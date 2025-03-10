use crate::{
    assets::AssetLoader,
    geometry::{Vec2, Vec3},
    render::{Backend, Context, TextureId},
};

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

#[derive(Clone, Debug)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
    pub uv: [Vec2; 3],
    pub rgb: [u32; 3],
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
