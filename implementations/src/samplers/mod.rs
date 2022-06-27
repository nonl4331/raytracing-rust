use crate::textures::Texture;
use rt_core::NoHit;
use rt_core::Ray;
use rt_core::Vec3;
use std::sync::Arc;

pub mod random_sampler;

pub struct Sky<T: Texture> {
	texture: Option<Arc<T>>,
}

impl<T: Texture> Sky<T> {
	pub fn new(texture: Option<&Arc<T>>) -> Self {
		let texture = texture.cloned();
		Sky { texture }
	}
}

impl<T: Texture> NoHit for Sky<T> {
	fn get_colour(&self, ray: &Ray) -> Vec3 {
		match &self.texture {
			Some(texture) => texture.colour_value(ray.direction, ray.origin),
			None => Vec3::zero(),
		}
	}
}
