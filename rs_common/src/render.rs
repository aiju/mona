use std::borrow::Cow;
use std::fmt::Debug;

use crate::{BarePrimitive, CoarseRasterIn};

#[derive(Debug, Clone)]
pub struct TextureType {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
}

#[derive(Clone)]
pub struct Texture<'a> {
    pub data: Cow<'a, [u8]>,
    pub ty: TextureType,
}

pub trait Backend {
    type Texture;
    type Error: Debug;
    fn load_texture(&mut self, texture: Texture) -> Result<Self::Texture, Self::Error>;
    fn use_texture(&mut self, texture: Option<&Self::Texture>);
    fn draw(&mut self, triangles: &[CoarseRasterIn]);
}

pub struct Context<B: Backend> {
    backend: B,
    textures: Vec<B::Texture>,
    current_texture: Option<TextureId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TextureId(u32);

impl<B: Backend> Context<B> {
    pub fn new(backend: B) -> Self {
        Context {
            backend,
            textures: Vec::new(),
            current_texture: None,
        }
    }
    pub fn backend(&self) -> &B {
        &self.backend
    }
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }
    pub fn load_texture(&mut self, texture: Texture) -> Result<TextureId, B::Error> {
        let id = TextureId(self.textures.len().try_into().unwrap());
        let tex = self.backend.load_texture(texture)?;
        self.textures.push(tex);
        Ok(id)
    }
    pub fn draw(&mut self) -> DrawCall<'_, B> {
        DrawCall {
            context: self,
            texture: None,
        }
    }
}

pub struct DrawCall<'a, B: Backend> {
    context: &'a mut Context<B>,
    texture: Option<TextureId>,
}

impl<B: Backend> DrawCall<'_, B> {
    pub fn textured(mut self, texture: TextureId) -> Self {
        self.texture = Some(texture);
        self
    }
    pub fn opt_textured(mut self, texture: Option<TextureId>) -> Self {
        self.texture = texture;
        self
    }
    pub fn run(self, triangles: &[BarePrimitive]) {
        let ctx = self.context;
        if ctx.current_texture != self.texture {
            let texture = self.texture.map(|id| &ctx.textures[id.0 as usize]);
            ctx.backend.use_texture(texture);
            ctx.current_texture = self.texture;
        }
        let tris = triangles
            .iter()
            .flat_map(|t| CoarseRasterIn::new(t))
            .collect::<Vec<_>>();
        ctx.backend.draw(&tris);
    }
}
