use rt_core::Vec3;

pub struct Coordinate {
	pub x: Vec3,
	pub y: Vec3,
	pub z: Vec3,
}

impl Coordinate {
	pub fn new_from_z(z: Vec3) -> Self {
		let x = if z.x.abs() > z.y.abs() {
			Vec3::new(-z.z, 0.0, z.x) / (z.x * z.x + z.z * z.z).sqrt()
		} else {
			Vec3::new(0.0, z.z, -z.y) / (z.y * z.y + z.z * z.z).sqrt()
		};
		Coordinate {
			x,
			y: x.cross(z),
			z,
		}
	}
	pub fn create_inverse(&self) -> Self {
		let x = Vec3::new(self.x.x, self.y.x, self.z.x);
		let y = Vec3::new(self.x.y, self.y.y, self.z.y);
		let z = Vec3::new(self.x.z, self.y.z, self.z.z);
		Coordinate { x, y, z }
	}
	pub fn to_coord(&self, vec: Vec3) -> Vec3 {
		vec.x * self.x + vec.y * self.y + vec.z * self.z
	}
}

#[cfg(test)]
mod tests {
	use crate::random_unit_vector;

	use super::*;

	#[test]
	fn inverse() {
		let z = random_unit_vector();
		let to = Coordinate::new_from_z(z);
		let from = to.create_inverse();
		let v = random_unit_vector();
		assert!(
			(v - from.to_coord(to.to_coord(v))).mag_sq() < 0.000001
				&& (v - to.to_coord(from.to_coord(v))).mag_sq() < 0.000001
		);
	}
}
