use crate::ray_tracing::{
    material::{offset_ray, Hit, Scatter},
    ray::Ray,
    texture::TextureTrait,
};

use crate::utility::{coord::Coordinate, math, math::Float, vec::Vec3};

use std::sync::Arc;

pub struct Lambertian<T: TextureTrait> {
    pub texture: Arc<T>,
    pub absorption: Float,
}

#[cfg(all(feature = "f64"))]
use std::f64::consts::PI;

#[cfg(not(feature = "f64"))]
use std::f32::consts::PI;

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
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (Float, bool) {
        let coordinate_system = Coordinate::new_from_z(hit.normal);
        let mut direction = math::hemisphere_sampling();
        coordinate_system.vec_to_coordinate(&mut direction);

        let point = offset_ray(hit.point, hit.normal, hit.error, true);
        *ray = Ray::new(point, direction, ray.time);
        (
            self.scattering_pdf(hit.point, ray.direction, hit.normal),
            false,
        )
    }
    fn scattering_pdf(&self, _: Vec3, direction: Vec3, normal: Vec3) -> Float {
        normal.dot(direction).max(0.0) / PI
    }
    fn scattering_albedo(&self, hit: &Hit, _: Vec3, _: Vec3) -> Vec3 {
        self.texture.colour_value(hit.uv, hit.point) * (1.0 - self.absorption)
    }
}
