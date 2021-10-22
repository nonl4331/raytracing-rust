use crate::ray_tracing::{
    material::{fresnel, offset_ray, Hit, MaterialTrait},
    ray::Ray,
    texture::{Texture, TextureTrait},
};

use crate::utility::{math, math::Float, vec::Vec3};

use std::sync::Arc;

pub struct CookTorrence {
    pub texture: Arc<Texture>,
    pub alpha: Float,
    pub absorbtion: Float,
    pub specular_chance: Float,
    pub f0: Vec3,
}

impl CookTorrence {
    pub fn new(
        texture: &Arc<Texture>,
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
        if math::random_float() < self.specular_chance {
            let mut direction = ray.direction;
            direction.reflect(hit.normal);
            let point = offset_ray(hit.point, hit.normal, hit.error, true);
            *ray = Ray::new(
                point,
                direction + self.alpha * math::random_unit_vector(),
                ray.time,
            );

            let cos_theta = (ray.direction.dot(hit.normal)).min(1.0);
            let g = geometry_attenuation(hit.normal, direction, ray.direction);
            let d = ggx(self.alpha, hit.normal, direction, ray.direction);
            let f = fresnel(cos_theta, self.f0);
            let denom = 4.0 * hit.normal.dot(direction) * hit.normal.dot(ray.direction);
            let colour = self.texture.colour_value(hit.uv, point) * g * d * f / denom;
            (colour, false)
        } else {
            let mut direction = math::random_unit_vector() + hit.normal;
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

fn geometry_attenuation(normal: Vec3, light_dir: Vec3, ray_dir: Vec3) -> Float {
    let half_angle = (light_dir + ray_dir) / (light_dir + ray_dir).mag();
    let temp = 2.0 * half_angle.dot(normal) / ray_dir.dot(half_angle);

    (temp * ray_dir.dot(normal))
        .min(temp * light_dir.dot(normal))
        .min(1.0)
}

fn ggx(alpha: Float, normal: Vec3, light_dir: Vec3, ray_dir: Vec3) -> Float {
    let half_angle = (light_dir + ray_dir) / (light_dir + ray_dir).mag();

    let ndoth = normal.dot(half_angle);

    let temp = ndoth * ndoth * (alpha * alpha - 1.0) + 1.0;

    let multiplier = 1.0; //if ndoth > 0.0 { 1.0 } else { 0.0 };

    multiplier * alpha * alpha / (temp * temp)
}
