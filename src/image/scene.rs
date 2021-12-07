use crate::acceleration::{bvh::Bvh, split::SplitType};
use crate::image::camera::Camera;
use crate::ray_tracing::{
    intersection::PrimitiveTrait,
    material::MaterialTrait,
    ray::{Colour, Ray},
    sky::Sky,
};
use crate::utility::{
    math::{random_float, random_in_unit_disk, Float},
    vec::Vec3,
};
use chrono::Local;
use rand::Rng;
use rayon::prelude::*;
use std::{
    iter::FromIterator,
    marker::PhantomData,
    marker::{Send, Sync},
    sync::{mpsc::channel, Arc, Mutex},
    time::{Duration, Instant},
};

const SAMPLES_DEFAULT: u32 = 30;
const WIDTH_DEFAULT: u32 = 800;
const HEIGHT_DEFAULT: u32 = 600;
const FILENAME_DEFAULT: &str = "out.png";

pub struct Parameters {
    pub samples: u32,
    pub width: u32,
    pub height: u32,
    pub filename: String,
}

impl Parameters {
    pub fn new(
        samples: Option<u32>,
        width: Option<u32>,
        height: Option<u32>,
        filename: Option<String>,
    ) -> Self {
        Parameters {
            samples: samples.unwrap_or(SAMPLES_DEFAULT),
            width: width.unwrap_or(WIDTH_DEFAULT),
            height: height.unwrap_or(HEIGHT_DEFAULT),
            filename: filename.unwrap_or_else(|| FILENAME_DEFAULT.to_string()),
        }
    }
}

pub struct Scene<P: PrimitiveTrait<M>, M: MaterialTrait> {
    pub primitives: Arc<Vec<P>>,
    pub bvh: Arc<Bvh>,
    pub camera: Camera,
    pub sky: Arc<Sky>,
    phantom: PhantomData<M>,
}

impl<P, M> Scene<P, M>
where
    P: PrimitiveTrait<M> + Sync + Send,
    M: MaterialTrait + Send + Sync,
    Vec<P>: FromIterator<P>,
{
    pub fn new(
        origin: Vec3,
        lookat: Vec3,
        vup: Vec3,
        fov: Float,
        aspect_ratio: Float,
        aperture: Float,
        focus_dist: Float,
        sky: Sky,
        split_type: SplitType,
        primitives: Vec<P>,
    ) -> Self {
        let mut primitives: Vec<P> = primitives;

        let time = Local::now();

        println!("\n{} - Bvh construction started at", time.format("%X"));

        let start = Instant::now();
        let bvh = Arc::new(Bvh::new(&mut primitives, split_type));
        let end = Instant::now();
        let duration = end.checked_duration_since(start).unwrap();

        println!("\tBvh construction finished in: {}ms", duration.as_millis());
        println!("\tNumber of BVH nodes: {}\n", bvh.number_nodes());

        let camera = Camera::new(origin, lookat, vup, fov, aspect_ratio, aperture, focus_dist);

        Scene {
            primitives: Arc::new(primitives),
            bvh,
            camera,
            sky: Arc::new(sky),
            phantom: PhantomData,
        }
    }

    fn get_image_part(
        &self,
        width: u32,
        height: u32,
        pixel_samples: u32,
        real_pixel_samples: u32,
    ) -> (Vec<Float>, u64) {
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
                let u = (rng.gen_range(0.0..1.0) + x as Float) / width as Float;
                let v = 1.0 - (rng.gen_range(0.0..1.0) + y as Float) / height as Float;

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
            colour /= real_pixel_samples as Float;

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

        let image: Arc<Mutex<Vec<Float>>> =
            Arc::new(Mutex::new(vec![0.0; (pixel_num * channels) as usize]));

        let threads = num_cpus::get();

        let sample_chunk_size = (pixel_samples as Float / threads as Float).floor() as u32;

        let last_chunk_size = pixel_samples - sample_chunk_size * (threads as u32 - 1);

        let mut chunk_sizes: Vec<u32>;

        let time = Local::now();

        println!("{} - Render started", time.format("%X"));
        println!("\tWidth: {}", width);
        println!("\tHeight: {}", height);
        println!("\tSamples per pixel: {}\n", pixel_samples);

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

        let ray_count: u64 = receiver.iter().sum();

        let image: Vec<u8> = (*(image.lock().unwrap()))
            .iter()
            .map(|value| (value.sqrt() * 255.0) as u8)
            .collect();

        let time = Local::now();

        println!("{} - Finised rendering image", time.format("%X"));
        println!("\tRender Time: {}", get_readable_duration(duration));
        println!("\tRays: {}", ray_count);
        println!(
            "\tMrays/s: {:.2}",
            (ray_count as f64 / duration.as_secs_f64()) / 1000000.0
        );
        line_break();
        image::save_buffer(filename, &image, width, height, image::ColorType::Rgb8).unwrap();
    }

    fn get_ray(&self, u: Float, v: Float) -> Ray {
        let rd = self.camera.lens_radius * random_in_unit_disk();
        let offset = rd.x * self.camera.u + rd.y * self.camera.v;
        Ray::new(
            self.camera.origin + offset,
            self.camera.lower_left + self.camera.horizontal * u + self.camera.vertical * v
                - self.camera.origin
                - offset,
            random_float(),
        )
    }
}

pub fn line_break() {
    println!("------------------------------");
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
