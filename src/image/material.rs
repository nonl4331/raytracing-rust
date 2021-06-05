use crate::image::ray::{Color, Ray};
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
    fn scatter_ray(&self, ray: &Ray, hit: &Hit, depth: u32) -> Color {
        match self {
            Material::Diffuse(diffuse) => diffuse.scatter_ray(ray, hit, depth),
            Material::Reflect(reflect) => reflect.scatter_ray(ray, hit, depth),
            Material::Refract(refract) => refract.scatter_ray(ray, hit, depth),
            Material::Emit(emit) => emit.scatter_ray(ray, hit, depth),
        }
    }
    fn color(&self, uv: Option<DVec2>, point: DVec3) -> Color {
        match self {
            Material::Diffuse(diffuse) => diffuse.color(uv, point),
            Material::Reflect(reflect) => reflect.color,
            Material::Refract(refract) => refract.color,
            Material::Emit(emit) => emit.color,
        }
    }
    fn requires_uv(&self) -> bool {
        match self {
            Material::Diffuse(diffuse) => diffuse.texture.requires_uv(),
            Material::Reflect(_) => false,
            Material::Refract(_) => false,
            Material::Emit(_) => false,
        }
    }
}

pub trait MaterialTrait {
    fn scatter_ray(&self, _: &Ray, _: &Hit, _: u32) -> Color {
        DVec3::new(0.0, 0.0, 0.0)
    }
    fn color(&self, _: Option<DVec2>, _: DVec3) -> Color {
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
        //let color = DVec3::one();
        Diffuse {
            texture,
            absorption,
        }
    }
}

pub struct Reflect {
    pub color: Color,
    pub fuzz: f64,
}

impl Reflect {
    pub fn new(color: DVec3, fuzz: f64) -> Self {
        Reflect { color, fuzz }
    }
}

pub struct Refract {
    pub color: Color,
    pub eta: f64,
}

impl Refract {
    pub fn new(color: DVec3, eta: f64) -> Self {
        Refract { color, eta }
    }
}

pub struct Emit {
    pub color: Color,
    pub strength: f64,
}

impl Emit {
    pub fn new(color: DVec3, strength: f64) -> Self {
        Emit { color, strength }
    }
}

impl MaterialTrait for Diffuse {
    fn scatter_ray(&self, ray: &Ray, hit: &Hit, depth: u32) -> Color {
        let mut direction = math::random_unit_vector() + hit.normal;
        if math::near_zero(direction) {
            direction = hit.normal;
        }
        let mut new_ray = Ray::new(
            hit.point,
            direction,
            ray.time,
            ray.sky,
            ray.hittables.clone(),
            ray.bvh.clone(),
        );
        return self.absorption * new_ray.get_color(depth + 1);
    }
    fn color(&self, uv: Option<DVec2>, point: DVec3) -> Color {
        self.texture.color_value(uv, point)
    }
}

impl MaterialTrait for Reflect {
    fn scatter_ray(&self, ray: &Ray, hit: &Hit, depth: u32) -> Color {
        let mut direction = ray.direction;
        direction.reflect(hit.normal);
        let mut new_ray = Ray::new(
            hit.point,
            direction + self.fuzz * math::random_unit_vector(),
            ray.time,
            ray.sky,
            ray.hittables.clone(),
            ray.bvh.clone(),
        );
        return new_ray.get_color(depth + 1);
    }
    fn color(&self, _: Option<DVec2>, _: DVec3) -> Color {
        self.color
    }
}

impl MaterialTrait for Refract {
    fn scatter_ray(&self, ray: &Ray, hit: &Hit, depth: u32) -> Color {
        let mut eta_fraction = 1.0 / self.eta;
        if !hit.out {
            eta_fraction = self.eta;
        }

        let cos_theta = ((-1.0 * ray.direction).dot(hit.normal)).min(1.0);

        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = eta_fraction * sin_theta > 1.0;
        if cannot_refract || reflectance(cos_theta, eta_fraction) > math::random_f64() {
            let ref_mat = Reflect {
                color: self.color,
                fuzz: 0.0,
            };
            return ref_mat.scatter_ray(ray, hit, depth);
        }

        let perp = eta_fraction * (ray.direction + cos_theta * hit.normal);
        let para = -1.0 * (1.0 - perp.mag_sq()).abs().sqrt() * hit.normal;
        let direction = perp + para;
        let mut new_ray = Ray::new(
            hit.point,
            direction,
            ray.time,
            ray.sky,
            ray.hittables.clone(),
            ray.bvh.clone(),
        );
        return new_ray.get_color(depth + 1);
    }
}

impl MaterialTrait for Emit {
    fn scatter_ray(&self, _: &Ray, _: &Hit, _: u32) -> Color {
        self.strength * self.color
    }
    fn color(&self, _: Option<DVec2>, _: DVec3) -> Color {
        self.color
    }
}

fn reflectance(cos: f64, eta_ratio: f64) -> f64 {
    let mut r0 = (1.0 - eta_ratio) / (1.0 + eta_ratio);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cos).powf(5.0)
}
