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
    geometry::{Matrix, Vec2, Vec3, Vec4},
    mesh::{self, Color, Triangle},
};

type Extras = Option<serde_json::Value>;
type Extensions = Option<serde_json::Value>;

macro_rules! define_id {
    ( $x:ident ) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
        #[serde(transparent)]
        struct $x(usize);
    };
}

define_id!(SceneId);
define_id!(NodeId);
define_id!(CameraId);
define_id!(MeshId);
define_id!(AccessorId);
define_id!(BufferViewId);
define_id!(BufferId);
define_id!(MaterialId);
define_id!(TextureId);
define_id!(SamplerId);
define_id!(ImageId);
define_id!(SkinId);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfRoot {
    asset: GltfAsset,
    scene: Option<SceneId>,
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
    #[serde(default)]
    materials: Vec<GltfMaterial>,
    #[serde(default)]
    textures: Vec<GltfTexture>,
    #[serde(default)]
    samplers: Vec<GltfSampler>,
    #[serde(default)]
    images: Vec<GltfImage>,
    extras: Extras,
    extensions: Extensions,
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
    nodes: Vec<NodeId>,
    name: Option<String>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GltfNode {
    camera: Option<CameraId>,
    #[serde(default)]
    children: Vec<NodeId>,
    skin: Option<SkinId>,
    matrix: Option<[f64; 16]>,
    mesh: Option<MeshId>,
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
    attributes: HashMap<String, AccessorId>,
    indices: Option<AccessorId>,
    material: Option<MaterialId>,
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
    buffer_view: Option<BufferViewId>,
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
    buffer: BufferId,
    #[serde(default)]
    byte_offset: usize,
    byte_length: usize,
    byte_stride: Option<usize>,
    target: Option<u32>,
    name: Option<String>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfMaterial {
    name: Option<String>,
    #[serde(default)]
    pbr_metallic_roughness: GltfPbrMetallicRoughness,
    normal_texture: Option<GltfTextureInfo>,
    occlusion_texture: Option<GltfTextureInfo>,
    emissive_texture: Option<GltfTextureInfo>,
    emissive_factor: Option<[f64; 3]>,
    #[serde(default)]
    alpha_mode: GltfAlphaMode,
    #[serde(default = "default_alpha_cutoff")]
    alpha_cutoff: f64,
    #[serde(default)]
    double_sided: bool,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfTextureInfo {
    index: TextureId,
    #[serde(default)]
    tex_coord: usize,
    scale: Option<f64>,
    strength: Option<f64>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfPbrMetallicRoughness {
    #[serde(default = "default_base_color_factor")]
    base_color_factor: [f64; 4],
    base_color_texture: Option<GltfTextureInfo>,
    #[serde(default = "one")]
    metallic_factor: f64,
    #[serde(default = "one")]
    roughness_factor: f64,
    metallic_roughness_texture: Option<GltfTextureInfo>,
    extras: Extras,
    extensions: Extensions,
}

impl Default for GltfPbrMetallicRoughness {
    fn default() -> GltfPbrMetallicRoughness {
        GltfPbrMetallicRoughness {
            base_color_factor: default_base_color_factor(),
            base_color_texture: None,
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            metallic_roughness_texture: None,
            extras: None,
            extensions: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum GltfAlphaMode {
    OPAQUE,
    MASK,
    BLEND,
}

impl Default for GltfAlphaMode {
    fn default() -> Self {
        GltfAlphaMode::OPAQUE
    }
}

fn default_base_color_factor() -> [f64; 4] {
    [1.0; 4]
}
fn one() -> f64 {
    1.0
}
fn default_alpha_cutoff() -> f64 {
    0.5
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfTexture {
    sampler: Option<SamplerId>,
    source: Option<ImageId>,
    name: Option<String>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfSampler {
    mag_filter: Option<u32>,
    min_filter: Option<u32>,
    wrap_s: Option<u32>,
    wrap_t: Option<u32>,
    name: Option<String>,
    extras: Extras,
    extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GltfImage {
    uri: Option<String>,
    mine_type: Option<String>,
    buffer_view: Option<BufferViewId>,
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
    #[error("index error")]
    IndexError,
}

macro_rules! define_id_lookup {
    ( $name:ident, $id_type:ident, $result_type:ident, $array:ident ) => {
        fn $name(&self, id: $id_type) -> Result<&$result_type, Error> {
            self.$array.get(id.0).ok_or(Error::IndexError)
        }
    };
}

impl GltfRoot {
    define_id_lookup!(scene, SceneId, GltfScene, scenes);
    define_id_lookup!(node, NodeId, GltfNode, nodes);
    define_id_lookup!(mesh, MeshId, GltfMesh, meshes);
    define_id_lookup!(accessor, AccessorId, GltfAccessor, accessors);
    define_id_lookup!(buffer_view, BufferViewId, GltfBufferView, buffer_views);
    define_id_lookup!(buffer, BufferId, GltfBuffer, buffers);
    define_id_lookup!(material, MaterialId, GltfMaterial, materials);
    define_id_lookup!(texture, TextureId, GltfTexture, textures);
    define_id_lookup!(sampler, SamplerId, GltfSampler, samplers);
    define_id_lookup!(image, ImageId, GltfImage, images);
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
    fn accessor<T: GltfElement>(&self, index: AccessorId) -> Result<Accessor<'_, T>, Error> {
        let accessor = self.json.accessor(index)?;
        assert!(accessor.sparse.is_none());
        let buffer_view = self.json.buffer_view(accessor.buffer_view.unwrap())?;
        let buffer = self.json.buffer(buffer_view.buffer)?;
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
        Ok(Accessor {
            data: &self.buffers[buffer_view.buffer.0]
                [buffer_view.byte_offset..buffer_view.byte_offset + expected_len],
            byte_stride,
            count: accessor.count,
            _phantom: PhantomData,
        })
    }
    fn translate_materials(&self, mesh: &mut mesh::Mesh) -> Result<(), Error> {
        for texture in &self.json.textures {
            let image = self.json.image(texture.source.unwrap())?;
            mesh.textures.push(image.uri.clone().unwrap());
        }
        for material in &self.json.materials {
            let mut mesh_material = mesh::Material::default();
            if let Some(texture) = &material.pbr_metallic_roughness.base_color_texture {
                mesh_material.texture = Some(texture.index.0);
            }
            mesh.triangles.push(Vec::new());
            mesh.materials.push(mesh_material);
        }
        Ok(())
    }
    fn walk_nodes(
        &self,
        index: NodeId,
        mut matrix: Matrix,
        fun: &mut impl FnMut(&GltfNode, Matrix) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let node = self.json.node(index)?;
        matrix = matrix * node.local_matrix();
        fun(node, matrix)?;
        for child in &node.children {
            self.walk_nodes(*child, matrix, fun)?;
        }
        Ok(())
    }
    fn material_color(&self, index: Option<MaterialId>) -> Result<Color, Error> {
        if let Some(index) = index {
            Ok(self
                .json
                .material(index)?
                .pbr_metallic_roughness
                .base_color_factor
                .into())
        } else {
            Ok(Color::WHITE)
        }
    }
    fn texcoord_accessor(
        &self,
        p: &GltfMeshPrimitive,
    ) -> Result<Option<Accessor<'_, [f32; 2]>>, Error> {
        let Some(material_id) = p.material else {
            return Ok(None);
        };
        let material = self.json.material(material_id)?;
        let Some(tex_info) = &material.pbr_metallic_roughness.base_color_texture else {
            return Ok(None);
        };
        let Some(&accessor_idx) = p
            .attributes
            .get(&format!("TEXCOORD_{}", tex_info.tex_coord))
        else {
            return Ok(None);
        };
        Ok(Some(self.accessor(accessor_idx)?))
    }
    fn gather_primitive(
        &self,
        p: &GltfMeshPrimitive,
        matrix: Matrix,
        mesh: &mut mesh::Mesh,
    ) -> Result<(), Error> {
        assert!(p.mode == GltfMeshPrimitiveMode::Triangles);
        let index_accessor: Accessor<'_, u16> = self.accessor(p.indices.unwrap())?;
        let position_accessor: Accessor<'_, [f32; 3]> = self.accessor(p.attributes["POSITION"])?;
        let texcoord_accessor = self.texcoord_accessor(p)?;
        let material_index = p.material.map(|x| x.0 + 1).unwrap_or_default();
        let color: Color = self.material_color(p.material)?;
        for ((p1, uv1), (p2, uv2), (p3, uv3)) in index_accessor
            .iter()
            .map(|i| {
                let vec: Vec3 = position_accessor.get(i as usize).into();
                let vec4: Vec4 = vec.into();
                let tex_coord: Vec2 = texcoord_accessor
                    .as_ref()
                    .map(|a| a.get(i as usize))
                    .unwrap_or_default()
                    .into();
                ((matrix * vec4).xyz(), tex_coord)
            })
            .tuples()
        {
            mesh.triangles[material_index].push(Triangle {
                vertices: [p1, p2, p3],
                uv: [uv1, uv2, uv3],
                color: [color; 3],
            });
        }
        Ok(())
    }
    pub fn gather_meshes(&self) -> Result<mesh::Mesh, Error> {
        let mut ret_mesh = mesh::Mesh {
            triangles: vec![vec![]],
            materials: vec![mesh::Material { texture: None }],
            textures: vec![],
        };
        self.translate_materials(&mut ret_mesh)?;
        let scene = self.json.scene(self.json.scene.unwrap())?;
        for node in &scene.nodes {
            self.walk_nodes(*node, Matrix::IDENTITY, &mut |node, matrix| {
                if let Some(mesh_idx) = node.mesh {
                    let mesh = self.json.mesh(mesh_idx)?;
                    for primitives in &mesh.primitives {
                        self.gather_primitive(&primitives, matrix, &mut ret_mesh)?;
                    }
                }
                Ok(())
            })?;
        }
        Ok(ret_mesh)
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

impl GltfElement for [f32; 2] {
    const COMPONENT_TYPE: GltfComponentType = GltfComponentType::F32;
    const ACCESSOR_TYPE: GltfAccessorType = GltfAccessorType::VEC2;
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
