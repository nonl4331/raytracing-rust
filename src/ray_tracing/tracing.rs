use crate::bvh::aabb::AABB;

use crate::math::{next_float, previous_float};

use crate::ray_tracing::{
    material::{Material, MaterialTrait},
    primitives::{AACuboid, AARect, Axis, Primitive, Sphere, Triangle, TriangleMesh},
    ray::Ray,
};

use std::f32::consts::PI;

use std::sync::Arc;

use ultraviolet::{Vec2, Vec3};

pub const EPSILON: f32 = 0.0001;

pub struct Hit {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Option<Vec2>,
    pub out: bool,
    pub material: Arc<Material>,
}

pub trait PrimitiveTrait {
    fn get_int(&self, _: &Ray) -> Option<Hit> {
        None
    }
    fn does_int(&self, _: &Ray) -> bool {
        false
    }
    fn get_internal(self) -> Vec<Primitive>;
    fn get_aabb(&self) -> Option<AABB> {
        None
    }
    fn requires_uv(&self) -> bool {
        false
    }
    fn get_uv(&self, _: Vec3) -> Option<Vec2> {
        None
    }
}

fn offset_ray(origin: Vec3, normal: Vec3, error: Vec3, incoming_dir: Vec3) -> Vec3 {
    let offset_val = normal.abs().dot(error);
    let mut offset = offset_val * normal;

    if normal.dot(incoming_dir) < 0.0 {
        offset = -offset;
    }

    let mut new_origin = origin + offset;

    if new_origin.x > 0.0 {
        new_origin.x = next_float(new_origin.x);
    } else {
        new_origin.x = previous_float(new_origin.x);
    }

    if new_origin.y > 0.0 {
        new_origin.y = next_float(new_origin.y);
    } else {
        new_origin.y = previous_float(new_origin.y);
    }

    if new_origin.z > 0.0 {
        new_origin.z = next_float(new_origin.z);
    } else {
        new_origin.z = previous_float(new_origin.z);
    }

    new_origin
}

impl PrimitiveTrait for Primitive {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_int(ray),
            Primitive::AARect(rect) => rect.get_int(ray),
            Primitive::AACuboid(aab) => aab.get_int(ray),
            Primitive::Triangle(triangle) => triangle.get_int(ray),
            Primitive::TriangleMesh(mesh) => mesh.get_int(ray),
            Primitive::None => panic!("get_int called on PrimitiveNone"),
        }
    }

    fn does_int(&self, ray: &Ray) -> bool {
        match self {
            Primitive::Sphere(sphere) => sphere.does_int(ray),
            Primitive::AARect(rect) => rect.does_int(ray),
            Primitive::AACuboid(aab) => aab.does_int(ray),
            Primitive::Triangle(triangle) => triangle.does_int(ray),
            Primitive::TriangleMesh(mesh) => mesh.does_int(ray),
            Primitive::None => panic!("does_int called on PrimitiveNone"),
        }
    }

    fn get_internal(self) -> Vec<Primitive> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_internal(),
            Primitive::AARect(rect) => rect.get_internal(),
            Primitive::AACuboid(aab) => aab.get_internal(),
            Primitive::Triangle(triangle) => triangle.get_internal(),
            Primitive::TriangleMesh(mesh) => mesh.get_internal(),
            Primitive::None => panic!("get_internal called on PrimitiveNone"),
        }
    }

    fn get_aabb(&self) -> Option<AABB> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_aabb(),
            Primitive::AARect(rect) => rect.get_aabb(),
            Primitive::AACuboid(aab) => aab.get_aabb(),
            Primitive::Triangle(triangle) => triangle.get_aabb(),
            Primitive::TriangleMesh(mesh) => mesh.get_aabb(),
            Primitive::None => panic!("get_aabb called on PrimitiveNone"),
        }
    }
    fn get_uv(&self, point: Vec3) -> Option<Vec2> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_uv(point),
            Primitive::AARect(rect) => rect.get_uv(point),
            Primitive::AACuboid(aab) => aab.get_uv(point),
            Primitive::Triangle(triangle) => triangle.get_uv(point),
            Primitive::TriangleMesh(mesh) => mesh.get_uv(point),
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
            Primitive::TriangleMesh(mesh) => mesh.material.requires_uv(),
            Primitive::None => panic!("requires_uv called on PrimitiveNone"),
        }
    }
}

impl PrimitiveTrait for Sphere {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let h = ray.direction.dot(oc); // b/2
        let c = oc.dot(oc) - self.radius * self.radius;
        let disc = h * h - a * c;
        if disc > 0.0 {
            let mut t = (-h - disc.sqrt()) / a;

            if t < 0.0 {
                t = (-h + disc.sqrt()) / a;
            }

            let point = ray.at(t);
            let mut normal = (point - self.center) / self.radius;
            let mut out = true;
            if normal.dot(ray.direction) > 0.0 {
                normal *= -1.0;
                out = false;
            }
            Some(Hit {
                t,
                point: point + EPSILON * normal,
                normal,
                uv: self.get_uv(point),
                out,
                material: self.material.clone(),
            })
        } else {
            None
        }
    }
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
    fn get_aabb(&self) -> Option<AABB> {
        Some(AABB::new(
            self.center - self.radius * Vec3::one(),
            self.center + self.radius * Vec3::one(),
        ))
    }
}

impl PrimitiveTrait for AARect {
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
    fn get_aabb(&self) -> Option<AABB> {
        Some(AABB::new(
            Axis::point_from_2d(&self.min, &self.axis, self.k - 0.0001),
            Axis::point_from_2d(&self.max, &self.axis, self.k + 0.0001),
        ))
    }
}

impl PrimitiveTrait for AACuboid {
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
    fn get_internal(mut self) -> Vec<Primitive> {
        self.rects
            .iter_mut()
            .map(|rect| Primitive::AARect(rect.clone()))
            .collect()
    }

    fn does_int(&self, ray: &Ray) -> bool {
        for side in self.rects.iter() {
            if side.does_int(ray) {
                return true;
            }
        }
        false
    }
    fn get_aabb(&self) -> Option<AABB> {
        None
    }
}

impl PrimitiveTrait for Triangle {
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
            e0 = (p1t.x as f64 * p2t.y as f64 - p1t.y as f64 * p2t.x as f64) as f32;
            e1 = (p2t.x as f64 * p0t.y as f64 - p2t.y as f64 * p0t.x as f64) as f32;
            e2 = (p0t.x as f64 * p1t.y as f64 - p0t.y as f64 * p1t.x as f64) as f32;
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

        let uv = b0 * Vec2::new(0.0, 0.0) + b1 * Vec2::new(1.0, 0.0) + b2 * Vec2::new(1.0, 1.0);

        let mut normal = b0 * self.normals[0] + b1 * self.normals[1] + b2 * self.normals[2];

        let mut out = true;
        if normal.dot(ray.direction) > 0.0 {
            normal *= -1.0;
            out = false;
        }

        let point =
            b0 * self.points[0] + b1 * self.points[1] + b2 * self.points[2] + 0.000000001 * normal;

        Some(Hit {
            t,
            point,
            normal,
            uv: Some(uv),
            out,
            material: self.material.clone(),
        })
    }
    fn get_internal(self) -> Vec<Primitive> {
        vec![Primitive::Triangle(self)]
    }
    fn does_int(&self, ray: &Ray) -> bool {
        self.get_int(ray).is_some()
    }
    fn get_aabb(&self) -> Option<AABB> {
        Some(AABB::new(
            self.points[0].min_by_component(self.points[1].min_by_component(self.points[2])),
            self.points[0].max_by_component(self.points[1].max_by_component(self.points[2])),
        ))
    }
}

impl PrimitiveTrait for TriangleMesh {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        let mut hit: Option<Hit> = None;
        for side in self.mesh.iter() {
            if let Some(current_hit) = side.get_int(ray) {
                // make sure ray is going forwards
                if current_hit.t > EPSILON {
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
    fn get_internal(mut self) -> Vec<Primitive> {
        self.mesh
            .iter_mut()
            .map(|triangle| Primitive::Triangle(triangle.clone()))
            .collect()
    }

    fn does_int(&self, ray: &Ray) -> bool {
        for triangle in self.mesh.iter() {
            if triangle.does_int(ray) {
                return true;
            }
        }
        false
    }
    fn get_aabb(&self) -> Option<AABB> {
        Some(AABB::new(self.min, self.max))
    }
}
