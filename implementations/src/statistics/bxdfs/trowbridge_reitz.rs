use crate::coord::Coordinate;
use crate::statistics::*;
use rand::Rng;

pub fn sample_h<R: Rng>(alpha: Float, rng: &mut R) -> Vec3 {
	let r1: Float = rng.gen();
	let r2: Float = rng.gen();
	let cos_theta = ((1.0 - r1) / (r1 * (alpha * alpha - 1.0) + 1.0)).sqrt();
	let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
	let phi_s = (TAU * r2).max(0.0).min(TAU);
	Vec3::new(phi_s.cos() * sin_theta, phi_s.sin() * sin_theta, cos_theta).normalised()
}

pub fn reference_d(h: Vec3, alpha: Float) -> Float {
	if h.z <= 0.0 {
		return 0.0;
	}

	let a_sq = alpha * alpha;
	let cos_theta = h.z;
	let cos_theta_sq = cos_theta * cos_theta;
	let sin_theta = (1.0 - cos_theta_sq).sqrt();
	let tan_theta = sin_theta / cos_theta;
	let tmp = a_sq + tan_theta * tan_theta;

	a_sq / (PI * cos_theta_sq * cos_theta_sq * tmp * tmp)
}

pub fn d(alpha: Float, cos_theta: Float) -> Float {
	if cos_theta <= 0.0 {
		return 0.0;
	}
	let a_sq = alpha * alpha;
	let tmp = cos_theta * cos_theta * (a_sq - 1.0) + 1.0;
	a_sq / (PI * tmp * tmp)
}

pub fn alternative_d(alpha: Float, h: Vec3, cos_theta: Float) -> Float {
	if cos_theta < 0.0 {
		return 0.0;
	}
	let a_sq = alpha * alpha;
	let tmp = (h.x * h.x + h.y * h.y) / a_sq + h.z * h.z;
	1.0 / (PI * a_sq * tmp * tmp)
}

pub fn pdf_h(alpha: Float, h: Vec3) -> Float {
	// technically the paper has an .abs() but it isn't needed since h.z < 0 => D(m) = 0
	d(alpha, h.z) * h.z
}

pub fn sample_local<R: Rng>(alpha: Float, incoming: Vec3, rng: &mut R) -> Vec3 {
	let h = sample_h(alpha, rng);
	incoming.reflected(h)
}

pub fn pdf_local(alpha: Float, incoming: Vec3, outgoing: Vec3) -> Float {
	let mut h = (outgoing - incoming).normalised();
	if h.z < 0.0 {
		h = (incoming - outgoing).normalised();
	}
	let d = d(alpha, h.z);
	d * h.z.abs() / (4.0 * outgoing.dot(h).abs())
}

pub fn sample<R: Rng>(alpha: Float, incoming: Vec3, normal: Vec3, rng: &mut R) -> Vec3 {
	let coord = Coordinate::new_from_z(normal);
	let h = coord.to_coord(sample_h(alpha, rng));
	incoming.reflected(h)
}

pub fn pdf(alpha: Float, incoming: Vec3, outgoing: Vec3, normal: Vec3) -> Float {
	let inverse = Coordinate::new_from_z(normal).create_inverse();
	let incoming = inverse.to_coord(incoming);
	let outgoing = inverse.to_coord(outgoing);
	let mut h = (outgoing - incoming).normalised();
	if h.z < 0.0 {
		h = (incoming - outgoing).normalised();
	}
	let d = d(alpha, h.z);
	d * h.z.abs() / (4.0 * outgoing.dot(h).abs())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::statistics::spherical_sampling::*;
	use rand::{rngs::ThreadRng, thread_rng, Rng};

	#[test]
	fn h() {
		let mut rng = thread_rng();
		let alpha = rng.gen();
		let pdf = |outgoing: Vec3| pdf_h(alpha, outgoing);
		let sample = |rng: &mut ThreadRng| sample_h(alpha, rng);
		test_spherical_pdf("tr_h", &pdf, &sample, false);
	}

	#[test]
	fn tr() {
		let mut rng = thread_rng();
		let incoming = generate_wi(&mut rng);
		let alpha = rng.gen();
		let pdf = |outgoing: Vec3| pdf_local(alpha, incoming, outgoing);
		let sample = |rng: &mut ThreadRng| sample_local(alpha, incoming, rng);
		test_spherical_pdf("tr", &pdf, &sample, false);
	}

	#[test]
	fn non_local() {
		let mut rng = thread_rng();
		let normal = random_unit_vector(&mut rng);
		let to_local = Coordinate::new_from_z(normal);
		let incoming = to_local.to_coord(generate_wi(&mut rng));
		let alpha = rng.gen();
		let pdf = |outgoing: Vec3| pdf(alpha, incoming, outgoing, normal);
		let sample = |rng: &mut ThreadRng| sample(alpha, incoming, normal, rng);
		test_spherical_pdf("tr_nl", &pdf, &sample, false);
	}
}
