use rand::Rng;
use rt_core::Sampler;
use rt_core::SamplerProgress;
use rt_core::Scatter;

use rayon::prelude::*;
use rt_core::Camera;

use rt_core::Primitive;

use rt_core::PrimitiveSampling;

use rt_core::NoHit;

use rt_core::Float;

use rt_core::Ray;

pub struct RandomSampler;

impl Sampler for RandomSampler {
	fn sample_image<C, P, M, T, F, A, S>(
		&self,
		samples_per_pixel: u64,
		width: u64,
		height: u64,
		camera: &C,
		sky: &S,
		acceleration_structure: &A,
		presentation_update: Option<F>,
		data: &mut Option<T>,
	) where
		C: Camera + Send + Sync,
		P: Primitive<M> + Sync + Send + 'static,
		M: Scatter + Send + Sync + 'static,
		F: Fn(&mut Option<T>, &SamplerProgress, u64) + Send + Sync,
		A: PrimitiveSampling<P, M> + Send + Sync,
		S: NoHit + Send + Sync,
		T: Send,
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
