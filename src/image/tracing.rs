use crate::image::math::near_zero;
use crate::image::ray::Ray;
use crate::image::scene::Material;
use crate::image::scene::Sphere;
use std::sync::Arc;
use ultraviolet::DVec3;

pub struct Hit {
    pub t: f64,
    pub point: DVec3,
    pub normal: DVec3,
    pub out: bool,
    pub material: Arc<Box<dyn Material>>,
}

pub trait Hittable {
    fn get_int(&self, _: &Ray) -> Option<Hit> {
        None
    }
}

impl Hittable for Sphere {
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
