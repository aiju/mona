use crate::{
    assets::AssetLoader,
    geometry::{Vec2, Vec3},
    render::{Backend, Context, TextureId},
};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    pub fn as_u32(self) -> u32 {
        (self.r as u32) | (self.g as u32) << 8 | (self.b as u32) << 16
    }
}

impl std::ops::Mul<f64> for Color {
    type Output = Color;
    fn mul(self, rhs: f64) -> Self::Output {
        let r = ((self.r as f64) * rhs) as u8;
        let g = ((self.g as f64) * rhs) as u8;
        let b = ((self.b as f64) * rhs) as u8;
        Color { r, g, b }
    }
}

impl From<[f64; 3]> for Color {
    fn from(value: [f64; 3]) -> Self {
        let f = |v: f64| (v * 255.0) as u8;
        Color {
            r: f(value[0]),
            g: f(value[1]),
            b: f(value[2]),
        }
    }
}

impl From<[f64; 4]> for Color {
    fn from(value: [f64; 4]) -> Self {
        let f = |v: f64| (v * 255.0) as u8;
        Color {
            r: f(value[0]),
            g: f(value[1]),
            b: f(value[2]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Material<T> {
    pub texture: Option<T>,
}
impl<T> Material<T> {
    fn map_texture<S>(&self, fun: impl Fn(&T) -> S) -> Material<S> {
        Material {
            texture: self.texture.as_ref().map(fun),
        }
    }
}

impl<T> Default for Material<T> {
    fn default() -> Self {
        Self { texture: Default::default() }
    }
}

#[derive(Clone, Debug)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
    pub uv: [Vec2; 3],
    pub color: [Color; 3],
}

#[derive(Default, Clone, Debug)]
pub struct Mesh {
    pub triangles: Vec<Vec<Triangle>>,
    pub materials: Vec<Material<usize>>,
    pub textures: Vec<String>,
}

impl Mesh {
    pub fn load<B: Backend>(
        &self,
        context: &mut Context<B>,
        mut loader: impl AssetLoader,
    ) -> LoadedMesh {
        let textures: Vec<_> = self
            .textures
            .iter()
            .map(|name| {
                let texture = loader.load_texture(name).unwrap();
                context.load_texture(texture).unwrap()
            })
            .collect();
        LoadedMesh {
            triangles: self.triangles.clone(),
            materials: self
                .materials
                .iter()
                .map(|m| m.map_texture(|&n| textures[n]))
                .collect(),
        }
    }
}

pub struct LoadedMesh {
    pub triangles: Vec<Vec<Triangle>>,
    pub materials: Vec<Material<TextureId>>,
}
