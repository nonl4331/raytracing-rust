use crate::{textures::Texture, utility::offset_ray};
use rt_core::{Float, Hit, Ray, Scatter, Vec3};
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
	fn get_emission(&self, hit: &Hit, _wo: Vec3) -> Vec3 {
		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		//TODO
		self.strength * self.texture.colour_value(Vec3::zero(), point)
	}
	fn is_light(&self) -> bool {
		true
	}
	fn scatter_ray(&self, _: &mut Ray, _: &Hit) -> bool {
		true
	}
}
