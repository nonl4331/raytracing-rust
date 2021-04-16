use ultraviolet::DVec3;

use std::sync::Arc;

use crate::image::ray::{Color, Ray};

use crate::image::tracing::Hit;

use crate::image::math;

pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
    pub material: Arc<Box<dyn Material>>,
}

impl Sphere {
    pub fn new(center: DVec3, radius: f64, material: Box<dyn Material + 'static>) -> Self {
        Sphere {
            center,
            radius,
            material: Arc::new(material),
        }
    }
}

pub trait Material {
    fn scatter_ray(&self, _: &Ray, _: &Hit, _: u32) -> Color {
        DVec3::new(0.0, 0.0, 0.0)
    }
    fn color(&self) -> Color {
        DVec3::new(1.0, 1.0, 1.0)
    }
}

pub struct Diffuse {
    color: Color,
    absorption: f64,
}

impl Diffuse {
    pub fn new(color: DVec3, absorption: f64) -> Self {
        Diffuse { color, absorption }
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

impl Material for Diffuse {
    fn scatter_ray(&self, ray: &Ray, hit: &Hit, depth: u32) -> Color {
        let mut direction = math::random_unit_vector() + hit.normal;
        if math::near_zero(direction) {
            direction = hit.normal;
        }
        let mut new_ray = Ray {
            origin: hit.point,
            direction,
            hittables: ray.hittables.clone(),
            hit: None,
        };
        return self.absorption * new_ray.get_color(depth + 1);
    }
    fn color(&self) -> Color {
        self.color
    }
}

impl Material for Reflect {
    fn scatter_ray(&self, ray: &Ray, hit: &Hit, depth: u32) -> Color {
        let mut direction = ray.direction;
        direction.reflect(hit.normal);
        let mut new_ray = Ray {
            origin: hit.point,
            direction: direction + self.fuzz * math::random_unit_vector(),
            hittables: ray.hittables.clone(),
            hit: None,
        };
        return new_ray.get_color(depth + 1);
    }
    fn color(&self) -> Color {
        self.color
    }
}

impl Material for Refract {
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
        let mut new_ray = Ray {
            origin: hit.point,
            direction,
            hittables: ray.hittables.clone(),
            hit: None,
        };
        return new_ray.get_color(depth + 1);
    }
}

fn reflectance(cos: f64, eta_ratio: f64) -> f64 {
    let mut r0 = (1.0 - eta_ratio) / (1.0 + eta_ratio);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cos).powf(5.0)
}
