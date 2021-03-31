use crate::image::ray::Ray;
use crate::image::scene::Sphere;
use ultraviolet::Vec3;

pub struct Hit {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub out: bool,
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
            let t = (-h - disc.sqrt()) / a;
            let point = ray.at(t);
            let mut normal = point - self.center;
            let mut out = true;
            normal.normalize();
            if normal.dot(ray.direction) > 0.0 {
                normal *= -1.0;
                out = false;
            }
            Some(Hit {
                t,
                point,
                normal,
                out,
            })
        } else {
            None
        }
    }
}
