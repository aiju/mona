use std::{f64::consts::PI, rc::Rc};

use rand::Rng;

use crate::{
    assets::AssetLoader,
    collision::{Aabb, Bvh, CapsuleCollider},
    entity::{Camera, EntityId, Transform, World},
    geometry::{Matrix, Quaternion, Vec2, Vec3},
    gltf::GltfImporter,
    input::{InputEvent, InputState, Key},
    mesh::{Color, Material, Mesh, Texture},
    render::{Backend, Context, HEIGHT, TextureId, Triangle4, WIDTH},
    *,
};

mod cat_room;
mod tetris;

#[allow(unused_variables)]
pub trait Scene<B: Backend> {
    fn input(&mut self, event: InputEvent) {}
    fn render(&mut self, context: &mut Context<B>);
    fn update(&mut self, delta: f64, input: &InputState) {}
}

pub fn create<B: Backend>(
    spec: &str,
    context: &mut Context<B>,
    loader: &mut AssetLoader,
) -> Option<Box<dyn Scene<B>>> {
    let (name, arg) = match spec.split_once(':') {
        Some((a, b)) => (a, Some(b)),
        None => (spec, None),
    };
    match (name, arg) {
        ("Cube", None) => Some(Box::new(Cube::new(context, loader))),
        ("CatRoom", None) => Some(Box::new(CatRoom::new(context, loader))),
        ("Gltf", Some(path)) => Some(Box::new(GltfScene::new(context, loader, path))),
        ("Sphere", None) => Some(Box::new(Sphere::default())),
        ("Tetris", None) => Some(Box::new(tetris::Tetris::new(context, loader))),
        _ => None,
    }
}

pub const CUBE: &'static [[[f64; 5]; 3]] = &[
    [
        [-1.0, -1.0, -1.0, 0.0, 0.0],
        [-1.0, -1.0, 1.0, 0.0, 1.0],
        [-1.0, 1.0, 1.0, 1.0, 1.0],
    ],
    [
        [-1.0, -1.0, -1.0, 0.0, 0.0],
        [-1.0, 1.0, 1.0, 1.0, 1.0],
        [-1.0, 1.0, -1.0, 1.0, 0.0],
    ],
    [
        [1.0, 1.0, 1.0, 1.0, 1.0],
        [1.0, -1.0, 1.0, 0.0, 1.0],
        [1.0, -1.0, -1.0, 0.0, 0.0],
    ],
    [
        [1.0, 1.0, 1.0, 1.0, 1.0],
        [1.0, -1.0, -1.0, 0.0, 0.0],
        [1.0, 1.0, -1.0, 1.0, 0.0],
    ],
    [
        [1.0, 1.0, 1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0, 0.0, 1.0],
        [-1.0, 1.0, -1.0, 0.0, 0.0],
    ],
    [
        [1.0, 1.0, 1.0, 1.0, 1.0],
        [-1.0, 1.0, -1.0, 0.0, 0.0],
        [1.0, 1.0, -1.0, 1.0, 0.0],
    ],
    [
        [1.0, -1.0, 1.0, 1.0, 1.0],
        [-1.0, -1.0, 1.0, 0.0, 1.0],
        [-1.0, -1.0, -1.0, 0.0, 0.0],
    ],
    [
        [1.0, -1.0, 1.0, 1.0, 1.0],
        [-1.0, -1.0, -1.0, 0.0, 0.0],
        [1.0, -1.0, -1.0, 1.0, 0.0],
    ],
    [
        [-1.0, -1.0, -1.0, 0.0, 0.0],
        [1.0, -1.0, -1.0, 1.0, 0.0],
        [1.0, 1.0, -1.0, 1.0, 1.0],
    ],
    [
        [-1.0, -1.0, -1.0, 0.0, 0.0],
        [1.0, 1.0, -1.0, 1.0, 1.0],
        [-1.0, 1.0, -1.0, 0.0, 1.0],
    ],
    [
        [-1.0, -1.0, 1.0, 0.0, 0.0],
        [1.0, -1.0, 1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0, 1.0, 1.0],
    ],
    [
        [-1.0, -1.0, 1.0, 0.0, 0.0],
        [1.0, 1.0, 1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0, 0.0, 1.0],
    ],
];

fn cube_mesh() -> Rc<Mesh> {
    let material = Rc::new(Material { texture: None });
    Rc::new(Mesh {
        vertices: [1.0, -1.0]
            .into_iter()
            .flat_map(|x| {
                [1.0, -1.0]
                    .into_iter()
                    .flat_map(move |y| [1.0, -1.0].map(|z| [x, y, z].into()))
            })
            .collect(),
        uv: vec![Vec2::default(); 8],
        color: vec![Color::WHITE; 8],
        triangle_indices: vec![
            [0, 4, 6],
            [0, 6, 2],
            [1, 3, 7],
            [1, 7, 5],
            [0, 1, 5],
            [0, 5, 4],
            [2, 6, 7],
            [2, 7, 3],
            [0, 2, 3],
            [0, 3, 1],
            [4, 5, 7],
            [4, 7, 6],
        ],
        material_ranges: vec![(material, 0..12)],
    })
}

pub struct Cube {
    texture: TextureId,
    time: f64,
}

impl Cube {
    fn new<B: Backend>(context: &mut Context<B>, loader: &mut AssetLoader) -> Cube {
        let image = Texture::from_file("cat", None);
        image.load_backend(context, loader);
        let texture = image.texture_id();
        Cube { texture, time: 0.0 }
    }
}

impl<B: Backend> Scene<B> for Cube {
    fn render(&mut self, context: &mut Context<B>) {
        let matrix = Matrix::projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0)
            * Matrix::translate(0.0, -0.0, 3.0)
            * Matrix::rotate(30.0 * self.time, [0.0, 1.0, 0.0]);
        let v: Vec<_> = CUBE
            .iter()
            .map(|v| Triangle4::new(*v))
            .map(move |p| p.transform(matrix))
            .collect();
        context.draw().textured(self.texture).run(&v);
    }
    fn update(&mut self, delta: f64, _input: &InputState) {
        self.time += delta;
    }
}

pub struct CatRoom {
    texture: TextureId,
    time: f64,
}

impl CatRoom {
    fn new<B: Backend>(context: &mut Context<B>, loader: &mut AssetLoader) -> CatRoom {
        let image = Texture::from_file("cat", None);
        image.load_backend(context, loader);
        let texture = image.texture_id();
        CatRoom { texture, time: 0.0 }
    }
}

impl<B: Backend> Scene<B> for CatRoom {
    fn render(&mut self, context: &mut Context<B>) {
        let matrix = Matrix::projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0)
            * Matrix::translate(0.0, -1.0, 3.0)
            * Matrix::rotate(-20.0, [1.0, 0.0, 0.0])
            * Matrix::rotate(30.0 * self.time, [0.0, 1.0, 0.0]);
        let v: Vec<_> = cat_room::CAT_ROOM
            .iter()
            .map(|v| Triangle4::new(*v))
            .map(move |p| p.transform(matrix))
            .collect();
        context.draw().textured(self.texture).run(&v);
    }
    fn update(&mut self, delta: f64, _input: &InputState) {
        self.time += delta;
    }
}

pub struct GltfScene {
    world: World,
    player: EntityId,
    camera_pivot: EntityId,
    camera: EntityId,
    time: f64,
}

fn randomize_mesh_colors(mesh: &Mesh) -> Mesh {
    let mut out_mesh = Mesh::default();
    let mut rng = rand::rng();
    for [i0, i1, i2] in mesh.triangle_indices.iter().cloned() {
        out_mesh.triangle_indices.push([out_mesh.vertices.len(), out_mesh.vertices.len() + 1, out_mesh.vertices.len() + 2]);
        out_mesh.vertices.extend([mesh.vertices[i0], mesh.vertices[i1], mesh.vertices[i2]]);
        out_mesh.uv.extend([Vec2::default(); 3]);
        let color = Color::from([rng.random(), rng.random(), rng.random()]);
        out_mesh.color.extend([color; 3]);
    }
    out_mesh.material_ranges.push((Rc::new(Material::default()), 0..out_mesh.triangle_indices.len()));
    out_mesh
}

impl GltfScene {
    fn new<B: Backend>(context: &mut Context<B>, loader: &mut AssetLoader, path: &str) -> Self {
        let file = loader.open_file(path).unwrap();
        let importer = GltfImporter::from_reader(file, loader, Some(path.to_string())).unwrap();
        let scene = importer.root_scene().unwrap().unwrap();
        let mut world = World::new();
        world.register::<Transform>(Default::default());
        world.register::<Rc<Mesh>>(Default::default());
        world.register::<Bvh<usize>>(Default::default());
        world.register::<Camera>(Default::default());
        scene.add_to_world(&mut world, |_| gltf::GltfAction::Keep);
        world.load(context, loader);
        println!("building bvh");
        let ids = world.iter::<Rc<Mesh>>().map(|x| x.0).collect::<Vec<_>>();
        for id in ids {
            world.set(id, Bvh::from_mesh(world.get::<Rc<Mesh>>(id)));
        }
        println!("done");
        let player = world.new_entity();
        world.set(
            player,
            Transform {
                local_position: Vec3::from([0.0, 2.0, -5.0]),
                local_rotation: Quaternion::default(),
                local_scale: Vec3::from([1.0, 1.0, 1.0]),
                local_to_world: Matrix::IDENTITY,
                parent: None,
            },
        );
        let camera_pivot = world.new_entity();
        world.set(
            camera_pivot,
            Transform {
                local_position: Vec3::from([0.0, 0.0, 0.0]),
                local_rotation: Quaternion::default(),
                local_scale: Vec3::from([1.0, 1.0, 1.0]),
                local_to_world: Matrix::IDENTITY,
                parent: Some(player),
            },
        );
        let camera = world.new_entity();
        world.set(
            camera,
            Transform {
                local_position: Vec3::from([0.0, 0.0, -2.0]),
                local_rotation: Quaternion::default(),
                local_scale: Vec3::from([1.0, 1.0, 1.0]),
                local_to_world: Matrix::IDENTITY,
                parent: Some(camera_pivot),
            },
        );
        world.set(camera, Camera { fov_angle: 90.0 });
        world.update_transforms();
        Self {
            world,
            player,
            camera,
            camera_pivot,
            time: 0.0,
        }
    }
}

fn render_aabb<B: Backend>(context: &mut Context<B>, aabb: &Aabb, view: Matrix) {
    let a = (aabb.max + aabb.min) * 0.5;
    let b = (aabb.max - aabb.min) * 0.5;
    let matrix = Matrix::translate(a.x, a.y, a.z) * Matrix::scale(b.x, b.y, b.z);
    let v: Vec<_> = CUBE
        .iter()
        .map(|v| Triangle4::new(*v))
        .map(move |p| p.transform(view * matrix))
        .collect();
    context.draw().run(&v);
}

impl<B: Backend> Scene<B> for GltfScene {
    fn render(&mut self, context: &mut Context<B>) {
        let Vec3 { x, y, z } = self.world.get::<Transform>(self.player).local_position;
        CapsuleCollider {
            base: [0.0, -1.0, 0.0].into(),
            tip: [0.0, 0.0, 0.0].into(),
            radius: 0.25,
        }
        .translate([x, y, z].into())
        .debug_render(
            context,
            self.world
                .get::<Camera>(self.camera)
                .view_matrix(self.world.get(self.camera)),
        );
        self.world.render(context, self.camera);
    }
    fn update(&mut self, delta: f64, input: &InputState) {
        let rot_x = (input.mouse_x() as f64) / 10.0;
        let rot_y = (input.mouse_y() as f64) / 10.0;
        let input_vector: Vec2 = [
            (input.is_key_down(Key::KeyD) as u32 as f64)
                - (input.is_key_down(Key::KeyA) as u32 as f64),
            (input.is_key_down(Key::KeyW) as u32 as f64)
                - (input.is_key_down(Key::KeyS) as u32 as f64),
        ]
        .into();
        let delta_position = input_vector.rotate(-rot_x);
        let Vec3 { x, y, z } = self.world.get::<Transform>(self.player).local_position;
        let mut new_x = x + delta_position.x * delta * 10.0;
        let mut new_z = z + delta_position.y * delta * 10.0;
        let mut new_y = y
            + ((input.is_key_down(Key::KeyE) as u32 as f64)
                - (input.is_key_down(Key::KeyQ) as u32 as f64))
                * delta
                * 10.0;

        let collider = CapsuleCollider {
            base: [0.0, -1.0, 0.0].into(),
            tip: [0.0, 0.0, 0.0].into(),
            radius: 0.25,
        }
        .translate([new_x, new_y, new_z].into());

        if self.world.check_collision(&collider).is_none() {
            let transform: &mut Transform = self.world.get_mut(self.player);
            transform.local_position = [new_x, new_y, new_z].into();
        }

        let transform: &mut Transform = self.world.get_mut(self.camera_pivot);
        transform.local_rotation = Quaternion::from_angle(rot_x, [0.0, 1.0, 0.0].into())
            * Quaternion::from_angle(rot_y, [1.0, 0.0, 0.0].into());
        self.world.update_transforms();
        self.time += delta;
    }
}

#[derive(Default)]
pub struct Sphere {
    time: f64,
}

impl<B: Backend> Scene<B> for Sphere {
    fn render(&mut self, context: &mut Context<B>) {
        let view = Matrix::projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0)
            * Matrix::translate(0.0, 0.0, 3.0);
        let theta_steps = 16;
        let phi_steps = 16;
        let coord = |i: i32, j: i32| {
            let theta = PI * (i as f64) / (theta_steps as f64);
            let phi = 2.0 * PI * (j as f64) / (phi_steps as f64);
            let x = phi.cos() * theta.sin();
            let y = phi.sin() * theta.sin();
            let z = theta.cos();
            [x, y, z, 1.0].into()
        };
        let mut tris = Vec::new();
        for i in 0..theta_steps {
            for j in 0..phi_steps {
                tris.push(Triangle4 {
                    vertices: [coord(i, j), coord(i + 1, j), coord(i + 1, j + 1)],
                    uv: [Vec2::default(); 3],
                    color: [Color::WHITE; 3],
                });
                tris.push(Triangle4 {
                    vertices: [coord(i, j), coord(i + 1, j + 1), coord(i, j + 1)],
                    uv: [Vec2::default(); 3],
                    color: [Color::WHITE; 3],
                });
            }
        }
        for t in &mut tris {
            for i in 0..3 {
                let normal = t.vertices[i].xyz();
                let tt = self.time;
                let direction: Vec3 = [tt.cos(), 0.0, -tt.sin()].into();
                let s = (normal * direction).clamp(0.0, 1.0);
                let l = 0.3 + 0.3 * s + 0.3 * s * s * s;
                t.color[i] = [l, l, l].into();
            }
        }
        let v: Vec<_> = tris.iter().map(|p| p.transform(view)).collect();
        context.draw().run(&v);
    }
    fn update(&mut self, delta: f64, _input: &InputState) {
        self.time += delta;
    }
}
