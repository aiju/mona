use crate::{
    assets::{AssetLoader, AssetLoaderError},
    geometry::{Matrix, Vec2, Vec3, Vec4},
    mesh::{self, Color},
};
use binary::Accessor;
use itertools::Itertools;
use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    io::Read,
    rc::{Rc, Weak},
};
use thiserror::Error;

mod binary;
mod json;

pub struct Texture {
    id: json::TextureId,
    image: String,
}

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
    color: Vec<Vec4>,
}

pub struct Mesh {
    primitives: Vec<Primitive>,
}

pub struct Node {
    mesh: Option<Rc<Mesh>>,
    parent: Option<Weak<Node>>,
    children: Vec<Rc<Node>>,
    local_matrix: Matrix,
    global_matrix: Matrix,
}

pub struct GltfImporter<L> {
    json: json::Root,
    loader: RefCell<L>,
    file_name: Option<String>,
    buffers: RefCell<HashMap<json::BufferId, Rc<Vec<u8>>>>,
    textures: RefCell<HashMap<json::TextureId, Rc<Texture>>>,
    materials: RefCell<HashMap<json::MaterialId, Rc<Material>>>,
    meshes: RefCell<HashMap<json::MeshId, Rc<Mesh>>>,
    nodes: RefCell<HashMap<json::NodeId, Rc<Node>>>,
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
            None => gltf_abort!()
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

fn get_or_insert<K: Eq + std::hash::Hash, V: Clone>(
    map: &RefCell<HashMap<K, V>>,
    key: K,
    fun: impl FnOnce() -> Result<V, Error>,
) -> Result<V, Error> {
    match map.borrow_mut().entry(key) {
        Entry::Occupied(entry) => Ok(entry.get().clone()),
        Entry::Vacant(entry) => {
            let value = fun()?;
            entry.insert(value.clone());
            Ok(value)
        }
    }
}

impl<L: AssetLoader> GltfImporter<L> {
    fn new(json: json::Root, loader: L, file_name: Option<String>) -> Self {
        GltfImporter {
            json,
            file_name,
            loader: RefCell::new(loader),
            buffers: Default::default(),
            textures: Default::default(),
            materials: Default::default(),
            meshes: Default::default(),
            nodes: Default::default(),
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
        loader: L,
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
                gltf.buffers
                    .borrow_mut()
                    .insert(json::BufferId(0), Rc::new(buf));
            }
            Ok(gltf)
        } else {
            reader.read_to_end(&mut buf)?;
            let json = serde_json::from_slice(&buf)?;
            Ok(Self::new(json, loader, file_name))
        }
    }
    fn buffer(&self, id: json::BufferId) -> Result<Rc<Vec<u8>>, Error> {
        get_or_insert(&self.buffers, id, || {
            let buffer = self.json.buffer(id)?;
            let name = gltf_unwrap!(buffer.uri.as_ref());
            let mut file = self
                .loader
                .borrow_mut()
                .open_file(name, self.file_name.as_ref().map(|x| &**x))?;
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
    fn mesh(&self, id: json::MeshId) -> Result<Rc<Mesh>, Error> {
        get_or_insert(&self.meshes, id, || {
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
                primitives.push(Primitive {
                    material,
                    indices,
                    position,
                    texcoord,
                    color: Vec::new(),
                });
            }
            Ok(Rc::new(Mesh { primitives }))
        })
    }
    fn texture(&self, id: json::TextureId) -> Result<Rc<Texture>, Error> {
        get_or_insert(&self.textures, id, || {
            let texture = self.json.texture(id)?;
            let image = self.json.image(gltf_unwrap!(texture.source))?;
            Ok(Rc::new(Texture {
                id,
                image: gltf_unwrap!(image.uri.clone()),
            }))
        })
    }
    fn material(&self, id: json::MaterialId) -> Result<Rc<Material>, Error> {
        get_or_insert(&self.materials, id, || {
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
    fn node(&self, id: json::NodeId, parent: Option<json::NodeId>) -> Result<Rc<Node>, Error> {
        get_or_insert(&self.nodes, id, || {
            let node = self.json.node(id)?;
            let mesh = node.mesh.map(|id| self.mesh(id)).transpose()?;
            let parent = parent.map(|id| Rc::downgrade(self.nodes.borrow().get(&id).unwrap()));
            let children = node
                .children
                .iter()
                .map(|c| self.node(*c, Some(id)))
                .collect::<Result<Vec<_>, _>>()?;
            let local_matrix = node.local_matrix();
            let global_matrix = parent.clone().map_or(local_matrix, |n| {
                n.upgrade().unwrap().global_matrix * local_matrix
            });
            Ok(Rc::new(Node {
                mesh,
                parent,
                children,
                local_matrix,
                global_matrix,
            }))
        })
    }
    pub fn scene(&self, id: json::SceneId) -> Result<Node, Error> {
        let scene = self.json.scene(id)?;
        let nodes = scene
            .nodes
            .iter()
            .map(|id| self.node(*id, None))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Node {
            children: nodes,
            mesh: None,
            parent: None,
            local_matrix: Matrix::IDENTITY,
            global_matrix: Matrix::IDENTITY,
        })
    }
    pub fn root_scene(&self) -> Result<Option<Node>, Error> {
        self.json.scene.map(|id| self.scene(id)).transpose()
    }
}

impl json::Node {
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

fn translate_material(
    material: &Material,
    mesh: &mut mesh::Mesh,
    cache: &mut HashMap<Option<json::MaterialId>, usize>,
) -> usize {
    *cache.entry(material.id).or_insert_with(|| {
        let idx = mesh.materials.len();
        mesh.triangles.push(vec![]);
        if let Some(t) = &material.texture {
            mesh.materials.push(mesh::Material {
                texture: Some(mesh.textures.len()),
            });
            mesh.textures.push(t.image.clone());
        } else {
            mesh.materials.push(mesh::Material { texture: None });
        }
        idx
    })
}

impl Node {
    fn walk(&self, fun: &mut impl FnMut(&Node)) {
        fun(self);
        for child in &self.children {
            child.walk(fun);
        }
    }
    pub fn to_mesh(&self) -> mesh::Mesh {
        let mut mesh_mesh = mesh::Mesh {
            triangles: vec![vec![]],
            materials: vec![mesh::Material::default()],
            textures: vec![],
        };
        let mut materials = HashMap::new();
        self.walk(&mut |node| {
            if let Some(mesh) = &node.mesh {
                for prim in &mesh.primitives {
                    let mat_idx =
                        translate_material(&prim.material, &mut mesh_mesh, &mut materials);
                    for (i0, i1, i2) in prim.indices.iter().map(|&i| i as usize).tuples() {
                        let vertices = [i0, i1, i2].map(|i| node.global_matrix * prim.position[i]);
                        let uv = [i0, i1, i2].map(|i| {
                            prim.texcoord
                                .get(prim.material.texcoord_idx)
                                .map_or(Vec2::default(), |a| a[i])
                        });
                        mesh_mesh.triangles[mat_idx].push(mesh::Triangle {
                            vertices,
                            uv,
                            color: [prim.material.color; 3],
                        });
                    }
                }
            }
        });
        mesh_mesh
    }
}
