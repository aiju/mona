use crate::{
    assets::{AssetLoader, AssetLoaderError},
    entity::{self, EntityId, World},
    geometry::{Matrix, Quaternion, Vec2, Vec3, Vec4},
    mesh::{self, Color, Texture, TextureState},
};
use binary::Accessor;
use itertools::Itertools;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io::Read,
    rc::Rc,
};
use thiserror::Error;

mod animation;
mod binary;
mod json;

pub use animation::{Animation, Skin};

//FIXME: maybe shouldnt be public ?
pub use json::AnimationId;

pub struct Material {
    id: Option<json::MaterialId>,
    texture: Option<Rc<Texture>>,
    texcoord_idx: usize,
    color: Color,
}

thread_local! {
    static DEFAULT_MATERIAL: Rc<Material> = Rc::new(Material {
        id: None,
        texture: None,
        color: Color::WHITE,
        texcoord_idx: 0,
    });
}

fn default_material() -> Rc<Material> {
    DEFAULT_MATERIAL.with(Clone::clone)
}

pub struct Primitive {
    material: Rc<Material>,
    indices: Vec<u32>,
    position: Vec<Vec3>,
    texcoord: Vec<Vec<Vec2>>,
    joints: Vec<Vec<(usize, f64)>>,
    color: Vec<Vec4>,
}

pub struct Mesh {
    primitives: Vec<Primitive>,
}

pub struct Node {
    pub name: Option<String>,
    mesh: Option<Rc<Mesh>>,
    skin: Option<Rc<Skin>>,
    children: Vec<Rc<Node>>,
    transform: Transform,
    used_by_animation: Cell<bool>,
}

#[derive(Clone, Copy, Debug)]
pub enum Transform {
    Matrix(Matrix),
    Trs {
        translate: Vec3,
        rotate: Option<Quaternion>,
        scale: Option<Vec3>,
    },
}

pub struct GltfImporter<'a> {
    json: json::Root,
    loader: &'a mut AssetLoader,
    file_name: Option<String>,
    buffers: Memoize<json::BufferId, Rc<Vec<u8>>>,
    textures: Memoize<json::TextureId, Rc<Texture>>,
    materials: Memoize<json::MaterialId, Rc<Material>>,
    meshes: Memoize<json::MeshId, Rc<Mesh>>,
    nodes: Memoize<json::NodeId, Rc<Node>>,
    animations: Memoize<json::AnimationId, Rc<Animation>>,
    skins: Memoize<json::SkinId, Rc<Skin>>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("json error")]
    JsonError(#[from] serde_json::Error),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("index error")]
    IndexError,
    #[error("invalid file")]
    InvalidFile,
    #[error("unsupported feature")]
    UnsupportedFeature,
    #[error("asset loader error")]
    AssetLoaderError(#[from] AssetLoaderError),
}

macro_rules! gltf_unwrap {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => gltf_abort!(),
        }
    };
}

macro_rules! gltf_assert {
    ( $e:expr ) => {
        $e.then_some(()).ok_or(Error::InvalidFile)?
    };
}

macro_rules! gltf_abort {
    () => {{
        gltf_assert!(false);
        unreachable!()
    }};
}

macro_rules! gltf_assert_supported {
    ( $e:expr ) => {
        $e.then_some(()).ok_or(Error::UnsupportedFeature)?
    };
}
pub(self) use {gltf_abort, gltf_assert, gltf_assert_supported, gltf_unwrap};

struct Memoize<K, V>(RefCell<HashMap<K, Option<V>>>);

impl<K, V> Default for Memoize<K, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> Memoize<K, V> {
    fn insert(&self, key: K, value: V) {
        self.0.borrow_mut().insert(key, Some(value));
    }
    fn get_or_insert(&self, key: K, fun: impl FnOnce() -> Result<V, Error>) -> Result<V, Error> {
        if let Some(value) = self.0.borrow().get(&key) {
            Ok(value.clone().expect("cycle during creation"))
        } else {
            self.0.borrow_mut().insert(key.clone(), None);
            let value = fun()?;
            self.0.borrow_mut().insert(key, Some(value.clone()));
            Ok(value)
        }
    }
}

impl<'a> GltfImporter<'a> {
    fn new(json: json::Root, loader: &'a mut AssetLoader, file_name: Option<String>) -> Self {
        GltfImporter {
            json,
            file_name,
            loader,
            buffers: Default::default(),
            textures: Default::default(),
            materials: Default::default(),
            meshes: Default::default(),
            nodes: Default::default(),
            animations: Default::default(),
            skins: Default::default(),
        }
    }
    fn read_chunk(
        reader: &mut impl std::io::Read,
        remaining_len: &mut u32,
        expected_type: u32,
    ) -> Result<Vec<u8>, Error> {
        let mut buf = [0; 8];
        reader.read_exact(&mut buf)?;
        let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let ty = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let len_w_header = len.checked_add(8);
        gltf_assert!(len_w_header.is_some());
        gltf_assert!(*remaining_len >= len_w_header.unwrap());
        gltf_assert!(ty == expected_type);
        *remaining_len -= len_w_header.unwrap();
        let mut buf = vec![0; len as usize];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }
    pub fn from_reader(
        mut reader: impl std::io::Read,
        loader: &'a mut AssetLoader,
        file_name: Option<String>,
    ) -> Result<Self, Error> {
        let mut buf = vec![0; 12];
        reader.read_exact(&mut buf)?;
        if u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) == 0x46546C67 {
            let mut remaining_len = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
            remaining_len = gltf_unwrap!(remaining_len.checked_sub(12));
            buf = Self::read_chunk(&mut reader, &mut remaining_len, 0x4E4F534A)?;
            let json = serde_json::from_slice(&buf)?;
            let gltf = Self::new(json, loader, file_name);
            if remaining_len > 0 {
                buf = Self::read_chunk(&mut reader, &mut remaining_len, 0x004E4942)?;
                gltf_assert!(gltf.json.buffers.len() > 0 && gltf.json.buffers[0].uri.is_none());
                gltf.buffers.insert(json::BufferId(0), Rc::new(buf));
            }
            Ok(gltf)
        } else {
            reader.read_to_end(&mut buf)?;
            let json = serde_json::from_slice(&buf)?;
            Ok(Self::new(json, loader, file_name))
        }
    }
    pub fn from_file(file_name: String, loader: &'a mut AssetLoader) -> Result<Self, Error> {
        let file = loader.open_file(&file_name)?;
        Self::from_reader(file, loader, Some(file_name))
    }
    fn buffer(&self, id: json::BufferId) -> Result<Rc<Vec<u8>>, Error> {
        self.buffers.get_or_insert(id, || {
            let buffer = self.json.buffer(id)?;
            let name = gltf_unwrap!(buffer.uri.as_ref());
            let mut file = self
                .loader
                .open_file_relative(name, self.file_name.as_ref().map(|x| &**x))?;
            let mut buf = vec![0; buffer.byte_length];
            file.read_exact(&mut buf)?;
            Ok(Rc::new(buf))
        })
    }
    fn accessor<T: Accessor>(&self, id: json::AccessorId) -> Result<Vec<T>, Error> {
        let accessor = self.json.accessor(id)?;
        let buffer_view = self.json.buffer_view(accessor.buffer_view.unwrap())?;
        let buffer = self.json.buffer(buffer_view.buffer)?;
        let element_len = accessor.component_type.len() * accessor.type_.len();
        let byte_stride = buffer_view.byte_stride.unwrap_or(element_len);
        gltf_assert!(byte_stride >= element_len);
        let expected_len = (byte_stride * accessor.count).saturating_sub(byte_stride - element_len);
        let provided_len = std::cmp::min(
            buffer.byte_length.saturating_sub(buffer_view.byte_offset),
            buffer_view.byte_length,
        );
        gltf_assert!(expected_len >= provided_len);
        let data = self.buffer(buffer_view.buffer)?;
        gltf_assert!(accessor.component_type == T::COMPONENT_TYPE);
        gltf_assert!(accessor.type_ == T::ACCESSOR_TYPE);
        Ok((0..accessor.count)
            .map(|i| unsafe { T::read(&data[buffer_view.byte_offset + byte_stride * i..]) })
            .collect())
    }
    fn index_accessor(&self, id: json::AccessorId) -> Result<Vec<u32>, Error> {
        match self.json.accessor(id)?.component_type {
            json::ComponentType::U8 => Ok(self
                .accessor::<u8>(id)?
                .into_iter()
                .map(|x| x as u32)
                .collect()),
            json::ComponentType::U16 => Ok(self
                .accessor::<u16>(id)?
                .into_iter()
                .map(|x| x as u32)
                .collect()),
            json::ComponentType::U32 => self.accessor(id),
            _ => gltf_abort!(),
        }
    }
    fn primitive_attribute_count(&self, primitive: &json::MeshPrimitive) -> Result<usize, Error> {
        let mut count = None;
        for (_, id) in &primitive.attributes {
            let c1 = self.json.accessor(*id)?.count;
            if let Some(c0) = count {
                gltf_assert!(c0 == c1);
            } else {
                count = Some(c1);
            }
        }
        Ok(gltf_unwrap!(count))
    }
    fn joint_accessor(&self, id: json::AccessorId) -> Result<Vec<[u16; 4]>, Error> {
        match self.json.accessor(id)?.component_type {
            json::ComponentType::U8 => Ok(self
                .accessor::<[u8; 4]>(id)?
                .into_iter()
                .map(|x| x.map(|y| y as u16))
                .collect()),
            json::ComponentType::U16 => self.accessor(id),
            _ => gltf_abort!(),
        }
    }
    fn primitive_joints(
        &self,
        prim: &json::MeshPrimitive,
        attr_count: usize,
    ) -> Result<Vec<Vec<(usize, f64)>>, Error> {
        let mut result = vec![vec![]; attr_count];
        for i in 0.. {
            match (
                prim.attributes.get(&format!("JOINTS_{}", i)),
                prim.attributes.get(&format!("WEIGHTS_{}", i)),
            ) {
                (Some(&joints_id), Some(&weight_id)) => {
                    let joints: Vec<[u16; 4]> = self.joint_accessor(joints_id)?;
                    let weights: Vec<[f64; 4]> = self.accessor(weight_id)?;
                    for (i, (j, w)) in joints.iter().zip(&weights).enumerate() {
                        for k in 0..4 {
                            result[i].push((j[k] as usize, w[k]));
                        }
                    }
                }
                (None, None) => {
                    break;
                }
                (_, _) => gltf_abort!(),
            }
        }
        Ok(result)
    }
    fn mesh(&self, id: json::MeshId) -> Result<Rc<Mesh>, Error> {
        self.meshes.get_or_insert(id, || {
            let mesh = self.json.mesh(id)?;
            let mut primitives = Vec::new();
            for prim in &mesh.primitives {
                gltf_assert_supported!(prim.mode == json::MeshPrimitiveMode::Triangles);
                gltf_assert_supported!(prim.attributes.contains_key("POSITION"));
                let material = prim
                    .material
                    .map_or(Ok(default_material()), |id| self.material(id))?;
                let attr_count = self.primitive_attribute_count(prim)?;
                let indices = prim.indices.map_or_else(
                    || Ok((0..attr_count as u32).collect()),
                    |id| self.index_accessor(id),
                )?;
                gltf_assert!(indices.len() % 3 == 0);
                let position = self.accessor::<Vec3>(prim.attributes["POSITION"])?;
                let mut texcoord = Vec::new();
                let mut i = 0;
                while let Some(&id) = prim.attributes.get(&format!("TEXCOORD_{}", i)) {
                    texcoord.push(self.accessor(id)?);
                    i += 1;
                }
                let joints = self.primitive_joints(&prim, attr_count)?;
                primitives.push(Primitive {
                    material,
                    indices,
                    position,
                    texcoord,
                    joints,
                    color: Vec::new(),
                });
            }
            Ok(Rc::new(Mesh { primitives }))
        })
    }
    fn texture(&self, id: json::TextureId) -> Result<Rc<Texture>, Error> {
        self.textures.get_or_insert(id, || {
            let texture = self.json.texture(id)?;
            let image = self.json.image(gltf_unwrap!(texture.source))?;
            if let Some(uri) = &image.uri {
                gltf_assert!(image.buffer_view.is_none());
                Ok(Texture::from_file(uri, self.file_name.as_deref()))
            } else {
                let buffer_view = self.json.buffer_view(gltf_unwrap!(image.buffer_view))?;
                let buffer = self.buffer(buffer_view.buffer)?;
                let data = buffer
                    [buffer_view.byte_offset..buffer_view.byte_offset + buffer_view.byte_length]
                    .to_vec();
                Ok(Rc::new(Texture {
                    state: RefCell::new(TextureState::Memory(data)),
                }))
            }
        })
    }
    fn material(&self, id: json::MaterialId) -> Result<Rc<Material>, Error> {
        self.materials.get_or_insert(id, || {
            let material = self.json.material(id)?;
            let texture = material
                .pbr_metallic_roughness
                .base_color_texture
                .as_ref()
                .map(|info| self.texture(info.index))
                .transpose()?;
            let color = material.pbr_metallic_roughness.base_color_factor.into();
            Ok(Rc::new(Material {
                id: Some(id),
                texcoord_idx: material
                    .pbr_metallic_roughness
                    .base_color_texture
                    .as_ref()
                    .map_or(0, |t| t.tex_coord),
                texture,
                color,
            }))
        })
    }
    fn node(&self, id: json::NodeId) -> Result<Rc<Node>, Error> {
        self.nodes.get_or_insert(id, || {
            let node = self.json.node(id)?;
            let mesh = node.mesh.map(|id| self.mesh(id)).transpose()?;
            let transform = node.transform();
            let children = node
                .children
                .iter()
                .map(|n| self.node(*n))
                .collect::<Result<Vec<_>, _>>()?;
            let skin = node.skin.map(|id| self.skin(id)).transpose()?;
            Ok(Rc::new(Node {
                name: node.name.clone(),
                mesh,
                children,
                transform,
                skin,
                used_by_animation: false.into(),
            }))
        })
    }
    pub fn scene(&self, id: json::SceneId) -> Result<Node, Error> {
        let scene = self.json.scene(id)?;
        let nodes = scene
            .nodes
            .iter()
            .map(|id| self.node(*id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Node {
            name: None,
            children: nodes,
            mesh: None,
            skin: None,
            transform: Transform::default(),
            used_by_animation: false.into(),
        })
    }
    pub fn root_scene(&self) -> Result<Option<Node>, Error> {
        self.json.scene.map(|id| self.scene(id)).transpose()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform::Trs {
            translate: Vec3::zero(),
            rotate: None,
            scale: None,
        }
    }
}

impl json::Node {
    fn transform(&self) -> Transform {
        if let Some(matrix) = self.matrix {
            Transform::Matrix(matrix.into())
        } else {
            let translate = self.translation.map_or(Vec3::zero(), Into::into);
            let rotate = self.rotation.map(Into::into);
            let scale = self.scale.map(Into::into);
            Transform::Trs {
                translate,
                rotate,
                scale,
            }
        }
    }
}

impl Transform {
    pub fn matrix(&self) -> Matrix {
        match *self {
            Transform::Matrix(matrix) => matrix,
            Transform::Trs {
                translate,
                rotate,
                scale,
            } => {
                let mut matrix = rotate.map_or(Matrix::IDENTITY, Into::into);
                if let Some(scale) = scale {
                    for i in 0..4 {
                        for j in 0..3 {
                            matrix.0[i][j] *= scale[j];
                        }
                    }
                }
                for i in 0..3 {
                    matrix.0[i][3] = translate[i];
                }
                matrix
            }
        }
    }
}

impl std::ops::Mul<Transform> for Transform {
    type Output = Transform;
    fn mul(self, rhs: Transform) -> Self::Output {
        match (self, rhs) {
            (
                Transform::Trs {
                    translate: t1,
                    rotate: None,
                    scale: None,
                },
                Transform::Trs {
                    translate: t2,
                    rotate: r2,
                    scale: s2,
                },
            ) => Transform::Trs {
                translate: t1 + t2,
                rotate: r2,
                scale: s2,
            },
            (_, _) => Transform::Matrix(self.matrix() * rhs.matrix()),
        }
    }
}

pub enum GltfAction {
    Keep,
    Skip,
    Split,
}

struct GltfTranslator<'a> {
    world: &'a mut World,
    id_stack: Vec<EntityId>,
    mesh_stack: Vec<mesh::Mesh>,
    material_cache: HashMap<Option<json::MaterialId>, Rc<mesh::Material>>,
}

fn translate_material(
    cache: &mut HashMap<Option<json::MaterialId>, Rc<mesh::Material>>,
    material: &Material,
) -> Rc<mesh::Material> {
    cache
        .entry(material.id)
        .or_insert_with(|| {
            Rc::new(mesh::Material {
                texture: material.texture.clone(),
            })
        })
        .clone()
}

impl GltfTranslator<'_> {
    fn add_mesh(&mut self, node: &Node, transform: Transform) {
        if let Some(mesh) = &node.mesh {
            let out_mesh = self.mesh_stack.last_mut().unwrap();
            let matrix = transform.matrix();
            for prim in &mesh.primitives {
                let mat_idx = translate_material(&mut self.material_cache, &prim.material);
                let index_start = out_mesh.vertices.len();
                out_mesh
                    .vertices
                    .extend(prim.position.iter().map(|v| matrix * *v));
                if let Some(texcoord) = prim.texcoord.get(prim.material.texcoord_idx) {
                    out_mesh.uv.extend(texcoord);
                } else {
                    out_mesh
                        .uv
                        .extend(std::iter::repeat(Vec2::default()).take(prim.position.len()));
                }
                out_mesh
                    .color
                    .extend(std::iter::repeat(prim.material.color).take(prim.position.len()));
                let tri_indices_start = out_mesh.triangle_indices.len();
                out_mesh.triangle_indices.extend(
                    prim.indices
                        .iter()
                        .map(|&i| index_start + i as usize)
                        .tuples()
                        .map(|(i, j, k)| [i, j, k]),
                );
                out_mesh.material_ranges.push((mat_idx, tri_indices_start..out_mesh.triangle_indices.len()));
            }
        }
    }
    fn push_entity(&mut self, node: &Node, transform: Transform) {
        let Transform::Trs {
            translate,
            rotate,
            scale,
        } = transform * node.transform
        else {
            panic!("matrix while translating from gltf");
        };
        let id = self.world.new_entity();
        self.world.set(
            id,
            entity::Transform {
                local_position: translate.into(),
                local_rotation: rotate.unwrap_or([0.0, 0.0, 0.0, 1.0].into()).into(),
                local_scale: scale.unwrap_or([1.0, 1.0, 1.0].into()).into(),
                local_to_world: Matrix::IDENTITY,
                parent: self.id_stack.last().copied(),
            },
        );
        self.mesh_stack.push(Default::default());
        self.id_stack.push(id);
    }
    fn pop_entity(&mut self) -> EntityId {
        let id = self.id_stack.pop().unwrap();
        let mesh = Rc::new(self.mesh_stack.pop().unwrap());
        self.world.set(id, mesh);
        id
    }
    fn add_to_entity(
        &mut self,
        node: &Node,
        transform: Transform,
        fun: &impl Fn(&Node) -> GltfAction,
    ) {
        let action = if node.used_by_animation.get() {
            GltfAction::Split
        } else {
            fun(node)
        };
        match action {
            GltfAction::Keep => {
                let new_transform = transform * node.transform;
                self.add_mesh(node, new_transform);
                for child in &node.children {
                    self.add_to_entity(child, new_transform, fun);
                }
            }
            GltfAction::Skip => {}
            GltfAction::Split => {
                self.push_entity(node, transform);
                self.add_mesh(node, Transform::default());
                for child in &node.children {
                    self.add_to_entity(child, Transform::default(), fun);
                }
                self.pop_entity();
            }
        }
    }
}

impl Node {
    pub fn add_to_world(&self, world: &mut World, fun: impl Fn(&Node) -> GltfAction) -> EntityId {
        let mut translator = GltfTranslator {
            world,
            id_stack: vec![],
            mesh_stack: vec![],
            material_cache: HashMap::new(),
        };
        translator.push_entity(self, Transform::default());
        translator.add_to_entity(self, Transform::default(), &fun);
        translator.pop_entity()
    }
}
