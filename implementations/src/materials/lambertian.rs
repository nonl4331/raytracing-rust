use crate::{
	rt_core::*,
	textures::Texture,
	utility::{coord::Coordinate, cosine_hemisphere_sampling, near_zero, offset_ray},
};
use std::sync::Arc;

#[derive(Debug)]
pub struct Lambertian<T: Texture> {
	pub texture: Arc<T>,
	pub absorption: Float,
}

#[cfg(all(feature = "f64"))]
use std::f64::consts::PI;

#[cfg(not(feature = "f64"))]
use std::f32::consts::PI;

impl<T> Lambertian<T>
where
	T: Texture,
{
	pub fn new(texture: &Arc<T>, absorption: Float) -> Self {
		Lambertian {
			texture: texture.clone(),
			absorption,
		}
	}
}

impl<T> Scatter for Lambertian<T>
where
	T: Texture,
{
	fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
		let coordinate_system = Coordinate::new_from_z(hit.normal);
		let mut direction = cosine_hemisphere_sampling();
		coordinate_system.vec_to_coordinate(&mut direction);

		if near_zero(direction) {
			direction = hit.normal;
		}

		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		*ray = Ray::new(point, direction, ray.time);

		false
	}
	fn scattering_pdf(&self, hit: &Hit, _: Vec3, wi: Vec3) -> Float {
		hit.normal.dot(wi).max(0.0) / PI
	}
	fn eval(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Vec3 {
		self.texture.colour_value(wo, hit.point)
			* (1.0 - self.absorption)
			* hit.normal.dot(wi).max(0.0)
			/ PI
	}
}
