use ultraviolet::{DVec2, DVec3};

use std::sync::Arc;

use crate::image::ray::{Color, Ray};

use crate::image::tracing::Hit;

use crate::image::math;

pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn get_axis_value(&self, point: DVec3) -> f64 {
        match self {
            Axis::X => point.x,
            Axis::Y => point.y,
            Axis::Z => point.z,
        }
    }

    pub fn point_without_axis(&self, point: DVec3) -> DVec2 {
        match self {
            Axis::X => DVec2::new(point.y, point.z),
            Axis::Y => DVec2::new(point.x, point.z),
            Axis::Z => DVec2::new(point.x, point.y),
        }
    }
    pub fn return_point_with_axis(&self, dir: DVec3) -> DVec3 {
        match self {
            Axis::X => DVec3::new(dir.x, 0.0, 0.0),
            Axis::Y => DVec3::new(0.0, dir.y, 0.0),
            Axis::Z => DVec3::new(0.0, 0.0, dir.z),
        }
    }
}

pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
    pub material: Arc<Material>,
}

pub struct AARect {
    pub min: DVec2,
    pub max: DVec2,
    pub k: f64,
    pub axis: Axis,
    pub material: Arc<Material>,
}

pub struct AABox {
    pub min: DVec3,
    pub max: DVec3,
    pub rects: [AARect; 6],
    pub material: Arc<Material>,
}

impl AARect {
    pub fn new(min: DVec2, max: DVec2, k: f64, axis: Axis, material: Material) -> Self {
        AARect {
            min,
            max,
            k,
            axis,
            material: Arc::new(material),
        }
    }
}

impl AABox {
    pub fn new(min: DVec3, max: DVec3, material: Material) -> Self {
        let rects = [
            AARect::new(
                Axis::X.point_without_axis(min),
                Axis::X.point_without_axis(max),
                min.x,
                Axis::X,
                material,
            ),
            AARect::new(
                Axis::X.point_without_axis(min),
                Axis::X.point_without_axis(max),
                max.x,
                Axis::X,
                material,
            ),
            AARect::new(
                Axis::Y.point_without_axis(min),
                Axis::Y.point_without_axis(max),
                min.y,
                Axis::Y,
                material,
            ),
            AARect::new(
                Axis::Y.point_without_axis(min),
                Axis::Y.point_without_axis(max),
                max.y,
                Axis::Y,
                material,
            ),
            AARect::new(
                Axis::Z.point_without_axis(min),
                Axis::Z.point_without_axis(max),
                min.z,
                Axis::Z,
                material,
            ),
            AARect::new(
                Axis::Z.point_without_axis(min),
                Axis::Z.point_without_axis(max),
                max.z,
                Axis::Z,
                material,
            ),
        ];
        AABox {
            min,
            max,
            rects,
            material: Arc::new(material),
        }
    }
}

impl Sphere {
    pub fn new(center: DVec3, radius: f64, material: Material) -> Self {
        Sphere {
            center,
            radius,
            material: Arc::new(material),
        }
    }
}

pub struct MovingSphere {
    pub start_pos: DVec3,
    pub end_pos: DVec3,
    pub radius: f64,
    pub material: Arc<Material>,
}

impl MovingSphere {
    pub fn new(start_pos: DVec3, end_pos: DVec3, radius: f64, material: Material) -> Self {
        MovingSphere {
            start_pos,
            end_pos,
            radius,
            material: Arc::new(material),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Material {
    Diffuse(Diffuse),
    Reflect(Reflect),
    Refract(Refract),
}

impl MaterialTrait for Material {
    fn scatter_ray(&self, ray: &Ray, hit: &Hit, depth: u32) -> Color {
        match self {
            Material::Diffuse(diffuse) => diffuse.scatter_ray(ray, hit, depth),
            Material::Reflect(reflect) => reflect.scatter_ray(ray, hit, depth),
            Material::Refract(refract) => refract.scatter_ray(ray, hit, depth),
        }
    }
    fn color(&self) -> Color {
        match self {
            Material::Diffuse(diffuse) => diffuse.color,
            Material::Reflect(reflect) => reflect.color,
            Material::Refract(refract) => refract.color,
        }
    }
}

pub trait MaterialTrait {
    fn scatter_ray(&self, _: &Ray, _: &Hit, _: u32) -> Color {
        DVec3::new(0.0, 0.0, 0.0)
    }
    fn color(&self) -> Color {
        DVec3::new(1.0, 1.0, 1.0)
    }
}
#[derive(Clone, Copy)]
pub struct Diffuse {
    color: Color,
    absorption: f64,
}

impl Diffuse {
    pub fn new(color: DVec3, absorption: f64) -> Self {
        Diffuse { color, absorption }
    }
}
#[derive(Clone, Copy)]
pub struct Reflect {
    pub color: Color,
    pub fuzz: f64,
}

impl Reflect {
    pub fn new(color: DVec3, fuzz: f64) -> Self {
        Reflect { color, fuzz }
    }
}
#[derive(Clone, Copy)]
pub struct Refract {
    pub color: Color,
    pub eta: f64,
}

impl Refract {
    pub fn new(color: DVec3, eta: f64) -> Self {
        Refract { color, eta }
    }
}

impl MaterialTrait for Diffuse {
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
            time: ray.time,
        };
        return self.absorption * new_ray.get_color(depth + 1);
    }
    fn color(&self) -> Color {
        self.color
    }
}

impl MaterialTrait for Reflect {
    fn scatter_ray(&self, ray: &Ray, hit: &Hit, depth: u32) -> Color {
        let mut direction = ray.direction;
        direction.reflect(hit.normal);
        let mut new_ray = Ray {
            origin: hit.point,
            direction: direction + self.fuzz * math::random_unit_vector(),
            hittables: ray.hittables.clone(),
            hit: None,
            time: ray.time,
        };
        return new_ray.get_color(depth + 1);
    }
    fn color(&self) -> Color {
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
        let mut new_ray = Ray {
            origin: hit.point,
            direction,
            hittables: ray.hittables.clone(),
            hit: None,
            time: ray.time,
        };
        return new_ray.get_color(depth + 1);
    }
}

fn reflectance(cos: f64, eta_ratio: f64) -> f64 {
    let mut r0 = (1.0 - eta_ratio) / (1.0 + eta_ratio);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cos).powf(5.0)
}
