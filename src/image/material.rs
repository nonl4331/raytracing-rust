use crate::image::ray::{Colour, Ray};
use crate::image::texture::{Texture, TextureTrait};
use ultraviolet::DVec2;
use ultraviolet::DVec3;

use crate::image::tracing::Hit;

use crate::image::math;

pub enum Material {
    Diffuse(Diffuse),
    Reflect(Reflect),
    Refract(Refract),
    Emit(Emit),
}

impl MaterialTrait for Material {
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (f64, bool) {
        match self {
            Material::Diffuse(diffuse) => diffuse.scatter_ray(ray, hit),
            Material::Reflect(reflect) => reflect.scatter_ray(ray, hit),
            Material::Refract(refract) => refract.scatter_ray(ray, hit),
            Material::Emit(emit) => emit.scatter_ray(ray, hit),
        }
    }
    fn colour(&self, uv: Option<DVec2>, point: DVec3) -> Colour {
        match self {
            Material::Diffuse(diffuse) => diffuse.colour(uv, point),
            Material::Reflect(reflect) => reflect.colour,
            Material::Refract(refract) => refract.colour,
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
    fn scatter_ray(&self, _: &mut Ray, _: &Hit) -> (f64, bool) {
        (1.0, true)
    }
    fn colour(&self, _: Option<DVec2>, _: DVec3) -> Colour {
        DVec3::new(1.0, 1.0, 1.0)
    }
    fn requires_uv(&self) -> bool {
        false
    }
}

pub struct Diffuse {
    texture: Texture,
    absorption: f64,
}

impl Diffuse {
    pub fn new(texture: Texture, absorption: f64) -> Self {
        Diffuse {
            texture,
            absorption,
        }
    }
}

pub struct Reflect {
    pub colour: Colour,
    pub fuzz: f64,
}

impl Reflect {
    pub fn new(colour: DVec3, fuzz: f64) -> Self {
        Reflect { colour, fuzz }
    }
}

pub struct Refract {
    pub colour: Colour,
    pub eta: f64,
}

impl Refract {
    pub fn new(colour: DVec3, eta: f64) -> Self {
        Refract { colour, eta }
    }
}

pub struct Emit {
    pub texture: Texture,
    pub strength: f64,
}

impl Emit {
    pub fn new(texture: Texture, strength: f64) -> Self {
        Emit { texture, strength }
    }
}

impl MaterialTrait for Diffuse {
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (f64, bool) {
        let mut direction = math::random_unit_vector() + hit.normal;
        if math::near_zero(direction) {
            direction = hit.normal;
        }
        *ray = Ray::new(
            hit.point,
            direction,
            ray.time,
            ray.sky,
            ray.hittables.clone(),
            ray.bvh.clone(),
        );
        (self.absorption, false)
    }
    fn colour(&self, uv: Option<DVec2>, point: DVec3) -> Colour {
        self.texture.colour_value(uv, point)
    }
}

impl MaterialTrait for Reflect {
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (f64, bool) {
        let mut direction = ray.direction;
        direction.reflect(hit.normal);
        *ray = Ray::new(
            hit.point,
            direction + self.fuzz * math::random_unit_vector(),
            ray.time,
            ray.sky,
            ray.hittables.clone(),
            ray.bvh.clone(),
        );
        (1.0, false)
    }
    fn colour(&self, _: Option<DVec2>, _: DVec3) -> Colour {
        self.colour
    }
}

impl MaterialTrait for Refract {
    fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> (f64, bool) {
        let mut eta_fraction = 1.0 / self.eta;
        if !hit.out {
            eta_fraction = self.eta;
        }

        let cos_theta = ((-1.0 * ray.direction).dot(hit.normal)).min(1.0);

        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = eta_fraction * sin_theta > 1.0;
        if cannot_refract || reflectance(cos_theta, eta_fraction) > math::random_f64() {
            let ref_mat = Reflect {
                colour: self.colour,
                fuzz: 0.0,
            };
            return ref_mat.scatter_ray(ray, hit);
        }

        let perp = eta_fraction * (ray.direction + cos_theta * hit.normal);
        let para = -1.0 * (1.0 - perp.mag_sq()).abs().sqrt() * hit.normal;
        let direction = perp + para;
        *ray = Ray::new(
            hit.point,
            direction,
            ray.time,
            ray.sky,
            ray.hittables.clone(),
            ray.bvh.clone(),
        );
        (1.0, false)
    }
}

impl MaterialTrait for Emit {
    fn scatter_ray(&self, _: &mut Ray, _: &Hit) -> (f64, bool) {
        (self.strength, true)
    }
    fn colour(&self, uv: Option<DVec2>, point: DVec3) -> Colour {
        self.texture.colour_value(uv, point)
    }
}

fn reflectance(cos: f64, eta_ratio: f64) -> f64 {
    let mut r0 = (1.0 - eta_ratio) / (1.0 + eta_ratio);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cos).powf(5.0)
}
