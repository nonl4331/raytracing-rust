use crate::{
	rt_core::*,
	textures::Texture,
	utility::{offset_ray, random_unit_vector},
};

#[derive(Debug, Clone)]
pub struct Reflect<'a, T: Texture> {
	pub texture: &'a T,
	pub fuzz: Float,
}

impl<'a, T> Reflect<'a, T>
where
	T: Texture,
{
	pub fn new(texture: &'a T, fuzz: Float) -> Self {
		Reflect { texture, fuzz }
	}
}

impl<'a, T> Scatter for Reflect<'a, T>
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
