use crate::geometry::{Vec2, Vec3};

pub struct Material {
}

pub struct MaterialId(u32);

pub struct Triangle {
    vertices: [Vec3; 3],
    uv: [Vec2; 3],
    rgb: [u32; 3],
    material: MaterialId,
}

pub struct Mesh {
    triangles: Vec<Triangle>
}