use crate::ray_tracing::{
    material::{offset_ray, Hit, MaterialTrait},
    ray::Ray,
    texture::{Texture, TextureTrait},
};

use crate::utility::{math, math::Float, vec::Vec3};

use std::sync::Arc;

pub struct Lambertian {
    pub texture: Arc<Texture>,
    pub absorption: Float,
}

impl Lambertian {
    pub fn new(texture: &Arc<Texture>, absorption: Float) -> Self {
        Lambertian {
            texture: texture.clone(),
            absorption,
        }
    }
}

impl MaterialTrait for Lambertian {
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (Vec3, bool) {
        let mut direction = math::random_unit_vector() + hit.normal;
        if math::near_zero(direction) {
            direction = hit.normal;
        }
        let point = offset_ray(hit.point, hit.normal, hit.error, true);
        *ray = Ray::new(point, direction, ray.time);
        (
            self.absorption * self.texture.colour_value(hit.uv, point),
            false,
        )
    }
}
