use crate::utility::random_float;
use rt_core::{Camera, Float, Ray, Vec3};

pub struct SimpleCamera {
	pub viewport_width: Float,
	pub viewport_height: Float,
	pub aspect_ratio: Float,
	pub origin: Vec3,
	pub vertical: Vec3,
	pub horizontal: Vec3,
	pub u: Vec3,
	pub v: Vec3,
	pub lower_left: Vec3,
	pub lens_radius: Float,
}

impl SimpleCamera {
	pub fn new(
		origin: Vec3,
		lookat: Vec3,
		vup: Vec3,
		fov: Float,
		aspect_ratio: Float,
		aperture: Float,
		focus_dist: Float,
	) -> Self {
		let viewport_width = 2.0 * (fov.to_radians() / 2.0).tan();
		let viewport_height = viewport_width / aspect_ratio;

		let w = (origin - lookat).normalised();
		let u = w.cross(vup).normalised();
		let v = u.cross(w);

		let horizontal = focus_dist * u * viewport_width;
		let vertical = focus_dist * v * viewport_height;

		let lower_left = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

		SimpleCamera {
			viewport_width,
			viewport_height,
			aspect_ratio,
			origin,
			vertical,
			horizontal,
			u,
			v,
			lower_left,
			lens_radius: aperture / 2.0,
		}
	}
}

impl Camera for SimpleCamera {
	fn get_ray(&self, u: Float, v: Float) -> Ray {
		Ray::new(
			self.origin,
			self.lower_left + self.horizontal * u + self.vertical * v - self.origin,
			random_float(),
		)
	}
}
