use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

type Extras = Option<serde_json::Value>;
type Extensions = Option<serde_json::Value>;

macro_rules! define_id {
    ( $x:ident ) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
        #[serde(transparent)]
        pub struct $x(pub usize);
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
pub struct Root {
    pub asset: Asset,
    pub scene: Option<SceneId>,
    #[serde(default)]
    pub scenes: Vec<Scene>,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub meshes: Vec<Mesh>,
    #[serde(default)]
    pub accessors: Vec<Accessor>,
    #[serde(default)]
    pub buffer_views: Vec<BufferView>,
    #[serde(default)]
    pub buffers: Vec<Buffer>,
    #[serde(default)]
    pub materials: Vec<Material>,
    #[serde(default)]
    pub textures: Vec<Texture>,
    #[serde(default)]
    pub samplers: Vec<Sampler>,
    #[serde(default)]
    pub images: Vec<Image>,
    pub extras: Extras,
    pub extensions: Extensions,
}

macro_rules! define_id_lookup {
    ( $name:ident, $id_type:ident, $result_type:ident, $array:ident ) => {
        pub fn $name(&self, id: $id_type) -> Result<&$result_type, super::Error> {
            self.$array.get(id.0).ok_or(super::Error::IndexError)
        }
    };
}

impl Root {
    define_id_lookup!(scene, SceneId, Scene, scenes);
    define_id_lookup!(node, NodeId, Node, nodes);
    define_id_lookup!(mesh, MeshId, Mesh, meshes);
    define_id_lookup!(accessor, AccessorId, Accessor, accessors);
    define_id_lookup!(buffer_view, BufferViewId, BufferView, buffer_views);
    define_id_lookup!(buffer, BufferId, Buffer, buffers);
    define_id_lookup!(material, MaterialId, Material, materials);
    define_id_lookup!(texture, TextureId, Texture, textures);
    define_id_lookup!(sampler, SamplerId, Sampler, samplers);
    define_id_lookup!(image, ImageId, Image, images);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub version: String,
    pub copyright: Option<String>,
    pub generator: Option<String>,
    pub min_version: Option<String>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    #[serde(default)]
    pub nodes: Vec<NodeId>,
    pub name: Option<String>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub camera: Option<CameraId>,
    #[serde(default)]
    pub children: Vec<NodeId>,
    pub skin: Option<SkinId>,
    pub matrix: Option<[f64; 16]>,
    pub mesh: Option<MeshId>,
    pub rotation: Option<[f64; 4]>,
    pub scale: Option<[f64; 3]>,
    pub translation: Option<[f64; 3]>,
    #[serde(default)]
    pub weights: Vec<f64>,
    pub name: Option<String>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub primitives: Vec<MeshPrimitive>,
    #[serde(default)]
    pub weights: Vec<f64>,
    pub name: Option<String>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshPrimitive {
    pub attributes: HashMap<String, AccessorId>,
    pub indices: Option<AccessorId>,
    pub material: Option<MaterialId>,
    #[serde(default)]
    pub mode: MeshPrimitiveMode,
    #[serde(default)]
    pub targets: Vec<serde_json::Value>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MeshPrimitiveMode {
    Points = 0,
    Lines = 1,
    LineLoop = 2,
    LineStrip = 3,
    Triangles = 4,
    TriangleStrip = 5,
    TriangleFan = 6,
}

impl Default for MeshPrimitiveMode {
    fn default() -> Self {
        MeshPrimitiveMode::Triangles
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessor {
    pub buffer_view: Option<BufferViewId>,
    #[serde(default)]
    pub byte_offset: usize,
    pub component_type: ComponentType,
    #[serde(default)]
    pub normalized: bool,
    pub count: usize,
    #[serde(rename = "type")]
    pub type_: AccessorType,
    pub max: Option<Vec<f64>>,
    pub min: Option<Vec<f64>>,
    pub name: Option<String>,
    pub sparse: Option<serde_json::Value>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    pub buffer: BufferId,
    #[serde(default)]
    pub byte_offset: usize,
    pub byte_length: usize,
    pub byte_stride: Option<usize>,
    pub target: Option<u32>,
    pub name: Option<String>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Material {
    pub name: Option<String>,
    #[serde(default)]
    pub pbr_metallic_roughness: PbrMetallicRoughness,
    pub normal_texture: Option<TextureInfo>,
    pub occlusion_texture: Option<TextureInfo>,
    pub emissive_texture: Option<TextureInfo>,
    pub emissive_factor: Option<[f64; 3]>,
    #[serde(default)]
    pub alpha_mode: AlphaMode,
    #[serde(default = "default_alpha_cutoff")]
    pub alpha_cutoff: f64,
    #[serde(default)]
    pub double_sided: bool,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureInfo {
    pub index: TextureId,
    #[serde(default)]
    pub tex_coord: usize,
    pub scale: Option<f64>,
    pub strength: Option<f64>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PbrMetallicRoughness {
    #[serde(default = "default_base_color_factor")]
    pub base_color_factor: [f64; 4],
    pub base_color_texture: Option<TextureInfo>,
    #[serde(default = "one")]
    pub metallic_factor: f64,
    #[serde(default = "one")]
    pub roughness_factor: f64,
    pub metallic_roughness_texture: Option<TextureInfo>,
    pub extras: Extras,
    pub extensions: Extensions,
}

impl Default for PbrMetallicRoughness {
    fn default() -> PbrMetallicRoughness {
        PbrMetallicRoughness {
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
pub enum AlphaMode {
    OPAQUE,
    MASK,
    BLEND,
}

impl Default for AlphaMode {
    fn default() -> Self {
        AlphaMode::OPAQUE
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
pub struct Texture {
    pub sampler: Option<SamplerId>,
    pub source: Option<ImageId>,
    pub name: Option<String>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sampler {
    pub mag_filter: Option<u32>,
    pub min_filter: Option<u32>,
    pub wrap_s: Option<u32>,
    pub wrap_t: Option<u32>,
    pub name: Option<String>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub uri: Option<String>,
    pub mine_type: Option<String>,
    pub buffer_view: Option<BufferViewId>,
    pub name: Option<String>,
    pub extras: Extras,
    pub extensions: Extensions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum ComponentType {
    I8 = 5120,
    U8 = 5121,
    I16 = 5122,
    U16 = 5123,
    U32 = 5125,
    F32 = 5126,
}

impl ComponentType {
    pub fn len(self) -> usize {
        match self {
            ComponentType::I8 => 1,
            ComponentType::U8 => 1,
            ComponentType::I16 => 2,
            ComponentType::U16 => 2,
            ComponentType::U32 => 4,
            ComponentType::F32 => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessorType {
    SCALAR,
    VEC2,
    VEC3,
    VEC4,
    MAT2,
    MAT3,
    MAT4,
}

impl AccessorType {
    pub fn len(self) -> usize {
        match self {
            AccessorType::SCALAR => 1,
            AccessorType::VEC2 => 2,
            AccessorType::VEC3 => 3,
            AccessorType::VEC4 => 4,
            AccessorType::MAT2 => 4,
            AccessorType::MAT3 => 9,
            AccessorType::MAT4 => 16,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Buffer {
    pub uri: Option<String>,
    pub byte_length: usize,
    pub name: Option<String>,
    pub extras: Extras,
    pub extensions: Extensions,
}
