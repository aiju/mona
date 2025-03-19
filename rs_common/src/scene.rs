use std::rc::Rc;

use crate::{
    assets::AssetLoader,
    collision::{Aabb, Bvh, CapsuleCollider},
    entity::{Game, Transform},
    geometry::Matrix,
    gltf::GltfImporter,
    input::{InputEvent, InputState, Key},
    mesh::{Mesh, Texture},
    render::{Backend, Context, TextureId},
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
        ("Obj", Some(path)) => Some(Box::new(ObjScene::new(context, loader, path))),
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
            .map(|v| BarePrimitive::new(*v))
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
            .map(|v| BarePrimitive::new(*v))
            .map(move |p| p.transform(matrix))
            .collect();
        context.draw().textured(self.texture).run(&v);
    }
    fn update(&mut self, delta: f64, _input: &InputState) {
        self.time += delta;
    }
}

pub struct ObjScene {
    model: Mesh,
    time: f64,
    x: f64,
    y: f64,
    z: f64,
    rot_x: f64,
    rot_y: f64,
}

impl ObjScene {
    fn new<B: Backend>(context: &mut Context<B>, loader: &mut AssetLoader, path: &str) -> Self {
        let model = obj_loader::load_obj_file(path);
        model.load(context, loader);
        Self {
            model,
            time: Default::default(),
            rot_x: 0.0,
            rot_y: 0.0,
            x: 0.0,
            y: 2.0,
            z: -5.0,
        }
    }
}

impl<B: Backend> Scene<B> for ObjScene {
    fn render(&mut self, context: &mut Context<B>) {
        let object = Matrix::IDENTITY;
        let view = Matrix::projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0)
            * Matrix::rotate(-self.rot_y, [1.0, 0.0, 0.0])
            * Matrix::rotate(-self.rot_x, [0.0, 1.0, 0.0])
            * Matrix::translate(-self.x, -self.y, -self.z);
        for (triangles, material) in self.model.triangles.iter().zip(&self.model.materials) {
            let v: Vec<_> = triangles
                .iter()
                .map(|t| {
                    BarePrimitive {
                        vertices: t.vertices.map(From::from),
                        uv: t.uv,
                        color: t.color,
                    }
                    .transform(object)
                    .lighting(0.5, 0.5, [0.707, 0.0, -0.707].into())
                    .transform(view)
                })
                .collect();
            context.draw().opt_textured(material.texture_id()).run(&v);
        }
    }
    fn update(&mut self, delta: f64, input: &InputState) {
        self.rot_x = (input.mouse_x() as f64) / 10.0;
        self.rot_y = (input.mouse_y() as f64) / 10.0;
        let input_vector: Vec2 = [
            (input.is_key_down(Key::KeyD) as u32 as f64)
                - (input.is_key_down(Key::KeyA) as u32 as f64),
            (input.is_key_down(Key::KeyW) as u32 as f64)
                - (input.is_key_down(Key::KeyS) as u32 as f64),
        ]
        .into();
        let delta_position = input_vector.rotate(-self.rot_x);
        self.x += delta_position.x * delta * 10.0;
        self.z += delta_position.y * delta * 10.0;
        self.y += ((input.is_key_down(Key::KeyE) as u32 as f64)
            - (input.is_key_down(Key::KeyQ) as u32 as f64))
            * delta
            * 10.0;
        self.time += delta;
    }
}

pub struct GltfScene {
    game: Game,
    time: f64,
    x: f64,
    y: f64,
    z: f64,
    rot_x: f64,
    rot_y: f64,
    visible: bool,
}

impl GltfScene {
    fn new<B: Backend>(context: &mut Context<B>, loader: &mut AssetLoader, path: &str) -> Self {
        let file = loader.open_file(path).unwrap();
        let importer = GltfImporter::from_reader(file, loader, Some(path.to_string())).unwrap();
        let scene = importer.root_scene().unwrap().unwrap();
        let mut game = Game::new();
        game.register::<Transform>(Default::default());
        game.register::<Rc<Mesh>>(Default::default());
        game.register::<Bvh<mesh::Triangle>>(Default::default());
        scene.add_to_game(&mut game, |_| gltf::GltfAction::Split);
        game.load(context, loader);
        println!("building bvh");
        let ids = game.iter::<Rc<Mesh>>().map(|x| x.0).collect::<Vec<_>>();
        for id in ids {
            let bvh = Bvh::from_primitives(
                game.get::<Rc<Mesh>>(id)
                    .triangles
                    .iter()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            game.set(id, bvh);
        }
        println!("done");
        game.update_transforms();
        Self {
            game,
            time: Default::default(),
            rot_x: 0.0,
            rot_y: 0.0,
            x: 0.0,
            y: 2.0,
            z: -5.0,
            visible: false,
        }
    }
}

fn render_aabb<B: Backend>(context: &mut Context<B>, aabb: &Aabb, view: Matrix) {
    let a = (aabb.max + aabb.min) * 0.5;
    let b = (aabb.max - aabb.min) * 0.5;
    let matrix = Matrix::translate(a.x, a.y, a.z) * Matrix::scale(b.x, b.y, b.z);
    let v: Vec<_> = CUBE
        .iter()
        .map(|v| BarePrimitive::new(*v))
        .map(move |p| p.transform(view * matrix))
        .collect();
    context.draw().run(&v);
}

impl<B: Backend> Scene<B> for GltfScene {
    fn render(&mut self, context: &mut Context<B>) {
        let collider = CapsuleCollider {
            base: [0.0, -1.0, 0.0].into(),
            tip: [0.0, 0.0, 0.0].into(),
            radius: 0.25,
        }
        .translate([self.x, self.y, self.z + 2.0].into());
        let view = Matrix::projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0)
            * Matrix::rotate(-self.rot_y, [1.0, 0.0, 0.0])
            * Matrix::rotate(-self.rot_x, [0.0, 1.0, 0.0])
            * Matrix::translate(-self.x, -self.y, -self.z);
        self.game.render(context, view);
        if self.visible {
            render_aabb(context, &collider.aabb(), view);
        }
    }
    fn update(&mut self, delta: f64, input: &InputState) {
        self.visible = !input.is_key_down(Key::Space);
        self.rot_x = (input.mouse_x() as f64) / 10.0;
        self.rot_y = (input.mouse_y() as f64) / 10.0;
        let input_vector: Vec2 = [
            (input.is_key_down(Key::KeyD) as u32 as f64)
                - (input.is_key_down(Key::KeyA) as u32 as f64),
            (input.is_key_down(Key::KeyW) as u32 as f64)
                - (input.is_key_down(Key::KeyS) as u32 as f64),
        ]
        .into();
        let delta_position = input_vector.rotate(-self.rot_x);
        let new_x = self.x + delta_position.x * delta * 10.0;
        let new_z = self.z + delta_position.y * delta * 10.0;
        let new_y = self.y
            + ((input.is_key_down(Key::KeyE) as u32 as f64)
                - (input.is_key_down(Key::KeyQ) as u32 as f64))
                * delta
                * 10.0;

        let collider = CapsuleCollider {
            base: [0.0, -1.0, 0.0].into(),
            tip: [0.0, 0.0, 0.0].into(),
            radius: 0.25,
        }
        .translate([new_x, new_y, new_z + 2.0].into());

        if !self.game.check_collision(&collider) {
            self.x = new_x;
            self.y = new_y;
            self.z = new_z;
        }
        self.game.update_transforms();
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
                tris.push(BarePrimitive {
                    vertices: [coord(i, j), coord(i + 1, j), coord(i + 1, j + 1)],
                    uv: [Vec2::default(); 3],
                    color: [Color::WHITE; 3],
                });
                tris.push(BarePrimitive {
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
