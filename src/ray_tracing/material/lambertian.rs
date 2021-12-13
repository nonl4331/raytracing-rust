use crate::ray_tracing::{
    material::{offset_ray, Hit, Scatter},
    ray::Ray,
    texture::TextureTrait,
};

use crate::utility::{math, math::Float, vec::Vec3};

use std::sync::Arc;

pub struct Lambertian<T: TextureTrait> {
    pub texture: Arc<T>,
    pub absorption: Float,
}

impl<T> Lambertian<T>
where
    T: TextureTrait,
{
    pub fn new(texture: &Arc<T>, absorption: Float) -> Self {
        Lambertian {
            texture: texture.clone(),
            absorption,
        }
    }
}

impl<T> Scatter for Lambertian<T>
where
    T: TextureTrait,
{
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
