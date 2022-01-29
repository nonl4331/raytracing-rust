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
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (Float, bool) {
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
    fn scattering_pdf(&self, point: Vec3, direction: Vec3, normal: Vec3) -> Float {
        match self {
            MaterialEnum::Lambertian(diffuse) => diffuse.scattering_pdf(point, direction, normal),
            MaterialEnum::Reflect(reflect) => reflect.scattering_pdf(point, direction, normal),
            MaterialEnum::Refract(refract) => refract.scattering_pdf(point, direction, normal),
            MaterialEnum::Emit(emit) => emit.scattering_pdf(point, direction, normal),
            MaterialEnum::CookTorrence(cook_torrence) => {
                cook_torrence.scattering_pdf(point, direction, normal)
            }
        }
    }
    fn scattering_albedo(&self, hit: &Hit, old_dir: Vec3, dir: Vec3) -> Vec3 {
        match self {
            MaterialEnum::Lambertian(diffuse) => diffuse.scattering_albedo(hit, old_dir, dir),
            MaterialEnum::Reflect(reflect) => reflect.scattering_albedo(hit, old_dir, dir),
            MaterialEnum::Refract(refract) => refract.scattering_albedo(hit, old_dir, dir),
            MaterialEnum::Emit(emit) => emit.scattering_albedo(hit, old_dir, dir),
            MaterialEnum::CookTorrence(cook_torrence) => {
                cook_torrence.scattering_albedo(hit, old_dir, dir)
            }
        }
    }

    fn is_light(&self) -> bool {
        match self {
            MaterialEnum::Lambertian(diffuse) => diffuse.is_light(),
            MaterialEnum::Reflect(reflect) => reflect.is_light(),
            MaterialEnum::Refract(refract) => refract.is_light(),
            MaterialEnum::Emit(emit) => emit.is_light(),
            MaterialEnum::CookTorrence(cook_torrence) => cook_torrence.is_light(),
        }
    }
    fn get_emission(&self, hit: &Hit) -> Vec3 {
        match self {
            MaterialEnum::Lambertian(diffuse) => diffuse.get_emission(hit),
            MaterialEnum::Reflect(reflect) => reflect.get_emission(hit),
            MaterialEnum::Refract(refract) => refract.get_emission(hit),
            MaterialEnum::Emit(emit) => emit.get_emission(hit),
            MaterialEnum::CookTorrence(cook_torrence) => cook_torrence.get_emission(hit),
        }
    }
}

pub trait Scatter {
    fn scatter_ray(&self, _: &mut Ray, _: &Hit) -> (Float, bool) {
        (0.0, true)
    }
    fn requires_uv(&self) -> bool {
        false
    }
    fn is_light(&self) -> bool {
        false
    }
    fn ls_chance(&self) -> Float {
        0.0
    }
    fn is_delta(&self) -> bool {
        false
    }
    fn scattering_pdf(&self, _: Vec3, _: Vec3, _: Vec3) -> Float {
        0.0
    }
    fn scattering_albedo(&self, _: &Hit, _: Vec3, _: Vec3) -> Vec3 {
        Vec3::one()
    }
    fn get_emission(&self, _: &Hit) -> Vec3 {
        Vec3::zero()
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
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (Float, bool) {
        let mut direction = ray.direction;
        direction.reflect(hit.normal);
        let point = offset_ray(hit.point, hit.normal, hit.error, true);
        *ray = Ray::new(
            point,
            direction + self.fuzz * math::random_unit_vector(),
            ray.time,
        );
        (1.0, false)
    }
    fn scattering_albedo(&self, hit: &Hit, _: Vec3, _: Vec3) -> Vec3 {
        let point = offset_ray(hit.point, hit.normal, hit.error, false);
        self.texture.colour_value(hit.uv, point)
    }
}

impl<T> Scatter for Refract<T>
where
    T: TextureTrait,
{
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (Float, bool) {
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
        (1.0, false)
    }
    fn scattering_albedo(&self, hit: &Hit, _: Vec3, _: Vec3) -> Vec3 {
        let point = offset_ray(hit.point, hit.normal, hit.error, false);
        self.texture.colour_value(hit.uv, point)
    }
}

impl<T> Scatter for Emit<T>
where
    T: TextureTrait,
{
    fn get_emission(&self, hit: &Hit) -> Vec3 {
        let point = offset_ray(hit.point, hit.normal, hit.error, true);
        self.strength * self.texture.colour_value(hit.uv, point)
    }
    fn is_light(&self) -> bool {
        true
    }
    fn scatter_ray(&self, _: &mut Ray, _: &Hit) -> (Float, bool) {
        (1.0, true)
    }
}

fn fresnel(cos: Float, f0: Vec3) -> Vec3 {
    f0 + (1.0 - f0) * (1.0 - cos).powf(5.0)
}
