use crate::{Float, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
	pub origin: Vec3,
	pub direction: Vec3,
	pub d_inverse: Vec3,
	pub shear: Vec3,
	pub time: Float,
}

impl Ray {
	pub fn new(origin: Vec3, mut direction: Vec3, time: Float) -> Self {
		direction.normalise();

		let max_axis =
			if direction.x.abs() > direction.y.abs() && direction.x.abs() > direction.z.abs() {
				0
			} else if direction.y.abs() > direction.z.abs() {
				1
			} else {
				2
			};

		let mut swaped_dir = direction;
		match max_axis {
			0 => {
				std::mem::swap(&mut swaped_dir.x, &mut swaped_dir.z);
			}
			1 => {
				std::mem::swap(&mut swaped_dir.x, &mut swaped_dir.z);
			}
			_ => {}
		}
		let shear_x = -swaped_dir.x / swaped_dir.z;
		let shear_y = -swaped_dir.y / swaped_dir.z;
		let shear_z = 1.0 / swaped_dir.z;

		Ray {
			origin,
			direction,
			d_inverse: Vec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
			shear: Vec3::new(shear_x, shear_y, shear_z),
			time,
		}
	}

	pub fn at(&self, t: Float) -> Vec3 {
		self.origin + self.direction * t
	}
}
