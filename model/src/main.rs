use std::f64::consts::PI;

mod obj_parser;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const TILE_SIZE: usize = 4;

#[derive(Default, Debug)]
struct Context {
    primitives: u64,
    coarse_tiles: u64,
    fine_tiles: u64,
    inside_pixels: u64,
    depth_pass_pixels: u64,
}

const VERTICES: &'static [[[f64; 5]; 3]] = &include!("model.rs");
/* 
    &[
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
*/

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

fn matmulv(m: Matrix, p: [f64; 4]) -> [f64; 4] {
    let x = m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3] * p[3];
    let y = m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3] * p[3];
    let z = m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3] * p[3];
    let w = m[3][0] * p[0] + m[3][1] * p[1] + m[3][2] * p[2] + m[3][3] * p[3];
    [x, y, z, w]
}

fn project(p: [f64; 4]) -> [f64; 4] {
    let [x, y, z, w] = p;
    [x / w, y / w, z / w, 1.0 / w]
}

fn clip_line(a: [f64; 4], b: [f64; 4], c: [f64; 4]) -> f64 {
    (c[0] * b[0] + c[1] * b[1] + c[2] * b[2] + c[3] * b[3])
        / (c[0] * (b[0] - a[0])
            + c[1] * (b[1] - a[1])
            + c[2] * (b[2] - a[2])
            + c[3] * (b[3] - a[3]))
}

fn lerp4(a: [f64; 4], b: [f64; 4], l: f64) -> [f64; 4] {
    [
        a[0] * l + b[0] * (1.0 - l),
        a[1] * l + b[1] * (1.0 - l),
        a[2] * l + b[2] * (1.0 - l),
        a[3] * l + b[3] * (1.0 - l),
    ]
}

fn lerp2(a: [f64; 2], b: [f64; 2], l: f64) -> [f64; 2] {
    [a[0] * l + b[0] * (1.0 - l), a[1] * l + b[1] * (1.0 - l)]
}

#[derive(Debug, Clone)]
struct BarePrimitive {
    vertices: [[f64; 4]; 3],
    uv: [[f64; 2]; 3],
}

impl BarePrimitive {
    fn new(data: [[f64; 5]; 3]) -> Self {
        let vertices = data.map(|d| [d[0], d[1], d[2], 1.0]);
        let uv = data.map(|d| [d[3], d[4]]);
        BarePrimitive { vertices, uv }
    }
    fn transform(&self, matrix: Matrix) -> Self {
        BarePrimitive {
            vertices: self.vertices.map(|v| matmulv(matrix, v)),
            uv: self.uv,
        }
    }
    fn clip_corner(
        &self,
        i: usize,
        j: usize,
        k: usize,
        plane: [f64; 4],
    ) -> ([f64; 4], [f64; 2], [f64; 4], [f64; 2]) {
        let a = clip_line(self.vertices[i], self.vertices[j], plane);
        let b = clip_line(self.vertices[i], self.vertices[k], plane);
        let va = lerp4(self.vertices[i], self.vertices[j], a);
        let uva = lerp2(self.uv[i], self.uv[j], a);
        let vb = lerp4(self.vertices[i], self.vertices[k], b);
        let uvb = lerp2(self.uv[i], self.uv[k], b);
        (va, uva, vb, uvb)
    }
    fn clip(&self, plane: [f64; 4]) -> Vec<Self> {
        let mut clipcode: u8 = 0;
        for i in 0..3 {
            if (0..4).map(|j| self.vertices[i][j] * plane[j]).sum::<f64>() > 0.0 {
                clipcode |= 1 << i;
            }
        }
        match clipcode {
            0b111 => vec![],
            0b100 | 0b010 | 0b001 => {
                let i = clipcode.trailing_zeros() as usize;
                let j = (i + 1) % 3;
                let k = (i + 2) % 3;
                let (va, uva, vb, uvb) = self.clip_corner(i, j, k, plane);
                vec![
                    BarePrimitive {
                        vertices: [va, self.vertices[j], self.vertices[k]],
                        uv: [uva, self.uv[j], self.uv[k]],
                    },
                    BarePrimitive {
                        vertices: [va, vb, self.vertices[k]],
                        uv: [uva, uvb, self.uv[k]],
                    },
                ]
            }
            0b110 | 0b101 | 0b011 => {
                let i = (7 ^ clipcode).trailing_zeros() as usize;
                let j = (i + 1) % 3;
                let k = (i + 2) % 3;
                let (va, uva, vb, uvb) = self.clip_corner(i, j, k, plane);
                vec![BarePrimitive {
                    vertices: [self.vertices[i], va, vb],
                    uv: [self.uv[i], uva, uvb],
                }]
            }
            0b000 => vec![self.clone()],
            _ => unreachable!(),
        }
    }
    fn project(&self) -> Self {
        BarePrimitive {
            vertices: self.vertices.map(project),
            uv: self.uv,
        }
    }
}

#[derive(Debug, Clone)]
struct Primitive {
    vertices: [[f64; 4]; 3],
    uv: [[f64; 2]; 3],
    edge_fns: [[f64; 3]; 3],
    signed_area: f64,
}

impl Primitive {
    fn new(p: &BarePrimitive) -> Self {
        let mut edge_fns = [[0.0; 3]; 3];
        for i in 0..3 {
            let ax = -(p.vertices[(i + 2) % 3][1] - p.vertices[(i + 1) % 3][1]);
            let ay = p.vertices[(i + 2) % 3][0] - p.vertices[(i + 1) % 3][0];
            let c = -(ax * p.vertices[(i + 1) % 3][0] + ay * p.vertices[(i + 1) % 3][1]);
            edge_fns[i] = [ax, ay, c];
        }
        let signed_area = edge_fns[0][2] + edge_fns[1][2] + edge_fns[2][2];
        Primitive {
            vertices: p.vertices,
            uv: p.uv,
            edge_fns,
            signed_area,
        }
    }
}

fn find_tiles(ctx: &mut Context, p: &Primitive) -> Vec<[usize; 2]> {
    /*if p.signed_area < 0.0 {
        return vec![];
    }*/
    ctx.primitives += 1;
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
    let y0 = ((min_y / t)
        .floor()
        .clamp(0.0, ((HEIGHT - 1) / TILE_SIZE) as f64)
        * t) as usize;
    let y1 = ((max_y / t)
        .ceil()
        .clamp(0.0, ((HEIGHT - 1) / TILE_SIZE) as f64)
        * t) as usize;
    let x0 = ((min_x / t)
        .floor()
        .clamp(0.0, ((WIDTH - 1) / TILE_SIZE) as f64)
        * t) as usize;
    let x1 = ((max_x / t)
        .ceil()
        .clamp(0.0, ((WIDTH - 1) / TILE_SIZE) as f64)
        * t) as usize;
    for y in (y0..=y1).step_by(TILE_SIZE) {
        for x in (x0..=x1).step_by(TILE_SIZE) {
            ctx.coarse_tiles += 1;
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
                ctx.fine_tiles += 1;
                tiles.push([x, y]);
            }
        }
    }
    tiles
}

fn render_primitive(
    ctx: &mut Context,
    p: &Primitive,
    buffer: &mut [u32],
    depth: &mut [f64],
    texture: &RgbImage,
) {
    for [gx, gy] in find_tiles(ctx, p) {
        for oy in 0..TILE_SIZE {
            for ox in 0..TILE_SIZE {
                let y = gy + oy;
                let x = gx + ox;
                let e = [0, 1, 2].map(|i| {
                    p.edge_fns[i][0] * x as f64 + p.edge_fns[i][1] * y as f64 + p.edge_fns[i][2]
                });
                let inside = e[0] >= 0.0 && e[1] >= 0.0 && e[2] >= 0.0
                    || e[0] <= 0.0 && e[1] <= 0.0 && e[2] <= 0.0;
                if !inside {
                    continue;
                }
                ctx.inside_pixels += 1;
                let area: f64 = (0..3).map(|i| e[i]).sum();
                let z = (0..3).map(|i| e[i] * p.vertices[i][2]).sum::<f64>() / area;
                let depth_ptr = &mut depth[y * WIDTH + x];
                if z >= *depth_ptr {
                    continue;
                }
                ctx.depth_pass_pixels += 1;
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
                let ty = ((v * (texture.height() as f64)) as u32).clamp(0, texture.height() - 1);
                let rgb = texture.get_pixel(tx, ty);
                buffer[y * WIDTH + x] =
                    rgb.0[2] as u32 | (rgb.0[1] as u32) << 8 | (rgb.0[0] as u32) << 16;
                *depth_ptr = z;
            }
        }
    }
}

fn render_frame(
    ctx: &mut Context,
    primitives: &[Primitive],
    mut buffer: &mut [u32],
    texture: &RgbImage,
) {
    buffer.fill(0);
    let mut depth = [f64::INFINITY; WIDTH * HEIGHT];
    for p in primitives {
        render_primitive(ctx, &p, &mut buffer, &mut depth, texture);
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
                translate(0.0, -1.0, 3.0),
            ),
            matmul(
                rotate(-20.0, [1.0, 0.0, 0.0]),
                rotate(30.0 * t, [0.0, 1.0, 0.0]),
            ),
        );
        let primitives: Vec<_> = VERTICES
            .iter()
            .flat_map(|v| {
                BarePrimitive::new(*v)
                    .transform(matrix)
                    .clip([0.0, 0.0, 1.0, 1.0])
            })
            .map(|p| Primitive::new(&p.project()))
            .collect();
        let mut ctx = Context::default();
        render_frame(&mut ctx, &primitives, &mut buffer, &texture);
        println!("{:?}", ctx);

        t += 10.0 / 60.0;

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
