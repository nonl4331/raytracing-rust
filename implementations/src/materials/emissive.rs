use crate::{rt_core::*, textures::Texture, utility::offset_ray};
use std::sync::Arc;

#[derive(Debug)]
pub struct Emit<T> {
	pub texture: Arc<T>,
	pub strength: Float,
}

impl<T> Emit<T>
where
	T: Texture,
{
	pub fn new(texture: &Arc<T>, strength: Float) -> Self {
		Emit {
			texture: texture.clone(),
			strength,
		}
	}
}

impl<T> Scatter for Emit<T>
where
	T: Texture,
{
	fn get_emission(&self, hit: &Hit, _: Vec3) -> Vec3 {
		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		self.strength * self.texture.colour_value(Vec3::zero(), point)
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
