use core::f64;
use std::{cell::Cell, ops::Range};

use crate::{
    geometry::{Triangle, Vec3},
    mesh::Mesh,
};

use super::Aabb;

pub trait BvhPrimitive {
    fn aabb(&self) -> Aabb;
    fn centroid(&self) -> Vec3;
}

#[derive(Clone, Debug)]
struct BvhNode {
    aabb: Aabb,
    left_or_first: usize,
    prim_count: usize,
}

impl BvhNode {
    fn is_leaf(&self) -> bool {
        self.prim_count != !0
    }
    fn range(&self) -> Range<usize> {
        self.left_or_first..self.left_or_first + self.prim_count
    }
    fn leaf(aabb: Aabb, prims: Range<usize>) -> BvhNode {
        BvhNode {
            aabb,
            left_or_first: prims.start,
            prim_count: prims.len(),
        }
    }
    fn non_leaf(aabb: Aabb, left: usize) -> BvhNode {
        BvhNode {
            aabb,
            left_or_first: left,
            prim_count: !0,
        }
    }
}

pub struct Bvh<I> {
    nodes: Vec<BvhNode>,
    indices: Vec<I>,
}

impl<I> Bvh<I> {
    pub fn from_primitives<P: BvhPrimitive>(
        primitive_indices: Vec<I>,
        primitives: impl Fn(&I) -> P,
    ) -> Self {
        let aabb = primitive_indices
            .iter()
            .map(|i| primitives(i).aabb())
            .collect();
        let mut bvh = Bvh {
            nodes: vec![BvhNode::leaf(aabb, 0..primitive_indices.len())],
            indices: primitive_indices,
        };
        bvh.subdivide(&primitives, 0);
        bvh
    }
    fn find_best_split<P: BvhPrimitive>(
        &self,
        primitives: impl Fn(&I) -> P,
        node_idx: usize,
    ) -> (f64, usize, f64) {
        const BIN_COUNT: usize = 8;
        let node = &self.nodes[node_idx];
        #[derive(Clone, Default, Debug)]
        struct Bin {
            aabb: Aabb,
            count: usize,
        }
        let mut best_cost = f64::INFINITY;
        let mut best_split = 0.0;
        let mut best_axis = 0;
        for axis in 0..3 {
            let mut bins = vec![Bin::default(); BIN_COUNT];
            let mut scale = bins.len() as f64 / (node.aabb.max[axis] - node.aabb.min[axis]);
            for p_idx in &self.indices[node.range()] {
                let bin_idx = (((primitives(p_idx).centroid()[axis] - node.aabb.min[axis]) * scale)
                    as usize)
                    .min(bins.len() - 1);
                bins[bin_idx].count += 1;
                bins[bin_idx].aabb.merge(&primitives(p_idx).aabb());
            }
            let mut costs = vec![0.0; bins.len() - 1];
            let mut left_sum = 0;
            let mut right_sum = 0;
            let mut left_aabb = Aabb::empty();
            let mut right_aabb = Aabb::empty();
            for i in 0..bins.len() - 1 {
                left_sum += bins[i].count;
                left_aabb.merge(&bins[i].aabb);
                costs[i] += (left_sum as f64) * left_aabb.surface_area();
            }
            for i in (0..bins.len() - 1).rev() {
                right_sum += bins[i + 1].count;
                right_aabb.merge(&bins[i + 1].aabb);
                costs[i] += (right_sum as f64) * right_aabb.surface_area();
            }
            scale = 1.0 / scale;
            for (i, &cost) in costs.iter().enumerate() {
                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    best_split = node.aabb.min[axis] + scale * ((i + 1) as f64);
                }
            }
        }
        (best_cost, best_axis, best_split)
    }
    fn subdivide<P: BvhPrimitive>(&mut self, primitives: &impl Fn(&I) -> P, node_idx: usize) {
        let node = &self.nodes[node_idx];
        let aabb = node.aabb;
        let old_cost = aabb.surface_area() * (node.range().len() as f64);
        let (cost, axis, split_pos) = self.find_best_split(primitives, node_idx);
        if cost >= old_cost {
            return;
        }
        let range = node.range();
        let mut i = range.start;
        let mut j = range.end - 1;
        while i <= j {
            if primitives(&self.indices[i]).centroid()[axis] < split_pos {
                i += 1;
            } else {
                self.indices.swap(i, j);
                j -= 1;
            }
        }
        if i == range.start || i == range.end {
            return;
        }
        let left_child = self.nodes.len();
        let right_child = self.nodes.len() + 1;
        self.nodes[node_idx] = BvhNode::non_leaf(aabb, left_child);
        self.nodes.push(BvhNode::leaf(
            self.indices[range.start..i]
                .iter()
                .map(|i| primitives(i).aabb())
                .collect(),
            range.start..i,
        ));
        self.nodes.push(BvhNode::leaf(
            self.indices[i..range.end]
                .iter()
                .map(|i| primitives(i).aabb())
                .collect(),
            i..range.end,
        ));
        self.subdivide(primitives, left_child);
        self.subdivide(primitives, right_child);
    }
    pub fn aabb_query<'a>(&'a self, aabb: &'a Aabb) -> AabbQuery<'a, I> {
        AabbQuery {
            aabb: &aabb,
            bvh: self,
            stack: vec![&self.nodes[0]],
            leaf_idx: 0,
        }
    }
}

impl Bvh<usize> {
    pub fn from_mesh(mesh: &Mesh) -> Self {
        Bvh::from_primitives((0..mesh.triangle_indices.len()).collect(), |i| {
            mesh.triangle(*i)
        })
    }
}

pub struct RayCaster {
    origin: Vec3,
    direction: Vec3,
    inv_direction: Vec3,
    min_t: Cell<f64>,
}

pub struct RayCasterFrame<'a, I> {
    ray_caster: &'a RayCaster,
    bvh: &'a Bvh<I>,
    stack: Vec<&'a BvhNode>,
    leaf_idx: usize,
}

impl RayCaster {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        RayCaster {
            origin,
            direction,
            inv_direction: Vec3::cw_div([1.0, 1.0, 1.0].into(), direction),
            min_t: f64::INFINITY.into(),
        }
    }
    pub fn intersect_bvh<'a, I>(&'a self, bvh: &'a Bvh<I>) -> RayCasterFrame<'a, I> {
        RayCasterFrame {
            ray_caster: self,
            bvh,
            stack: vec![&bvh.nodes[0]],
            leaf_idx: 0,
        }
    }
    pub fn intersect_triangle(&self, triangle: &Triangle) -> bool {
        let v1 = triangle.vertices[1] - triangle.vertices[0];
        let v2 = triangle.vertices[2] - triangle.vertices[0];
        let h = self.direction.cross(v2);
        let det = v1 * h;
        if det <= f64::EPSILON && det >= -f64::EPSILON {
            return false;
        }
        let inv_det = 1.0 / det;
        let s = self.origin - triangle.vertices[0];
        let u = inv_det * (s * h);
        if u < 0.0 || u > 1.0 {
            return false;
        }
        let q = s.cross(v1);
        let v = inv_det * (self.direction * q);
        if v < 0.0 || u + v > 1.0 {
            return false;
        }
        let t = inv_det * (v2 * q);
        if t < self.min_t.get() {
            self.min_t.set(t);
            true
        } else {
            false
        }
    }
    fn intersect_aabb(&self, aabb: &Aabb) -> f64 {
        let Aabb { min, max } = aabb.clone();
        let t1 = (min - self.origin).cw_mul(self.inv_direction);
        let t2 = (max - self.origin).cw_mul(self.inv_direction);
        let tmin = Vec3::cw_min(t1, t2).xyz_max();
        let tmax = Vec3::cw_max(t1, t2).xyz_min();
        if tmax >= tmin && tmin < self.min_t.get() && tmax > 0.0 {
            tmin
        } else {
            f64::INFINITY
        }
    }
}

impl<'a, I> Iterator for RayCasterFrame<'a, I> {
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.last() {
            if node.is_leaf() {
                let range = node.range();
                if self.leaf_idx >= range.len() {
                    self.leaf_idx = 0;
                    self.stack.pop();
                } else {
                    let r = &self.bvh.indices[range.start + self.leaf_idx];
                    self.leaf_idx += 1;
                    return Some(r);
                }
            } else {
                let mut c1 = &self.bvh.nodes[node.left_or_first];
                let mut c2 = &self.bvh.nodes[node.left_or_first + 1];
                let mut d1 = self.ray_caster.intersect_aabb(&c1.aabb);
                let mut d2 = self.ray_caster.intersect_aabb(&c2.aabb);
                self.stack.pop();
                if d1 > d2 {
                    std::mem::swap(&mut d1, &mut d2);
                    std::mem::swap(&mut c1, &mut c2);
                }
                if d1 < f64::INFINITY {
                    if d2 < f64::INFINITY {
                        self.stack.push(c2);
                    }
                    self.stack.push(c1);
                }
            }
        }
        None
    }
}

pub struct AabbQuery<'a, I> {
    aabb: &'a Aabb,
    bvh: &'a Bvh<I>,
    stack: Vec<&'a BvhNode>,
    leaf_idx: usize,
}

impl<'a, I> Iterator for AabbQuery<'a, I> {
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.last() {
            if node.is_leaf() {
                let range = node.range();
                if self.leaf_idx >= range.len() {
                    self.leaf_idx = 0;
                    self.stack.pop();
                } else {
                    let idx = &self.bvh.indices[range.start + self.leaf_idx];
                    self.leaf_idx += 1;
                    return Some(idx);
                }
            } else {
                let left = &self.bvh.nodes[node.left_or_first];
                let right = &self.bvh.nodes[node.left_or_first + 1];
                self.stack.pop();
                if self.aabb.intersects(&right.aabb) {
                    self.stack.push(right);
                }
                if self.aabb.intersects(&left.aabb) {
                    self.stack.push(left);
                }
            }
        }
        None
    }
}

impl BvhPrimitive for Triangle {
    fn aabb(&self) -> Aabb {
        let mut aabb = Aabb::empty();
        aabb.grow(self.vertices[0]);
        aabb.grow(self.vertices[1]);
        aabb.grow(self.vertices[2]);
        aabb
    }
    fn centroid(&self) -> Vec3 {
        (self.vertices[0] + self.vertices[1] + self.vertices[2]) * (1.0 / 3.0)
    }
}

impl<P: BvhPrimitive> BvhPrimitive for Bvh<P> {
    fn aabb(&self) -> Aabb {
        self.nodes[0].aabb
    }
    fn centroid(&self) -> Vec3 {
        let Aabb { min, max } = self.aabb();
        (min + max) * 0.5
    }
}
