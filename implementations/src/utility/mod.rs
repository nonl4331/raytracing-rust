use crate::rt_core::{Float, Vec3, PI};
use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};

pub mod coord;

pub fn check_side(normal: &mut Vec3, ray_direction: &Vec3) -> bool {
	if normal.dot(*ray_direction) > 0.0 {
		*normal = -*normal;
		false
	} else {
		true
	}
}

pub fn random_unit_vector() -> Vec3 {
	let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
	let (mut x, mut y, mut z) = (1.0, 1.0, 1.0);
	while x * x + y * y + z * z > 1.0 {
		x = rng.gen_range(-1.0..1.0);
		y = rng.gen_range(-1.0..1.0);
		z = rng.gen_range(-1.0..1.0);
	}

	Vec3::new(x, y, z).normalised()
}

pub fn cosine_hemisphere_sampling() -> Vec3 {
	let cos_theta = (1.0 - random_float()).sqrt();
	let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
	let phi = 2.0 * PI * random_float();
	Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
}

pub fn specular_sampling(n: Float) -> Vec3 {
	let a = random_float().powf(1.0 / (n + 1.0));
	let term = (1.0 - a * a).sqrt();
	let phi = 2.0 * PI * random_float();
	Vec3::new(term * phi.cos(), term * phi.sin(), a)
}

pub fn random_float() -> Float {
	let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
	rng.gen()
}

pub fn near_zero(vec: Vec3) -> bool {
	let s = 0.001;
	vec.x.abs() < s && vec.y.abs() < s && vec.z.abs() < s
}

pub fn next_float(mut float: Float) -> Float {
	if float.is_infinite() && float > 0.0 {
		return float;
	}

	if float == -0.0 {
		float = 0.0
	}

	Float::from_bits(if float >= 0.0 {
		Float::to_bits(float) + 1
	} else {
		Float::to_bits(float) - 1
	})
}

pub fn previous_float(mut float: Float) -> Float {
	if float.is_infinite() && float < 0.0 {
		return float;
	}

	if float == 0.0 {
		float = -0.0
	}

	Float::from_bits(if float <= 0.0 {
		Float::to_bits(float) + 1
	} else {
		Float::to_bits(float) - 1
	})
}

pub fn gamma(n: u32) -> Float {
	let nm = n as Float * 0.5 * Float::EPSILON;
	nm / (1.0 - nm)
}

pub fn offset_ray(origin: Vec3, normal: Vec3, error: Vec3, is_brdf: bool) -> Vec3 {
	let offset_val = normal.abs().dot(error);
	let mut offset = offset_val * normal;

	if !is_brdf {
		offset = -offset;
	}

	let mut new_origin = origin + offset;

	if offset.x > 0.0 {
		new_origin.x = next_float(new_origin.x);
	} else {
		new_origin.x = previous_float(new_origin.x);
	}

	if offset.y > 0.0 {
		new_origin.y = next_float(new_origin.y);
	} else {
		new_origin.y = previous_float(new_origin.y);
	}

	if offset.z > 0.0 {
		new_origin.z = next_float(new_origin.z);
	} else {
		new_origin.z = previous_float(new_origin.z);
	}

	new_origin
}

pub fn sort_by_indices<T>(vec: &mut [T], mut indices: Vec<usize>) {
	for index in 0..vec.len() {
		if indices[index] != index {
			let mut current_index = index;
			loop {
				let target_index = indices[current_index];
				indices[current_index] = current_index;
				if indices[target_index] == target_index {
					break;
				}
				vec.swap(current_index, target_index);
				current_index = target_index;
			}
		}
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn sort_vec_by_indices() {
		let indices = vec![0, 4, 2, 1, 3];
		let mut values = ["a", "b", "c", "d", "e"];

		sort_by_indices(&mut values, indices);

		assert_eq!(values, ["a", "e", "c", "b", "d"]);
	}
}
