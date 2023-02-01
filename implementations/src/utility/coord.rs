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
		let a = self.x.x;
		let b = self.y.x;
		let c = self.z.x;
		let d = self.x.y;
		let e = self.y.y;
		let f = self.z.y;
		let g = self.x.z;
		let h = self.y.z;
		let i = self.z.z;

		let tmp = Vec3::new(e * i - f * h, f * g - d * i, d * h - e * g);
		let one_over_det = 1.0 / (a * tmp.x + b * tmp.y + c * tmp.z);
		let x = one_over_det * tmp;
		let y = one_over_det * Vec3::new(c * h - b * i, a * i - c * g, b * g - a * h);
		let z = one_over_det * Vec3::new(b * f - c * e, c * d - a * f, a * e - b * d);
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
