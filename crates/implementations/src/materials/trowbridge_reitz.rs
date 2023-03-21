use crate::{
	materials::refract, statistics::bxdfs::trowbridge_reitz, textures::Texture, utility::offset_ray,
};
use rand::{rngs::SmallRng, thread_rng, SeedableRng};
use rt_core::*;

#[derive(Debug, Clone)]
pub struct TrowbridgeReitz<'a, T: Texture> {
	pub texture: &'a T,
	pub alpha: Float,
	pub ior: Vec3,
	pub metallic: Float,
}

impl<'a, T> TrowbridgeReitz<'a, T>
where
	T: Texture,
{
	pub fn new(texture: &'a T, roughness: Float, ior: Vec3, metallic: Float) -> Self {
		Self {
			texture,
			alpha: roughness * roughness,
			ior,
			metallic,
		}
	}

	fn fresnel(&self, hit: &Hit, wo: Vec3, wi: Vec3, h: Vec3) -> Vec3 {
		let f0 = ((1.0 - self.ior) / (1.0 + self.ior)).abs();
		let f0 = f0 * f0;
		let f0 = lerp(f0, self.texture.colour_value(wi, hit.point), self.metallic);
		refract::fresnel((-wo).dot(h), f0)
	}
}

impl<'a, T> Scatter for TrowbridgeReitz<'a, T>
where
	T: Texture,
{
	fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
		let direction = trowbridge_reitz::sample(
			self.alpha,
			ray.direction,
			hit.normal,
			&mut SmallRng::from_rng(thread_rng()).unwrap(),
		);

		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		*ray = Ray::new(point, direction, ray.time);

		false
	}
	fn scattering_pdf(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Float {
		let a = trowbridge_reitz::pdf(self.alpha, wo, wi, hit.normal);
		if a == 0.0 {
			INFINITY
		} else {
			a
		}
	}
	fn eval(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Vec3 {
		let h = (wi - wo).normalised();

		if wi.dot(hit.normal) < 0.0 || h.dot(wo) > 0.0 {
			return Vec3::zero();
		}

		let f = self.fresnel(hit, wo, wi, h);
		let g = trowbridge_reitz::g2(self.alpha, hit.normal, h, wo, wi);
		let d = trowbridge_reitz::d(self.alpha, hit.normal.dot(h));

		f * g * d / (4.0 * (-wo).dot(hit.normal).abs() * wi.dot(hit.normal))
	}
	fn eval_over_scattering_pdf(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Vec3 {
		let h = (wi - wo).normalised();

		if wo.dot(h) > 0.0 || wi.dot(hit.normal) < 0.0 {
			return Vec3::zero();
		}

		self.fresnel(hit, wo, wi, h)
			* trowbridge_reitz::g2(self.alpha, hit.normal, h, wo, wi)
			* (-wo).dot(h)
			/ (wo.dot(hit.normal) * h.dot(hit.normal)).abs()
	}
}

fn lerp(a: Vec3, b: Vec3, t: Float) -> Vec3 {
	(1.0 - t) * a + t * b
}
