use crate::distributions::Distribution2D;
use crate::generate_values;
use crate::next_float;
use crate::random_float;
use rand::rngs::SmallRng;
use rand::thread_rng;
use rand::SeedableRng;
use rt_core::*;

use crate::Texture;

#[derive(Debug, Clone)]
pub struct Sky<'a, T: Texture, M: Scatter> {
	texture: &'a T,
	mat: &'a M,
	pub distribution: Option<Distribution2D>,
	sampler_res: (usize, usize),
}

impl<'a, T: Texture, M: Scatter> Sky<'a, T, M> {
	pub fn new(texture: &'a T, mat: &'a M, sampler_res: (usize, usize)) -> Self {
		let values = generate_values(texture, sampler_res);

		let distribution = if sampler_res.0 | sampler_res.1 != 0 {
			Some(Distribution2D::new(&values, sampler_res.0))
		} else {
			None
		};

		Sky {
			texture,
			mat,
			distribution,
			sampler_res,
		}
	}
}

impl<'a, T: Texture, M: Scatter> NoHit<M> for Sky<'a, T, M> {
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
	fn get_si(&self, _ray: &Ray) -> SurfaceIntersection<M> {
		SurfaceIntersection {
			hit: Hit {
				t: 0.0,
				point: Vec3::zero(),
				error: Vec3::zero(),
				normal: Vec3::zero(),
				uv: None,
				out: false,
			},
			material: self.mat,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	//use crate::spherical_sampling::test_spherical_pdf;
	use crate::AllMaterials;
	use crate::AllTextures;
	use crate::Emit;
	use crate::Lerp;
	//use rand::rngs::ThreadRng;

	#[test]
	fn sky_sampling() {
		let tex = AllTextures::Lerp(Lerp::new(Vec3::zero(), Vec3::one()));
		let mat = AllMaterials::Emit(Emit::new(&tex, 1.0));

		let _sky = Sky::new(&tex, &mat, (60, 30));

		/*let pdf = |outgoing: Vec3| sky.pdf(outgoing);
		let sample = |_: &mut ThreadRng| sky.sample();
		test_spherical_pdf("lerp sky sampling", &pdf, &sample, false);*/
		todo!()
	}
}
