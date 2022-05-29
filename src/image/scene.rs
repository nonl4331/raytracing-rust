use crate::acceleration::bvh::PrimitiveSampling;
use crate::image::camera::{Camera, Sampler, SamplerProgress};
use crate::ray_tracing::intersection::Primitive;
use crate::ray_tracing::{material::Scatter, sky::Sky};

use std::marker::PhantomData;
use std::{
	marker::{Send, Sync},
	sync::Arc,
};

pub struct Scene<P: Primitive<M>, M: Scatter, S: Sampler, A: PrimitiveSampling<P, M>> {
	pub acceleration_structure: Arc<A>,
	pub camera: Arc<Camera>,
	pub sampler: Arc<S>,
	pub sky: Arc<Sky>,
	phantom_data: PhantomData<(M, P)>,
}

impl<P, M, S, A> Scene<P, M, S, A>
where
	P: Primitive<M> + Send + Sync + 'static,
	M: Scatter + Send + Sync + 'static,
	S: Sampler,
	A: PrimitiveSampling<P, M> + Send + Sync,
{
	pub fn new(
		camera: Arc<Camera>,
		sky: Arc<Sky>,
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
	pub fn generate_image_threaded<T>(
		&self,
		width: u64,
		height: u64,
		samples: u64,
		presentation_update: Option<impl Fn(&mut Option<T>, &SamplerProgress, u64) + Send + Sync>,
		data: &mut Option<T>,
	) where
		T: Send,
	{
		self.sampler.sample_image(
			samples,
			width,
			height,
			&self.camera,
			&self.sky,
			&*self.acceleration_structure,
			presentation_update,
			data,
		)
	}
}
