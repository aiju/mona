use std::{
    f64::consts::PI,
    ops::{Add, Mul, Sub},
};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Vec4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Matrix(pub [[f64; 4]; 4]);

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Quaternion(pub Vec4);

#[derive(Clone, Debug)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
}

impl From<[f32; 2]> for Vec2 {
    fn from(value: [f32; 2]) -> Self {
        Vec2 {
            x: value[0] as f64,
            y: value[1] as f64,
        }
    }
}

impl From<[f64; 2]> for Vec2 {
    fn from(value: [f64; 2]) -> Self {
        Vec2 {
            x: value[0],
            y: value[1],
        }
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(value: [f32; 3]) -> Self {
        Vec3 {
            x: value[0] as f64,
            y: value[1] as f64,
            z: value[2] as f64,
        }
    }
}

impl From<[f64; 3]> for Vec3 {
    fn from(value: [f64; 3]) -> Self {
        Vec3 {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl From<[f64; 4]> for Vec4 {
    fn from(value: [f64; 4]) -> Self {
        Vec4 {
            x: value[0],
            y: value[1],
            z: value[2],
            w: value[3],
        }
    }
}

impl From<[f64; 4]> for Quaternion {
    fn from(value: [f64; 4]) -> Self {
        Quaternion(value.into())
    }
}

impl From<[f64; 16]> for Matrix {
    fn from(value: [f64; 16]) -> Self {
        unsafe { Matrix(std::mem::transmute::<[f64; 16], [[f64; 4]; 4]>(value)) }
    }
}

impl From<Vec3> for Vec4 {
    fn from(value: Vec3) -> Self {
        Vec4 {
            x: value.x,
            y: value.y,
            z: value.z,
            w: 1.0,
        }
    }
}

impl std::ops::Deref for Vec2 {
    type Target = [f64; 2];

    fn deref(&self) -> &Self::Target {
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

impl std::ops::Deref for Vec3 {
    type Target = [f64; 3];

    fn deref(&self) -> &Self::Target {
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

impl std::ops::Deref for Vec4 {
    type Target = [f64; 4];

    fn deref(&self) -> &Self::Target {
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

impl Matrix {
    pub const IDENTITY: Matrix = Matrix([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    pub fn transpose(self) -> Matrix {
        Matrix([0, 1, 2, 3].map(|i| [0, 1, 2, 3].map(|j| self.0[j][i])))
    }
    pub fn inverse_3x4(self) -> Matrix {
        let [
            [m00, m01, m02, m03],
            [m10, m11, m12, m13],
            [m20, m21, m22, m23],
            [_, _, _, _],
        ] = self.0;
        let f = 1.0
            / (m01 * m12 * m20 - m02 * m11 * m20 + m02 * m10 * m21
                - m00 * m12 * m21
                - m01 * m10 * m22
                + m00 * m11 * m22);
        let mut m = Matrix([
            [
                (m11 * m22 - m12 * m21) * f,
                (m02 * m21 - m01 * m22) * f,
                (m01 * m12 - m02 * m11) * f,
                0.0,
            ],
            [
                (m12 * m20 - m10 * m22) * f,
                (m00 * m22 - m02 * m20) * f,
                (m02 * m10 - m00 * m12) * f,
                0.0,
            ],
            [
                (m10 * m21 - m11 * m20) * f,
                (m01 * m20 - m00 * m21) * f,
                (m00 * m11 - m01 * m10) * f,
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let x = m * Vec4::from([-m03, -m13, -m23, 1.0]);
        m.0[0][3] = x[0];
        m.0[1][3] = x[1];
        m.0[2][3] = x[2];
        m
    }
    pub fn rotate(angle: f64, axis: [f64; 3]) -> Matrix {
        let c = (angle * PI / 180.0).cos();
        let s = (angle * PI / 180.0).sin();
        let t = 1.0 - c;
        let n = f64::hypot(axis[0], f64::hypot(axis[1], axis[2]));
        let x = axis[0] / n;
        let y = axis[1] / n;
        let z = axis[2] / n;
        Matrix([
            [t * x * x + c, t * x * y - s * z, t * x * z + s * y, 0.0],
            [t * x * y + s * z, t * y * y + c, t * y * z - s * x, 0.0],
            [t * x * z - s * y, t * y * z + s * x, t * z * z + c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
    pub fn translate(x: f64, y: f64, z: f64) -> Matrix {
        Matrix([
            [1.0, 0.0, 0.0, x],
            [0.0, 1.0, 0.0, y],
            [0.0, 0.0, 1.0, z],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
    pub fn scale(x: f64, y: f64, z: f64) -> Matrix {
        Matrix([
            [x, 0.0, 0.0, 0.0],
            [0.0, y, 0.0, 0.0],
            [0.0, 0.0, z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
    pub fn projection(fov_y: f64, width: f64, height: f64, near: f64, far: f64) -> Matrix {
        let f = (fov_y / 2.0 * PI / 180.0).tan();
        Matrix([
            [width / (2.0 * f), 0.0, width / 2.0, 0.0],
            [0.0, -width / (2.0 * f), height / 2.0, 0.0],
            [0.0, 0.0, far / (near - far), far * near / (near - far)],
            [0.0, 0.0, 1.0, 0.0],
        ])
    }
}

impl Add for Matrix {
    type Output = Matrix;
    fn add(self, rhs: Self) -> Self::Output {
        Matrix([0, 1, 2, 3].map(|i| [0, 1, 2, 3].map(|j| self.0[i][j] + rhs.0[i][j])))
    }
}

impl Sub for Matrix {
    type Output = Matrix;
    fn sub(self, rhs: Self) -> Self::Output {
        Matrix([0, 1, 2, 3].map(|i| [0, 1, 2, 3].map(|j| self.0[i][j] - rhs.0[i][j])))
    }
}

impl Mul<f64> for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: f64) -> Self::Output {
        Matrix([0, 1, 2, 3].map(|i| [0, 1, 2, 3].map(|j| self.0[i][j] * rhs)))
    }
}

impl Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut r = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    r[i][j] += self.0[i][k] * rhs.0[k][j];
                }
            }
        }
        Matrix(r)
    }
}

impl Mul<Vec3> for Matrix {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        let mut out = [self.0[0][3], self.0[1][3], self.0[2][3]];
        for i in 0..3 {
            for j in 0..3 {
                out[i] += self.0[i][j] * rhs[j];
            }
        }
        out.into()
    }
}

impl Mul<Vec4> for Matrix {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        let mut out = [0.0; 4];
        for i in 0..4 {
            for j in 0..4 {
                out[i] += self.0[i][j] * rhs[j];
            }
        }
        out.into()
    }
}

impl Vec2 {
    pub fn lerp(self, other: Vec2, l: f64) -> Vec2 {
        [
            self[0] * l + other[0] * (1.0 - l),
            self[1] * l + other[1] * (1.0 - l),
        ]
        .into()
    }
    pub fn rotate(self, angle: f64) -> Vec2 {
        let c = (angle * PI / 180.0).cos();
        let s = (angle * PI / 180.0).sin();
        [self[0] * c - self[1] * s, self[0] * s + self[1] * c].into()
    }
}

impl Vec3 {
    pub fn zero() -> Vec3 {
        Vec3::default()
    }
    pub fn len_sq(self) -> f64 {
        self * self
    }
    pub fn len(self) -> f64 {
        (self * self).sqrt()
    }
    pub fn dist_sq(self, other: Self) -> f64 {
        (self - other).len_sq()
    }
    pub fn dist(self, other: Self) -> f64 {
        (self - other).len()
    }
    pub fn normalize(self) -> Vec3 {
        // TODO: can be more intelligent here
        let l = (self * self).sqrt();
        if l < 1e-20 {
            Self::zero()
        } else {
            let l = 1.0 / l;
            [self.x * l, self.y * l, self.z * l].into()
        }
    }
    pub fn cross(self, other: Self) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    pub fn cw_min(self, other: Self) -> Vec3 {
        Vec3 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }
    pub fn cw_max(self, other: Self) -> Vec3 {
        Vec3 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }
    pub fn all_ge(self, other: Self) -> bool {
        self.x >= other.x && self.y >= other.y && self.z >= other.z
    }
    pub fn largest_axis(self) -> usize {
        let mut axis = 0;
        if self.y > self.x {
            axis = 1;
        }
        if self.z > self[axis] {
            axis = 2;
        }
        axis
    }
    pub fn xyz_max(self) -> f64 {
        self.x.max(self.y.max(self.z))
    }
    pub fn xyz_min(self) -> f64 {
        self.x.min(self.y.min(self.z))
    }
    pub fn cw_mul(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
    pub fn cw_div(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self::Output {
        [self.x + rhs.x, self.y + rhs.y, self.z + rhs.z].into()
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Self::Output {
        [self.x - rhs.x, self.y - rhs.y, self.z - rhs.z].into()
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul for Vec3 {
    type Output = f64;

    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Vec4 {
    pub fn xyz(self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
    pub fn project(self) -> Vec4 {
        let Vec4 { x, y, z, w } = self;
        [x / w, y / w, z / w, 1.0 / w].into()
    }
    pub fn lerp(self, other: Vec4, l: f64) -> Vec4 {
        [
            self[0] * l + other[0] * (1.0 - l),
            self[1] * l + other[1] * (1.0 - l),
            self[2] * l + other[2] * (1.0 - l),
            self[3] * l + other[3] * (1.0 - l),
        ]
        .into()
    }
}

impl Add for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Self) -> Self::Output {
        [
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        ]
        .into()
    }
}

impl Sub for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Self) -> Self::Output {
        [
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        ]
        .into()
    }
}

impl Mul<f64> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec4 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl Triangle {
    pub fn transform(&self, matrix: Matrix) -> Self {
        Triangle {
            vertices: self.vertices.map(|v| matrix * v),
        }
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        [0.0, 0.0, 0.0, 1.0].into()
    }
}

impl Quaternion {
    pub fn from_angle(angle: f64, axis: Vec3) -> Quaternion {
        let c = f64::cos(angle * PI / 360.0);
        let s = f64::sin(angle * PI / 360.0);
        let Vec3 { x, y, z } = axis.normalize();
        [x * s, y * s, z * s, c].into()
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: Quaternion) -> Self::Output {
        let Vec4 {
            x: x1,
            y: y1,
            z: z1,
            w: w1,
        } = self.0;
        let Vec4 {
            x: x2,
            y: y2,
            z: z2,
            w: w2,
        } = rhs.0;
        Quaternion(Vec4 {
            w: w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2,
            x: w2 * x1 + w1 * x2 - y2 * z1 + y1 * z2,
            y: w2 * y1 + w1 * y2 - x1 * z2 + z1 * x2,
            z: w2 * z1 + w1 * z2 - x2 * y1 + x1 * y2,
        })
    }
}

impl From<Quaternion> for Matrix {
    fn from(q: Quaternion) -> Matrix {
        let Vec4 {
            x: qi,
            y: qj,
            z: qk,
            w: qr,
        } = q.0;
        let s = 2.0 / (qi * qi + qj * qj + qk * qk + qr * qr);
        Matrix([
            [
                1.0 - s * (qj * qj + qk * qk),
                s * (qi * qj - qk * qr),
                s * (qi * qk + qj * qr),
                0.0,
            ],
            [
                s * (qi * qj + qk * qr),
                1.0 - s * (qi * qi + qk * qk),
                s * (qj * qk - qi * qr),
                0.0,
            ],
            [
                s * (qi * qk - qj * qr),
                s * (qj * qk + qi * qr),
                1.0 - s * (qi * qi + qj * qj),
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}
