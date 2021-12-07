use crate::ray_tracing::{
    material::{fresnel, offset_ray, Hit, MaterialTrait},
    ray::Ray,
    texture::{TextureEnum, TextureTrait},
};

use crate::utility::{math, math::Float, vec::Vec3};

use std::sync::Arc;

#[cfg(all(feature = "f64"))]
use std::f64::consts::PI;

#[cfg(not(feature = "f64"))]
use std::f32::consts::PI;

pub struct CookTorrence {
    pub texture: Arc<TextureEnum>,
    pub alpha: Float,
    pub absorbtion: Float,
    pub specular_chance: Float,
    pub f0: Vec3,
}

impl CookTorrence {
    pub fn new(
        texture: &Arc<TextureEnum>,
        alpha: Float,
        absorbtion: Float,
        specular_chance: Float,
        f0: Vec3,
    ) -> Self {
        if alpha < 0.0 || alpha > 1.0 {
            panic!("Alpha value for CookTorrence must be between 0 and 1 inclusive")
        }
        if absorbtion < 0.0 || absorbtion > 1.0 {
            panic!("absorbtion value for CookTorrence must be between 0 and 1 inclusive")
        }
        if specular_chance < 0.0 || specular_chance > 1.0 {
            panic!("specular_chance value for CookTorrence must be between 0 and 1 inclusive")
        }
        if f0.component_min() < 0.0 {
            panic!("f0 values for CookTorrence must be greater than 0");
        }
        CookTorrence {
            texture: texture.clone(),
            alpha,
            absorbtion,
            specular_chance,
            f0,
        }
    }
}

impl MaterialTrait for CookTorrence {
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (Vec3, bool) {
        let random_dir = (math::random_unit_vector() + hit.normal).normalised();
        if math::random_float() < self.specular_chance {
            let point = offset_ray(hit.point, hit.normal, hit.error, true);

            let mut direction = ray.direction;
            direction.reflect(hit.normal);
            direction += self.alpha * math::random_unit_vector();

            let cos_theta = random_dir.dot(hit.normal).min(1.0);
            let half_angle = (random_dir - ray.direction).normalised();

            let g = geometry_attenuation(hit.normal, half_angle, -ray.direction, random_dir);
            let d = ggx_distribution(self.alpha, hit.normal, half_angle);
            let f = fresnel(cos_theta, self.f0);
            let denom = 2.0 * hit.normal.dot(-ray.direction);
            let colour = PI * f * d * g / denom;

            *ray = Ray::new(point, direction, ray.time);

            (colour, false)
        } else {
            let mut direction = random_dir;
            if math::near_zero(direction) {
                direction = hit.normal;
            }
            let point = offset_ray(hit.point, hit.normal, hit.error, true);
            *ray = Ray::new(point, direction, ray.time);
            (
                self.absorbtion * self.texture.colour_value(hit.uv, point),
                false,
            )
        }
    }
}

fn geometry_attenuation(normal: Vec3, half_angle: Vec3, ray_in: Vec3, ray_out: Vec3) -> Float {
    let temp = 2.0 * half_angle.dot(normal) / ray_in.dot(half_angle);

    (temp * ray_in.dot(normal))
        .min(temp * ray_out.dot(normal))
        .min(1.0)
}

/*fn ggx_distribution(alpha: Float, normal: Vec3, half_angle: Vec3) -> Float {
    let alpha_sq = alpha * alpha;

    let ndoth = normal.dot(half_angle).max(0.0);

    let ndoth_sq = ndoth * ndoth;

    let denom = ndoth_sq * alpha_sq + (1.0 - ndoth_sq);
    let denom = PI * denom * denom;

    alpha_sq / denom
}*/

fn ggx_distribution(alpha: Float, normal: Vec3, half_angle: Vec3) -> Float {
    let ndoth = normal.dot(half_angle);
    let ndoth_sq = ndoth * ndoth;

    let alpha_sq = alpha * alpha;

    let alpha_ndoth_sq = ndoth_sq * alpha_sq;

    let base = 1.0 / (PI * alpha_ndoth_sq * ndoth_sq);
    let exponent = (ndoth_sq - 1.0) / alpha_ndoth_sq;

    base.powf(exponent)
}
