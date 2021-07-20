use crate::bvh::bvh::{SplitType, BVH};

use crate::image::camera::Camera;

use crate::math::{random_f32, random_in_unit_disk};

use crate::parameters::Parameters;

use crate::ray_tracing::{
    ray::{Colour, Ray},
    sky::Sky,
    tracing::{Primitive, PrimitiveTrait},
};

use rand::Rng;

use rayon::prelude::*;

use std::sync::{mpsc::channel, Arc, Mutex};

use std::time::{Duration, Instant};

use ultraviolet::vec::Vec3;

pub type PrimitivesType = Arc<Vec<Primitive>>;

pub struct Scene {
    pub primitives: PrimitivesType,
    pub bvh: Arc<BVH>,
    pub camera: Camera,
    pub sky: Arc<Sky>,
}

impl Scene {
    pub fn new(
        origin: Vec3,
        lookat: Vec3,
        vup: Vec3,
        fov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_dist: f32,
        sky: Sky,
        mut primitives: Vec<Primitive>,
    ) -> Self {
        let primitives: Vec<Primitive> = primitives
            .drain(..)
            .flat_map(|primitive| primitive.get_internal())
            .collect();

        let primitives: PrimitivesType = Arc::new(primitives);

        let bvh = Arc::new(BVH::new(&primitives, SplitType::SAH));

        let camera = Camera::new(origin, lookat, vup, fov, aspect_ratio, aperture, focus_dist);

        Scene {
            primitives,
            bvh,
            camera,
            sky: Arc::new(sky),
        }
    }

    fn get_image_part(
        &self,
        width: u32,
        height: u32,
        pixel_samples: u32,
        real_pixel_samples: u32,
    ) -> (Vec<f32>, u64) {
        let channels = 3;
        let pixel_num = height * width;

        let mut rgb_vec = Vec::with_capacity((pixel_num * channels) as usize);

        let mut rng = rand::thread_rng();

        let mut ray_count = 0;

        for pixel_i in 0..pixel_num {
            let x = pixel_i % width;
            let y = (pixel_i - x) / width;
            let mut colour = Colour::new(0.0, 0.0, 0.0);
            for _ in 0..pixel_samples {
                let u = (rng.gen_range(0.0..1.0) + x as f32) / width as f32;
                let v = 1.0 - (rng.gen_range(0.0..1.0) + y as f32) / height as f32;

                let mut ray = self.get_ray(u, v);
                let result = Ray::get_colour(
                    &mut ray,
                    self.sky.clone(),
                    self.bvh.clone(),
                    self.primitives.clone(),
                );
                colour += result.0;
                ray_count += result.1;
            }
            colour /= real_pixel_samples as f32;

            rgb_vec.push(colour.x);
            rgb_vec.push(colour.y);
            rgb_vec.push(colour.z);
        }
        (rgb_vec, ray_count)
    }

    pub fn generate_image_threaded(&self, options: Parameters) {
        let pixel_samples = options.samples;
        let width = options.width;
        let filename = options.filename;
        let height = options.height;

        let channels = 3;

        let pixel_num = height * width;

        let image: Arc<Mutex<Vec<f32>>> =
            Arc::new(Mutex::new(vec![0.0; (pixel_num * channels) as usize]));

        let threads = num_cpus::get();

        let sample_chunk_size = (pixel_samples as f32 / threads as f32).floor() as u32;

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

        let (sender, receiver) = channel();

        let start = Instant::now();

        chunk_sizes
            .par_iter()
            .for_each_with(sender, |sender, &chunk_size| {
                let result = self.get_image_part(width, height, chunk_size, pixel_samples);
                sender.send(result.1).unwrap();

                let mut main_image = image.lock().unwrap();

                for (value, sample) in (*main_image).iter_mut().zip(result.0.iter()) {
                    *value += sample;
                }
            });

        let end = Instant::now();
        let duration = end.checked_duration_since(start).unwrap();

        let ray_count = receiver
            .iter()
            .fold(0, |rays, partial_rays| rays + partial_rays);

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
        println!("Render Time: {}", get_readable_duration(duration));
        println!("Rays: {}", ray_count);
        println!(
            "Mrays/s: {:.2}",
            (ray_count as f32 / duration.as_secs_f32()) / 1000000.0
        );
        println!("------------------------------");
        image::save_buffer(filename, &image, width, height, image::ColorType::Rgb8).unwrap();
    }

    fn get_ray(&self, u: f32, v: f32) -> Ray {
        let rd = self.camera.lens_radius * random_in_unit_disk();
        let offset = rd.x * self.camera.u + rd.y * self.camera.v;
        Ray::new(
            self.camera.origin + offset,
            self.camera.lower_left + self.camera.horizontal * u + self.camera.vertical * v
                - self.camera.origin
                - offset,
            random_f32(),
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
