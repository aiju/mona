use crate::*;

pub trait Scene {
    fn prep(&mut self) -> impl Iterator<Item = BarePrimitive>;
    fn update(&mut self, delta: f64);
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
    fn prep(&mut self) -> impl Iterator<Item = BarePrimitive> {
        let matrix = matmul(&[
            projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0),
            translate(0.0, -0.0, 3.0),
            rotate(30.0 * self.time, [0.0, 1.0, 0.0]),
        ]);
        CUBE.iter()
            .map(|v| BarePrimitive::new(*v))
            .map(move |p| p.transform(matrix))
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
    fn prep(&mut self) -> impl Iterator<Item = BarePrimitive> {
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
        tri.into_iter()
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
    fn prep(&mut self) -> impl Iterator<Item = BarePrimitive> {
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
    }
    fn update(&mut self, delta: f64) {
        self.time += delta;
    }
}
