use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetLoaderError {
    #[error("io error")]
    IoError(#[from] std::io::Error),
}

pub fn resolve_path(name: impl AsRef<Path>, parent: Option<impl AsRef<Path>>) -> PathBuf {
    let mut buf = PathBuf::new();
    if let Some(p) = parent {
        buf.push(p.as_ref().parent().unwrap());
    }
    buf.push(name);
    buf
}

#[derive(Default)]
pub struct AssetLoader {}

impl AssetLoader {
    pub fn open_file(&self, path: impl AsRef<Path>) -> Result<std::fs::File, AssetLoaderError> {
        Ok(std::fs::File::open(path)?)
    }
    pub fn open_file_relative(
        &self,
        path: impl AsRef<Path>,
        parent: Option<impl AsRef<Path>>,
    ) -> Result<std::fs::File, AssetLoaderError> {
        Ok(std::fs::File::open(resolve_path(path, parent))?)
    }
}
