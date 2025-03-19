use std::{any::TypeId, collections::HashMap, rc::Rc};

use itertools::Itertools;

use crate::{
    assets::AssetLoader,
    collision::{Bvh, CapsuleCollider},
    geometry::{Matrix, Vec3, Vec4},
    gltf,
    mesh::Mesh,
    render::{Backend, Context},
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct EntityId(usize);

pub trait AbstractStorage: 'static {
    fn new_id(&mut self, new_id: EntityId);
    fn remove(&mut self, id: EntityId);
}

pub trait Storage<T: 'static>: AbstractStorage {
    fn get(&self, id: EntityId) -> Option<&T>;
    fn get_mut(&mut self, id: EntityId) -> Option<&mut T>;
    fn set(&mut self, id: EntityId, value: T) -> Option<T>;
    fn iter(&self) -> impl Iterator<Item = (EntityId, &T)>;
    fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)>;
}

pub trait Component: Sized + 'static {
    type Storage: Storage<Self>;
}

#[derive(Default)]
pub struct CompIndex {
    storage: HashMap<TypeId, Box<dyn AbstractStorage>>,
}

impl CompIndex {
    pub fn insert<T: AbstractStorage>(&mut self, value: T) -> Option<T> {
        self.storage
            .insert(TypeId::of::<T>(), Box::new(value))
            .map(|x| unsafe { *Box::from_raw(Box::into_raw(x).cast()) })
    }
    pub fn remove<T: AbstractStorage>(&mut self) -> Option<T> {
        self.storage
            .remove(&TypeId::of::<T>())
            .map(|x| unsafe { *Box::from_raw(Box::into_raw(x).cast()) })
    }
    pub fn get<T: AbstractStorage>(&self) -> &T {
        let b = self
            .storage
            .get(&TypeId::of::<T>())
            .expect("use of unregistered component");
        unsafe { &*(&**b as *const dyn AbstractStorage as *const T) }
    }
    pub fn get_mut<T: AbstractStorage>(&mut self) -> &mut T {
        let b = self
            .storage
            .get_mut(&TypeId::of::<T>())
            .expect("use of unregistered component");
        unsafe { &mut *(&mut **b as *mut dyn AbstractStorage as *mut T) }
    }
    pub fn iter(&self) -> impl Iterator<Item = &dyn AbstractStorage> {
        self.storage.values().map(|x| &**x)
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn AbstractStorage> {
        self.storage.values_mut().map(|x| &mut **x)
    }
}

impl<T: 'static> AbstractStorage for Vec<Option<T>> {
    fn new_id(&mut self, new_id: EntityId) {
        debug_assert!(self.len() == new_id.0);
        self.push(None);
    }
    fn remove(&mut self, id: EntityId) {
        self[id.0] = None;
    }
}

impl<T: 'static> Storage<T> for Vec<Option<T>> {
    fn get(&self, id: EntityId) -> Option<&T> {
        self[id.0].as_ref()
    }
    fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        self[id.0].as_mut()
    }
    fn set(&mut self, id: EntityId, value: T) -> Option<T> {
        std::mem::replace(&mut self[id.0], Some(value))
    }
    fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        (**self)
            .iter()
            .enumerate()
            .filter_map(|x| x.1.as_ref().map(|y| (EntityId(x.0), y)))
    }
    fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        (**self)
            .iter_mut()
            .enumerate()
            .filter_map(|x| x.1.as_mut().map(|y| (EntityId(x.0), y)))
    }
}

pub struct World {
    entity_ctr: usize,
    comp_index: CompIndex,
    missing_storage: usize,
}

impl World {
    pub fn new() -> World {
        World {
            entity_ctr: 0,
            comp_index: Default::default(),
            missing_storage: 0,
        }
    }
    pub fn register<T: Component>(&mut self, value: T::Storage) {
        self.comp_index.insert(value);
    }
    pub fn new_entity(&mut self) -> EntityId {
        assert!(self.missing_storage == 0);
        let ret = EntityId(self.entity_ctr);
        self.entity_ctr += 1;
        for storage in self.comp_index.iter_mut() {
            storage.new_id(ret);
        }
        ret
    }
    pub fn delete_entity(&mut self, id: EntityId) {
        assert!(self.missing_storage == 0);
        for storage in self.comp_index.iter_mut() {
            storage.remove(id);
        }
    }
    pub fn storage<T: Component>(&self) -> &T::Storage {
        self.comp_index.get::<T::Storage>()
    }
    pub fn storage_mut<T: Component>(&mut self) -> &mut T::Storage {
        self.comp_index.get_mut::<T::Storage>()
    }
    pub fn get<T: Component>(&self, id: EntityId) -> &T {
        self.comp_index
            .get::<T::Storage>()
            .get(id)
            .expect("tried to get component that doesn't exist for entity")
    }
    pub fn get_mut<T: Component>(&mut self, id: EntityId) -> &mut T {
        self.comp_index
            .get_mut::<T::Storage>()
            .get_mut(id)
            .expect("tried to get component that doesn't exist for entity")
    }
    pub fn set<T: Component>(&mut self, id: EntityId, value: T) -> Option<T> {
        self.comp_index.get_mut::<T::Storage>().set(id, value)
    }
    pub fn iter<T: Component>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.comp_index.get::<T::Storage>().iter()
    }
    pub fn iter_mut<T: Component>(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.comp_index.get_mut::<T::Storage>().iter_mut()
    }
    pub fn iter2<T: Component, U: Component>(&self) -> impl Iterator<Item = (EntityId, &T, &U)> {
        let i1 = self.comp_index.get::<T::Storage>().iter();
        let i2 = self.comp_index.get::<U::Storage>().iter();
        i1.merge_join_by(i2, |a, b| a.0.cmp(&b.0))
            .flat_map(|x| match x {
                itertools::EitherOrBoth::Both((id, a), (_, b)) => Some((id, a, b)),
                itertools::EitherOrBoth::Left(_) => None,
                itertools::EitherOrBoth::Right(_) => None,
            })
    }
    pub fn with_storage<T: Component>(&mut self, fun: impl FnOnce(&mut Self, &mut T::Storage)) {
        self.missing_storage += 1;
        let mut storage = self
            .comp_index
            .remove::<T::Storage>()
            .expect("with_storage on unregistered/removed component");
        fun(self, &mut storage);
        self.comp_index.insert(storage);
        self.missing_storage -= 1;
    }
}

pub struct Transform {
    pub local_position: Vec3,
    pub local_rotation: Vec4,
    pub local_scale: Vec3,
    pub local_to_world: Matrix,
    pub parent: Option<EntityId>,
}

impl Component for Transform {
    type Storage = Vec<Option<Self>>;
}

impl Component for Rc<Mesh> {
    type Storage = Vec<Option<Self>>;
}

impl Component for Bvh<usize> {
    type Storage = Vec<Option<Self>>;
}

impl World {
    pub fn load<B: Backend>(&self, context: &mut Context<B>, loader: &mut AssetLoader) {
        for (_, mesh) in self.iter::<Rc<Mesh>>() {
            mesh.load(context, loader);
        }
    }
    pub fn render<B: Backend>(&self, context: &mut Context<B>, view: Matrix) {
        for (_, transform, mesh) in self.iter2::<Transform, Rc<Mesh>>() {
            for (material, idx_range) in &mesh.material_ranges {
                let v = idx_range
                    .clone()
                    .map(|i| {
                        mesh.triangle4(i)
                            .transform(transform.local_to_world)
                            .lighting(0.5, 0.5, [0.707, 0.0, -0.707].into())
                            .transform(view)
                    })
                    .collect::<Vec<_>>();
                context.draw().opt_textured(material.texture_id()).run(&v);
            }
        }
    }
    pub fn update_transforms(&mut self) {
        let update_transform = |t: &mut Transform, parent_transform: gltf::Transform| {
            t.local_to_world = (parent_transform
                * gltf::Transform::Trs {
                    translate: t.local_position,
                    rotate: Some(t.local_rotation),
                    scale: Some(t.local_scale),
                })
            .matrix();
        };
        // TODO: this is probably way more expensive than it has to be
        let mut work_list = Vec::new();
        let mut children: HashMap<EntityId, Vec<EntityId>> = HashMap::new();
        for (id, transform) in self.iter::<Transform>() {
            if let Some(parent) = transform.parent {
                children.entry(parent).or_default().push(id);
            } else {
                work_list.push(id);
            }
        }
        let mut i = 0;
        while let Some(id) = work_list.get(i) {
            let opt_parent = self.get::<Transform>(*id).parent;
            let parent_transform = opt_parent.map_or(gltf::Transform::default(), |p_id| {
                gltf::Transform::Matrix(self.get::<Transform>(p_id).local_to_world)
            });
            update_transform(self.get_mut(*id), parent_transform);
            if let Some(l) = children.get(id) {
                work_list.extend(l);
            }
            i += 1;
        }
    }
    pub fn check_collision(&self, collider: &CapsuleCollider) -> bool {
        for (idx, transform, bvh) in self.iter2::<Transform, Bvh<usize>>() {
            let mesh = self.get::<Rc<Mesh>>(idx);
            let aabb = collider
                .aabb()
                .transform(transform.local_to_world.inverse_3x4());
            for idx in bvh.aabb_query(&aabb) {
                if collider
                    .intersect_triangle(&mesh.triangle(*idx).transform(transform.local_to_world))
                    .is_some()
                {
                    return true;
                }
            }
        }
        false
    }
}
