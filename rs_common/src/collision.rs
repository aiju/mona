use core::f64;

use crate::{geometry::Vec3, mesh::Triangle};

mod bvh;
mod geometry;

pub use bvh::{Bvh, BvhPrimitive};
pub use geometry::*;

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for Aabb {
    fn default() -> Self {
        Aabb::empty()
    }
}

impl Aabb {
    pub fn empty() -> Aabb {
        Aabb {
            min: [f64::INFINITY; 3].into(),
            max: [-f64::INFINITY; 3].into(),
        }
    }
    pub fn grow(&mut self, v: Vec3) {
        self.min = self.min.cw_min(v);
        self.max = self.max.cw_max(v);
    }
    pub fn merge(&mut self, other: &Aabb) {
        self.min = self.min.cw_min(other.min);
        self.max = self.max.cw_max(other.max);
    }
    pub fn surface_area(&self) -> f64 {
        let v = self.max - self.min;
        if v[0] > 0.0 && v[1] > 0.0 && v[2] > 0.0 {
            2.0 * (v[0] * v[1] + v[1] * v[2] + v[2] * v[0])
        } else {
            0.0
        }
    }
    pub fn intersects(&self, other: &Aabb) -> bool {
        Vec3::all_ge(self.max, other.min) && Vec3::all_ge(other.max, self.min)
    }
}

impl Extend<Aabb> for Aabb {
    fn extend<T: IntoIterator<Item = Aabb>>(&mut self, iter: T) {
        for other in iter {
            self.merge(&other);
        }
    }
}

impl FromIterator<Aabb> for Aabb {
    fn from_iter<T: IntoIterator<Item = Aabb>>(iter: T) -> Self {
        let mut aabb = Aabb::empty();
        aabb.extend(iter);
        aabb
    }
}

pub fn collide_triangle_and_sphere(
    triangle: &Triangle,
    center: Vec3,
    radius: f64,
) -> Option<(Vec3, f64)> {
    closest_point_on_triangle(triangle, center, radius).map(|closest| {
        let vec = center - closest;
        let len = vec.len();
        (vec * (1.0 / len), radius - len)
    })
}

pub fn collide_triangle_and_capsule(
    triangle: &Triangle,
    base: Vec3,
    tip: Vec3,
    radius: f64,
) -> Option<(Vec3, f64)> {
    let capsule_vec = (tip - base).normalize();
    let base_center = base + capsule_vec * radius;
    let tip_center = tip - capsule_vec * radius;
    closest_points_of_triangle_and_line_segment(triangle, base_center, tip_center, radius)
        .map(|(a, b, d)| ((b - a) * (1.0 / d), radius - d))
}

pub struct SphereCollider {
    pub center: Vec3,
    pub radius: f64,
}

impl SphereCollider {
    pub fn aabb(&self) -> Aabb {
        let v = [self.radius, self.radius, self.radius].into();
        Aabb {
            min: self.center - v,
            max: self.center + v,
        }
    }
}

pub struct CapsuleCollider {
    pub base: Vec3,
    pub tip: Vec3,
    pub radius: f64,
}

impl CapsuleCollider {
    pub fn aabb(&self) -> Aabb {
        let capsule_vec = (self.tip - self.base).normalize();
        let base_center = self.base + capsule_vec * self.radius;
        let tip_center = self.tip - capsule_vec * self.radius;
        let mut r = SphereCollider {
            center: base_center,
            radius: self.radius,
        }
        .aabb();
        r.merge(
            &SphereCollider {
                center: tip_center,
                radius: self.radius,
            }
            .aabb(),
        );
        r
    }
    pub fn translate(&self, offset: Vec3) -> Self {
        CapsuleCollider {
            base: self.base + offset,
            tip: self.tip + offset,
            radius: self.radius,
        }
    }
    pub fn intersect_triangle(&self, triangle: &Triangle) -> Option<(Vec3, f64)> {
        collide_triangle_and_capsule(triangle, self.base, self.tip, self.radius)
    }
}
