use crate::math;

use crate::ray_tracing::{
    ray::{Colour, Ray},
    texture::{Texture, TextureTrait},
    tracing::Hit,
};

use std::sync::Arc;

use ultraviolet::{Vec2, Vec3};

pub enum Material {
    Diffuse(Diffuse),
    Reflect(Reflect),
    Refract(Refract),
    Emit(Emit),
}

impl MaterialTrait for Material {
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (f32, bool) {
        match self {
            Material::Diffuse(diffuse) => diffuse.scatter_ray(ray, hit),
            Material::Reflect(reflect) => reflect.scatter_ray(ray, hit),
            Material::Refract(refract) => refract.scatter_ray(ray, hit),
            Material::Emit(emit) => emit.scatter_ray(ray, hit),
        }
    }
    fn colour(&self, uv: Option<Vec2>, point: Vec3) -> Colour {
        match self {
            Material::Diffuse(diffuse) => diffuse.colour(uv, point),
            Material::Reflect(reflect) => reflect.colour(uv, point),
            Material::Refract(refract) => refract.colour(uv, point),
            Material::Emit(emit) => emit.colour(uv, point),
        }
    }
    fn requires_uv(&self) -> bool {
        match self {
            Material::Diffuse(diffuse) => diffuse.texture.requires_uv(),
            Material::Reflect(_) => false,
            Material::Refract(_) => false,
            Material::Emit(emit) => emit.texture.requires_uv(),
        }
    }
}

pub trait MaterialTrait {
    fn scatter_ray(&self, _: &mut Ray, _: &Hit) -> (f32, bool) {
        (1.0, true)
    }
    fn colour(&self, _: Option<Vec2>, _: Vec3) -> Colour {
        Vec3::new(1.0, 1.0, 1.0)
    }
    fn requires_uv(&self) -> bool {
        false
    }
}

pub struct Diffuse {
    texture: Arc<Texture>,
    absorption: f32,
}

impl Diffuse {
    pub fn new(texture: &Arc<Texture>, absorption: f32) -> Self {
        Diffuse {
            texture: texture.clone(),
            absorption,
        }
    }
}

pub struct Reflect {
    pub texture: Arc<Texture>,
    pub fuzz: f32,
}

impl Reflect {
    pub fn new(texture: &Arc<Texture>, fuzz: f32) -> Self {
        Reflect {
            texture: texture.clone(),
            fuzz,
        }
    }
}

pub struct Refract {
    pub texture: Arc<Texture>,
    pub eta: f32,
}

impl Refract {
    pub fn new(texture: &Arc<Texture>, eta: f32) -> Self {
        Refract {
            texture: texture.clone(),
            eta,
        }
    }
}

pub struct Emit {
    pub texture: Arc<Texture>,
    pub strength: f32,
}

impl Emit {
    pub fn new(texture: &Arc<Texture>, strength: f32) -> Self {
        Emit {
            texture: texture.clone(),
            strength,
        }
    }
}

impl MaterialTrait for Diffuse {
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (f32, bool) {
        let mut direction = math::random_unit_vector() + hit.normal;
        if math::near_zero(direction) {
            direction = hit.normal;
        }
        *ray = Ray::new(hit.point, direction, ray.time);
        (self.absorption, false)
    }
    fn colour(&self, uv: Option<Vec2>, point: Vec3) -> Colour {
        self.texture.colour_value(uv, point)
    }
}

impl MaterialTrait for Reflect {
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (f32, bool) {
        let mut direction = ray.direction;
        direction.reflect(hit.normal);
        *ray = Ray::new(
            hit.point,
            direction + self.fuzz * math::random_unit_vector(),
            ray.time,
        );
        (1.0, false)
    }
    fn colour(&self, uv: Option<Vec2>, point: Vec3) -> Colour {
        self.texture.colour_value(uv, point)
    }
}

impl MaterialTrait for Refract {
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (f32, bool) {
        let mut eta_fraction = 1.0 / self.eta;
        if !hit.out {
            eta_fraction = self.eta;
        }

        let cos_theta = ((-1.0 * ray.direction).dot(hit.normal)).min(1.0);

        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = eta_fraction * sin_theta > 1.0;
        if cannot_refract || reflectance(cos_theta, eta_fraction) > math::random_f32() {
            let ref_mat = Reflect::new(&self.texture.clone(), 0.0);
            return ref_mat.scatter_ray(ray, hit);
        }

        let perp = eta_fraction * (ray.direction + cos_theta * hit.normal);
        let para = -1.0 * (1.0 - perp.mag_sq()).abs().sqrt() * hit.normal;
        let direction = perp + para;
        *ray = Ray::new(hit.point, direction, ray.time);
        (1.0, false)
    }
    fn colour(&self, uv: Option<Vec2>, point: Vec3) -> Colour {
        self.texture.colour_value(uv, point)
    }
}

impl MaterialTrait for Emit {
    fn scatter_ray(&self, _: &mut Ray, _: &Hit) -> (f32, bool) {
        (self.strength, true)
    }
    fn colour(&self, uv: Option<Vec2>, point: Vec3) -> Colour {
        self.texture.colour_value(uv, point)
    }
}

fn reflectance(cos: f32, eta_ratio: f32) -> f32 {
    let mut r0 = (1.0 - eta_ratio) / (1.0 + eta_ratio);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cos).powf(5.0)
}
