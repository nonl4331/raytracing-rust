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

pub fn pdf_h(h: Vec3, alpha: Float) -> Float {
	// technically the paper has an .abs() but it isn't needed since h.z < 0 => D(m) = 0
	d(alpha, h.z) * h.z
}

pub fn sample<R: Rng>(alpha: Float, incoming: Vec3, rng: &mut R) -> Vec3 {
	let h = sample_h(alpha, rng);
	2.0 * incoming.dot(h) * h - incoming
}

pub fn pdf_outgoing(alpha: Float, incoming: Vec3, outgoing: Vec3, normal: Vec3) -> Float {
	let mut h = (incoming + outgoing).normalised();
	if h.dot(normal) < 0.0 {
		h = -(incoming + outgoing).normalised();
	}
	let cos_theta = normal.dot(h);
	let d = d(alpha, cos_theta);
	d * h.dot(normal).abs() / (4.0 * outgoing.dot(h).abs())
}
