use crate::image::camera::Camera;
use crate::image::generate::check_percent;
use crate::image::math;
use crate::image::ray::{Color, Ray};
use crate::image::tracing::{Hit, Hittable};
use image::Rgb;
use std::sync::Mutex;

use image::ImageBuffer;
use rand::Rng;

use std::sync::Arc;

use std::sync::RwLock;
use ultraviolet::vec::DVec3;

use rayon::prelude::*;

pub type HittablesType = Arc<RwLock<Vec<Box<dyn Hittable + Send + Sync>>>>;

pub struct Scene {
    pub hittables: HittablesType,
    pub camera: Camera,
}

impl Scene {
    pub fn new(
        origin: DVec3,
        lookat: DVec3,
        vup: DVec3,
        fov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
        starting_hittables: Option<Vec<Box<dyn Hittable + Send + Sync>>>,
    ) -> Self {
        let hittables: HittablesType;

        hittables = match starting_hittables {
            Some(value) => Arc::new(RwLock::new(value)),
            None => Arc::new(RwLock::new(vec![])),
        };

        let camera = Camera::new(
            origin,
            lookat,
            vup,
            fov,
            aspect_ratio,
            aperture,
            focus_dist,
            hittables.clone(),
        );

        Scene { hittables, camera }
    }

    pub fn _add(&mut self, hittable: Box<dyn Hittable + Send + Sync>) {
        let mut vec = self.hittables.write().unwrap();
        vec.push(hittable);
    }

    fn get_image(
        &self,
        width: u32,
        height: u32,
        pixel_samples: u32,
    ) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let percent = ((width * height) as f64 / 100.0) as u32;

        let mut image = ImageBuffer::new(width, height);

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
        image
    }

    fn get_image_part(
        &self,
        width: u32,
        height: u32,
        pixel_samples: u32,
        real_pixel_samples: u32,
    ) -> Vec<f64> {
        let channels = 3;
        let pixel_num = height * width;

        let mut rgb_vec = Vec::with_capacity((pixel_num * channels) as usize);

        let mut rng = rand::thread_rng();

        for pixel_i in 0..pixel_num {
            let x = pixel_i % width;
            let y = (pixel_i - x) / width;
            let mut color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..pixel_samples {
                let u = (rng.gen_range(0.0..1.0) + x as f64) / width as f64;
                let v = (rng.gen_range(0.0..1.0) + y as f64) / height as f64;

                let mut ray = self.camera.get_ray(u, v);
                color += ray.get_color(0);
            }
            color /= real_pixel_samples as f64;

            rgb_vec.push(color.x);
            rgb_vec.push(color.y);
            rgb_vec.push(color.z);
        }
        rgb_vec
    }

    #[allow(dead_code)]
    pub fn generate_image(&self, filename: &str, width: u32, pixel_samples: u32) {
        let height = (width as f64 / self.camera.aspect_ratio) as u32;
        let image = self.get_image(width, height, pixel_samples);
        println!("Image done generating:");
        println!("Width: {}", width);
        println!("Height: {}", height);
        println!("Samples per pixel: {}", pixel_samples);
        image.save(filename).unwrap();
    }

    // note generate_image_threaded multi-threading is broken
    #[allow(dead_code)]
    pub fn generate_image_threaded(&self, filename: &str, width: u32, pixel_samples: u32) {
        let height = (width as f64 / self.camera.aspect_ratio) as u32;

        let mut image = image::RgbImage::new(width, height).into_vec();

        let channels = 3;

        let threads = num_cpus::get();

        let pixel_chunk_size = ((width * height) as f64 / threads as f64).ceil() as u32;

        image
            .par_chunks_mut((pixel_chunk_size * channels) as usize)
            .enumerate()
            .for_each(|(chunk_index, chunk)| {
                let mut color = Color::new(0.0, 0.0, 0.0);
                let mut rng = rand::thread_rng();

                for (index, value) in chunk.iter_mut().enumerate() {
                    let rgb_i = index % 3;
                    if rgb_i == 0 {
                        color = Color::new(0.0, 0.0, 0.0);
                        let pixel_i = chunk_index as u32 * pixel_chunk_size + index as u32 / 3;
                        let x = pixel_i % width;
                        let y = (pixel_i - x) / width;

                        for _ in 0..pixel_samples {
                            let u = (rng.gen_range(0.0..1.0) + x as f64) / width as f64;
                            let v = (rng.gen_range(0.0..1.0) + y as f64) / height as f64;

                            let mut ray = self.camera.get_ray(u, v);
                            color += ray.get_color(0);
                        }
                        color /= pixel_samples as f64;

                        *value = (color.x.sqrt() * 255.0) as u8;
                    } else if rgb_i % 3 == 1 {
                        *value = (color.y.sqrt() * 255.0) as u8;
                    } else {
                        *value = (color.z.sqrt() * 255.0) as u8;
                    }
                }
            });
        println!("Image done generating:");
        println!("Width: {}", width);
        println!("Height: {}", height);
        println!("Samples per pixel: {}", pixel_samples);
        image::save_buffer(filename, &image, width, height, image::ColorType::Rgb8).unwrap();
    }

    pub fn generate_image_sample_threaded(&self, filename: &str, width: u32, pixel_samples: u32) {
        let height = (width as f64 / self.camera.aspect_ratio) as u32;

        let channels = 3;

        let pixel_num = height * width;

        let image: Arc<Mutex<Vec<f64>>> =
            Arc::new(Mutex::new(vec![0.0; (pixel_num * channels) as usize]));

        let threads = num_cpus::get();

        let sample_chunk_size = (pixel_samples as f64 / threads as f64).floor() as u32;

        let last_chunk_size = pixel_samples - sample_chunk_size * (threads as u32 - 1);

        let mut chunk_sizes: Vec<u32>;

        if (threads as u32) < pixel_samples {
            chunk_sizes = vec![1; pixel_samples as usize];
        } else if last_chunk_size == sample_chunk_size {
            chunk_sizes = vec![sample_chunk_size; threads];
        } else {
            chunk_sizes = vec![sample_chunk_size; threads - 1];
            chunk_sizes.push(last_chunk_size);
        }

        chunk_sizes.par_iter().for_each(|&chunk_size| {
            let image_part = self.get_image_part(width, height, chunk_size, pixel_samples);

            let mut main_image = image.lock().unwrap();

            for (value, sample) in (*main_image).iter_mut().zip(image_part.iter()) {
                *value += sample;
            }
        });

        let image: Vec<u8> = (*(image.lock().unwrap()))
            .iter()
            .map(|value| (value.sqrt() * 255.0) as u8)
            .collect();

        println!("Image done generating:");
        println!("Width: {}", width);
        println!("Height: {}", height);
        println!("Samples per pixel: {}", pixel_samples);
        image::save_buffer(filename, &image, width, height, image::ColorType::Rgb8).unwrap();
    }
}

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
