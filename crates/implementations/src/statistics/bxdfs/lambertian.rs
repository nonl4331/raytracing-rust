use crate::coord::Coordinate;
use crate::statistics::*;
use rand::Rng;

pub fn sample_local<R: Rng>(_: Vec3, rng: &mut R) -> Vec3 {
	let cos_theta = (1.0 - rng.gen::<Float>()).sqrt();
	let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
	let phi = 2.0 * PI * rng.gen::<Float>();
	Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
}

pub fn pdf_local(_: Vec3, outgoing: Vec3) -> Float {
	outgoing.z.max(0.0) / PI
}

pub fn sample<R: Rng>(incoming: Vec3, normal: Vec3, rng: &mut R) -> Vec3 {
	Coordinate::new_from_z(normal).to_coord(sample_local(incoming, rng))
}

pub fn pdf(_: Vec3, outgoing: Vec3, normal: Vec3) -> Float {
	outgoing.dot(normal).max(0.0) / PI
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::statistics::spherical_sampling::*;
	use rand::{rngs::ThreadRng, thread_rng};

	#[test]
	fn lambertian() {
		let mut rng = thread_rng();
		let incoming = generate_wi(&mut rng);
		let pdf = |outgoing: Vec3| pdf_local(incoming, outgoing);
		let sample = |rng: &mut ThreadRng| sample_local(incoming, rng);
		test_spherical_pdf("lambertian", &pdf, &sample, false);
	}

	#[test]
	fn non_local() {
		let mut rng = thread_rng();
		let normal = random_unit_vector(&mut rng);
		let to_local = Coordinate::new_from_z(normal);
		let incoming = to_local.to_coord(generate_wi(&mut rng));
		let pdf = |outgoing: Vec3| pdf(incoming, outgoing, normal);
		let sample = |rng: &mut ThreadRng| sample(incoming, normal, rng);
		test_spherical_pdf("lambertian_nl", &pdf, &sample, false);
	}
}
