#![allow(dead_code)]

use std::f64::consts::PI;

pub const WIDTH: usize = 640;
pub const HEIGHT: usize = 480;
pub const TILE_SIZE: usize = 4;

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

pub type Matrix = [[f64; 4]; 4];

pub fn rotate(angle: f64, axis: [f64; 3]) -> Matrix {
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

pub fn translate(x: f64, y: f64, z: f64) -> Matrix {
    [
        [1.0, 0.0, 0.0, x],
        [0.0, 1.0, 0.0, y],
        [0.0, 0.0, 1.0, z],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn projection(fov_y: f64, width: f64, height: f64, near: f64, far: f64) -> Matrix {
    let f = (fov_y / 2.0 * PI / 180.0).tan();
    [
        [width / (2.0 * f), 0.0, width / 2.0, 0.0],
        [0.0, -width / (2.0 * f), height / 2.0, 0.0],
        [0.0, 0.0, far / (near - far), far * near / (near - far)],
        [0.0, 0.0, 1.0, 0.0],
    ]
}

pub fn matmul(args: &[Matrix]) -> Matrix {
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

pub fn matmulv(m: Matrix, p: [f64; 4]) -> [f64; 4] {
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
pub struct BarePrimitive {
    pub vertices: [[f64; 4]; 3],
    pub uv: [[f64; 2]; 3],
}

#[derive(Debug, Clone)]
pub struct BBox {
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
}

impl BarePrimitive {
    pub fn new(data: [[f64; 5]; 3]) -> Self {
        let vertices = data.map(|d| [d[0], d[1], d[2], 1.0]);
        let uv = data.map(|d| [d[3], d[4]]);
        BarePrimitive { vertices, uv }
    }
    pub fn transform(&self, matrix: Matrix) -> Self {
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
}

#[derive(Debug, Clone)]
pub struct CoarseRasterIn {
    pub edge_mat: [[f64; 3]; 3],
    pub uv: [[f64; 2]; 3],
    pub bbox: BBox,
}

impl CoarseRasterIn {
    pub fn new(p: &BarePrimitive) -> Option<Self> {
        let bbox = p.bbox()?;
        let mut edge_mat = p.edge_mat()?;
        for i in 0..3 {
            edge_mat[2][i] += edge_mat[0][i] * (bbox.min_x * TILE_SIZE) as f64;
            edge_mat[2][i] += edge_mat[1][i] * (bbox.min_y * TILE_SIZE) as f64;
        }
        Some(CoarseRasterIn {
            edge_mat,
            uv: p.uv,
            bbox,
        })
    }
}
