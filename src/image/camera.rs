use crate::{
	acceleration::PrimitiveSampling,
	ray_tracing::{intersection::Primitive, material::Scatter, sky::Sky, Ray},
	utility::{random_float, vec::Vec3, Float},
};
use rand::Rng;
use rayon::prelude::*;

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
	fn sample_image<P, M, T, F, A>(
		&self,
		_: u64,
		_: u64,
		_: u64,
		_: &Camera,
		_: &Sky,
		_: &A,
		_: Option<F>,
		_: &mut Option<T>,
	) where
		P: Primitive<M> + Sync + Send + 'static,
		M: Scatter + Send + Sync + 'static,
		F: Fn(&mut Option<T>, &SamplerProgress, u64) + Send + Sync,
		A: PrimitiveSampling<P, M> + Send + Sync,
		T: Send;
}

pub struct RandomSampler;

impl Sampler for RandomSampler {
	fn sample_image<P, M, T, F, A>(
		&self,
		samples_per_pixel: u64,
		width: u64,
		height: u64,
		camera: &Camera,
		sky: &Sky,
		acceleration_structure: &A,
		presentation_update: Option<F>,
		data: &mut Option<T>,
	) where
		P: Primitive<M> + Sync + Send + 'static,
		M: Scatter + Send + Sync + 'static,
		F: Fn(&mut Option<T>, &SamplerProgress, u64) + Send + Sync,
		T: Send,
		A: PrimitiveSampling<P, M> + Send + Sync,
	{
		let channels = 3;
		let pixel_num = width * height;

		let mut accumulator_buffers = (
			SamplerProgress::new(pixel_num, channels),
			SamplerProgress::new(pixel_num, channels),
		);

		let pixel_chunk_size = 10000;
		let chunk_size = pixel_chunk_size * channels;

		for i in 0..samples_per_pixel {
			let (previous, current) = if i % 2 == 0 {
				(&accumulator_buffers.0, &mut accumulator_buffers.1)
			} else {
				(&accumulator_buffers.1, &mut accumulator_buffers.0)
			};

			rayon::scope(|s| {
				if i != 0 {
					s.spawn(|_| match presentation_update.as_ref() {
						Some(f) => f(data, previous, i),
						None => (),
					});
				}

				current.rays_shot = current
					.current_image
					.par_chunks_mut(chunk_size as usize)
					.enumerate()
					.map(|(chunk_i, chunk)| {
						let mut rng = rand::thread_rng();
						let mut rays_shot = 0;
						for chunk_pixel_i in 0..(chunk.len() / 3) {
							let pixel_i = chunk_pixel_i as u64 + pixel_chunk_size * chunk_i as u64;
							let x = pixel_i as u64 % width;
							let y = (pixel_i as u64 - x) / width;
							let u = (rng.gen_range(0.0..1.0) + x as Float) / width as Float;
							let v = 1.0 - (rng.gen_range(0.0..1.0) + y as Float) / height as Float;

							let mut ray = camera.get_ray(u, v); // remember to add le DOF
							let result = Ray::get_colour(&mut ray, sky, acceleration_structure);

							chunk[chunk_pixel_i * channels as usize] = result.0.x;
							chunk[chunk_pixel_i * channels as usize + 1] = result.0.y;
							chunk[chunk_pixel_i * channels as usize + 2] = result.0.z;
							rays_shot += result.1;
						}
						rays_shot
					})
					.sum();
			});
		}

		let (previous, _) = if samples_per_pixel % 2 == 0 {
			(&accumulator_buffers.0, &mut accumulator_buffers.1)
		} else {
			(&accumulator_buffers.1, &mut accumulator_buffers.0)
		};
		match presentation_update.as_ref() {
			Some(f) => f(data, previous, samples_per_pixel),
			None => (),
		}
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
