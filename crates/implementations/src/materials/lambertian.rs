use crate::{textures::Texture, utility::offset_ray};
use rand::{rngs::SmallRng, thread_rng, SeedableRng};
use rt_core::*;

#[derive(Debug, Clone)]
pub struct Lambertian<'a, T: Texture> {
	pub texture: &'a T,
	pub albedo: Float,
}

#[cfg(all(feature = "f64"))]
use std::f64::consts::PI;

#[cfg(not(feature = "f64"))]
use std::f32::consts::PI;

impl<'a, T> Lambertian<'a, T>
where
	T: Texture,
{
	pub fn new(texture: &'a T, albedo: Float) -> Self {
		Lambertian { texture, albedo }
	}
}

impl<'a, T> Scatter for Lambertian<'a, T>
where
	T: Texture,
{
	fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
		let direction = crate::statistics::bxdfs::lambertian::sample(
			ray.direction,
			hit.normal,
			&mut SmallRng::from_rng(thread_rng()).unwrap(),
		);

		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		*ray = Ray::new(point, direction, ray.time);

		false
	}
	fn scattering_pdf(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Float {
		crate::statistics::bxdfs::lambertian::pdf(wo, wi, hit.normal)
	}
	fn eval(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Vec3 {
		self.texture.colour_value(wo, hit.point) * self.albedo * hit.normal.dot(wi).max(0.0) / PI
	}
	fn eval_over_scattering_pdf(&self, hit: &Hit, wo: Vec3, _: Vec3) -> Vec3 {
		self.texture.colour_value(wo, hit.point) * self.albedo
	}
}
