use crate::utility::vec::Vec3;

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
	pub fn vec_to_coordinate(&self, vec: &mut Vec3) {
		*vec = vec.x * self.x + vec.y * self.y + vec.z * self.z;
	}
	pub fn get_z(&self) -> Vec3 {
		self.z
	}
}
