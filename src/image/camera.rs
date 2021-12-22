use crate::acceleration::bvh::Bvh;
use crate::material::Scatter;
use crate::ray::Colour;
use crate::ray_tracing::intersection::Primitive;
use crate::ray_tracing::sky::Sky;
use std::iter::FromIterator;

use std::sync::Arc;

use std::sync::RwLock;
use std::thread;

use rand::Rng;

use crate::ray_tracing::ray::Ray;
use crate::utility::{
    math::{random_float, Float},
    vec::Vec3,
};

pub struct SamplerProgress {
    pub samples_completed: u64,
    pub rays_shot: u64,
    pub current_image: Vec<Float>,
}

impl SamplerProgress {
    pub fn new(pixel_num: u64, channels: u64) -> Self {
        SamplerProgress {
            samples_completed: 0,
            rays_shot: 0,
            current_image: vec![0.0; (pixel_num * channels) as usize],
        }
    }
}

pub trait Sampler {
    fn sample_image<P, M: 'static>(
        &self,
        _: u64,
        _: u64,
        _: u64,
        _: Arc<Camera>,
        _: Arc<Sky>,
        _: Arc<Bvh<P, M>>,
    ) -> Vec<Arc<RwLock<SamplerProgress>>>
    where
        P: 'static + Primitive<M> + Sync + Send,
        M: Scatter + Send + Sync,
        Vec<P>: FromIterator<P>,
    {
        unimplemented!()
    }
}

pub struct RandomSampler;

impl Sampler for RandomSampler {
    fn sample_image<P, M: 'static>(
        &self,
        samples_per_pixel: u64,
        width: u64,
        height: u64,
        camera: Arc<Camera>,
        sky: Arc<Sky>,
        bvh: Arc<Bvh<P, M>>,
    ) -> Vec<Arc<RwLock<SamplerProgress>>>
    where
        P: 'static + Primitive<M> + Sync + Send,
        M: Scatter + Send + Sync,
        Vec<P>: FromIterator<P>,
    {
        let channels = 3;
        let pixel_num = width * height;

        let mut progresses: Vec<Arc<RwLock<SamplerProgress>>> = Vec::new();

        let threads = num_cpus::get();

        let upper_sample_count = (samples_per_pixel as Float / threads as Float).ceil() as u64;
        let lower_sample_count = (samples_per_pixel as Float / threads as Float).floor() as u64;

        for thread_num in 0..threads {
            let thread_progress = Arc::new(RwLock::new(SamplerProgress::new(pixel_num, channels)));

            progresses.push(thread_progress.clone());

            let thread_sky = sky.clone();
            let thread_bvh = bvh.clone();
            let thread_camera = camera.clone();

            let thread_samples = if samples_per_pixel % threads as u64 <= thread_num as u64 {
                lower_sample_count
            } else {
                upper_sample_count
            };

            thread::spawn(move || {
                let mut rng = rand::thread_rng();

                for thread_i in 0..thread_samples {
                    for pixel_i in 0..pixel_num {
                        let x = pixel_i % width;
                        let y = (pixel_i - x) / width;
                        let mut colour = Colour::new(0.0, 0.0, 0.0);
                        let u = (rng.gen_range(0.0..1.0) + x as Float) / width as Float;
                        let v = 1.0 - (rng.gen_range(0.0..1.0) + y as Float) / height as Float;

                        let mut ray = thread_camera.get_ray(u, v); // remember to add le DOF
                        let result =
                            Ray::get_colour(&mut ray, thread_sky.clone(), thread_bvh.clone());

                        let mut sample_progress = thread_progress.write().unwrap();
                        colour += result.0;
                        sample_progress.rays_shot += result.1;

                        sample_progress.current_image[(pixel_i * channels) as usize] += (colour.x
                            - sample_progress.current_image[(pixel_i * channels) as usize])
                            / (thread_i + 1) as Float;
                        sample_progress.current_image[(pixel_i * channels + 1) as usize] += (colour
                            .y
                            - sample_progress.current_image[(pixel_i * channels + 1) as usize])
                            / (thread_i + 1) as Float;
                        sample_progress.current_image[(pixel_i * channels + 2) as usize] += (colour
                            .z
                            - sample_progress.current_image[(pixel_i * channels + 2) as usize])
                            / (thread_i + 1) as Float;
                    }
                    let mut sample_progress = thread_progress.write().unwrap();
                    sample_progress.samples_completed += 1;
                }
            });
        }

        progresses
    }
}

pub struct Camera {
    pub viewport_width: Float,
    pub viewport_height: Float,
    pub aspect_ratio: Float,
    pub origin: Vec3,
    pub vertical: Vec3,
    pub horizontal: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub lower_left: Vec3,
    pub lens_radius: Float,
}

impl Camera {
    pub fn new(
        origin: Vec3,
        lookat: Vec3,
        vup: Vec3,
        fov: Float,
        aspect_ratio: Float,
        aperture: Float,
        focus_dist: Float,
    ) -> Self {
        let viewport_width = 2.0 * (fov.to_radians() / 2.0).tan();
        let viewport_height = viewport_width / aspect_ratio;

        let w = (origin - lookat).normalised();
        let u = w.cross(vup).normalised();
        let v = u.cross(w);

        let horizontal = focus_dist * u * viewport_width;
        let vertical = focus_dist * v * viewport_height;

        let lower_left = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        Camera {
            viewport_width,
            viewport_height,
            aspect_ratio,
            origin,
            vertical,
            horizontal,
            u,
            v,
            lower_left,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn get_ray(&self, u: Float, v: Float) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left + self.horizontal * u + self.vertical * v - self.origin,
            random_float(),
        )
    }
}
