use crate::image::math;
use crate::image::ray::Ray;
use crate::image::tracing::{Hit, Hittable};
use ultraviolet::vec::Vec3;

pub const MAIN_SPHERE: Sphere = Sphere {
    center: Vec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    },
    radius: 0.5,
};
pub const LARGE_SPHERE: Sphere = Sphere {
    center: Vec3 {
        x: 0.0,
        y: 100.5,
        z: 1.0,
    },
    radius: 100.0,
};
pub const SMALL_SPHERE: Sphere = Sphere {
    center: Vec3 {
        x: 0.5,
        y: 0.0,
        z: 1.0,
    },
    radius: 0.5,
};
pub const SCENE: [&dyn Hittable; 3] = [&MAIN_SPHERE, &SMALL_SPHERE, &LARGE_SPHERE];

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

pub trait Material {
    fn scatter_ray(&self, _: &Hit, _: u32) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
}

pub struct Diffuse {
    pub absorption: f32,
}

impl Material for Diffuse {
    fn scatter_ray(&self, hit: &Hit, depth: u32) -> Vec3 {
        let direction = math::random_unit_vector() + hit.normal;
        let mut new_ray = Ray {
            origin: hit.point,
            direction,
            hit: None,
        };
        return self.absorption * new_ray.get_color(depth + 1);
    }
}
