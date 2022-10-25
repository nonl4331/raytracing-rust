use crate::textures::Texture;
use crate::utility::distribution::*;
use crate::{generate_pdf, next_float, random_float};
use rt_core::Float;
use rt_core::PI;
use rt_core::{NoHit, Ray, Vec3};
use std::sync::Arc;

pub mod random_sampler;

pub struct Sky<T: Texture> {
	texture: Arc<T>,
	pub pdf: Vec<Float>,
	cdf: CDF2D,
	sampler_res: (usize, usize),
}

impl<T: Texture> Sky<T> {
	pub fn new(texture: &Arc<T>, sampler_res: (usize, usize)) -> Self {
		let texture = texture.clone();

		let pdf = generate_pdf(&*texture, sampler_res);

		let cdf = CDF2D::from_pdf(&pdf, sampler_res.0);

		Sky {
			texture,
			pdf,
			cdf,
			sampler_res,
		}
	}
}

impl<T: Texture> NoHit for Sky<T> {
	fn get_colour(&self, ray: &Ray) -> Vec3 {
		self.texture.colour_value(ray.direction, ray.origin)
	}
	fn pdf(&self, wi: Vec3) -> Float {
		let sin_theta = (1.0 - wi.z * wi.z).sqrt();
		if sin_theta <= 0.0 {
			return 0.0;
		}
		let theta = wi.z.acos();
		let mut phi = (wi.y).atan2(wi.x);

		if phi < 0.0 {
			phi += 2.0 * PI;
		}
		let u = phi / (2.0 * PI);
		let v = theta / PI;

		let u_bin = next_float((u * self.sampler_res.0 as Float).floor()) as usize;
		let v_bin = next_float((v * self.sampler_res.1 as Float).floor()) as usize;

		let index =
			(v_bin * self.sampler_res.0 + u_bin).min(self.sampler_res.0 * self.sampler_res.1 - 1);

		self.pdf[index] / (2.0 * PI * PI * sin_theta)
	}
	fn can_sample(&self) -> bool {
		true
	}
	fn sample(&self) -> Vec3 {
		let uv = self.cdf.sample();

		let u = next_float(uv.0 as Float + random_float()) / self.sampler_res.0 as Float;
		let v = next_float(uv.1 as Float + random_float()) / self.sampler_res.1 as Float;

		let phi = u * 2.0 * PI;
		let theta = v * PI;

		Vec3::from_spherical(theta.sin(), theta.cos(), phi.sin(), phi.cos())
	}
}
