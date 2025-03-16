use crate::{
    animation::{Sampler, SamplerMode},
    assets::{AssetLoader, AssetLoaderError},
    geometry::{Matrix, Vec2, Vec3, Vec4},
    mesh::{self, Color, GameObject, Texture, TextureState},
};
use binary::Accessor;
use itertools::Itertools;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io::Read,
    rc::{Rc, Weak},
};
use thiserror::Error;

mod binary;
mod json;

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
    color: Vec<Vec4>,
}

pub struct Mesh {
    primitives: Vec<Primitive>,
}

pub struct Node {
    pub name: Option<String>,
    mesh: Option<Rc<Mesh>>,
    parent: Option<Weak<Node>>,
    children: Vec<Rc<Node>>,
    transform: Transform,
    game_object: RefCell<Option<Rc<GameObject>>>,
    used_by_animation: Cell<bool>,
}

#[derive(Clone, Copy, Debug)]
pub enum Transform {
    Matrix(Matrix),
    Trs {
        translate: Vec3,
        rotate: Option<Vec4>,
        scale: Option<Vec3>,
    },
}

#[derive(Clone)]
pub struct Animation {
    name: Option<String>,
    channels: Vec<(usize, Rc<Node>, json::AnimationPath)>,
    data: Vec<AnimationData>,
}

#[derive(Clone, Debug)]
pub struct AnimationData {
    input: Vec<f64>,
    interpolation: json::AnimationInterpolation,
    output: AnimationOutput,
}

#[derive(Clone, Debug)]
pub enum AnimationOutput {
    Scalar(Vec<f64>),
    Vec3(Vec<Vec3>),
    Vec4(Vec<Vec4>),
}

pub struct GltfImporter<'a> {
    json: json::Root,
    loader: &'a mut AssetLoader,
    file_name: Option<String>,
    buffers: RefCell<HashMap<json::BufferId, Rc<Vec<u8>>>>,
    textures: RefCell<HashMap<json::TextureId, Rc<Texture>>>,
    materials: RefCell<HashMap<json::MaterialId, Rc<Material>>>,
    meshes: RefCell<HashMap<json::MeshId, Rc<Mesh>>>,
    nodes: RefCell<HashMap<json::NodeId, Rc<Node>>>,
    animations: RefCell<HashMap<json::AnimationId, Rc<Animation>>>,
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

fn get_or_insert<K: Eq + std::hash::Hash, V: Clone>(
    map: &RefCell<HashMap<K, V>>,
    key: K,
    fun: impl FnOnce() -> Result<V, Error>,
) -> Result<V, Error> {
    if let Some(value) = map.borrow().get(&key) {
        Ok(value.clone())
    } else {
        let value = fun()?;
        map.borrow_mut().insert(key, value.clone());
        Ok(value)
    }
}

fn new_cyclic_fallible<T, E>(
    create_fn: impl FnOnce(&Weak<T>) -> Result<T, E>,
    any_value: impl FnOnce() -> T,
) -> Result<Rc<T>, E> {
    let mut error = None;
    let value = Rc::new_cyclic(|weak| match create_fn(weak) {
        Ok(value) => value,
        Err(err) => {
            error = Some(err);
            any_value()
        }
    });
    error.map_or(Ok(value), Err)
}

fn garbage_node() -> Node {
    Node {
        name: None,
        mesh: None,
        parent: None,
        children: vec![],
        transform: Transform::default(),
        used_by_animation: false.into(),
        game_object: None.into(),
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
    pub fn from_file(file_name: String, loader: &'a mut AssetLoader) -> Result<Self, Error> {
        let file = loader.open_file(&file_name)?;
        Self::from_reader(file, loader, Some(file_name))
    }
    fn buffer(&self, id: json::BufferId) -> Result<Rc<Vec<u8>>, Error> {
        get_or_insert(&self.buffers, id, || {
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
    fn node(&self, id: json::NodeId, parent: Option<Weak<Node>>) -> Result<Rc<Node>, Error> {
        get_or_insert(&self.nodes, id, || {
            new_cyclic_fallible(
                |weak_self| {
                    let node = self.json.node(id)?;
                    let mesh = node.mesh.map(|id| self.mesh(id)).transpose()?;
                    let transform = node.transform();
                    let children = node
                        .children
                        .iter()
                        .map(|n| self.node(*n, Some(weak_self.clone())))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(Node {
                        name: node.name.clone(),
                        mesh,
                        parent,
                        children,
                        transform,
                        game_object: None.into(),
                        used_by_animation: false.into(),
                    })
                },
                garbage_node,
            )
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
            name: None,
            children: nodes,
            mesh: None,
            parent: None,
            game_object: None.into(),
            transform: Transform::default(),
            used_by_animation: false.into(),
        })
    }
    pub fn root_scene(&self) -> Result<Option<Node>, Error> {
        self.json.scene.map(|id| self.scene(id)).transpose()
    }
    fn animation_output_accessor(&self, id: json::AccessorId) -> Result<AnimationOutput, Error> {
        let accessor = self.json.accessor(id)?;
        Ok(match accessor.type_ {
            json::AccessorType::SCALAR => AnimationOutput::Scalar(self.accessor(id)?),
            json::AccessorType::VEC3 => AnimationOutput::Vec3(self.accessor(id)?),
            json::AccessorType::VEC4 => AnimationOutput::Vec4(self.accessor(id)?),
            _ => gltf_abort!(),
        })
    }
    fn validate_animation_channel(
        &self,
        sampler: usize,
        data: &[AnimationData],
        path: json::AnimationPath,
    ) -> Result<(), Error> {
        let data = gltf_unwrap!(data.get(sampler));
        Ok(match (path, &data.output) {
            (json::AnimationPath::Translation, AnimationOutput::Vec3(_)) => {}
            (json::AnimationPath::Rotation, AnimationOutput::Vec4(_)) => {}
            (json::AnimationPath::Scale, AnimationOutput::Vec3(_)) => {}
            (json::AnimationPath::Weights, AnimationOutput::Scalar(_)) => {}
            _ => gltf_abort!(),
        })
    }
    pub fn animation(&self, id: json::AnimationId) -> Result<Rc<Animation>, Error> {
        get_or_insert(&self.animations, id, || {
            let animation = self.json.animation(id)?;
            let data = animation
                .samplers
                .iter()
                .map(|s| {
                    let input = self.accessor(s.input)?;
                    let output = self.animation_output_accessor(s.output)?;
                    Ok(AnimationData {
                        input,
                        interpolation: s.interpolation,
                        output,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let channels = animation
                .channels
                .iter()
                .map(|c| {
                    let node = self
                        .nodes
                        .borrow()
                        .get(&gltf_unwrap!(c.target.node))
                        .expect("called .animation before .node")
                        .clone();
                    node.used_by_animation.set(true);
                    self.validate_animation_channel(c.sampler, &data, c.target.path)?;
                    Ok((c.sampler, node, c.target.path))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Ok(Rc::new(Animation {
                name: animation.name.clone(),
                channels,
                data,
            }))
        })
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
    fn matrix(&self) -> Matrix {
        match *self {
            Transform::Matrix(matrix) => matrix,
            Transform::Trs {
                translate,
                rotate,
                scale,
            } => {
                let mut matrix = rotate.map_or(Matrix::IDENTITY, Matrix::rotate_quaternion);
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

fn translate_material(
    material: &Material,
    mesh: &mut mesh::Mesh,
    cache: &mut HashMap<Option<json::MaterialId>, usize>,
) -> usize {
    *cache.entry(material.id).or_insert_with(|| {
        let idx = mesh.materials.len();
        mesh.triangles.push(vec![]);
        mesh.materials.push(Rc::new(mesh::Material {
            texture: material.texture.clone(),
        }));
        idx
    })
}

pub enum GltfAction {
    Keep,
    Skip,
    Split,
}

impl Node {
    fn add_mesh_to_game_object(
        &self,
        game_object: &mut mesh::GameObject,
        cache: &mut HashMap<Option<json::MaterialId>, usize>,
        transform: Transform,
    ) {
        if let Some(mesh) = &self.mesh {
            let out_mesh = game_object.mesh.get_or_insert_default();
            let matrix = transform.matrix();
            for prim in &mesh.primitives {
                let mat_idx = translate_material(&prim.material, out_mesh, cache);
                for (i0, i1, i2) in prim.indices.iter().map(|&i| i as usize).tuples() {
                    let vertices = [i0, i1, i2].map(|i| matrix * prim.position[i]);
                    let uv = [i0, i1, i2].map(|i| {
                        prim.texcoord
                            .get(prim.material.texcoord_idx)
                            .map_or(Vec2::default(), |a| a[i])
                    });
                    out_mesh.triangles[mat_idx].push(mesh::Triangle {
                        vertices,
                        uv,
                        color: [prim.material.color; 3],
                    });
                }
            }
        }
    }
    fn new_game_object(&self, transform: Transform) -> mesh::GameObject {
        let Transform::Trs {
            translate,
            rotate,
            scale,
        } = transform * self.transform
        else {
            panic!("matrix in new_game_object");
        };
        GameObject {
            mesh: None,
            name: None,
            position: translate.into(),
            rotation: rotate.unwrap_or([0.0, 0.0, 0.0, 1.0].into()).into(),
            scale: scale.unwrap_or([1.0, 1.0, 1.0].into()).into(),
            children: vec![].into(),
            update_fn: RefCell::new(None),
        }
    }
    fn add_to_game_object(
        &self,
        game_object: &mut mesh::GameObject,
        cache: &mut HashMap<Option<json::MaterialId>, usize>,
        transform: Transform,
        fun: &impl Fn(&Node) -> GltfAction,
    ) {
        let action = if self.used_by_animation.get() {
            GltfAction::Split
        } else {
            fun(self)
        };
        match action {
            GltfAction::Keep => {
                let new_transform = transform * self.transform;
                self.add_mesh_to_game_object(game_object, cache, new_transform);
                for child in &self.children {
                    child.add_to_game_object(game_object, cache, new_transform, fun);
                }
            }
            GltfAction::Skip => {}
            GltfAction::Split => {
                let mut new_object = self.new_game_object(transform);
                let mut new_cache = HashMap::new();
                self.add_mesh_to_game_object(&mut new_object, &mut new_cache, Transform::default());
                for child in &self.children {
                    child.add_to_game_object(
                        &mut new_object,
                        &mut new_cache,
                        Transform::default(),
                        fun,
                    );
                }
                let new_object = Rc::new(new_object);
                self.game_object.replace(Some(new_object.clone()));
                game_object.children.borrow_mut().push(new_object);
            }
        }
    }
    pub fn to_game_object(&self, fun: impl Fn(&Node) -> GltfAction) -> mesh::GameObject {
        let mut game_object = self.new_game_object(Transform::default());
        let mut cache = HashMap::new();
        self.add_to_game_object(&mut game_object, &mut cache, Transform::default(), &fun);
        game_object
    }
}

impl TryInto<Vec<f64>> for AnimationOutput {
    type Error = Error;
    fn try_into(self) -> Result<Vec<f64>, Self::Error> {
        match self {
            AnimationOutput::Scalar(vec) => Ok(vec),
            _ => gltf_abort!()
        }
    }
}

impl TryInto<Vec<Vec3>> for AnimationOutput {
    type Error = Error;
    fn try_into(self) -> Result<Vec<Vec3>, Self::Error> {
        match self {
            AnimationOutput::Vec3(vec) => Ok(vec),
            _ => gltf_abort!()
        }
    }
}

impl TryInto<Vec<Vec4>> for AnimationOutput {
    type Error = Error;
    fn try_into(self) -> Result<Vec<Vec4>, Self::Error> {
        match self {
            AnimationOutput::Vec4(vec) => Ok(vec),
            _ => gltf_abort!()
        }
    }
}

impl AnimationData {
    pub fn to_sampler<T>(&self) -> Sampler<T>  where AnimationOutput: TryInto<Vec<T>> {
        let mode = match self.interpolation {
            json::AnimationInterpolation::LINEAR => SamplerMode::Linear,
            json::AnimationInterpolation::STEP => SamplerMode::Step,
            json::AnimationInterpolation::CUBICSPLINE => todo!(),
        };
        Sampler {
            mode,
            keyframes: self.input.clone(),
            samples: self.output.clone().try_into().unwrap_or_else(|_| unreachable!()),
            time: 0.0,
            index: 0
        }
    }
}

impl Animation {
    pub fn to_game_object(&self) -> mesh::GameObject {
        let mut update_fns: Vec<Box<dyn FnMut(f64)>> = Vec::new();
        for (sampler_idx, node, path) in &self.channels {
            let data = &self.data[*sampler_idx];
            let go = node.game_object.borrow().as_ref().unwrap().clone();
            match path {
                json::AnimationPath::Translation => {
                    let mut sampler = data.to_sampler();
                    update_fns.push(Box::new(move |delay| {
                        *go.position.borrow_mut() = sampler.sample();
                        sampler.advance(delay);
                    }));
                }
                json::AnimationPath::Rotation => {
                    let mut sampler = data.to_sampler();
                    update_fns.push(Box::new(move |delay| {
                        *go.rotation.borrow_mut() = sampler.sample();
                        sampler.advance(delay);
                    }));
                }
                json::AnimationPath::Scale => {
                    let mut sampler = data.to_sampler();
                    update_fns.push(Box::new(move |delay| {
                        *go.scale.borrow_mut() = sampler.sample();
                        sampler.advance(delay);
                    }));
                }
                _ => {
                    eprintln!("unsupported animation: {path:?}");
                }
            }
        }
        GameObject {
            mesh: None,
            name: None,
            position: Vec3::from([0.0, 0.0, 0.0]).into(),
            rotation: Vec4::from([0.0, 0.0, 0.0, 1.0]).into(),
            scale: Vec3::from([1.0, 1.0, 1.0]).into(),
            children: Default::default(),
            update_fn: RefCell::new(Some(Box::new(move |_, delta| {
                update_fns.iter_mut().for_each(|f| f(delta));
            }))),
        }
    }
}