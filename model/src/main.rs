use std::f64::consts::PI;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const TILE_SIZE: usize = 4;

const VERTICES: &'static [[[f64; 5]; 3]] = &[
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

type Matrix = [[f64; 4]; 4];

fn rotate(angle: f64, axis: [f64; 3]) -> Matrix {
    let c = (angle * PI / 180.0).cos();
    let s = (angle * PI / 180.0).sin();
    let t = 1.0 - c;
    let n = f64::hypot(axis[0], f64::hypot(axis[1], axis[2]));
    let x = axis[0] / n;
    let y = axis[1] / n;
    let z = axis[2] / n;
    [
        [t * x * x + c, t * x * y - s * z, t * x * z + s * y, 0.0],
        [t * x * y + s * z, t * y * y + c, t * y * z - s * x, 0.0],
        [t * x * z - s * y, t * y * z + s * x, t * z * z + c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn translate(x: f64, y: f64, z: f64) -> Matrix {
    [
        [1.0, 0.0, 0.0, x],
        [0.0, 1.0, 0.0, y],
        [0.0, 0.0, 1.0, z],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn projection(fov_y: f64, width: f64, height: f64, near: f64, far: f64) -> Matrix {
    let f = (fov_y / 2.0 * PI / 180.0).tan();
    let t = near * f;
    let r = t * width / height;
    [
        [width * near / r / 2.0, 0.0, -width / 2.0, 0.0],
        [0.0, height * near / t / 2.0, -height / 2.0, 0.0],
        [
            0.0,
            0.0,
            (near + far) / (near - far),
            2.0 * far * near / (far - near),
        ],
        [0.0, 0.0, -1.0, 0.0],
    ]
}

fn matmul(m: Matrix, n: Matrix) -> Matrix {
    let mut r = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                r[i][j] += m[i][k] * n[k][j];
            }
        }
    }
    r
}

fn transform(m: Matrix, p: [f64; 3]) -> [f64; 4] {
    let x = m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3];
    let y = m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3];
    let z = m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3];
    let w = m[3][0] * p[0] + m[3][1] * p[1] + m[3][2] * p[2] + m[3][3];
    [x / w, y / w, z / w, 1.0 / w]
}

#[derive(Debug)]
struct Primitive {
    vertices: [[f64; 4]; 3],
    uv: [[f64; 2]; 3],
    edge_fns: [[f64; 3]; 3],
}

impl Primitive {
    fn new(data: [[f64; 5]; 3], matrix: Matrix) -> Self {
        let vertices = data.map(|d| transform(matrix, [d[0], d[1], d[2]]));
        let uv = data.map(|d| [d[3], d[4]]);
        let mut edge_fns = [[0.0; 3]; 3];
        for i in 0..3 {
            let ax = -(vertices[(i + 2) % 3][1] - vertices[(i + 1) % 3][1]);
            let ay = vertices[(i + 2) % 3][0] - vertices[(i + 1) % 3][0];
            let c = -(ax * vertices[(i + 1) % 3][0] + ay * vertices[(i + 1) % 3][1]);
            edge_fns[i] = [ax, ay, c];
        }
        Primitive {
            vertices,
            uv,
            edge_fns,
        }
    }
}

fn find_tiles(p: &Primitive) -> Vec<[usize; 2]> {
    let mut tiles = Vec::new();
    let min_y = (0..3)
        .map(|i| p.vertices[i][1])
        .min_by(f64::total_cmp)
        .unwrap();
    let max_y = (0..3)
        .map(|i| p.vertices[i][1])
        .max_by(f64::total_cmp)
        .unwrap();
    let min_x = (0..3)
        .map(|i| p.vertices[i][0])
        .min_by(f64::total_cmp)
        .unwrap();
    let max_x = (0..3)
        .map(|i| p.vertices[i][0])
        .max_by(f64::total_cmp)
        .unwrap();
    let t = TILE_SIZE as f64;
    let y0 = ((min_y / t).floor().clamp(0.0, ((HEIGHT - 1) / TILE_SIZE) as f64) * t) as usize;
    let y1 = ((max_y / t).ceil().clamp(0.0, ((HEIGHT - 1) / TILE_SIZE) as f64) * t) as usize;
    let x0 = ((min_x / t).floor().clamp(0.0, ((WIDTH - 1) / TILE_SIZE) as f64) * t) as usize;
    let x1 = ((max_x / t).ceil().clamp(0.0, ((WIDTH - 1) / TILE_SIZE) as f64) * t) as usize;
    for y in (y0..=y1).step_by(TILE_SIZE) {
        for x in (x0..=x1).step_by(TILE_SIZE) {
            let e: [[f64; 3]; 4] = [
                [0, 1, 2].map(|i| {
                    p.edge_fns[i][0] * x as f64 + p.edge_fns[i][1] * y as f64 + p.edge_fns[i][2]
                }),
                [0, 1, 2].map(|i| {
                    p.edge_fns[i][0] * (x + TILE_SIZE - 1) as f64
                        + p.edge_fns[i][1] * y as f64
                        + p.edge_fns[i][2]
                }),
                [0, 1, 2].map(|i| {
                    p.edge_fns[i][0] * x as f64
                        + p.edge_fns[i][1] * (y + TILE_SIZE - 1) as f64
                        + p.edge_fns[i][2]
                }),
                [0, 1, 2].map(|i| {
                    p.edge_fns[i][0] * (x + TILE_SIZE - 1) as f64
                        + p.edge_fns[i][1] * (y + TILE_SIZE - 1) as f64
                        + p.edge_fns[i][2]
                }),
            ];
            let can_pos = (0..3).all(|j| (0..4).any(|i| e[i][j] >= 0.0));
            let can_neg = (0..3).all(|j| (0..4).any(|i| e[i][j] <= 0.0));
            if can_pos || can_neg {
                tiles.push([x, y]);
            }
        }
    }
    tiles
}

fn render_frame(primitives: &[Primitive], buffer: &mut [u32], texture: &RgbImage) {
    buffer.fill(0);
    let mut depth = [f64::INFINITY; WIDTH * HEIGHT];
    for p in primitives {
        for [gx, gy] in find_tiles(p) {
            for oy in 0..TILE_SIZE {
                for ox in 0..TILE_SIZE {
                    let y = gy + oy;
                    let x = gx + ox;
                    if depth[y * WIDTH + x] == f64::INFINITY {
                        buffer[y * WIDTH + x] = 0xff;
                    }
                    let e = [0, 1, 2].map(|i| {
                        p.edge_fns[i][0] * x as f64 + p.edge_fns[i][1] * y as f64 + p.edge_fns[i][2]
                    });
                    let inside = e[0] >= 0.0 && e[1] >= 0.0 && e[2] >= 0.0
                        || e[0] <= 0.0 && e[1] <= 0.0 && e[2] <= 0.0;
                    if !inside {
                        continue;
                    }
                    let area: f64 = (0..3).map(|i| e[i]).sum();
                    let z = (0..3).map(|i| e[i] * p.vertices[i][2]).sum::<f64>() / area;
                    let depth_ptr = &mut depth[y * WIDTH + x];
                    if z >= *depth_ptr {
                        continue;
                    }
                    let persp: f64 = (0..3).map(|i| e[i] * p.vertices[i][3]).sum();
                    let u = (0..3)
                        .map(|i| e[i] * p.vertices[i][3] * p.uv[i][0])
                        .sum::<f64>()
                        / persp;
                    let v = (0..3)
                        .map(|i| e[i] * p.vertices[i][3] * p.uv[i][1])
                        .sum::<f64>()
                        / persp;
                    let tx = ((u * (texture.width() as f64)) as u32).clamp(0, texture.width() - 1);
                    let ty =
                        ((v * (texture.height() as f64)) as u32).clamp(0, texture.height() - 1);
                    let rgb = texture.get_pixel(tx, ty);
                    buffer[y * WIDTH + x] =
                        rgb.0[2] as u32 | (rgb.0[1] as u32) << 8 | (rgb.0[0] as u32) << 16;
                    *depth_ptr = z;
                }
            }
        }
    }
}

use image::{ImageReader, RgbImage};
use minifb::{Key, Window, WindowOptions};

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let texture = ImageReader::open("/home/aiju/cat.jpg")
        .unwrap()
        .decode()
        .unwrap()
        .into_rgb8();

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);

    let mut t = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let matrix = matmul(
            matmul(
                projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0),
                translate(0.0, 0.0, 3.0),
            ),
            rotate(30.0 * t, [1.0, 0.0, 0.0]),
        );
        let primitives: Vec<_> = VERTICES
            .iter()
            .map(|v| Primitive::new(*v, matrix))
            .collect();
        render_frame(&primitives, &mut buffer, &texture);

        t += 10.0 / 60.0;

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
