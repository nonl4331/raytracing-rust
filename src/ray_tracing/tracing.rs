use crate::acceleration::aabb::Aabb;
use crate::math::Float;

use crate::math::{gamma, next_float, previous_float};

use crate::ray_tracing::{
    material::{Material, MaterialTrait},
    primitives::{AACuboid, AARect, Axis, MeshTriangle, Primitive, Sphere, Triangle},
    ray::Ray,
};

use std::f32::consts::PI;

use std::sync::Arc;

use ultraviolet::{Vec2, Vec3};

pub const EPSILON: Float = 0.00000003;

pub struct Hit {
    pub t: Float,
    pub point: Vec3,
    pub error: Vec3,
    pub normal: Vec3,
    pub uv: Option<Vec2>,
    pub out: bool,
    pub material: Arc<Material>,
}

pub trait Intersection {
    fn get_int(&self, _: &Ray) -> Option<Hit> {
        None
    }
    fn does_int(&self, ray: &Ray) -> bool {
        self.get_int(ray).is_some()
    }
}

pub trait PrimitiveTrait: Intersection {
    fn get_internal(self) -> Vec<Primitive>;
    fn get_aabb(&self) -> Option<Aabb> {
        None
    }
    fn requires_uv(&self) -> bool {
        false
    }
    fn get_uv(&self, _: Vec3) -> Option<Vec2> {
        None
    }
}

pub fn offset_ray(origin: Vec3, normal: Vec3, error: Vec3, is_brdf: bool) -> Vec3 {
    let offset_val = normal.abs().dot(error);
    let mut offset = offset_val * normal;

    if !is_brdf {
        offset = -offset;
    }

    let mut new_origin = origin + offset;

    if offset.x > 0.0 {
        new_origin.x = next_float(new_origin.x);
    } else {
        new_origin.x = previous_float(new_origin.x);
    }

    if offset.y > 0.0 {
        new_origin.y = next_float(new_origin.y);
    } else {
        new_origin.y = previous_float(new_origin.y);
    }

    if offset.z > 0.0 {
        new_origin.z = next_float(new_origin.z);
    } else {
        new_origin.z = previous_float(new_origin.z);
    }

    new_origin
}

pub fn check_side(normal: &mut Vec3, ray_direction: &Vec3) -> bool {
    if normal.dot(*ray_direction) > 0.0 {
        *normal = -*normal;
        false
    } else {
        true
    }
}

impl Intersection for Primitive {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_int(ray),
            Primitive::AARect(rect) => rect.get_int(ray),
            Primitive::AACuboid(aab) => aab.get_int(ray),
            Primitive::Triangle(triangle) => triangle.get_int(ray),
            Primitive::MeshTriangle(triangle) => triangle.get_int(ray),
            Primitive::None => panic!("get_int called on PrimitiveNone"),
        }
    }

    fn does_int(&self, ray: &Ray) -> bool {
        match self {
            Primitive::Sphere(sphere) => sphere.does_int(ray),
            Primitive::AARect(rect) => rect.does_int(ray),
            Primitive::AACuboid(aab) => aab.does_int(ray),
            Primitive::Triangle(triangle) => triangle.does_int(ray),
            Primitive::MeshTriangle(triangle) => triangle.does_int(ray),
            Primitive::None => panic!("does_int called on PrimitiveNone"),
        }
    }
}

impl PrimitiveTrait for Primitive {
    fn get_internal(self) -> Vec<Primitive> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_internal(),
            Primitive::AARect(rect) => rect.get_internal(),
            Primitive::AACuboid(aab) => aab.get_internal(),
            Primitive::Triangle(triangle) => triangle.get_internal(),
            Primitive::MeshTriangle(triangle) => triangle.get_internal(),
            Primitive::None => panic!("get_internal called on PrimitiveNone"),
        }
    }

    fn get_aabb(&self) -> Option<Aabb> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_aabb(),
            Primitive::AARect(rect) => rect.get_aabb(),
            Primitive::AACuboid(aab) => aab.get_aabb(),
            Primitive::Triangle(triangle) => triangle.get_aabb(),
            Primitive::MeshTriangle(triangle) => triangle.get_aabb(),
            Primitive::None => panic!("get_aabb called on PrimitiveNone"),
        }
    }
    fn get_uv(&self, point: Vec3) -> Option<Vec2> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_uv(point),
            Primitive::AARect(rect) => rect.get_uv(point),
            Primitive::AACuboid(aab) => aab.get_uv(point),
            Primitive::Triangle(triangle) => triangle.get_uv(point),
            Primitive::MeshTriangle(triangle) => triangle.get_uv(point),
            Primitive::None => panic!("get_uv called on PrimitiveNone"),
        };
        None
    }
    fn requires_uv(&self) -> bool {
        match self {
            Primitive::Sphere(sphere) => (*sphere.material).requires_uv(),
            Primitive::AARect(rect) => rect.material.requires_uv(),
            Primitive::AACuboid(aab) => aab.material.requires_uv(),
            Primitive::Triangle(triangle) => triangle.material.requires_uv(),
            Primitive::MeshTriangle(triangle) => triangle.material.requires_uv(),
            Primitive::None => panic!("requires_uv called on PrimitiveNone"),
        }
    }
    /*fn is_brdf(&self) -> bool {
        match self {
            Primitive::Sphere(sphere) => (*sphere.material).is_brdf(),
            Primitive::AARect(rect) => rect.material.is_brdf(),
            Primitive::AACuboid(aab) => aab.material.is_brdf(),
            Primitive::Triangle(triangle) => triangle.material.is_brdf(),
            Primitive::MeshTriangle(triangle) => triangle.material.is_brdf(),
            Primitive::None => panic!("requires_uv called on PrimitiveNone"),
        }
    }*/
}

impl Intersection for Sphere {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        crate::ray_tracing::intersection::sphere::sphere_intersection(self, ray)
    }
}

#[allow(clippy::suspicious_operation_groupings)]
impl PrimitiveTrait for Sphere {
    fn get_internal(self) -> Vec<Primitive> {
        vec![Primitive::Sphere(self)]
    }
    fn get_uv(&self, point: Vec3) -> Option<Vec2> {
        if self.material.requires_uv() {
            let x = (self.center.x - point.x) / self.radius;
            let y = (self.center.y - point.y) / self.radius;
            let z = (self.center.z - point.z) / self.radius;
            let phi = (-1.0 * z).atan2(x) + PI;
            let theta = (-1.0 * y).acos();

            return Some(Vec2::new(phi / (2.0 * PI), theta / PI));
        }
        None
    }
    fn get_aabb(&self) -> Option<Aabb> {
        Some(Aabb::new(
            self.center - self.radius * Vec3::one(),
            self.center + self.radius * Vec3::one(),
        ))
    }
}

impl Intersection for AARect {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        let t = (self.k - self.axis.get_axis_value(ray.origin))
            / self.axis.get_axis_value(ray.direction);
        let point = ray.at(t);
        let point_2d = self.axis.point_without_axis(point);

        // x & y are not the x & y axis but rather the two axis that are not self.axis
        if point_2d.x > self.min.x
            && point_2d.x < self.max.x
            && point_2d.y > self.min.y
            && point_2d.y < self.max.y
        {
            Some(Hit {
                t,
                point: point + EPSILON * self.axis.return_point_with_axis(Vec3::one()),
                error: Vec3::zero(),
                normal: self
                    .axis
                    .return_point_with_axis(-1.0 * ray.direction)
                    .normalized(),
                uv: self.get_uv(point),
                out: true,
                material: self.material.clone(),
            })
        } else {
            None
        }
    }

    fn does_int(&self, ray: &Ray) -> bool {
        let t = (self.k - self.axis.get_axis_value(ray.origin))
            / self.axis.get_axis_value(ray.direction);
        let point = ray.at(t);
        let point_2d = self.axis.point_without_axis(point);

        // x & y are not the x & y axis but rather the two axis that are not self.axis
        point_2d.x > self.min.x
            && point_2d.x < self.max.x
            && point_2d.y > self.min.y
            && point_2d.y < self.max.y
    }
}

impl PrimitiveTrait for AARect {
    fn get_internal(self) -> Vec<Primitive> {
        vec![Primitive::AARect(self)]
    }
    fn get_uv(&self, point: Vec3) -> Option<Vec2> {
        if self.material.requires_uv() {
            let pwa = self.axis.point_without_axis(point);
            return Some(Vec2::new(
                (pwa.x - self.min.x) / self.max.x,
                (pwa.y - self.min.y) / self.max.y,
            ));
        }
        None
    }
    fn get_aabb(&self) -> Option<Aabb> {
        Some(Aabb::new(
            Axis::point_from_2d(&self.min, &self.axis, self.k - 0.0001),
            Axis::point_from_2d(&self.max, &self.axis, self.k + 0.0001),
        ))
    }
}

impl Intersection for AACuboid {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        let mut hit: Option<Hit> = None;
        for side in self.rects.iter() {
            if let Some(current_hit) = side.get_int(ray) {
                // make sure ray is going forwards
                if current_hit.t > 0.0 {
                    // check if hit already exists
                    if hit.is_some() {
                        // check if t value is close to 0 than previous hit
                        if current_hit.t < hit.as_ref().unwrap().t {
                            hit = Some(current_hit);
                        }
                        continue;
                    }

                    // if hit doesn't exist set current hit to hit
                    hit = Some(current_hit);
                }
            }
        }
        hit
    }

    fn does_int(&self, ray: &Ray) -> bool {
        for side in self.rects.iter() {
            if side.does_int(ray) {
                return true;
            }
        }
        false
    }
}

impl PrimitiveTrait for AACuboid {
    fn get_internal(mut self) -> Vec<Primitive> {
        self.rects
            .iter_mut()
            .map(|rect| Primitive::AARect(rect.clone()))
            .collect()
    }

    fn get_aabb(&self) -> Option<Aabb> {
        None
    }
}

impl Intersection for Triangle {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        let mut p0t = self.points[0] - ray.origin;
        let mut p1t = self.points[1] - ray.origin;
        let mut p2t = self.points[2] - ray.origin;

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

        let delta_t = 3.0
            * (gamma(3) * max_e * max_z_t + delta_e * max_z_t + delta_z * max_e * inv_det.abs());

        if t < delta_t {
            return None;
        }

        let uv = b0 * Vec2::new(0.0, 0.0) + b1 * Vec2::new(1.0, 0.0) + b2 * Vec2::new(1.0, 1.0);

        let mut normal = b0 * self.normals[0] + b1 * self.normals[1] + b2 * self.normals[2];

        let out = check_side(&mut normal, &ray.direction);

        let x_abs_sum = (b0 * self.points[0].x).abs() + (b1 * self.points[1].x).abs();
        let y_abs_sum = (b0 * self.points[0].y).abs() + (b1 * self.points[1].y).abs();
        let z_abs_sum = (b0 * self.points[0].z).abs() + (b1 * self.points[1].z).abs();

        let point_error = gamma(7) * Vec3::new(x_abs_sum, y_abs_sum, z_abs_sum)
            + gamma(6)
                * Vec3::new(
                    b2 * self.points[2].x,
                    b2 * self.points[2].y,
                    b2 * self.points[2].z,
                );

        let point = b0 * self.points[0] + b1 * self.points[1] + b2 * self.points[2];

        Some(Hit {
            t,
            point,
            error: point_error,
            normal,
            uv: Some(uv),
            out,
            material: self.material.clone(),
        })
    }
}

impl PrimitiveTrait for Triangle {
    fn get_internal(self) -> Vec<Primitive> {
        vec![Primitive::Triangle(self)]
    }

    fn get_aabb(&self) -> Option<Aabb> {
        Some(Aabb::new(
            self.points[0].min_by_component(self.points[1].min_by_component(self.points[2])),
            self.points[0].max_by_component(self.points[1].max_by_component(self.points[2])),
        ))
    }
}

impl Intersection for MeshTriangle {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        let points = [
            (*self.mesh).vertices[self.point_indices[0]],
            (*self.mesh).vertices[self.point_indices[1]],
            (*self.mesh).vertices[self.point_indices[2]],
        ];
        let normals = [
            (*self.mesh).normals[self.normal_indices[0]],
            (*self.mesh).normals[self.normal_indices[1]],
            (*self.mesh).normals[self.normal_indices[2]],
        ];

        let mut p0t = points[0] - ray.origin;
        let mut p1t = points[1] - ray.origin;
        let mut p2t = points[2] - ray.origin;

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

        let delta_t = 3.0
            * (gamma(3) * max_e * max_z_t + delta_e * max_z_t + delta_z * max_e * inv_det.abs());

        if t < delta_t {
            return None;
        }

        let uv = b0 * Vec2::new(0.0, 0.0) + b1 * Vec2::new(1.0, 0.0) + b2 * Vec2::new(1.0, 1.0);

        let mut normal = b0 * normals[0] + b1 * normals[1] + b2 * normals[2];

        let out = check_side(&mut normal, &ray.direction);

        let x_abs_sum = (b0 * points[0].x).abs() + (b1 * points[1].x).abs();
        let y_abs_sum = (b0 * points[0].y).abs() + (b1 * points[1].y).abs();
        let z_abs_sum = (b0 * points[0].z).abs() + (b1 * points[1].z).abs();

        let point_error = gamma(7) * Vec3::new(x_abs_sum, y_abs_sum, z_abs_sum)
            + gamma(6) * Vec3::new(b2 * points[2].x, b2 * points[2].y, b2 * points[2].z);

        let point = b0 * points[0] + b1 * points[1] + b2 * points[2];

        Some(Hit {
            t,
            point,
            error: point_error,
            normal,
            uv: Some(uv),
            out,
            material: self.material.clone(),
        })
    }
}

impl PrimitiveTrait for MeshTriangle {
    fn get_internal(self) -> Vec<Primitive> {
        vec![Primitive::MeshTriangle(self)]
    }

    fn get_aabb(&self) -> Option<Aabb> {
        let points = [
            (*self.mesh).vertices[self.point_indices[0]],
            (*self.mesh).vertices[self.point_indices[1]],
            (*self.mesh).vertices[self.point_indices[2]],
        ];

        Some(Aabb::new(
            points[0].min_by_component(points[1].min_by_component(points[2])),
            points[0].max_by_component(points[1].max_by_component(points[2])),
        ))
    }
}
