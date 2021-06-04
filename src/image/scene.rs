use crate::image::bvh::BVH;
use crate::image::camera::Camera;

use crate::image::math::random_f64;
use crate::image::math::random_in_unit_disk;
use crate::image::ray::Ray;
use crate::image::tracing::Hittable;
use crate::parameters::Parameters;

use crate::image::ray::Color;

use crate::image::sky::Sky;

use std::sync::Mutex;

use rand::Rng;

use std::sync::Arc;

use ultraviolet::vec::DVec3;

use rayon::prelude::*;

use std::time::{Duration, Instant};

pub type HittablesType = Arc<Vec<Hittable>>;

pub struct Scene {
    pub hittables: HittablesType,
    pub bvh: Arc<BVH>,
    pub camera: Camera,
    pub sky: Sky,
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
        sky: Sky,
        hittables: Vec<Hittable>,
    ) -> Self {
        let hittables: HittablesType = Arc::new(hittables);

        let bvh = Arc::new(BVH::new(&hittables));

        let camera = Camera::new(
            origin,
            lookat,
            vup,
            fov,
            aspect_ratio,
            aperture,
            focus_dist,
            sky,
        );

        Scene {
            hittables,
            bvh,
            camera,
            sky,
        }
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
                let v = 1.0 - (rng.gen_range(0.0..1.0) + y as f64) / height as f64;

                let mut ray = self.get_ray(u, v);
                color += ray.get_color(0);
            }
            color /= real_pixel_samples as f64;

            rgb_vec.push(color.x);
            rgb_vec.push(color.y);
            rgb_vec.push(color.z);
        }
        rgb_vec
    }

    pub fn generate_image_threaded(&self, options: Parameters) {
        let pixel_samples = options.samples;
        let width = options.width;
        let filename = options.filename;
        let height = options.height;

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

        let start = Instant::now();

        chunk_sizes.par_iter().for_each(|&chunk_size| {
            let image_part = self.get_image_part(width, height, chunk_size, pixel_samples);

            let mut main_image = image.lock().unwrap();

            for (value, sample) in (*main_image).iter_mut().zip(image_part.iter()) {
                *value += sample;
            }
        });

        let end = Instant::now();

        let image: Vec<u8> = (*(image.lock().unwrap()))
            .iter()
            .map(|value| (value.sqrt() * 255.0) as u8)
            .collect();

        println!("------------------------------");
        println!("Finised rendering image!");
        println!("------------------------------");
        println!("Width: {}", width);
        println!("Height: {}", height);
        println!("Samples per pixel: {}", pixel_samples);
        println!(
            "Render Time: {}",
            get_readable_duration(end.checked_duration_since(start).unwrap())
        );
        println!("------------------------------");
        image::save_buffer(filename, &image, width, height, image::ColorType::Rgb8).unwrap();
    }

    fn get_ray(&self, u: f64, v: f64) -> Ray {
        let rd = self.camera.lens_radius * random_in_unit_disk();
        let offset = rd.x * self.camera.u + rd.y * self.camera.v;
        Ray::new(
            self.camera.origin + offset,
            self.camera.lower_left + self.camera.horizontal * u + self.camera.vertical * v
                - self.camera.origin
                - offset,
            random_f64(),
            self.sky,
            self.hittables.clone(),
            self.bvh.clone(),
        )
    }
}

fn get_readable_duration(duration: Duration) -> String {
    let days = duration.as_secs() / 86400;

    let days_string = match days {
        0 => "".to_string(),
        1 => format!("{} day, ", days),
        _ => format!("{} days, ", days),
    };

    let hours = (duration.as_secs() - days * 86400) / 3600;
    let hours_string = match hours {
        0 => "".to_string(),
        1 => format!("{} hour, ", hours),
        _ => format!("{} hours, ", hours),
    };

    let minutes = (duration.as_secs() - days * 86400 - hours * 3600) / 60;
    let minutes_string = match minutes {
        0 => "".to_string(),
        1 => format!("{} minute, ", minutes),
        _ => format!("{} minutes, ", minutes),
    };

    let seconds = duration.as_secs() % 60;
    let seconds_string = match seconds {
        0 => "~0 seconds".to_string(),
        1 => format!("{} second", seconds),
        _ => format!("{} seconds", seconds),
    };
    days_string + &hours_string + &minutes_string + &seconds_string
}
