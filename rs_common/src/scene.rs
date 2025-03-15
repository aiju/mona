use crate::{
    assets::AssetLoader,
    geometry::Matrix,
    gltf::GltfImporter,
    input::{InputEvent, InputState, Key},
    mesh::LoadedMesh,
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
    loader: impl AssetLoader,
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
    fn new<B: Backend>(context: &mut Context<B>, mut loader: impl AssetLoader) -> Cube {
        let image = loader.load_texture("cat").unwrap();
        let texture = context.load_texture(image).unwrap();
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
    fn new<B: Backend>(context: &mut Context<B>, mut loader: impl AssetLoader) -> CatRoom {
        let image = loader.load_texture("cat").unwrap();
        let texture = context.load_texture(image).unwrap();
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
    model: LoadedMesh,
    time: f64,
    x: f64,
    y: f64,
    z: f64,
    rot_x: f64,
    rot_y: f64,
}

impl ObjScene {
    fn new<B: Backend>(context: &mut Context<B>, loader: impl AssetLoader, path: &str) -> Self {
        let model = obj_loader::load_obj_file(path).load(context, loader);
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
            context.draw().opt_textured(material.texture).run(&v);
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
    model: LoadedMesh,
    time: f64,
    x: f64,
    y: f64,
    z: f64,
    rot_x: f64,
    rot_y: f64,
}

impl GltfScene {
    fn new<B: Backend>(context: &mut Context<B>, mut loader: impl AssetLoader, path: &str) -> Self {
        let file = loader.open_file(path, None).unwrap();
        let importer =
            GltfImporter::from_reader(file, &mut loader, Some(path.to_string())).unwrap();
        let scene = importer.root_scene().unwrap().unwrap();
        let model = scene.to_mesh().load(context, loader);
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

impl<B: Backend> Scene<B> for GltfScene {
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
            context.draw().opt_textured(material.texture).run(&v);
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
                let direction = [tt.cos(), 0.0, -tt.sin()].into();
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
