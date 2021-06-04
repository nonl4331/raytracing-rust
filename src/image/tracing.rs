use crate::image::aabb::AABB;
use crate::image::hittables::{AABox, AARect, MovingSphere, Sphere};
use crate::image::material::{Material, MaterialTrait};
use crate::image::math::near_zero;
use crate::image::ray::Ray;
use std::f64::consts::PI;
use std::sync::Arc;
use ultraviolet::{DVec2, DVec3};

pub struct Hit {
    pub t: f64,
    pub point: DVec3,
    pub normal: DVec3,
    pub uv: Option<DVec2>,
    pub out: bool,
    pub material: Arc<Material>,
}

pub trait HittableTrait {
    fn get_int(&self, _: &Ray) -> Option<Hit> {
        None
    }
    fn does_int(&self, _: &Ray) -> bool {
        false
    }
    fn get_aabb(&self) -> Option<AABB> {
        None
    }
    fn requires_uv(&self) -> bool {
        false
    }
    fn get_uv(&self, _: DVec3) -> Option<DVec2> {
        None
    }
}

pub enum Hittable {
    Sphere(Sphere),
    MovingSphere(MovingSphere),
    AARect(AARect),
    AABox(AABox),
}

impl HittableTrait for Hittable {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        match self {
            Hittable::Sphere(sphere) => sphere.get_int(ray),
            Hittable::MovingSphere(sphere) => sphere.get_int(ray),
            Hittable::AARect(rect) => rect.get_int(ray),
            Hittable::AABox(rect) => rect.get_int(ray),
        }
    }

    fn does_int(&self, ray: &Ray) -> bool {
        match self {
            Hittable::Sphere(sphere) => sphere.does_int(ray),
            Hittable::MovingSphere(sphere) => sphere.does_int(ray),
            Hittable::AARect(rect) => rect.does_int(ray),
            Hittable::AABox(rect) => rect.does_int(ray),
        }
    }

    fn get_aabb(&self) -> Option<AABB> {
        match self {
            Hittable::Sphere(sphere) => Some(sphere.aabb),
            Hittable::MovingSphere(sphere) => Some(sphere.aabb),
            Hittable::AARect(rect) => Some(rect.aabb),
            Hittable::AABox(rect) => Some(rect.aabb),
        }
    }
    fn get_uv(&self, point: DVec3) -> Option<DVec2> {
        match self {
            Hittable::Sphere(sphere) => sphere.get_uv(point),
            Hittable::MovingSphere(sphere) => sphere.get_uv(point),
            Hittable::AARect(rect) => rect.get_uv(point),
            Hittable::AABox(rect) => rect.get_uv(point),
        };
        None
    }
    fn requires_uv(&self) -> bool {
        match self {
            Hittable::Sphere(sphere) => (*sphere.material).requires_uv(),
            Hittable::MovingSphere(sphere) => sphere.material.requires_uv(),
            Hittable::AARect(rect) => rect.material.requires_uv(),
            Hittable::AABox(rect) => rect.material.requires_uv(),
        }
    }
}

impl HittableTrait for Sphere {
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
    fn get_uv(&self, point: DVec3) -> Option<DVec2> {
        if self.material.requires_uv() {
            let phi = (-1.0 * point.z).atan2(point.x) + PI;
            let theta = (-1.0 * point.y).acos();

            return Some(DVec2::new(phi / (2.0 * PI), theta / PI));
        }
        None
    }
}

impl HittableTrait for MovingSphere {
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
    fn get_uv(&self, point: DVec3) -> Option<DVec2> {
        if self.material.requires_uv() {
            let phi = (-1.0 * point.z).atan2(point.x) + PI;
            let theta = (-1.0 * point.y).acos();

            return Some(DVec2::new(phi / (2.0 * PI), theta / PI));
        }
        None
    }
}

impl HittableTrait for AARect {
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
    fn get_uv(&self, point: DVec3) -> Option<DVec2> {
        if self.material.requires_uv() {
            let pwa = self.axis.point_without_axis(point);
            return Some(DVec2::new(
                (pwa.x - self.min.x) / self.max.x,
                (pwa.y - self.min.y) / self.max.y,
            ));
        }
        None
    }
}

impl HittableTrait for AABox {
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

    fn does_int(&self, ray: &Ray) -> bool {
        for side in self.rects.iter() {
            if side.does_int(ray) {
                return true;
            }
        }
        false
    }
}
