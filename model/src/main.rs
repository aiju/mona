#![allow(dead_code)]

use std::f64::consts::PI;

mod obj_parser;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const TILE_SIZE: usize = 8;
const FINE_TILE_SIZE: usize = 4;

#[derive(Default, Debug)]
struct Context {
    primitives: u64,
    coarse_tiles: u64,
    fine_tiles: u64,
    inside_pixels: u64,
    depth_pass_pixels: u64,
}

const VERTICES: &'static [[[f64; 5]; 3]] = &include!("model.rs");
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
    [
        [width / (2.0 * f), 0.0, width / 2.0, 0.0],
        [0.0, -width / (2.0 * f), height / 2.0, 0.0],
        [0.0, 0.0, far / (near - far), far * near / (near - far)],
        [0.0, 0.0, 1.0, 0.0],
    ]
}

fn matmul(args: &[Matrix]) -> Matrix {
    args.iter()
        .copied()
        .reduce(|m, n| {
            let mut r = [[0.0; 4]; 4];
            for i in 0..4 {
                for j in 0..4 {
                    for k in 0..4 {
                        r[i][j] += m[i][k] * n[k][j];
                    }
                }
            }
            r
        })
        .unwrap_or([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
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

#[derive(Debug, Clone)]
struct BBox {
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
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
    fn edge_mat(&self) -> Option<[[f64; 3]; 3]> {
        let [x0, y0, _, w0] = self.vertices[0];
        let [x1, y1, _, w1] = self.vertices[1];
        let [x2, y2, _, w2] = self.vertices[2];
        let mut d = x0 * y1 * w2 + y0 * w1 * x2 + w0 * x1 * y2;
        d -= x0 * w1 * y2 + y0 * x1 * w2 + w0 * y1 * x2;
        if d.abs() < 1e-9 {
            None
        } else {
            d = 1.0 / d;
            Some([
                [
                    (w2 * y1 - w1 * y2) * d,
                    (w0 * y2 - w2 * y0) * d,
                    (w1 * y0 - w0 * y1) * d,
                ],
                [
                    (w1 * x2 - w2 * x1) * d,
                    (w2 * x0 - w0 * x2) * d,
                    (w0 * x1 - w1 * x0) * d,
                ],
                [
                    (x1 * y2 - x2 * y1) * d,
                    (x2 * y0 - x0 * y2) * d,
                    (x0 * y1 - x1 * y0) * d,
                ],
            ])
        }
    }
    fn extent(&self, xy: usize) -> Option<[f64; 2]> {
        let size = if xy != 0 { HEIGHT } else { WIDTH } as f64;
        let mut min = size;
        let mut max = 0.0;
        let mut lr = [0; 3];
        let mut anyvis = true;
        for i in 0..3 {
            if self.vertices[i][xy] < 0.0 {
                lr[i] |= 1;
            }
            let r = size * self.vertices[i][3] - self.vertices[i][xy];
            if r < 0.0 {
                lr[i] |= 2;
            }
            if lr[i] == 0 {
                anyvis = true;
                if self.vertices[i][xy] - min * self.vertices[i][3] < 0.0 {
                    min = self.vertices[i][xy] / self.vertices[i][3];
                }
                if self.vertices[i][xy] - max * self.vertices[i][3] > 0.0 {
                    max = self.vertices[i][xy] / self.vertices[i][3];
                }
            }
        }
        if lr[0] | lr[1] | lr[2] == 0 {
            Some([min, max])
        } else if lr[0] & lr[1] & lr[2] != 0 {
            None
        } else if !anyvis {
            println!("bailed");
            Some([0.0, size])
        } else {
            for i in 0..3 {
                if lr[i] & 1 != 0 && self.vertices[i][xy] - min * self.vertices[i][3] < 0.0 {
                    min = 0.0;
                }
                if lr[i] & 2 != 0 && self.vertices[i][xy] - max * self.vertices[i][3] > 0.0 {
                    max = size;
                }
            }
            Some([min, max])
        }
    }
    fn bbox(&self) -> Option<BBox> {
        let [x0, x1] = self.extent(0)?;
        let [y0, y1] = self.extent(1)?;
        Some(BBox {
            min_x: (x0 / TILE_SIZE as f64).clamp(0.0, ((WIDTH - 1) / TILE_SIZE) as f64) as usize,
            min_y: (y0 / TILE_SIZE as f64).clamp(0.0, ((HEIGHT - 1) / TILE_SIZE) as f64) as usize,
            max_x: (x1 / TILE_SIZE as f64).clamp(0.0, ((WIDTH - 1) / TILE_SIZE) as f64) as usize,
            max_y: (y1 / TILE_SIZE as f64).clamp(0.0, ((HEIGHT - 1) / TILE_SIZE) as f64) as usize,
        })
    }
    fn dummy_bbox(&self) -> Option<BBox> {
        Some(BBox {
            min_x: 0,
            min_y: 0,
            max_x: (WIDTH - 1) / TILE_SIZE,
            max_y: (HEIGHT - 1) / TILE_SIZE,
        })
    }
}

#[derive(Debug, Clone)]
struct CoarseRasterIn {
    edge_mat: [[f64; 3]; 3],
    uv: [[f64; 2]; 3],
    bbox: BBox,
}

impl CoarseRasterIn {
    fn new(p: &BarePrimitive) -> Option<Self> {
        let bbox = p.bbox()?;
        let edge_mat = p.edge_mat()?;
        Some(CoarseRasterIn {
            edge_mat,
            uv: p.uv,
            bbox,
        })
    }
}

#[derive(Debug, Clone)]
struct Tile {
    pos: [usize; 2],
    edge_vec: [f64; 3],
}

fn coarse_raster(ctx: &mut Context, p: &CoarseRasterIn) -> Vec<Tile> {
    ctx.primitives += 1;
    let mut tiles = Vec::new();
    let mut e_left = [0, 1, 2].map(|i| {
        p.edge_mat[0][i] * (p.bbox.min_x * TILE_SIZE) as f64
            + p.edge_mat[1][i] * (p.bbox.min_y * TILE_SIZE) as f64
            + p.edge_mat[2][i]
    });
    for y in p.bbox.min_y..=p.bbox.max_y {
        let mut e = e_left;
        for x in p.bbox.min_x..=p.bbox.max_x {
            ctx.coarse_tiles += 1;
            let mut c = [[0.0; 3]; 4];
            for i in 0..3 {
                c[0][i] = e[i];
                c[1][i] = e[i] + p.edge_mat[0][i] * (TILE_SIZE - 1) as f64;
                c[2][i] = e[i] + p.edge_mat[1][i] * (TILE_SIZE - 1) as f64;
                c[3][i] = e[i] + (p.edge_mat[0][i] + p.edge_mat[1][i]) * (TILE_SIZE - 1) as f64;
            }
            let ok = (0..3).all(|j| (0..4).any(|i| c[i][j] >= 0.0));
            if ok {
                ctx.fine_tiles += ((TILE_SIZE / FINE_TILE_SIZE) * (TILE_SIZE / FINE_TILE_SIZE)) as u64;
                tiles.push(Tile {
                    pos: [x, y],
                    edge_vec: e,
                });
            }
            for i in 0..3 {
                e[i] += p.edge_mat[0][i] * TILE_SIZE as f64;
            }
        }
        for i in 0..3 {
            e_left[i] += p.edge_mat[1][i] * TILE_SIZE as f64;
        }
    }
    tiles
}

fn fine_raster(
    ctx: &mut Context,
    p: &CoarseRasterIn,
    tile: &Tile,
    buffer: &mut [u32],
    depth: &mut [f64],
    texture: &RgbImage,
) {
    for oy in 0..TILE_SIZE {
        for ox in 0..TILE_SIZE {
            let y = tile.pos[1] * TILE_SIZE + oy;
            let x = tile.pos[0] * TILE_SIZE + ox;
            let e = [0, 1, 2].map(|i| {
                tile.edge_vec[i] + p.edge_mat[0][i] * ox as f64 + p.edge_mat[1][i] * oy as f64
            });
            let inside = e.iter().all(|&p| p >= 0.0);
            if !inside {
                continue;
            }
            ctx.inside_pixels += 1;
            let wr = e[0] + e[1] + e[2];
            let depth_ptr = &mut depth[y * WIDTH + x];
            if wr <= *depth_ptr {
                continue;
            }
            *depth_ptr = wr;
            ctx.depth_pass_pixels += 1;
            let u = (0..3).map(|i| p.uv[i][0] * e[i]).sum::<f64>() / wr;
            let v = (0..3).map(|i| p.uv[i][1] * e[i]).sum::<f64>() / wr;
            let tx = ((u * (texture.width() as f64)) as u32).clamp(0, texture.width() - 1);
            let ty = ((v * (texture.height() as f64)) as u32).clamp(0, texture.height() - 1);
            let rgb = texture.get_pixel(tx, ty);
            buffer[y * WIDTH + x] =
                rgb.0[2] as u32 | (rgb.0[1] as u32) << 8 | (rgb.0[0] as u32) << 16;
        }
    }
}
/*
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
    */

fn render_frame(
    ctx: &mut Context,
    primitives: &[BarePrimitive],
    matrix: Matrix,
    buffer: &mut [u32],
    texture: &RgbImage,
) {
    buffer.fill(0);
    let mut depth = [-f64::INFINITY; WIDTH * HEIGHT];
    let data = primitives
        .iter()
        .map(|p| p.transform(matrix))
        .flat_map(|p| CoarseRasterIn::new(&p))
        .collect::<Vec<_>>();
    for p in &data {
        for tile in coarse_raster(ctx, p) {
            fine_raster(ctx, &p, &tile, buffer, &mut depth, texture);
        }
    }
    for p in &data {
        for x in p.bbox.min_x * TILE_SIZE..(p.bbox.max_x + 1) * TILE_SIZE {
            buffer[p.bbox.min_y * TILE_SIZE * WIDTH + x] = 0xffffff;
            buffer[((p.bbox.max_y + 1) * TILE_SIZE - 1) * WIDTH + x] = 0xffffff;
        }
        for y in p.bbox.min_y * TILE_SIZE..(p.bbox.max_y + 1) * TILE_SIZE {
            buffer[p.bbox.min_x * TILE_SIZE + y * WIDTH] = 0xffffff;
            buffer[((p.bbox.max_x + 1) * TILE_SIZE - 1) + y * WIDTH] = 0xffffff;
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
        let matrix = matmul(&[
            projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0),
            translate(0.0, -1.0, 3.0),
            rotate(-20.0, [1.0, 0.0, 0.0]),
            rotate(30.0 * t, [0.0, 1.0, 0.0]),
        ]);
        let primitives: Vec<_> = VERTICES.iter().map(|v| BarePrimitive::new(*v)).collect();
        let mut ctx = Context::default();
        render_frame(&mut ctx, &primitives, matrix, &mut buffer, &texture);
        println!("{:?}", ctx);

        t += 10.0 / 60.0;

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
