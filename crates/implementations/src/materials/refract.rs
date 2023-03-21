use crate::{
	materials::reflect::Reflect,
	textures::Texture,
	utility::{offset_ray, random_float},
};
use rt_core::*;

#[derive(Debug, Clone)]
pub struct Refract<'a, T: Texture> {
	pub texture: &'a T,
	pub eta: Float,
}

impl<'a, T> Refract<'a, T>
where
	T: Texture,
{
	pub fn new(texture: &'a T, eta: Float) -> Self {
		Refract { texture, eta }
	}
}

impl<'a, T> Scatter for Refract<'a, T>
where
	T: Texture,
{
	fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
		let mut eta_fraction = 1.0 / self.eta;
		if !hit.out {
			eta_fraction = self.eta;
		}

		let cos_theta = ((-ray.direction).dot(hit.normal)).min(1.0);

		let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
		let cannot_refract = eta_fraction * sin_theta > 1.0;
		let f0 = (1.0 - eta_fraction) / (1.0 + eta_fraction);
		let f0 = f0 * f0 * Vec3::one();
		if cannot_refract || fresnel(cos_theta, f0).x > random_float() {
			let ref_mat = Reflect::new(self.texture, 0.0);
			return ref_mat.scatter_ray(ray, hit);
		}

		let perp = eta_fraction * (ray.direction + cos_theta * hit.normal);
		let para = -1.0 * (1.0 - perp.mag_sq()).abs().sqrt() * hit.normal;
		let direction = perp + para;
		let point = offset_ray(hit.point, hit.normal, hit.error, false);
		*ray = Ray::new(point, direction, ray.time);
		false
	}
	fn eval(&self, hit: &Hit, wo: Vec3, _: Vec3) -> Vec3 {
		self.texture.colour_value(wo, hit.point)
	}
	fn is_delta(&self) -> bool {
		true
	}
}

pub fn fresnel(cos: Float, f0: Vec3) -> Vec3 {
	f0 + (1.0 - f0) * (1.0 - cos).powf(5.0)
}
