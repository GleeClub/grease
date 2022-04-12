//! Music file utilities, for uploading files and other relevant operations.

use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use async_graphql::Result;

pub struct MusicFile {
    pub path: PathBuf,
    pub content: Vec<u8>,
}

impl MusicFile {
    const MUSIC_BASE_PATH: PathBuf = PathBuf::from("../httpsdocs/music/");

    fn file_name<'a>(path: impl AsRef<Path> + 'a) -> Result<&'a OsStr> {
        path.as_ref()
            .file_name()
            .ok_or("Failed to get file name")
            .into()
    }

    pub fn named(path: impl AsRef<Path>) -> Result<PathBuf> {
        Ok(Self::MUSIC_BASE_PATH.join(Self::file_name(path)?))
    }

    pub fn upload(&self) -> Result<PathBuf> {
        let path = Self::named(self.path)?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .map_err(|err| format!("Error opening file: {}", err))?;

        file.set_len(0)
            .map_err(|err| format!("unable to clear file for link: {}", err))?;
        file.write_all(&self.content)
            .map_err(|err| format!("error writing to file: {}", err))?;

        Self::named(self.path)
    }

    pub fn exists(path: impl AsRef<Path>) -> Result<PathBuf> {
        let file_name = Self::named(path)?;

        if std::fs::try_exists(file_name).unwrap_or(false) {
            Ok(file_name)
        } else {
            Err(
                "the file doesn't exist yet and must be uploaded before a link to it can be made"
                    .into()
            )
        }
    }
}
