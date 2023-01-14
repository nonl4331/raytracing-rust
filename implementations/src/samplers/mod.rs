use crate::{generate_values, next_float, random_float, textures::Texture};
use rand::{rngs::SmallRng, thread_rng, SeedableRng};
use rt_core::*;
use statistics::distributions::*;
use std::sync::Arc;

pub mod random_sampler;

pub struct Sky<T: Texture> {
	texture: Arc<T>,
	pub distribution: Option<Distribution2D>,
	sampler_res: (usize, usize),
}

impl<T: Texture> Sky<T> {
	pub fn new(texture: &Arc<T>, sampler_res: (usize, usize)) -> Self {
		let texture = texture.clone();

		let values = generate_values(&*texture, sampler_res);

		let distribution = if sampler_res.0 | sampler_res.1 != 0 {
			Some(Distribution2D::new(&values, sampler_res.0))
		} else {
			None
		};

		Sky {
			texture,
			distribution,
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
		self.sampler_res.0 as Float
			* self.sampler_res.1 as Float
			* self.distribution.as_ref().unwrap().pdf(u, v)
			/ (sin_theta * TAU * PI)
	}
	fn can_sample(&self) -> bool {
		self.sampler_res.0 | self.sampler_res.1 != 0
	}
	fn sample(&self) -> Vec3 {
		let uv = self
			.distribution
			.as_ref()
			.unwrap()
			.sample(&mut SmallRng::from_rng(thread_rng()).unwrap());

		let u = next_float(uv.0 as Float + random_float()) / self.sampler_res.0 as Float;
		let v = next_float(uv.1 as Float + random_float()) / self.sampler_res.1 as Float;

		let phi = u * 2.0 * PI;
		let theta = v * PI;

		Vec3::from_spherical(theta.sin(), theta.cos(), phi.sin(), phi.cos())
	}
}

#[cfg(test)]
mod tests {
	use crate::*;
	use rand::rngs::ThreadRng;
	use rt_core::*;
	use statistics::spherical_sampling::test_spherical_pdf;

	#[test]
	fn sky_sampling() {
		let tex = std::sync::Arc::new(AllTextures::Lerp(Lerp::new(Vec3::zero(), Vec3::one())));

		let sky = Sky::new(&tex, (60, 30));

		let pdf = |outgoing: Vec3| sky.pdf(outgoing);
		let sample = |_: &mut ThreadRng| sky.sample();
		test_spherical_pdf("lerp sky sampling", &pdf, &sample, false);
	}
}
