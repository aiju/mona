use std::f64::consts::PI;

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
#[derive(Copy, Clone)]
pub struct Matrix(pub [[f64; 4]; 4]);

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
    pub fn rotate_quaternion(q: [f64; 4]) -> Matrix {
        let [qi, qj, qk, qr] = q;
        let s = 2.0 / (qi * qi + qj * qj + qk * qk + qr * qr);
        Matrix([
            [1.0 - s * (qj * qj + qk * qk), s * (qi * qj - qk * qr), s * (qi * qk + qj * qr), 0.0],
            [s * (qi * qj + qk * qr), 1.0 - s * (qi * qi + qk * qk), s * (qj * qk - qi * qr), 0.0],
            [s * (qi * qk - qj * qr), s * (qj * qk + qi * qr), 1.0 - s * (qi * qi + qj * qj), 0.0],
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

impl std::ops::Mul<Matrix> for Matrix {
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

impl std::ops::Mul<Vec3> for Matrix {
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

impl std::ops::Mul<Vec4> for Matrix {
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
    pub fn len(self) -> f64 {
        (self * self).sqrt()
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
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self::Output {
        [self.x + rhs.x, self.y + rhs.y, self.z + rhs.z].into()
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Self::Output {
        [self.x - rhs.x, self.y - rhs.y, self.z - rhs.z].into()
    }
}

impl std::ops::Mul for Vec3 {
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
