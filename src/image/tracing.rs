use crate::image::hittables::Material;
use crate::image::hittables::MovingSphere;
use crate::image::hittables::{AARect, Sphere};
use crate::image::math::near_zero;
use crate::image::ray::Ray;
use std::sync::Arc;
use ultraviolet::DVec3;

pub struct Hit {
    pub t: f64,
    pub point: DVec3,
    pub normal: DVec3,
    pub out: bool,
    pub material: Arc<Material>,
}

unsafe impl Send for Sphere {}
unsafe impl Sync for Sphere {}

pub trait HittableTrait {
    fn get_int(&self, _: &Ray) -> Option<Hit> {
        None
    }
    fn does_int(&self, _: &Ray) -> bool {
        false
    }
}

pub enum Hittable {
    Sphere(Sphere),
    MovingSphere(MovingSphere),
    AARect(AARect),
}

impl HittableTrait for Hittable {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        match self {
            Hittable::Sphere(sphere) => sphere.get_int(ray),
            Hittable::MovingSphere(sphere) => sphere.get_int(ray),
            Hittable::AARect(rect) => rect.get_int(ray),
        }
    }

    fn does_int(&self, ray: &Ray) -> bool {
        match self {
            Hittable::Sphere(sphere) => sphere.does_int(ray),
            Hittable::MovingSphere(sphere) => sphere.does_int(ray),
            Hittable::AARect(rect) => rect.does_int(ray),
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
                out,
                material: self.material.clone(),
            })
        } else {
            None
        }
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
                out,
                material: self.material.clone(),
            })
        } else {
            None
        }
    }
}

impl HittableTrait for AARect {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        let t = (self.z - ray.origin.z) / ray.direction.z;
        let point = ray.at(t);
        let yval = point.y;
        let xval = point.x;
        if xval > self.min.x && xval < self.max.x && yval > self.min.y && yval < self.max.y {
            Some(Hit {
                t,
                point,
                normal: DVec3::new(0.0, 0.0, -ray.direction.z).normalized(),
                out: true,
                material: self.material.clone(),
            })
        } else {
            None
        }
    }

    fn does_int(&self, ray: &Ray) -> bool {
        let t = (self.z - ray.origin.z) / ray.direction.z;
        let point = ray.at(t);
        let yval = point.y;
        let xval = point.x;

        xval > self.min.x && xval < self.max.x && yval > self.min.y && yval < self.max.y
    }
}
