use crate::{
	materials::refract,
	statistics,
	textures::Texture,
	utility::{coord::Coordinate, offset_ray},
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

	fn microfacet_ndf_ggx(&self, hit: &Hit, h: Vec3) -> Float {
		let noh = hit.normal.dot(h);
		let alpha_sq = self.alpha * self.alpha;
		let noh_sq = noh * noh;
		let den = noh_sq * (alpha_sq - 1.0) + 1.0;
		alpha_sq / (PI * den * den)
	}
}

impl<'a, T> Scatter for TrowbridgeReitz<'a, T>
where
	T: Texture,
{
	fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
		let coord = Coordinate::new_from_z(hit.normal);

		let mut h = statistics::bxdfs::trowbridge_reitz::sample_h(
			self.alpha,
			&mut SmallRng::from_rng(thread_rng()).unwrap(),
		);

		coord.vec_to_coordinate(&mut h);

		let direction = ray.direction.reflected(h);

		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		*ray = Ray::new(point, direction, ray.time);

		false
	}
	fn scattering_pdf(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Float {
		let wo = -wo;
		let a = statistics::bxdfs::trowbridge_reitz::pdf_outgoing(self.alpha, wo, wi, hit.normal);
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
			* self.microfacet_ndf_ggx(hit, h)
			/ (4.0 * wo.dot(hit.normal) * wi.dot(hit.normal));

		spec_component * hit.normal.dot(wi).abs()
	}
	fn eval_over_scattering_pdf(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Vec3 {
		let wo = -wo;
		let h = (wi + wo).normalised();

		if wo.dot(h) < 0.0 || wi.dot(hit.normal) < 0.0 {
			return Vec3::zero();
		}

		self.microfacet_ndf_ggx(hit, h);

		self.fresnel(hit, wo, wi, h) * self.geometry_ggx(h, wo, wi)
			/ self.geometry_partial_ggx(h, wo)
	}
}

fn lerp(a: Vec3, b: Vec3, t: Float) -> Vec3 {
	(1.0 - t) * a + t * b
}
