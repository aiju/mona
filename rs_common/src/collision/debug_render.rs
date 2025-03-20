use std::f64::consts::PI;

use crate::{
    geometry::{Matrix, Vec2, Vec3, Vec4},
    mesh::Color,
    render::{Backend, Context, Triangle4},
};

use super::CapsuleCollider;

impl CapsuleCollider {
    pub fn debug_render<B: Backend>(&self, context: &mut Context<B>, view: Matrix) {
        const NPHI: u32 = 10;
        const NTHETA: u32 = 5;
        let mut v = Vec::new();
        let vec = (self.tip - self.base).normalize();
        let base_center = self.base + vec * self.radius;
        let tip_center = self.tip - vec * self.radius;
        let normal1 = if vec.x.abs() < 0.1 {
            vec.cross([1.0, 0.0, 0.0].into()).normalize()
        } else {
            vec.cross([0.0, 1.0, 0.0].into()).normalize()
        };
        let normal2 = vec.cross(normal1);
        let bottom = |phi: f64, theta: f64| {
            base_center + (normal1 * phi.cos() + normal2 * phi.sin()) * self.radius * theta.cos()
                - vec * self.radius * theta.sin()
        };
        let top = |phi: f64, theta: f64| {
            tip_center + (normal1 * phi.cos() + normal2 * phi.sin()) * self.radius * theta.cos()
                + vec * self.radius * theta.sin()
        };
        let mut tri = |a: Vec3, b: Vec3, c: Vec3| {
            v.push(
                Triangle4 {
                    vertices: [a, b, c].map(Vec4::from),
                    uv: [Vec2::default(); 3],
                    color: [Color::WHITE; 3],
                }
                .lighting(0.5, 0.5, [0.707, 0.0, -0.707].into())
                .transform(view),
            )
        };
        for i in 0..NPHI {
            let phi1 = (i as f64 * 2.0 * PI) / (NPHI as f64);
            let phi2 = ((i + 1) as f64 * 2.0 * PI) / (NPHI as f64);
            tri(bottom(phi1, 0.0), bottom(phi2, 0.0), top(phi1, 0.0));
            tri(bottom(phi2, 0.0), top(phi2, 0.0), top(phi1, 0.0));
            for j in 0..NTHETA {
                let th1 = (j as f64 * 0.5 * PI) / (NTHETA as f64);
                let th2 = ((j + 1) as f64 * 0.5 * PI) / (NTHETA as f64);
                tri(top(phi1, th1), top(phi2, th1), top(phi2, th2));
                tri(top(phi1, th1), top(phi2, th2), top(phi1, th2));
                tri(bottom(phi1, th1), bottom(phi2, th2), bottom(phi2, th1));
                tri(bottom(phi1, th1), bottom(phi1, th2), bottom(phi2, th2));
            }
        }
        context.draw().run(&v);
    }
}
