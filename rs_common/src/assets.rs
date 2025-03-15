use thiserror::Error;
use crate::render::Texture;
use std::{fmt::Debug, io::{Read, Seek}};

#[derive(Error, Debug)]
pub enum AssetLoaderError {
    #[error("io error")]
    IoError(#[from] std::io::Error),
}

pub trait AssetLoader {
    type File: Read + Seek;
    fn open_file(&mut self, name: &str, parent: Option<&str>) -> Result<Self::File, AssetLoaderError>;
    fn load_texture(&mut self, name: &str) -> Result<Texture, AssetLoaderError>;
}

impl<T: AssetLoader> AssetLoader for &mut T {
    type File = T::File;

    fn open_file(&mut self, name: &str, parent: Option<&str>) -> Result<Self::File, AssetLoaderError> {
        (**self).open_file(name, parent)
    }

    fn load_texture(&mut self, name: &str) -> Result<Texture, AssetLoaderError> {
        (**self).load_texture(name)
    }
}