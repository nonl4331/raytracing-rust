use crate::ray_tracing::{
	texture::{TextureEnum, TextureTrait},
	Colour, Ray,
};
use std::sync::Arc;

pub struct Sky {
	texture: Option<Arc<TextureEnum>>,
}

impl Sky {
	pub fn new(texture: Option<&Arc<TextureEnum>>) -> Self {
		let texture = texture.cloned();
		Sky { texture }
	}

	pub fn get_colour(&self, ray: &Ray) -> Colour {
		match &self.texture {
			Some(texture) => texture.colour_value(ray.direction, ray.origin),
			None => Colour::zero(),
		}
	}
}
