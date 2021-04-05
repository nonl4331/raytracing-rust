use crate::image::camera::Camera;
use crate::image::generate::check_percent;
use crate::image::math;
use crate::image::ray::{Color, Ray};
use crate::image::tracing::{Hit, Hittable};

use image::ImageBuffer;
use rand::Rng;

use std::sync::Arc;

use std::sync::RwLock;
use ultraviolet::vec::DVec3;

pub type HittablesType = Arc<RwLock<Vec<Box<dyn Hittable>>>>;

pub struct Scene {
    pub hittables: HittablesType,
    pub camera: Camera,
}

impl Scene {
    pub fn new(aspect_ratio: f64, focal_length: f64, viewport_height: f64) -> Self {
        let hittables = Arc::new(RwLock::new(vec![]));
        let camera = Camera::new(
            viewport_height,
            aspect_ratio,
            focal_length,
            hittables.clone(),
        );

        Scene { hittables, camera }
    }

    pub fn add(&mut self, hittable: Box<dyn Hittable>) {
        let mut vec = self.hittables.write().unwrap();
        vec.push(hittable);
    }

    pub fn generate_image(&self, width: u32, pixel_samples: u32) {
        let height = (width as f64 / self.camera.aspect_ratio) as u32;

        let mut image = ImageBuffer::new(width, height);

        let percent = ((width * height) as f64 / 100.0) as u32;

        let mut rng = rand::thread_rng();
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            check_percent(percent, width, x, y);
            let mut color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..pixel_samples {
                let u = (rng.gen_range(0.0..1.0) + x as f64) / width as f64;
                let v = (rng.gen_range(0.0..1.0) + y as f64) / height as f64;

                let mut ray = self.camera.get_ray(u, v);
                color += ray.get_color(0);
            }
            color /= pixel_samples as f64;

            *pixel = image::Rgb([
                (color.x.sqrt() * 255.0) as u8,
                (color.y.sqrt() * 255.0) as u8,
                (color.z.sqrt() * 255.0) as u8,
            ]);
        }
        println!("Image done generating:");
        println!("Width: {}", width);
        println!("Height: {}", height);
        println!("Samples per pixel: {}", pixel_samples);
        image.save("test.png").unwrap();
    }
}

pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
    pub material: Arc<Box<dyn Material>>,
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
    pub color: Color,
    pub absorption: f64,
}

pub struct Reflect {
    pub color: Color,
    pub fuzz: f64,
}

pub struct Refract {
    pub color: Color,
    pub eta: f64,
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
