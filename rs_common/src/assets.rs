use crate::render::Texture;
use std::fmt::Debug;

pub trait AssetLoader {
    type Error: Debug;
    fn load_texture(&mut self, name: &str) -> Result<Texture, Self::Error>;
}
