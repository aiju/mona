use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    marker::PhantomData,
    path::Path,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;

use crate::{
    geometry::{Matrix, Vec3, Vec4},
    mesh::{self, Color, Triangle},
};

type Extras = Option<serde_json::Value>;
type Extensions = Option<serde_json::Value>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfRoot {
    asset: GltfAsset,
    scene: Option<usize>,
    #[serde(default)]
    scenes: Vec<GltfScene>,
    #[serde(default)]
    nodes: Vec<GltfNode>,
    #[serde(default)]
    meshes: Vec<GltfMesh>,
    #[serde(default)]
    accessors: Vec<GltfAccessor>,
    #[serde(default)]
    buffer_views: Vec<GltfBufferView>,
    #[serde(default)]
    buffers: Vec<GltfBuffer>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfAsset {
    version: String,
    copyright: Option<String>,
    generator: Option<String>,
    min_version: Option<String>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GltfScene {
    #[serde(default)]
    nodes: Vec<usize>,
    name: Option<String>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GltfNode {
    camera: Option<usize>,
    #[serde(default)]
    children: Vec<usize>,
    skin: Option<usize>,
    matrix: Option<[f64; 16]>,
    mesh: Option<usize>,
    rotation: Option<[f64; 4]>,
    scale: Option<[f64; 3]>,
    translation: Option<[f64; 3]>,
    #[serde(default)]
    weights: Vec<f64>,
    name: Option<String>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GltfMesh {
    primitives: Vec<GltfMeshPrimitive>,
    #[serde(default)]
    weights: Vec<f64>,
    name: Option<String>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GltfMeshPrimitive {
    attributes: HashMap<String, usize>,
    indices: Option<usize>,
    material: Option<usize>,
    #[serde(default)]
    mode: GltfMeshPrimitiveMode,
    #[serde(default)]
    targets: Vec<serde_json::Value>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum GltfMeshPrimitiveMode {
    Points = 0,
    Lines = 1,
    LineLoop = 2,
    LineStrip = 3,
    Triangles = 4,
    TriangleStrip = 5,
    TriangleFan = 6,
}

impl Default for GltfMeshPrimitiveMode {
    fn default() -> Self {
        GltfMeshPrimitiveMode::Triangles
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfAccessor {
    buffer_view: Option<usize>,
    #[serde(default)]
    byte_offset: usize,
    component_type: GltfComponentType,
    #[serde(default)]
    normalized: bool,
    count: usize,
    #[serde(rename = "type")]
    type_: GltfAccessorType,
    max: Option<Vec<f64>>,
    min: Option<Vec<f64>>,
    name: Option<String>,
    sparse: Option<serde_json::Value>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfBufferView {
    buffer: usize,
    #[serde(default)]
    byte_offset: usize,
    byte_length: usize,
    byte_stride: Option<usize>,
    target: Option<u32>,
    name: Option<String>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum GltfComponentType {
    I8 = 5120,
    U8 = 5121,
    I16 = 5122,
    U16 = 5123,
    U32 = 5125,
    F32 = 5126,
}

impl GltfComponentType {
    fn len(self) -> usize {
        match self {
            GltfComponentType::I8 => 1,
            GltfComponentType::U8 => 1,
            GltfComponentType::I16 => 2,
            GltfComponentType::U16 => 2,
            GltfComponentType::U32 => 4,
            GltfComponentType::F32 => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GltfAccessorType {
    SCALAR,
    VEC2,
    VEC3,
    VEC4,
    MAT2,
    MAT3,
    MAT4,
}

impl GltfAccessorType {
    fn len(self) -> usize {
        match self {
            GltfAccessorType::SCALAR => 1,
            GltfAccessorType::VEC2 => 2,
            GltfAccessorType::VEC3 => 3,
            GltfAccessorType::VEC4 => 4,
            GltfAccessorType::MAT2 => 4,
            GltfAccessorType::MAT3 => 9,
            GltfAccessorType::MAT4 => 16,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfBuffer {
    uri: Option<String>,
    byte_length: usize,
    name: Option<String>,
    extras: Extras,
    extensions: Extensions,
}

pub struct Gltf {
    json: GltfRoot,
    buffers: Vec<Vec<u8>>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("json error")]
    JsonError(#[from] serde_json::Error),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("missing buffer")]
    MissingBuffer,
}

impl Gltf {
    fn load_buffers(path: &str, json: &GltfRoot) -> Result<Vec<Vec<u8>>, Error> {
        let mut buffers = Vec::new();
        for buffer in &json.buffers {
            let name = buffer.uri.as_ref().ok_or(Error::MissingBuffer)?;
            let buf_path = Path::new(path).parent().unwrap().join(name);
            let mut file = std::fs::File::open(buf_path)?;
            let mut buf = vec![0; buffer.byte_length];
            file.read_exact(&mut buf)?;
            buffers.push(buf);
        }
        Ok(buffers)
    }
    pub fn from_reader(path: &str, reader: impl std::io::Read) -> Result<Gltf, Error> {
        let json = serde_json::from_reader(reader)?;
        let buffers = Self::load_buffers(path, &json)?;
        Ok(Gltf { json, buffers })
    }
    pub fn from_file(path: &str) -> Result<Gltf, Error> {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);
        Self::from_reader(path, buf_reader)
    }
    fn accessor<T: GltfElement>(&self, index: usize) -> Accessor<'_, T> {
        let accessor = &self.json.accessors[index];
        let buffer_view = &self.json.buffer_views[accessor.buffer_view.unwrap()];
        let buffer = &self.json.buffers[buffer_view.buffer];
        let element_len = accessor.component_type.len() * accessor.type_.len();
        let byte_stride = buffer_view.byte_stride.unwrap_or(element_len);
        assert!(byte_stride >= element_len);
        let expected_len = (byte_stride * accessor.count).saturating_sub(byte_stride - element_len);
        let provided_len = std::cmp::min(
            buffer.byte_length.saturating_sub(buffer_view.byte_offset),
            buffer_view.byte_length,
        );
        assert!(expected_len >= provided_len);
        assert!(accessor.component_type == T::COMPONENT_TYPE && accessor.type_ == T::ACCESSOR_TYPE);
        Accessor {
            data: &self.buffers[buffer_view.buffer]
                [buffer_view.byte_offset..buffer_view.byte_offset + expected_len],
            byte_stride,
            count: accessor.count,
            _phantom: PhantomData,
        }
    }
    fn walk_nodes(
        &self,
        index: usize,
        mut matrix: Matrix,
        fun: &mut impl FnMut(&GltfNode, Matrix),
    ) {
        let node = &self.json.nodes[index];
        matrix = matrix * node.local_matrix();
        fun(node, matrix);
        for child in &node.children {
            self.walk_nodes(*child, matrix, fun);
        }
    }
    fn gather_primitive(&self, p: &GltfMeshPrimitive, matrix: Matrix, mesh: &mut mesh::Mesh) {
        assert!(p.mode == GltfMeshPrimitiveMode::Triangles);
        let index_accessor: Accessor<'_, u16> = self.accessor(p.indices.unwrap());
        let position_accessor: Accessor<'_, [f32; 3]> = self.accessor(p.attributes["POSITION"]);
        for (p1, p2, p3) in index_accessor
            .iter()
            .map(|i| {
                let vec: Vec3 = position_accessor.get(i as usize).into();
                let vec4: Vec4 = vec.into();
                (matrix * vec4).xyz()
            })
            .tuples()
        {
            mesh.triangles.last_mut().unwrap().push(Triangle {
                vertices: [p1, p2, p3],
                uv: Default::default(),
                color: [Color::WHITE; 3],
            });
        }
    }
    pub fn gather_meshes(&self) -> mesh::Mesh {
        let mut ret_mesh = mesh::Mesh {
            triangles: vec![vec![]],
            materials: vec![mesh::Material { texture: None }],
            textures: vec![],
        };
        let scene = &self.json.scenes[self.json.scene.unwrap()];
        for node in &scene.nodes {
            self.walk_nodes(*node, Matrix::IDENTITY, &mut |node, matrix| {
                if let Some(mesh_idx) = node.mesh {
                    let mesh = &self.json.meshes[mesh_idx];
                    for primitives in &mesh.primitives {
                        self.gather_primitive(&primitives, matrix, &mut ret_mesh);
                    }
                }
            });
        }
        ret_mesh
    }
}

impl GltfNode {
    fn local_matrix(&self) -> Matrix {
        if let Some(matrix) = self.matrix {
            matrix.into()
        } else {
            let mut matrix = if let Some(scale) = self.scale {
                Matrix::scale(scale[0], scale[1], scale[2])
            } else {
                Matrix::IDENTITY
            };
            if let Some(rotation) = self.rotation {
                matrix = Matrix::rotate_quaternion(rotation) * matrix;
            }
            if let Some(translation) = self.translation {
                matrix = Matrix::translate(translation[0], translation[1], translation[2]) * matrix;
            }
            matrix
        }
    }
}

pub trait GltfElement {
    const COMPONENT_TYPE: GltfComponentType;
    const ACCESSOR_TYPE: GltfAccessorType;
    unsafe fn read(data: &[u8]) -> Self;
}

impl GltfElement for u16 {
    const COMPONENT_TYPE: GltfComponentType = GltfComponentType::U16;
    const ACCESSOR_TYPE: GltfAccessorType = GltfAccessorType::SCALAR;
    unsafe fn read(data: &[u8]) -> Self {
        #[cfg(target_endian = "little")]
        unsafe {
            *data.as_ptr().cast()
        }
    }
}

impl GltfElement for u32 {
    const COMPONENT_TYPE: GltfComponentType = GltfComponentType::U32;
    const ACCESSOR_TYPE: GltfAccessorType = GltfAccessorType::SCALAR;
    unsafe fn read(data: &[u8]) -> Self {
        #[cfg(target_endian = "little")]
        unsafe {
            *data.as_ptr().cast()
        }
    }
}

impl GltfElement for [f32; 3] {
    const COMPONENT_TYPE: GltfComponentType = GltfComponentType::F32;
    const ACCESSOR_TYPE: GltfAccessorType = GltfAccessorType::VEC3;
    unsafe fn read(data: &[u8]) -> Self {
        #[cfg(target_endian = "little")]
        unsafe {
            *data.as_ptr().cast()
        }
    }
}

pub struct Accessor<'a, T> {
    data: &'a [u8],
    count: usize,
    byte_stride: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: GltfElement> Accessor<'a, T> {
    pub fn get(&self, index: usize) -> T {
        assert!(index < self.count);
        unsafe { T::read(&self.data[self.byte_stride * index..]) }
    }
    pub fn iter(&self) -> impl Iterator<Item = T> {
        (0..self.count).map(|n| self.get(n))
    }
}
