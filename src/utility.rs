use implementations::rt_core::*;
use implementations::*;

pub fn rotate_around_point(
	point: &mut Vec3,
	center_point: Vec3,
	sin_angles: Vec3,
	cos_angles: Vec3,
) {
	*point -= center_point;

	rotate_around_axis(point, Axis::X, sin_angles.x, cos_angles.x);
	rotate_around_axis(point, Axis::Y, sin_angles.y, cos_angles.y);
	rotate_around_axis(point, Axis::Z, sin_angles.z, cos_angles.z);

	*point += center_point;
}

pub fn rotate_around_axis(point: &mut Vec3, axis: Axis, sin: Float, cos: Float) {
	match axis {
		Axis::X => {
			let old_y = point.y;
			point.y = cos * point.y - sin * point.z;
			point.z = sin * old_y + cos * point.z;
		}
		Axis::Y => {
			let old_x = point.x;
			point.x = cos * point.x - sin * point.z;
			point.z = sin * old_x + cos * point.z;
		}
		Axis::Z => {
			let old_y = point.y;
			point.y = cos * point.y - sin * point.x;
			point.x = sin * old_y + cos * point.x;
		}
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn rotation() {
		let center_point = Vec3::new(1.0, 0.0, 0.0);

		let mut point = Vec3::new(2.0, 0.0, 0.0);

		let angles = Vec3::new(0.0, 45.0 * PI / 180.0, 0.0);

		let cos_angles = Vec3::new(angles.x.cos(), angles.y.cos(), angles.z.cos());
		let sin_angles = Vec3::new(angles.x.sin(), angles.y.sin(), angles.z.sin());

		rotate_around_point(&mut point, center_point, sin_angles, cos_angles);

		assert!((point - Vec3::new(1.707107, 0.0, 0.7071069)).abs().mag() < 0.000001);
	}
}
