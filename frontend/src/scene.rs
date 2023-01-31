use implementations::rt_core::*;
use implementations::*;
use rand::Rng;
use rayon::prelude::*;

use bumpalo::Bump;
use ouroboros::self_referencing;

#[self_referencing]
pub struct SceneHolder<S: Sampler, A: AccelerationStructure + Send + Sync, N: NoHit + Send + Sync> {
	arena: Bump,
	#[borrows(arena)]
	#[covariant]
	scene: Scene<'this, S, A, N>,
}

pub struct Scene<'a, S: Sampler, A: AccelerationStructure + Send + Sync, N: NoHit + Send + Sync> {
	arena: &'a Bump,
	camera: SimpleCamera,
	_sampler: S,
	core_scene: Option<SceneData<A, N>>,
}

pub struct SceneData<A: AccelerationStructure + Send + Sync, N: NoHit + Send + Sync> {
	acceleration_structure: A,
	sky: N,
}

impl<
		'a,
		S: Sampler + Send + Sync,
		A: AccelerationStructure + Send + Sync,
		N: NoHit + Send + Sync,
	> Scene<'a, S, A, N>
{
	pub fn render<T, F>(
		&self,
		render_options: RenderOptions,
		mut presentation_update: Option<(&mut T, F)>,
	) where
		F: Fn(&mut T, &SamplerProgress, u64),
	{
		let channels = 3;
		let pixel_num = render_options.width * render_options.height;

		let mut accumulator_buffers = (
			SamplerProgress::new(pixel_num, channels),
			SamplerProgress::new(pixel_num, channels),
		);

		let pixel_chunk_size = 10000;
		let chunk_size = pixel_chunk_size * channels;

		let camera = &self.camera;
		let core_scene = self.core_scene.as_ref().unwrap();
		let (sky, acceleration_structure) = (&core_scene.sky, &core_scene.acceleration_structure);

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
									/ render_options.width as Float;
								let v = 1.0
									- (rng.gen_range(0.0..1.0) + y as Float)
										/ render_options.height as Float;

								let mut ray = camera.get_ray(u, v); // remember to add le DOF
								let result = match render_options.render_method {
									RenderMethod::Naive => {
										Ray::get_colour_naive(&mut ray, sky, acceleration_structure)
									}
									RenderMethod::MIS => {
										Ray::get_colour(&mut ray, sky, acceleration_structure)
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
					f(data, previous, i)
				};
			}
		}

		let (previous, _) = if render_options.samples_per_pixel % 2 == 0 {
			(&accumulator_buffers.0, &mut accumulator_buffers.1)
		} else {
			(&accumulator_buffers.1, &mut accumulator_buffers.0)
		};
		if let Some((ref mut data, f)) = presentation_update.as_mut() {
			f(data, previous, render_options.samples_per_pixel)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use implementations::random_sampler::RandomSampler;
	use implementations::sphere::Sphere;
	use std::collections::HashMap;

	#[test]
	pub fn create_scene() {
		type TextureType = AllTextures;
		type MaterialType<'a> = AllMaterials<'a, TextureType>;

		let mut texture_search: HashMap<&str, &TextureType> = HashMap::new();
		let mut material_search: HashMap<&str, &MaterialType> = HashMap::new();

		let mut scene = SceneHolderBuilder {
			arena: Bump::new(),
			scene_builder: |bump| Scene {
				arena: bump,
				core_scene: None,
				camera: SimpleCamera::new(
					Vec3::new(-5.0, -3.0, 3.0),
					Vec3::new(0.0, 0.0, 0.5),
					Vec3::new(0.0, 0.0, 1.0),
					34.0,
					16.0 / 9.0,
					0.0,
					10.0,
				),
				_sampler: RandomSampler {},
			},
		}
		.build();

		scene.with_mut(|scene| {
			// add textures
			texture_search.insert(
				"Grey",
				scene
					.arena
					.alloc(AllTextures::SolidColour(SolidColour::new(Vec3::new(
						0.5, 0.5, 0.5,
					)))),
			);

			texture_search.insert(
				"White",
				scene
					.arena
					.alloc(AllTextures::SolidColour(SolidColour::new(Vec3::one()))),
			);

			texture_search.insert(
				"black white lerp",
				scene
					.arena
					.alloc(AllTextures::Lerp(Lerp::new(Vec3::zero(), Vec3::one()))),
			);

			// add materials
			material_search.insert(
				"Light",
				scene
					.arena
					.alloc(AllMaterials::Emit(Emit::new(texture_search["White"], 1.5))),
			);

			material_search.insert(
				"Diffuse",
				scene.arena.alloc(AllMaterials::Lambertian(Lambertian::new(
					texture_search["Grey"],
					0.5,
				))),
			);
			let mut primitives = bumpalo::collections::Vec::new_in(scene.arena);

			// add primitives
			primitives.extend([
				AllPrimitives::Sphere(Sphere::new(
					Vec3::new(0.0, 0.0, -1000.0),
					1000.0,
					material_search["Diffuse"],
				)),
				AllPrimitives::Sphere(Sphere::new(
					Vec3::new(0.0, 0.0, 0.5),
					0.5,
					material_search["Light"],
				)),
			]);

			// construct bvh
			let bvh = Bvh::new(primitives, split::SplitType::Sah);

			scene.scene.core_scene = Some(SceneData {
				acceleration_structure: bvh,
				sky: Sky::new(texture_search["black white lerp"], (0, 0)),
			});

			scene.scene.render::<(), fn(&mut _, &_, _)>(
				RenderOptions {
					samples_per_pixel: 1,
					render_method: RenderMethod::MIS,
					width: 100,
					height: 50,
				},
				None,
			);
		});
	}
}
