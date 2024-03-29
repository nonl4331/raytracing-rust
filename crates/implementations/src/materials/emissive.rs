use crate::{textures::Texture, utility::offset_ray};
use rt_core::*;

#[derive(Debug, Clone)]
pub struct Emit<'a, T: Texture> {
	pub texture: &'a T,
	pub strength: Float,
}

impl<'a, T> Emit<'a, T>
where
	T: Texture,
{
	pub fn new(texture: &'a T, strength: Float) -> Self {
		Emit { texture, strength }
	}
}

impl<'a, T> Scatter for Emit<'a, T>
where
	T: Texture,
{
	fn get_emission(&self, hit: &Hit, wo: Vec3) -> Vec3 {
		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		self.strength * self.texture.colour_value(wo, point)
	}
	fn scattering_pdf(&self, _hit: &Hit, _wo: Vec3, _wi: Vec3) -> Float {
		unreachable!()
	}
	fn is_light(&self) -> bool {
		true
	}
	fn eval(&self, _hit: &Hit, _: Vec3, _: Vec3) -> Vec3 {
		unreachable!()
	}
	fn scatter_ray(&self, _: &mut Ray, _: &Hit) -> bool {
		true
	}
}
