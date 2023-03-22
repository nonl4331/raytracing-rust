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

pub fn d(alpha: Float, cos_theta: Float) -> Float {
	if cos_theta <= 0.0 {
		return 0.0;
	}
	let a_sq = alpha * alpha;
	let tmp = cos_theta * cos_theta * (a_sq - 1.0) + 1.0;
	a_sq / (PI * tmp * tmp)
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
	let mut h = (outgoing + incoming).normalised();
	if h.z < 0.0 {
		h = -h;
	}
	let d = d(alpha, h.z);
	d * h.z.abs() / (4.0 * outgoing.dot(h).abs())
}

pub fn sample<R: Rng>(alpha: Float, incoming: Vec3, normal: Vec3, rng: &mut R) -> Vec3 {
	let coord = Coordinate::new_from_z(normal);
	let local_h = sample_h(alpha, rng);
	let h = coord.to_coord(local_h);

	incoming.reflected(h)
}

pub fn pdf(alpha: Float, incoming: Vec3, outgoing: Vec3, normal: Vec3) -> Float {
	let inverse = Coordinate::new_from_z(normal).create_inverse();
	let incoming = inverse.to_coord(incoming);
	let outgoing = inverse.to_coord(outgoing);
	let mut h = (outgoing + incoming).normalised();
	if h.z < 0.0 {
		h = -h;
	}
	let d = d(alpha, h.z);
	d * h.z.abs() / (4.0 * outgoing.dot(h).abs())
}

pub fn g2(alpha: Float, normal: Vec3, h: Vec3, incoming: Vec3, outgoing: Vec3) -> Float {
	//let incoming = -incoming;
	if incoming.dot(h) / incoming.dot(normal) <= 0.0
		|| outgoing.dot(h) / outgoing.dot(normal) <= 0.0
	{
		return 0.0;
	}

	let alpha_sq = alpha * alpha;
	let one_minus_alpha_sq = 1.0 - alpha_sq;
	let cos_i = normal.dot(incoming);
	let cos_i_sq = cos_i * cos_i;
	let tmp_a = alpha_sq + one_minus_alpha_sq * cos_i_sq;
	let cos_o = normal.dot(outgoing);
	let cos_o_sq = cos_o * cos_o;
	let tmp_b = alpha_sq + one_minus_alpha_sq * cos_o_sq;
	2.0 * cos_i * cos_o / (cos_o * tmp_a.sqrt() + cos_i * tmp_b.sqrt())
}

pub fn g1(alpha: Float, normal: Vec3, h: Vec3, v: Vec3) -> Float {
	if v.dot(h) / v.dot(normal) <= 0.0 {
		return 0.0;
	}
	let cos = normal.dot(v);
	let cos_sq = cos * cos;
	let alpha_sq = alpha * alpha;
	let tmp = alpha_sq + (1.0 - alpha_sq) * cos_sq;
	2.0 * cos / (tmp.sqrt() + cos)
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
		let incoming = -generate_wi(&mut rng);
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
		let incoming = to_local.to_coord(-generate_wi(&mut rng));
		let alpha = rng.gen();
		let pdf = |outgoing: Vec3| pdf(alpha, incoming, outgoing, normal);
		let sample = |rng: &mut ThreadRng| sample(alpha, incoming, normal, rng);
		test_spherical_pdf("tr_nl", &pdf, &sample, false);
	}

	#[test]
	fn g1_cos_test() {
		let mut rng = thread_rng();
		let incoming = -generate_wi(&mut rng);
		let cos_theta = incoming.z;
		let alpha = rng.gen();
		let test = |h: Vec3| {
			g1(alpha, Vec3::new(0.0, 0.0, 1.0), h, incoming)
				* incoming.dot(h).max(0.0)
				* d(alpha, h.z)
		};

		let integral = integrate_over_sphere(&test);
		assert!((integral - cos_theta).abs() < 0.0001);
	}

	#[test]
	fn projected_area_test_local() {
		let mut rng = thread_rng();
		let alpha = rng.gen();
		let test = |h: Vec3| d(alpha, h.z) * h.z;
		let integral = integrate_over_sphere(&test);
		assert!((integral - 1.0).abs() < 0.0001);
	}

	#[test]
	fn projected_area_test_non_local() {
		let mut rng = thread_rng();
		let normal = random_unit_vector(&mut rng);
		let alpha = rng.gen();
		let test = |h: Vec3| d(alpha, h.dot(normal)) * h.dot(normal);
		let integral = integrate_over_sphere(&test);
		assert!((integral - 1.0).abs() < 0.0001);
	}

	#[test]
	fn weak_furnace_test() {
		let mut rng = thread_rng();
		let wo = -generate_wi(&mut rng);

		let alpha: Float = rng.gen();

		let test = |wi: Vec3| {
			let mut h = (wi + wo).normalised();
			if h.z < 0.0 {
				h = -h;
			}
			let denom = 4.0 * wo.z.abs();
			if denom < 0.000000001 {
				0.0
			} else {
				g1(alpha, Vec3::new(0.0, 0.0, 1.0), h, wo) * d(alpha, h.z) / denom
			}
		};

		let integral = integrate_over_sphere(&test);
		assert!((integral - 1.0).abs() < 0.0001);
	}

	#[test]
	fn g2_test() {
		let mut rng = thread_rng();
		let a = -generate_wi(&mut rng);
		let alpha = rng.gen();
		let test = |b: Vec3| {
			let mut h = (a + b).normalised();
			if h.z < 0.0 {
				h = -h;
			}
			let denom = 4.0 * a.z.abs();
			if denom < 0.000000001 {
				0.0
			} else {
				g2(alpha, Vec3::new(0.0, 0.0, 1.0), h, a, b) * d(alpha, h.z) / denom
			}
		};

		let integral = integrate_over_sphere(&test);
		assert!(integral <= 1.0);
	}

	#[test]
	fn g2_test_non_local() {
		let mut rng = thread_rng();
		let a = -generate_wi(&mut rng);
		let normal = random_unit_vector(&mut rng);
		let alpha = rng.gen();
		let test = |b: Vec3| {
			let mut h = (a + b).normalised();
			if h.dot(normal) < 0.0 {
				h = -h;
			}
			let denom = 4.0 * a.dot(-normal).abs();
			if denom < 0.000000001 {
				0.0
			} else {
				g2(alpha, normal, h, a, b) * d(alpha, a.dot(-normal)) / denom
			}
		};

		let integral = integrate_over_sphere(&test);
		assert!(integral <= 1.0);
	}
}
