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
            .ok_or_else(|| "Failed to get file name")
    }

    pub fn named(path: impl AsRef<Path>) -> Result<PathBuf> {
        Ok(Self::MUSIC_BASE_PATH.join(Self::file_name(path)))
    }

    pub fn upload(&self) -> Result<String> {
        let path = Self::named(self.path)?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .map_err(|err| format!("Error opening file: {}", err))?;

        file.set_len(0)
            .map_err(|err| format!("unable to clear file for link: {}", err))?;
        file.write_all(&self.content.0)
            .map_err(|err| format!("error writing to file: {}", err))?;

        Ok(Self::file_name(self.path).to_string_lossy().to_string())
    }

    pub fn exists(path: impl AsRef<Path>) -> Result<()> {
        let file_name = path
            .as_ref()
            .file_name()
            .ok_or(|| "File name must end in an absolute path".to_owned())?;

        let existing_path = Self::MUSIC_BASE_PATH.append(&file_name);
        let path_exists = std::fs::try_exists(existing_path).unwrap_or_default();

        if path_exists {
            Ok(())
        } else {
            Err(
                "the file doesn't exist yet and must be uploaded before a link to it can be made"
                    .to_owned(),
            )
        }
    }
}
