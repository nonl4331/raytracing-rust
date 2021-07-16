use crate::bvh::aabb::AABB;

use crate::math::near_zero;

use crate::ray_tracing::{
    material::{Material, MaterialTrait},
    primitives::{AABox, AARect, MovingSphere, Sphere},
    ray::Ray,
};

use std::f32::consts::PI;

use std::sync::Arc;

use ultraviolet::{Vec2, Vec3};

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

pub enum Primitive {
    Sphere(Sphere),
    MovingSphere(MovingSphere),
    AARect(AARect),
    AABox(AABox),
}

impl PrimitiveTrait for Primitive {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_int(ray),
            Primitive::MovingSphere(sphere) => sphere.get_int(ray),
            Primitive::AARect(rect) => rect.get_int(ray),
            Primitive::AABox(aab) => aab.get_int(ray),
        }
    }

    fn does_int(&self, ray: &Ray) -> bool {
        match self {
            Primitive::Sphere(sphere) => sphere.does_int(ray),
            Primitive::MovingSphere(sphere) => sphere.does_int(ray),
            Primitive::AARect(rect) => rect.does_int(ray),
            Primitive::AABox(aab) => aab.does_int(ray),
        }
    }

    fn get_internal(self) -> Vec<Primitive> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_internal(),
            Primitive::MovingSphere(sphere) => sphere.get_internal(),
            Primitive::AARect(rect) => rect.get_internal(),
            Primitive::AABox(aab) => aab.get_internal(),
        }
    }

    fn get_aabb(&self) -> Option<AABB> {
        match self {
            Primitive::Sphere(sphere) => Some(sphere.aabb),
            Primitive::MovingSphere(sphere) => Some(sphere.aabb),
            Primitive::AARect(rect) => Some(rect.aabb),
            Primitive::AABox(aab) => Some(aab.aabb),
        }
    }
    fn get_uv(&self, point: Vec3) -> Option<Vec2> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_uv(point),
            Primitive::MovingSphere(sphere) => sphere.get_uv(point),
            Primitive::AARect(rect) => rect.get_uv(point),
            Primitive::AABox(aab) => aab.get_uv(point),
        };
        None
    }
    fn requires_uv(&self) -> bool {
        match self {
            Primitive::Sphere(sphere) => (*sphere.material).requires_uv(),
            Primitive::MovingSphere(sphere) => sphere.material.requires_uv(),
            Primitive::AARect(rect) => rect.material.requires_uv(),
            Primitive::AABox(aab) => aab.material.requires_uv(),
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
            let point = ray.at(t);

            // check intersection with self
            if near_zero(point - ray.origin) || t < 0.0 {
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
                point,
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
}

impl PrimitiveTrait for MovingSphere {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        let time_center = self.start_pos + (self.end_pos - self.start_pos) * ray.time;
        let oc = ray.origin - time_center;
        let a = ray.direction.dot(ray.direction);
        let h = ray.direction.dot(oc); // b/2
        let c = oc.dot(oc) - self.radius * self.radius;
        let disc = h * h - a * c;
        if disc > 0.0 {
            let mut t = (-h - disc.sqrt()) / a;
            let point = ray.at(t);

            // check intersection with self
            if near_zero(point - ray.origin) || t < 0.0 {
                t = (-h + disc.sqrt()) / a;
            }

            let point = ray.at(t);
            let mut normal = (point - time_center) / self.radius;
            let mut out = true;
            if normal.dot(ray.direction) > 0.0 {
                normal *= -1.0;
                out = false;
            }
            Some(Hit {
                t,
                point,
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
        vec![Primitive::MovingSphere(self)]
    }
    fn get_uv(&self, point: Vec3) -> Option<Vec2> {
        if self.material.requires_uv() {
            let phi = (-1.0 * point.z).atan2(point.x) + PI;
            let theta = (-1.0 * point.y).acos();

            return Some(Vec2::new(phi / (2.0 * PI), theta / PI));
        }
        None
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
                point,
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
}

impl PrimitiveTrait for AABox {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        let mut hit: Option<Hit> = None;
        for side in self.rects.iter() {
            if let Some(current_hit) = side.get_int(ray) {
                // make sure ray is going forwards
                if current_hit.t > 0.001 {
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
}
