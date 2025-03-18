use crate::{geometry::Vec3, mesh::Triangle};

pub fn closest_point_on_line_segment(a: Vec3, b: Vec3, p: Vec3) -> Vec3 {
    let ab = b - a;
    let t = (p - a) * ab / (ab * ab);
    a + ab * t.clamp(0.0, 1.0)
}

pub fn closest_point_on_triangle(
    triangle: &Triangle,
    point: Vec3,
    max_distance: f64,
) -> Option<Vec3> {
    let [p0, p1, p2] = triangle.vertices;
    let normal = Vec3::cross(p1 - p0, p2 - p0).normalize();
    let dist = (point - p0) * normal;
    if dist.abs() > max_distance {
        return None;
    }
    let p = point - normal * dist;
    let c0 = Vec3::cross(p - p0, p1 - p0);
    let c1 = Vec3::cross(p - p1, p2 - p1);
    let c2 = Vec3::cross(p - p2, p0 - p2);
    if c0 * normal <= 0.0 && c1 * normal <= 0.0 && c2 * normal <= 0.0 {
        return Some(p);
    }

    let q0 = closest_point_on_line_segment(p0, p1, point);
    let q1 = closest_point_on_line_segment(p1, p2, point);
    let q2 = closest_point_on_line_segment(p2, p0, point);
    let max_distance_sq = max_distance * max_distance;

    [q0, q1, q2]
        .into_iter()
        .map(|q| (q, q.dist_sq(point)))
        .filter(|x| x.1 <= max_distance_sq)
        .min_by(|a, b| f64::total_cmp(&a.1, &b.1))
        .map(|x| x.0)
}

pub fn barycentric_coordinates(triangle: &Triangle, point: Vec3) -> (f64, f64, f64) {
    let [p0, p1, p2] = triangle.vertices;
    let v0 = p1 - p0;
    let v1 = p2 - p0;
    let v2 = point - p0;
    let d00 = v0 * v0;
    let d01 = v0 * v1;
    let d02 = v0 * v2;
    let d11 = v1 * v1;
    let d12 = v1 * v2;
    let f = 1.0 / (d00 * d11 - d01 * d01);
    let s = (d11 * d02 - d01 * d12) * f;
    let t = (d00 * d12 - d01 * d02) * f;
    let r = 1.0 - s - t;
    (r, s, t)
}

pub fn closest_points_of_line_segments(p1: Vec3, q1: Vec3, p2: Vec3, q2: Vec3) -> (Vec3, Vec3) {
    let d1 = q1 - p1;
    let d2 = q2 - p2;
    let r = p1 - p2;
    let a = d1 * d1;
    let e = d2 * d2;
    let c = d1 * r;
    let f = d2 * r;
    let b = d1 * d2;
    let denom = a * e - b * b;
    let mut s = if denom != 0.0 {
        ((b * f - c * e) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = (b * s + f) / e;
    if t < 0.0 {
        t = 0.0;
        s = (-c / a).clamp(0.0, 1.0);
    } else if t > 1.0 {
        t = 1.0;
        s = ((b - c) / a).clamp(0.0, 1.0);
    }
    (p1 + d1 * s, p2 + d2 * t)
}

pub fn intersection_of_triangle_and_line(triangle: &Triangle, p: Vec3, q: Vec3) -> Option<(f64, Vec3)> {
    let [p0, p1, p2] = triangle.vertices;
    let normal = Vec3::cross(p1 - p0, p2 - p0).normalize();
    let a = (p0 - p) * normal;
    let b = (q - p) * normal;
    if b.abs() < f64::EPSILON {
        None
    } else {
        let t = a / b;
        let v = p * (1.0 - t) + q * t;
        let (b0, b1, b2) = barycentric_coordinates(triangle, v);
        (b0 >= 0.0 && b1 >= 0.0 && b2 >= 0.0).then_some((t, v))
    }
}

pub fn closest_points_of_triangle_and_line_segment(
    triangle: &Triangle,
    p: Vec3,
    q: Vec3,
    max_distance: f64,
) -> Option<(Vec3, Vec3, f64)> {
    let [p0, p1, p2] = triangle.vertices;
    let mut best_distance_sq = max_distance * max_distance;
    let mut result = None;
    // TODO: i'm pretty sure this function could be streamlined...
    if let Some((t, v)) = intersection_of_triangle_and_line(triangle, p, q) {
        if t >= 0.0 && t <= 1.0 {
            return Some((v, v, 0.0));
        }
    }
    let mut f = |px, qx| {
        let (a, b) = closest_points_of_line_segments(px, qx, p, q);
        let d_sq = a.dist_sq(b);
        if d_sq <= best_distance_sq {
            result = Some((a, b));
            best_distance_sq = d_sq;
        }
    };
    f(p0, p1);
    f(p1, p2);
    f(p2, p0);
    let mut g = |px| {
        let (r, s, t) = barycentric_coordinates(triangle, px);
        if r >= 0.0 && s >= 0.0 && t >= 0.0 {
            let a = p0 * r + p1 * s + p2 * t;
            let d_sq = px.dist_sq(a);
            if d_sq <= best_distance_sq {
                result = Some((a, px));
                best_distance_sq = d_sq;
            }
        }
    };
    g(p);
    g(q);
    result.map(|x| (x.0, x.1, best_distance_sq.sqrt()))
}
