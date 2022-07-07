use crate::{
    textures::Texture,
    utility::{random_float, specular_sampling, coord::Coordinate, cosine_hemisphere_sampling, near_zero, offset_ray},
};
use rt_core::{Float, Hit, Ray, Scatter, Vec3, PI};
use std::sync::Arc;

#[derive(Debug)]
pub struct Phong<T> {
    pub ks: Float,
    pub kd: Float,
    pub exponent: Float,
    pub texture: Arc<T>,
}

impl<T> Phong<T> 
where
    T: Texture,
{
    pub fn new(texture: &Arc<T>, kd: Float, ks: Float, exponent: Float) -> Self {
        Self { ks, kd, exponent, texture: texture.clone()}
    }
}

impl<T> Scatter for Phong<T>
where
    T: Texture,
{
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
        let rand = random_float();
        let direction = if rand > self.kd + self.ks {
            return true;
        } else if rand > self.kd {
            // sample diffuse
            let coordinate_system = Coordinate::new_from_z(hit.normal);
            let mut direction = cosine_hemisphere_sampling();
            coordinate_system.vec_to_coordinate(&mut direction);
 
            if near_zero(direction) {
                direction = hit.normal;
            }
            direction
        } else {
            // sample specular
            let mut spec_dir = ray.direction;
            spec_dir.reflect(hit.normal);
            let coordinate_system = Coordinate::new_from_z(spec_dir);
            let mut direction = specular_sampling(self.exponent);
            coordinate_system.vec_to_coordinate(&mut direction);
 
            if direction.dot(hit.normal) < 0.0 {
                return true;
            }
            if near_zero(direction) {
                direction = spec_dir;
            }
            direction
        };
        let point = offset_ray(hit.point, hit.normal, hit.error, true);
        *ray = Ray::new(point, direction, ray.time);
        false
    }
    fn scattering_pdf(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Float {
        let mut spec_dir = wo;
        spec_dir.reflect(hit.normal);
        let cos_alpha = spec_dir.dot(wi).max(0.0);
 
        let diffuse = hit.normal.dot(wi).max(0.0) / PI;
        let spec = (self.exponent + 1.0) * cos_alpha.powf(self.exponent) / (2.0 * PI);
        self.kd * diffuse + self.ks * spec
    }
    fn eval(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Vec3 {
        let mut spec_dir = wo;
        spec_dir.reflect(hit.normal);
        let cos_alpha = spec_dir.dot(wi).max(0.0);
 
        let diff = self.texture.colour_value(wo, hit.point)* hit.normal.dot(wi).max(0.0)
            / PI;
        let spec = Vec3::one() * (self.exponent + 2.0) * cos_alpha.powf(self.exponent) / (2.0 * PI);
 
        self.kd * diff + self.ks * spec
    }
}