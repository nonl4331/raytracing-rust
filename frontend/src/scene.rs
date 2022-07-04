use implementations::{SimpleCamera, Sky, Texture};
use rt_core::{AccelerationStructure, Primitive, Sampler, SamplerProgress, Scatter};
use std::{marker::PhantomData, sync::Arc};

pub struct Scene<
	P: Primitive<M>,
	M: Scatter,
	S: Sampler,
	A: AccelerationStructure<P, M>,
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
	P: Primitive<M> + Send + Sync + 'static,
	M: Scatter + Send + Sync + 'static,
	S: Sampler,
	A: AccelerationStructure<P, M> + Send + Sync,
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
		width: u64,
		height: u64,
		samples: u64,
		presentation_update: Option<(&mut D, impl Fn(&mut D, &SamplerProgress, u64))>,
	) {
		self.sampler.sample_image(
			samples,
			width,
			height,
			&*self.camera,
			&*self.sky,
			&*self.acceleration_structure,
			presentation_update,
		)
	}
}
