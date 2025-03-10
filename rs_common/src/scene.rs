use crate::*;

pub trait Scene {
    fn prep(&mut self) -> Vec<BarePrimitive>;
    fn update(&mut self, delta: f64);
}

pub fn create(spec: &str) -> Option<Box<dyn Scene>> {
    let (name, arg) = match spec.split_once(':') {
        Some((a, b)) => (a, Some(b)),
        None => (spec, None),
    };
    match (name, arg) {
        ("Cube", None) => Some(Box::new(Cube::default())),
        ("DoubleCube", None) => Some(Box::new(DoubleCube::default())),
        ("CatRoom", None) => Some(Box::new(CatRoom::default())),
        ("Obj", Some(path)) => Some(Box::new(ObjScene::new(path))),
        ("Sphere", None) => Some(Box::new(Sphere::default())),
        _ => None,
    }
}

const CUBE: &'static [[[f64; 5]; 3]] = &[
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

#[derive(Default)]
pub struct Cube {
    time: f64,
}

impl Scene for Cube {
    fn prep(&mut self) -> Vec<BarePrimitive> {
        let matrix = matmul(&[
            projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0),
            translate(0.0, -0.0, 3.0),
            rotate(30.0 * self.time, [0.0, 1.0, 0.0]),
        ]);
        CUBE.iter()
            .map(|v| BarePrimitive::new(*v))
            .map(move |p| p.transform(matrix))
            .collect()
    }
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }
}

#[derive(Default)]
pub struct DoubleCube {
    time: f64,
}

impl Scene for DoubleCube {
    fn prep(&mut self) -> Vec<BarePrimitive> {
        let mut tri = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let matrix = matmul(&[
                    projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0),
                    translate((x as f64 - 1.5) * 3.0, (y as f64 - 1.5) * 3.0, 8.0),
                    rotate(
                        (x as f64 - 1.5).signum() * 30.0 * self.time,
                        [0.0, 1.0, 0.0],
                    ),
                ]);
                tri.extend(
                    CUBE.iter()
                        .map(|v| BarePrimitive::new(*v))
                        .map(move |p| p.transform(matrix)),
                );
            }
        }
        tri
    }
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }
}

#[derive(Default)]
pub struct CatRoom {
    time: f64,
}

impl Scene for CatRoom {
    fn prep(&mut self) -> Vec<BarePrimitive> {
        let matrix = matmul(&[
            projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0),
            translate(0.0, -1.0, 3.0),
            rotate(-20.0, [1.0, 0.0, 0.0]),
            rotate(30.0 * self.time, [0.0, 1.0, 0.0]),
        ]);
        include!("cat_room.rs")
            .iter()
            .map(|v| BarePrimitive::new(*v))
            .map(move |p| p.transform(matrix))
            .collect()
    }
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }
}

pub struct ObjScene {
    model: Vec<BarePrimitive>,
    time: f64,
}

impl ObjScene {
    fn new(path: &str) -> Self {
        let model = obj_loader::load_obj_file(path);
        Self {
            model,
            time: Default::default(),
        }
    }
}

impl Scene for ObjScene {
    fn prep(&mut self) -> Vec<BarePrimitive> {
        let object = rotate(30.0 * self.time, [0.0, 1.0, 0.0]);
        let view = matmul(&[
            projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0),
            translate(0.0, -2.0, 5.0),
        ]);
        self.model
            .iter()
            .map(move |p| {
                p.transform(object)
                    .lighting(0.3, 0.3, [0.707, 0.0, -0.707])
                    .transform(view)
            })
            .collect()
    }
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }
}

#[derive(Default)]
pub struct Sphere {
    time: f64,
}

impl Scene for Sphere {
    fn prep(&mut self) -> Vec<BarePrimitive> {
        let view = matmul(&[
            projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0),
            translate(0.0, 0.0, 3.0),
        ]);
        let theta_steps = 16;
        let phi_steps = 16;
        let coord = |i: i32, j: i32| {
            let theta = PI * (i as f64) / (theta_steps as f64);
            let phi = 2.0 * PI * (j as f64) / (phi_steps as f64);
            let x = phi.cos() * theta.sin();
            let y = phi.sin() * theta.sin();
            let z = theta.cos();
            [x, y, z, 1.0]
        };
        let mut tris = Vec::new();
        for i in 0..theta_steps {
            for j in 0..phi_steps {
                tris.push(BarePrimitive {
                    vertices: [coord(i, j), coord(i + 1, j), coord(i + 1, j + 1)],
                    uv: [[0.0; 2]; 3],
                    rgb: [!0; 3],
                });
                tris.push(BarePrimitive {
                    vertices: [coord(i, j), coord(i + 1, j + 1), coord(i, j + 1)],
                    uv: [[0.0; 2]; 3],
                    rgb: [!0; 3],
                });
            }
        }
        for t in &mut tris {
            for i in 0..3 {
                let normal = xyz(t.vertices[i]);
                let tt = self.time;
                let direction = [tt.cos(), 0.0, -tt.sin()];
                let s = dot(normal, direction).clamp(0.0, 1.0);
                let l = 0.3 + 0.3 * s + 0.3 * s * s * s;
                t.rgb[i] = ((l * 255.0) as u32) * 0x10101;
            }
        }
        tris.iter().map(|p| p.transform(view)).collect()
    }
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }
}
