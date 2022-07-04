use crate::{
	textures::Texture,
	utility::{offset_ray, random_unit_vector},
};
use rt_core::{Float, Hit, Ray, Scatter, Vec3};
use std::sync::Arc;

#[derive(Debug)]
pub struct Reflect<T: Texture> {
	pub texture: Arc<T>,
	pub fuzz: Float,
}

impl<T> Reflect<T>
where
	T: Texture,
{
	pub fn new(texture: &Arc<T>, fuzz: Float) -> Self {
		Reflect {
			texture: texture.clone(),
			fuzz,
		}
	}
}

impl<T> Scatter for Reflect<T>
where
	T: Texture,
{
	fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
		let mut direction = ray.direction;
		direction.reflect(hit.normal);
		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		*ray = Ray::new(
			point,
			direction + self.fuzz * random_unit_vector(),
			ray.time,
		);
		false
	}
	fn eval(&self, hit: &Hit, wo: Vec3, _: Vec3) -> Vec3 {
		self.texture.colour_value(wo, hit.point)
	}
	fn is_delta(&self) -> bool {
		true
	}
}
