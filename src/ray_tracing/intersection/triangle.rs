use crate::ray_tracing::material::Material;
use crate::ray_tracing::ray::Ray;

use crate::utility::vec::Vec2;
use crate::utility::vec::Vec3;
use std::sync::Arc;

use crate::ray_tracing::primitives::Triangle;

use crate::ray_tracing::primitives::MeshTriangle;

use crate::ray_tracing::primitives::Axis;

use crate::ray_tracing::tracing::Hit;

use crate::utility::math::Float;

use crate::utility::math::gamma;

use crate::ray_tracing::tracing::check_side;

const TRIANGLE_INTERSECTION: TriangleIntersection = TriangleIntersection::One;

enum TriangleIntersection {
    One,
}

pub trait TriangleTrait {
    fn get_point(&self, index: usize) -> Vec3;
    fn get_normal(&self, index: usize) -> Vec3;
    fn get_material(&self) -> Arc<Material>;
}

impl TriangleTrait for Triangle {
    fn get_point(&self, index: usize) -> Vec3 {
        self.points[index]
    }
    fn get_normal(&self, index: usize) -> Vec3 {
        self.normals[index]
    }
    fn get_material(&self) -> Arc<Material> {
        self.material.clone()
    }
}

impl TriangleTrait for MeshTriangle {
    fn get_point(&self, index: usize) -> Vec3 {
        (*self.mesh).vertices[self.point_indices[index]]
    }
    fn get_normal(&self, index: usize) -> Vec3 {
        (*self.mesh).normals[self.normal_indices[index]]
    }
    fn get_material(&self) -> Arc<Material> {
        self.material.clone()
    }
}

pub fn triangle_intersection<T: TriangleTrait>(triangle: &T, ray: &Ray) -> Option<Hit> {
    match TRIANGLE_INTERSECTION {
        TriangleIntersection::One => triangle_intersection_one::<T>(triangle, ray),
    }
}

fn triangle_intersection_one<T: TriangleTrait>(triangle: &T, ray: &Ray) -> Option<Hit> {
    let mut p0t = triangle.get_point(0) - ray.origin;
    let mut p1t = triangle.get_point(1) - ray.origin;
    let mut p2t = triangle.get_point(2) - ray.origin;

    let max_axis = Axis::get_max_abs_axis(&ray.direction);
    Axis::swap_z(&mut p0t, &max_axis);
    Axis::swap_z(&mut p1t, &max_axis);
    Axis::swap_z(&mut p2t, &max_axis);

    p0t.x += ray.shear.x * p0t.z;
    p0t.y += ray.shear.y * p0t.z;
    p1t.x += ray.shear.x * p1t.z;
    p1t.y += ray.shear.y * p1t.z;
    p2t.x += ray.shear.x * p2t.z;
    p2t.y += ray.shear.y * p2t.z;

    let mut e0 = p1t.x * p2t.y - p1t.y * p2t.x;
    let mut e1 = p2t.x * p0t.y - p2t.y * p0t.x;
    let mut e2 = p0t.x * p1t.y - p0t.y * p1t.x;
    if e0 == 0.0 || e1 == 0.0 || e2 == 0.0 {
        e0 = (p1t.x as f64 * p2t.y as f64 - p1t.y as f64 * p2t.x as f64) as Float;
        e1 = (p2t.x as f64 * p0t.y as f64 - p2t.y as f64 * p0t.x as f64) as Float;
        e2 = (p0t.x as f64 * p1t.y as f64 - p0t.y as f64 * p1t.x as f64) as Float;
    }

    if (e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0) {
        return None;
    }

    let det = e0 + e1 + e2;
    if det == 0.0 {
        return None;
    }

    p0t *= ray.shear.z;
    p1t *= ray.shear.z;
    p2t *= ray.shear.z;

    let t_scaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;
    if (det < 0.0 && t_scaled >= 0.0) || (det > 0.0 && t_scaled <= 0.0) {
        return None;
    }

    let inv_det = 1.0 / det;

    let b0 = e0 * inv_det;
    let b1 = e1 * inv_det;
    let b2 = e2 * inv_det;

    let t = inv_det * t_scaled;

    let max_z_t = Vec3::new(p0t.z.abs(), p1t.z.abs(), p2t.z.abs()).component_max();
    let delta_z = gamma(3) * max_z_t;

    let max_x_t = Vec3::new(p0t.x.abs(), p1t.x.abs(), p2t.x.abs()).component_max();
    let max_y_t = Vec3::new(p0t.y.abs(), p1t.y.abs(), p2t.y.abs()).component_max();
    let delta_x = gamma(5) * (max_x_t + max_z_t);
    let delta_y = gamma(5) * (max_y_t + max_z_t);

    let delta_e = 2.0 * (gamma(2) * max_x_t * max_y_t + delta_y * max_x_t + delta_x + max_x_t);

    let max_e = Vec3::new(e0.abs(), e1.abs(), e2.abs()).component_max();

    let delta_t =
        3.0 * (gamma(3) * max_e * max_z_t + delta_e * max_z_t + delta_z * max_e * inv_det.abs());

    if t < delta_t {
        return None;
    }

    let uv = b0 * Vec2::new(0.0, 0.0) + b1 * Vec2::new(1.0, 0.0) + b2 * Vec2::new(1.0, 1.0);

    let mut normal =
        b0 * triangle.get_normal(0) + b1 * triangle.get_normal(1) + b2 * triangle.get_normal(2);

    let out = check_side(&mut normal, &ray.direction);

    let x_abs_sum = (b0 * triangle.get_point(0).x).abs() + (b1 * triangle.get_point(1).x).abs();
    let y_abs_sum = (b0 * triangle.get_point(0).y).abs() + (b1 * triangle.get_point(1).y).abs();
    let z_abs_sum = (b0 * triangle.get_point(0).z).abs() + (b1 * triangle.get_point(1).z).abs();

    let point_error = gamma(7) * Vec3::new(x_abs_sum, y_abs_sum, z_abs_sum)
        + gamma(6)
            * Vec3::new(
                b2 * triangle.get_point(2).x,
                b2 * triangle.get_point(2).y,
                b2 * triangle.get_point(2).z,
            );

    let point =
        b0 * triangle.get_point(0) + b1 * triangle.get_point(1) + b2 * triangle.get_point(2);

    Some(Hit {
        t,
        point,
        error: point_error,
        normal,
        uv: Some(uv),
        out,
        material: triangle.get_material(),
    })
}
