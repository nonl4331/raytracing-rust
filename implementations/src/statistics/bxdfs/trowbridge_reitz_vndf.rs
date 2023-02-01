use rand::Rng;
use rt_core::*;

// All in local frame
pub fn d_ansiotropic(a_x: Float, a_y: Float, h: Vec3) -> Float {
	let tmp = h.x * h.x / (a_x * a_x) + h.y * h.y / (a_y * a_y) + h.z * h.z;
	1.0 / (PI * a_x * a_y * tmp * tmp)
}

pub fn d_isotropic(a: Float, h: Vec3) -> Float {
	let a_sq = a * a;

	let tmp = (h.x * h.x + h.y * h.y) / a_sq + h.z * h.z;

	1.0 / (PI * a_sq * tmp * tmp)
}

pub fn lambda_ansiotropic(a_x: Float, a_y: Float, v: Vec3) -> Float {
	let tmp = 1.0 + (a_x * a_x * v.x * v.x + a_y * a_y * v.y * v.y) / (v.z * v.z);
	0.5 * (tmp.sqrt() - 1.0)
}

pub fn lambda_isotropic(a: Float, v: Vec3) -> Float {
	let tmp = 1.0 + (a * a * (v.x * v.x + v.y * v.y)) / (v.z * v.z);
	0.5 * (tmp.sqrt() - 1.0)
}

pub fn g1_ansiotropic(a_x: Float, a_y: Float, v: Vec3) -> Float {
	1.0 / (1.0 + lambda_ansiotropic(a_x, a_y, v))
}
pub fn g1_isotropic(a: Float, v: Vec3) -> Float {
	1.0 / (1.0 + lambda_isotropic(a, v))
}

pub fn vndf_ansiotropic(a_x: Float, a_y: Float, h: Vec3, v: Vec3) -> Float {
	if h.z < 0.0 {
		return 0.0;
	}
	g1_ansiotropic(a_x, a_y, v) * v.dot(h).max(0.0) * d_ansiotropic(a_x, a_y, h) / v.z
}

pub fn vndf_isotropic(a: Float, h: Vec3, v: Vec3) -> Float {
	if h.z < 0.0 {
		return 0.0;
	}
	g1_isotropic(a, v) * v.dot(h).max(0.0) * d_isotropic(a, h) / v.z
}
pub fn sample_vndf_ansiotropic<R: Rng>(a_x: Float, a_y: Float, v: Vec3, rng: &mut R) -> Vec3 {
	// chapter 1 act 1
	// transform from ellipsoid configuration to hemisphere by multipling by the scaling factors on the x and y axies (a_x, a_y)
	let v_hemisphere = Vec3::new(a_x * v.x, a_y * v.y, v.z).normalised();

	// chapter 2 act 1

	// interlude
	let len_sq = v_hemisphere.x * v_hemisphere.x + v_hemisphere.y * v_hemisphere.y;

	let basis_two = if len_sq > 0.0 {
		Vec3::new(-v_hemisphere.y, v_hemisphere.x, 0.0) / len_sq.sqrt()
	} else {
		Vec3::new(1.0, 0.0, 0.0) // len_sq = 0 implies v_hemisphere = Z, so X is a valid orthonormal basis
	};
	let basis_three = v_hemisphere.cross(basis_two); // v_hemisphere is first basis

	// chapter 3 act 1
	let r = rng.gen::<Float>().sqrt();
	let phi = TAU * rng.gen::<Float>();
	let mut t = r * Vec2::new(phi.cos(), phi.sin());
	let s = 0.5 * (1.0 + v_hemisphere.z);
	t.y = (1.0 - s) * (1.0 - t.x * t.x).sqrt() + s * t.y;

	// chapter 4 act 1
	let h_hemisphere = t.x * basis_two
		+ t.y * basis_three
		+ (1.0 - t.x * t.x - t.y * t.y).max(0.0).sqrt() * v_hemisphere;

	// chapter 5 final act

	// not dividing since microfacet normal is a covector
	Vec3::new(
		a_x * h_hemisphere.x,
		a_y * h_hemisphere.y,
		h_hemisphere.z.max(0.0), // avoid numerical errors
	)
	.normalised()
}

pub fn sample_vndf_isotropic<R: Rng>(a: Float, v: Vec3, rng: &mut R) -> Vec3 {
	sample_vndf_ansiotropic(a, a, v, rng) // cause I'm lazy
}

pub fn sample_outgoing_isotropic<R: Rng>(a: Float, incoming: Vec3, rng: &mut R) -> Vec3 {
	let h = sample_vndf_isotropic(a, incoming, rng);
	2.0 * incoming.dot(h) * h - incoming
}

pub fn pdf_outgoing(alpha: Float, incoming: Vec3, outgoing: Vec3) -> Float {
	let mut h = (incoming + outgoing).normalised();
	if h.z < 0.0 {
		h = -(incoming + outgoing).normalised();
	}
	let vndf = vndf_ansiotropic(alpha, alpha, h, incoming);
	vndf / (4.0 * incoming.dot(h))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::coord::Coordinate;
	use crate::statistics::spherical_sampling::*;
	use rand::{rngs::ThreadRng, thread_rng, Rng};

	#[test]
	fn trowbridge_reitz_vndf_h() {
		let mut rng = thread_rng();
		let incoming: Vec3 = generate_wi(&mut rng);
		let alpha: Float = rng.gen();
		let pdf = |outgoing: Vec3| vndf_isotropic(alpha, outgoing, incoming);
		let sample = |rng: &mut ThreadRng| sample_vndf_isotropic(alpha, incoming, rng);
		test_spherical_pdf("trowbrige reitz vndf h", &pdf, &sample, false);
	}

	#[test]
	fn trowbridge_reitz_vndf_h_non_local() {
		let mut rng = thread_rng();

		let normal = random_unit_vector(&mut rng);

		let to_local = Coordinate::new_from_z(normal);
		let from_local = Coordinate::create_inverse(&to_local);

		let incoming: Vec3 = to_local.vec_to_coordinate(generate_wi(&mut rng));

		let alpha: Float = rng.gen();
		let pdf = |outgoing: Vec3| {
			vndf_isotropic(
				alpha,
				from_local.vec_to_coordinate(outgoing),
				from_local.vec_to_coordinate(incoming),
			)
		};
		let sample = |rng: &mut ThreadRng| {
			to_local.vec_to_coordinate(sample_vndf_isotropic(
				alpha,
				from_local.vec_to_coordinate(incoming),
				rng,
			))
		};
		test_spherical_pdf("trowbrige reitz vndf h non local", &pdf, &sample, false);
	}

	#[test]
	fn trowbridge_reitz_vndf() {
		let mut rng = thread_rng();
		let incoming: Vec3 = generate_wi(&mut rng);
		let alpha: Float = rng.gen();
		let pdf = |outgoing: Vec3| pdf_outgoing(alpha, incoming, outgoing);
		let sample = |rng: &mut ThreadRng| sample_outgoing_isotropic(alpha, incoming, rng);
		test_spherical_pdf("trowbrige reitz vndf", &pdf, &sample, false);
	}

	#[test]
	fn trowbridge_reitz_vndf_non_local() {
		let mut rng = thread_rng();

		let normal = random_unit_vector(&mut rng);

		let to_local = Coordinate::new_from_z(normal);
		let from_local = Coordinate::create_inverse(&to_local);

		let incoming: Vec3 = to_local.vec_to_coordinate(generate_wi(&mut rng));

		let alpha: Float = rng.gen();
		let pdf = |outgoing: Vec3| {
			pdf_outgoing(
				alpha,
				from_local.vec_to_coordinate(incoming),
				from_local.vec_to_coordinate(outgoing),
			)
		};
		let sample = |rng: &mut ThreadRng| {
			to_local.vec_to_coordinate(sample_outgoing_isotropic(
				alpha,
				from_local.vec_to_coordinate(incoming),
				rng,
			))
		};
		test_spherical_pdf("trowbrige reitz vndf non local", &pdf, &sample, false);
	}
}
