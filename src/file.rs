//! Music file utilities, for uploading files and other relevant operations

use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use async_graphql::Result;

pub struct MusicFile {
    pub path: PathBuf,
    pub content: Vec<u8>,
}

impl MusicFile {
    const MUSIC_BASE_PATH: &'static str = "../httpsdocs/music/";

    fn file_name(path: impl AsRef<Path>) -> Result<OsString> {
        path.as_ref()
            .file_name()
            .map(|file_name| file_name.to_os_string())
            .ok_or_else(|| "Failed to get file name".into())
    }

    pub fn named(path: impl AsRef<Path>) -> Result<PathBuf> {
        Ok(PathBuf::from(Self::MUSIC_BASE_PATH).join(Self::file_name(path)?))
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = Self::named(&self.path)?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .map_err(|err| format!("Error opening file: {}", err))?;

        file.set_len(0)
            .map_err(|err| format!("unable to clear file for link: {}", err))?;
        file.write_all(&self.content)
            .map_err(|err| format!("error writing to file: {}", err))?;

        Ok(path)
    }

    pub fn exists(path: impl AsRef<Path>) -> Result<bool> {
        let file_name = Self::named(path)?;

        Ok(std::fs::try_exists(file_name).unwrap_or(false))
    }

    pub fn ensure_exists(path: impl AsRef<Path>) -> Result<()> {
        if Self::exists(path)? {
            Ok(())
        } else {
            Err("the file doesn't exist yet and must be \
                 uploaded before a link to it can be made"
                .into())
        }
    }
}
