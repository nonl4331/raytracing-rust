use crate::{AccelerationStructure, Float, Primitive, Ray, Scatter, Vec3};

pub trait Sampler {
	fn sample_image<C, P, M, T, F, A, S>(
		&self,
		_: u64,
		_: u64,
		_: u64,
		_: &C,
		_: &S,
		_: &A,
		_: Option<F>,
		_: &mut Option<T>,
	) where
		C: Camera + Send + Sync,
		P: Primitive<M> + Sync + Send + 'static,
		M: Scatter + Send + Sync + 'static,
		F: Fn(&mut Option<T>, &SamplerProgress, u64) + Send + Sync,
		A: AccelerationStructure<P, M> + Send + Sync,
		S: NoHit + Send + Sync,
		T: Send;
}

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

pub trait Camera {
	fn get_ray(&self, u: Float, v: Float) -> Ray;
}

pub trait NoHit {
	fn get_colour(&self, ray: &Ray) -> Vec3;
}