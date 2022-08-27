use crate::{
	materials::refract,
	textures::Texture,
	utility::{coord::Coordinate, offset_ray, random_float},
};
use rt_core::{Float, Hit, Ray, Scatter, Vec3};
use std::{f32::INFINITY, sync::Arc};

#[derive(Debug)]
pub struct CookTorrence<T: Texture> {
	pub texture: Arc<T>,
	pub alpha: Float,
	pub ior: Vec3,
	pub metallic: Float,
	pub ks: Float,
}

#[cfg(all(feature = "f64"))]
use std::f64::consts::PI;

#[cfg(not(feature = "f64"))]
use std::f32::consts::PI;

impl<T> CookTorrence<T>
where
	T: Texture,
{
	pub fn new(texture: &Arc<T>, roughness: Float, ior: Vec3, metallic: Float, ks: Float) -> Self {
		Self {
			texture: texture.clone(),
			alpha: roughness * roughness,
			ior,
			metallic,
			ks,
		}
	}

	fn fresnel(&self, hit: &Hit, wo: Vec3, wi: Vec3, h: Vec3) -> Vec3 {
		let f0 = ((1.0 - self.ior) / (1.0 + self.ior)).abs();
		let f0 = f0 * f0;
		let f0 = lerp(f0, self.texture.colour_value(wi, hit.point), self.metallic);
		refract::fresnel(wo.dot(h), f0)
	}

	fn geometry_partial_ggx(&self, hit: &Hit, h: Vec3, v: Vec3) -> Float {
		let voh = v.dot(h);
		let chi = chi(voh / v.dot(hit.normal));
		let voh_sq = voh * voh;
		let tan_sq = (1.0 - voh_sq) / voh_sq;
		(2.0 * chi) / (1.0 + (1.0 + self.alpha * self.alpha * tan_sq).sqrt())
	}

	fn geometry_ggx(&self, hit: &Hit, h: Vec3, wo: Vec3, wi: Vec3) -> Float {
		self.geometry_partial_ggx(hit, h, wo) * self.geometry_partial_ggx(hit, h, wi)
	}

	fn distribution_ggx(&self, hit: &Hit, h: Vec3) -> Float {
		let noh = hit.normal.dot(h);
		let alpha_sq = self.alpha * self.alpha;
		let noh_sq = noh * noh;
		let den = noh_sq * (alpha_sq - 1.0) + 1.0;
		chi(noh) * alpha_sq / (PI * den * den)
	}
}

impl<T> Scatter for CookTorrence<T>
where
	T: Texture,
{
	fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
		let r1 = random_float();
		let r2 = random_float();
		let cos_theta = ((1.0 - r1) / (r1 * (self.alpha * self.alpha - 1.0) + 1.0)).sqrt();
		let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
		let phi_s = 2.0 * PI * r2;

		let mut h =
			Vec3::new(phi_s.cos() * sin_theta, phi_s.sin() * sin_theta, cos_theta).normalised();
		let coordinate_system = Coordinate::new_from_z(hit.normal);
		coordinate_system.vec_to_coordinate(&mut h);
		let wo = -ray.direction;

		if h.dot(hit.normal) < 0.0 {
			h = -h;
		}

		let direction = 2.0 * (h.dot(wo)) * h - wo;

		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		*ray = Ray::new(point, direction, ray.time);

		false
	}
	fn scattering_pdf(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Float {
		let wo = -wo;
		let h = (wo + wi).normalised();
		let a = self.distribution_ggx(hit, h) * h.dot(hit.normal) / (4.0 * wo.dot(h).abs());
		if a == 0.0 {
			INFINITY
		} else {
			a
		}
	}
	fn eval(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Vec3 {
		let wi = if wi.dot(hit.normal) < 0.0 { -wi } else { wi };
		let wo = -wo;
		let h = (wi + wo).normalised();

		let spec_component = self.fresnel(hit, wo, wi, h)
			* self.geometry_ggx(hit, h, wo, wi)
			* self.distribution_ggx(hit, h)
			/ (4.0 * wo.dot(hit.normal) * wi.dot(hit.normal));

		let diffuse_component = self.texture.colour_value(wi, hit.point) / PI;
		((1.0 - self.ks) * diffuse_component + self.ks * spec_component) * hit.normal.dot(wi).abs()
	}
}

fn lerp(a: Vec3, b: Vec3, t: Float) -> Vec3 {
	(1.0 - t) * a + t * b
}

fn chi(val: Float) -> Float {
	if val > 0.0 {
		1.0
	} else {
		0.0
	}
}
