//pub mod parse;

pub use implementations::{self, rt_core};
use std::collections::HashMap;

use implementations::{SimpleCamera, Sky, Texture};
use rt_core::*;
use std::{marker::PhantomData, sync::Arc};

pub struct Scene<
	P: Primitive,
	M: Scatter,
	S: Sampler,
	A: AccelerationStructure<Object = P, Material = M>,
	T: Texture,
> {
	pub acceleration_structure: Arc<A>,
	pub camera: Arc<SimpleCamera>,
	pub sampler: Arc<S>,
	pub sky: Arc<Sky<T>>,
	phantom_data: PhantomData<(M, P)>,
}

impl<P, M, S, A, T> Scene<P, M, S, A, T>
where
	P: Primitive + Send + Sync + 'static,
	M: Scatter + Send + Sync + 'static,
	S: Sampler,
	A: AccelerationStructure<Object = P, Material = M> + Send + Sync,
	T: Texture + Send + Sync,
{
	pub fn new(
		camera: Arc<SimpleCamera>,
		sky: Arc<Sky<T>>,
		sampler: Arc<S>,
		acceleration_structure: Arc<A>,
	) -> Self {
		Scene {
			acceleration_structure,
			camera,
			sampler,
			sky,
			phantom_data: PhantomData,
		}
	}
	pub fn generate_image_threaded<D>(
		&self,
		render_options: RenderOptions,
		presentation_update: Option<(&mut D, impl Fn(&mut D, &SamplerProgress, u64))>,
	) {
		self.sampler.sample_image(
			render_options,
			&*self.camera,
			&*self.sky,
			&*self.acceleration_structure,
			presentation_update,
		)
	}
}

use bumpalo::Bump;
use ouroboros::self_referencing;

/*
#[self_referencing]
struct ActualScene<'b, A: AccelerationStructure, M: Scatter + 'static> {
	bump: Bump,
	#[borrows(bump)]
	#[covariant]
	scene: Scene<'b, 'this, A, M>,
}

pub struct Scene<'b, 'a, A: AccelerationStructure<'a>, M: Scatter> {
	acceleration_structure: Option<A>,
	arena: &'a Bump,
	material_search: HashMap<&'b str, &'a M>,
}
*/

#[self_referencing]
pub struct ActualScene<
	'b,
	P: Primitive,
	M: Scatter + 'static,
	S: Sampler,
	A: AccelerationStructure<Object = P, Material = M>,
	T: Texture + 'static,
	N: NoHit,
> {
	arena: Bump,
	#[borrows(arena)]
	#[covariant]
	scene: NewScene<'this, 'b, P, M, S, A, T, N>,
}

pub struct NewScene<
	'a,
	'b,
	P: Primitive,
	M: Scatter,
	S: Sampler,
	A: AccelerationStructure<Object = P, Material = M>,
	T: Texture,
	N: NoHit,
> {
	arena: &'a Bump,
	core_scene: Option<CoreScene<P, M, S, A, N>>,
	texture_search: HashMap<&'b str, &'a T>,
	material_search: HashMap<&'b str, &'a M>,
}

pub struct CoreScene<
	P: Primitive,
	M: Scatter,
	S: Sampler,
	A: AccelerationStructure<Object = P, Material = M>,
	N: NoHit,
> {
	acceleration_structure: A,
	camera: SimpleCamera,
	sampler: S,
	sky: N,
}

#[cfg(test)]
mod tests {
	use super::*;
	use implementations::random_sampler::RandomSampler;
	use implementations::sphere::Sphere;

	#[test]
	pub fn create_scene() {
		type TextureType = AllTextures;
		type MaterialType = AllMaterials<TextureType>;
		type PrimitiveType = AllPrimitives<MaterialType>;

		let mut scene: ActualScene<
			'_,
			PrimitiveType,
			MaterialType,
			RandomSampler,
			Bvh<PrimitiveType, MaterialType>,
			AllTextures,
			Sky<AllTextures>,
		> = ActualSceneBuilder {
			arena: Bump::new(),
			scene_builder: |bump| NewScene {
				arena: bump,
				core_scene: None,
				texture_search: HashMap::new(),
				material_search: HashMap::new(),
			},
		}
		.build();

		use implementations::*;

		// add textures
		scene.with_scene_mut(|scene| {
			let tex = scene
				.arena
				.alloc(AllTextures::SolidColour(SolidColour::new(Vec3::new(
					0.5, 0.5, 0.5,
				))));
			scene.texture_search.insert("Grey", tex);
			let tex = scene
				.arena
				.alloc(AllTextures::SolidColour(SolidColour::new(Vec3::one())));
			scene.texture_search.insert("White", tex);
			let tex = scene
				.arena
				.alloc(AllTextures::Lerp(Lerp::new(Vec3::zero(), Vec3::one())));
			scene.texture_search.insert("black white lerp", tex);
		});

		// add materials
		scene.with_scene_mut(|scene| {
			let mat = scene.arena.alloc(AllMaterials::Emit(Emit::new(
				&Arc::new(scene.texture_search["White"].clone()), // till migration to references
				1.5,
			)));
			scene.material_search.insert("Light", mat as &_);

			let mat = scene.arena.alloc(AllMaterials::Lambertian(Lambertian::new(
				&Arc::new(scene.texture_search["Grey"].clone()),
				0.5,
			)));
			scene.material_search.insert("Diffuse", mat as &_);
		});

		// create primitives and rest of scene
		scene.with_scene_mut(|scene| {
			let primitives = vec![
				AllPrimitives::Sphere(Sphere::new(
					Vec3::new(0.0, 0.0, -1000.0),
					1000.0,
					&Arc::new(scene.material_search["Diffuse"].clone()),
				)),
				AllPrimitives::Sphere(Sphere::new(
					Vec3::new(0.0, 0.0, 0.5),
					0.5,
					&Arc::new(scene.material_search["Light"].clone()),
				)),
			];

			scene.core_scene = Some(CoreScene {
				acceleration_structure: Bvh::new(primitives, split::SplitType::Sah),
				camera: SimpleCamera::new(
					Vec3::new(-5.0, -3.0, 3.0),
					Vec3::new(0.0, 0.0, 0.5),
					Vec3::new(0.0, 0.0, 1.0),
					34.0,
					16.0 / 9.0,
					0.0,
					10.0,
				),
				sampler: RandomSampler {},
				sky: Sky::new(
					&Arc::new(scene.texture_search["black white lerp"].clone()),
					(0, 0),
				),
			});
		});
	}
}
