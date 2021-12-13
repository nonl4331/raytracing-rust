pub mod cook_torrence;

pub mod lambertian;

use crate::utility::{math, math::Float};

use crate::ray_tracing::{
    intersection::{offset_ray, Hit},
    ray::Ray,
    texture::TextureTrait,
};

use std::sync::Arc;

use crate::utility::vec::Vec3;

pub use lambertian::Lambertian;

pub use cook_torrence::CookTorrence;

pub enum MaterialEnum<T: TextureTrait> {
    Lambertian(Lambertian<T>),
    Reflect(Reflect<T>),
    Refract(Refract<T>),
    Emit(Emit<T>),
    CookTorrence(CookTorrence<T>),
}

impl<T> Scatter for MaterialEnum<T>
where
    T: TextureTrait,
{
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (Vec3, bool) {
        match self {
            MaterialEnum::Reflect(reflect) => reflect.scatter_ray(ray, hit),
            MaterialEnum::Lambertian(diffuse) => diffuse.scatter_ray(ray, hit),
            MaterialEnum::Refract(refract) => refract.scatter_ray(ray, hit),
            MaterialEnum::Emit(emit) => emit.scatter_ray(ray, hit),
            MaterialEnum::CookTorrence(cook_torrence) => cook_torrence.scatter_ray(ray, hit),
        }
    }
    fn requires_uv(&self) -> bool {
        match self {
            MaterialEnum::Lambertian(diffuse) => diffuse.texture.requires_uv(),
            MaterialEnum::Reflect(reflect) => reflect.texture.requires_uv(),
            MaterialEnum::Refract(refract) => refract.texture.requires_uv(),
            MaterialEnum::Emit(emit) => emit.texture.requires_uv(),
            MaterialEnum::CookTorrence(cook_torrence) => cook_torrence.texture.requires_uv(),
        }
    }
}

pub trait Scatter {
    fn scatter_ray(&self, _: &mut Ray, _: &Hit) -> (Vec3, bool) {
        (Vec3::one(), true)
    }
    fn requires_uv(&self) -> bool {
        false
    }
}

pub struct Reflect<T: TextureTrait> {
    pub texture: Arc<T>,
    pub fuzz: Float,
}

impl<T> Reflect<T>
where
    T: TextureTrait,
{
    pub fn new(texture: &Arc<T>, fuzz: Float) -> Self {
        Reflect {
            texture: texture.clone(),
            fuzz,
        }
    }
}

pub struct Refract<T: TextureTrait> {
    pub texture: Arc<T>,
    pub eta: Float,
}

impl<T> Refract<T>
where
    T: TextureTrait,
{
    pub fn new(texture: &Arc<T>, eta: Float) -> Self {
        Refract {
            texture: texture.clone(),
            eta,
        }
    }
}

pub struct Emit<T> {
    pub texture: Arc<T>,
    pub strength: Float,
}

impl<T> Emit<T>
where
    T: TextureTrait,
{
    pub fn new(texture: &Arc<T>, strength: Float) -> Self {
        Emit {
            texture: texture.clone(),
            strength,
        }
    }
}

impl<T> Scatter for Reflect<T>
where
    T: TextureTrait,
{
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (Vec3, bool) {
        let mut direction = ray.direction;
        direction.reflect(hit.normal);
        let point = offset_ray(hit.point, hit.normal, hit.error, true);
        *ray = Ray::new(
            point,
            direction + self.fuzz * math::random_unit_vector(),
            ray.time,
        );
        (self.texture.colour_value(hit.uv, point), false)
    }
}

impl<T> Scatter for Refract<T>
where
    T: TextureTrait,
{
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (Vec3, bool) {
        let mut eta_fraction = 1.0 / self.eta;
        if !hit.out {
            eta_fraction = self.eta;
        }

        let cos_theta = ((-1.0 * ray.direction).dot(hit.normal)).min(1.0);

        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = eta_fraction * sin_theta > 1.0;
        let f0 = (1.0 - eta_fraction) / (1.0 + eta_fraction);
        let f0 = f0 * f0 * Vec3::one();
        if cannot_refract || fresnel(cos_theta, f0).x > math::random_float() {
            let ref_mat = Reflect::new(&self.texture.clone(), 0.0);
            return ref_mat.scatter_ray(ray, hit);
        }

        let perp = eta_fraction * (ray.direction + cos_theta * hit.normal);
        let para = -1.0 * (1.0 - perp.mag_sq()).abs().sqrt() * hit.normal;
        let direction = perp + para;
        let point = offset_ray(hit.point, hit.normal, hit.error, false);
        *ray = Ray::new(point, direction, ray.time);
        (self.texture.colour_value(hit.uv, point), false)
    }
}

impl<T> Scatter for Emit<T>
where
    T: TextureTrait,
{
    fn scatter_ray(&self, _: &mut Ray, hit: &Hit) -> (Vec3, bool) {
        let point = offset_ray(hit.point, hit.normal, hit.error, true);
        (
            self.strength * self.texture.colour_value(hit.uv, point),
            true,
        )
    }
}

fn fresnel(cos: Float, f0: Vec3) -> Vec3 {
    f0 + (1.0 - f0) * (1.0 - cos).powf(5.0)
}
