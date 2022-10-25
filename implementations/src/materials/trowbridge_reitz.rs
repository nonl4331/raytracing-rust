use crate::utility::coord::Coordinate;
use crate::{
	materials::refract,
	textures::Texture,
	utility::{offset_ray, random_float},
};
use rt_core::{Float, Hit, Ray, Scatter, Vec3};
use std::sync::Arc;

#[derive(Debug)]
pub struct TrowbridgeReitz<T: Texture> {
	pub texture: Arc<T>,
	pub alpha: Float,
	pub ior: Vec3,
	pub metallic: Float,
}

#[cfg(all(feature = "f64"))]
use std::f64::{consts::PI, INFINITY};

#[cfg(not(feature = "f64"))]
use std::f32::{consts::PI, INFINITY};

impl<T> TrowbridgeReitz<T>
where
	T: Texture,
{
	pub fn new(texture: &Arc<T>, roughness: Float, ior: Vec3, metallic: Float) -> Self {
		Self {
			texture: texture.clone(),
			alpha: roughness * roughness,
			ior,
			metallic,
		}
	}

	fn fresnel(&self, hit: &Hit, wo: Vec3, wi: Vec3, h: Vec3) -> Vec3 {
		let f0 = ((1.0 - self.ior) / (1.0 + self.ior)).abs();
		let f0 = f0 * f0;
		let f0 = lerp(f0, self.texture.colour_value(wi, hit.point), self.metallic);
		refract::fresnel(wo.dot(h), f0)
	}

	fn geometry_partial_ggx(&self, h: Vec3, v: Vec3) -> Float {
		1.0 / (1.0 + self.lambda_ggx(h, v))
	}

	fn geometry_ggx(&self, h: Vec3, wo: Vec3, wi: Vec3) -> Float {
		1.0 / (1.0 + self.lambda_ggx(h, wo) + self.lambda_ggx(h, wi))
	}

	fn lambda_ggx(&self, h: Vec3, v: Vec3) -> Float {
		let voh = v.dot(h);
		let voh_sq = voh * voh;
		let tan_sq = (1.0 - voh_sq) / voh_sq;

		((1.0 + self.alpha * self.alpha * tan_sq).sqrt() - 1.0) * 0.5
	}

	fn distribution_ggx(&self, hit: &Hit, h: Vec3) -> Float {
		let noh = hit.normal.dot(h);
		let alpha_sq = self.alpha * self.alpha;
		let noh_sq = noh * noh;
		let den = noh_sq * (alpha_sq - 1.0) + 1.0;
		alpha_sq / (PI * den * den)
	}

	fn sample_h(&self, hit: &Hit, _: Vec3) -> Vec3 {
		let coord = Coordinate::new_from_z(hit.normal);

		let r1 = random_float();
		let r2 = random_float();
		let cos_theta = ((1.0 - r1) / (r1 * (self.alpha * self.alpha - 1.0) + 1.0)).sqrt();
		let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
		let phi_s = (2.0 * PI * r2).max(0.0).min(2.0 * PI);

		let mut h =
			Vec3::new(phi_s.cos() * sin_theta, phi_s.sin() * sin_theta, cos_theta).normalised();

		coord.vec_to_coordinate(&mut h);

		h
	}
}

impl<T> Scatter for TrowbridgeReitz<T>
where
	T: Texture,
{
	fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
		let wo = -ray.direction;

		let h = self.sample_h(hit, wo);

		let direction = ray.direction.reflected(h);

		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		*ray = Ray::new(point, direction, ray.time);

		false
	}
	fn scattering_pdf(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Float {
		let wo = -wo;
		let h = (wo + wi).normalised();
		let a = self.distribution_ggx(hit, h) * h.dot(hit.normal).abs() / (4.0 * wo.dot(h));
		if a == 0.0 {
			INFINITY
		} else {
			a
		}
	}
	fn eval(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Vec3 {
		let wo = -wo;
		let h = (wi + wo).normalised();

		if wi.dot(hit.normal) < 0.0 || h.dot(wo) < 0.0 {
			return Vec3::zero();
		}

		let spec_component = self.fresnel(hit, wo, wi, h)
			* self.geometry_ggx(h, wo, wi)
			* self.distribution_ggx(hit, h)
			/ (4.0 * wo.dot(hit.normal) * wi.dot(hit.normal));

		spec_component * hit.normal.dot(wi).abs()
	}
	fn eval_over_scattering_pdf(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Vec3 {
		let wo = -wo;
		let h = (wi + wo).normalised();

		if wo.dot(h) < 0.0 || wi.dot(hit.normal) < 0.0 {
			return Vec3::zero();
		}

		self.distribution_ggx(hit, h);

		self.fresnel(hit, wo, wi, h) * self.geometry_ggx(h, wo, wi)
			/ self.geometry_partial_ggx(h, wo)
	}
}

fn lerp(a: Vec3, b: Vec3, t: Float) -> Vec3 {
	(1.0 - t) * a + t * b
}
