use rt_core::*;

pub mod random_sampler;

use clap::ValueEnum;

pub trait Sampler: Sync {
	fn sample_image<C, P, M, T, F, A, S>(
		&self,
		_render_options: RenderOptions,
		_camera: &C,
		_sky: &S,
		_acceleration_structure: &A,
		_update_function: Option<(&mut T, F)>,
	) where
		C: Camera,
		P: Primitive,
		M: Scatter,
		F: Fn(&mut T, &SamplerProgress, u64) -> bool,
		A: AccelerationStructure<Object = P, Material = M>,
		S: NoHit;
}

#[derive(Copy, Clone, Debug)]
pub struct RenderOptions {
	pub samples_per_pixel: u64,
	pub render_method: RenderMethod,
	pub width: u64,
	pub height: u64,
}

impl Default for RenderOptions {
	fn default() -> Self {
		Self {
			samples_per_pixel: 100,
			render_method: RenderMethod::MIS,
			width: 1920,
			height: 1080,
		}
	}
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum RenderMethod {
	Naive,
	MIS,
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

pub trait Camera: Sync {
	fn get_ray(&self, u: Float, v: Float) -> Ray;
}
