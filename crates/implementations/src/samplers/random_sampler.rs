use crate::integrators::*;
use crate::*;
use rand::Rng;
use rayon::prelude::*;
use rt_core::*;

pub struct RandomSampler;

impl Sampler for RandomSampler {
	fn sample_image<C, P, M, T, F, A>(
		&self,
		render_options: RenderOptions,
		camera: &C,
		acceleration_structure: &A,
		mut presentation_update: Option<(&mut T, F)>,
	) where
		C: Camera,
		P: Primitive,
		M: Scatter,
		F: Fn(&mut T, &SamplerProgress, u64) -> bool,
		A: AccelerationStructure<Object = P, Material = M>,
	{
		let channels = 3;
		let pixel_num = render_options.width * render_options.height;

		let mut accumulator_buffers = (
			SamplerProgress::new(pixel_num, channels),
			SamplerProgress::new(pixel_num, channels),
		);

		let pixel_chunk_size = 10000;
		let chunk_size = pixel_chunk_size * channels;

		for i in 0..render_options.samples_per_pixel {
			let (previous, current) = if i % 2 == 0 {
				(&accumulator_buffers.0, &mut accumulator_buffers.1)
			} else {
				(&accumulator_buffers.1, &mut accumulator_buffers.0)
			};

			rayon::scope(|s| {
				s.spawn(|_| {
					current.rays_shot = current
						.current_image
						.par_chunks_mut(chunk_size as usize)
						.enumerate()
						.map(|(chunk_i, chunk)| {
							let mut rng = rand::thread_rng();
							let mut rays_shot = 0;
							for chunk_pixel_i in 0..(chunk.len() / 3) {
								let pixel_i =
									chunk_pixel_i as u64 + pixel_chunk_size * chunk_i as u64;
								let x = pixel_i % render_options.width;
								let y = (pixel_i - x) / render_options.width;
								let u = (rng.gen_range(0.0..1.0) + x as Float)
									/ (render_options.width - 1) as Float;
								let v = 1.0
									- (rng.gen_range(0.0..1.0) + y as Float)
										/ (render_options.height - 1) as Float;

								let mut ray = camera.get_ray(u, v); // remember to add le DOF
								let result = match render_options.render_method {
									RenderMethod::Naive => NaiveIntegrator::get_colour(
										&mut ray,
										acceleration_structure,
									),
									RenderMethod::MIS => {
										MisIntegrator::get_colour(&mut ray, acceleration_structure)
									}
								};

								chunk[chunk_pixel_i * channels as usize] = result.0.x;
								chunk[chunk_pixel_i * channels as usize + 1] = result.0.y;
								chunk[chunk_pixel_i * channels as usize + 2] = result.0.z;
								rays_shot += result.1;
							}
							rays_shot
						})
						.sum();
				});
			});
			if i != 0 {
				if let Some((ref mut data, f)) = presentation_update.as_mut() {
					if f(data, previous, i) {
						return;
					}
				};
			}
		}

		let (previous, _) = if render_options.samples_per_pixel % 2 == 0 {
			(&accumulator_buffers.0, &mut accumulator_buffers.1)
		} else {
			(&accumulator_buffers.1, &mut accumulator_buffers.0)
		};
		if let Some((ref mut data, f)) = presentation_update.as_mut() {
			f(data, previous, render_options.samples_per_pixel);
		}
	}
}
