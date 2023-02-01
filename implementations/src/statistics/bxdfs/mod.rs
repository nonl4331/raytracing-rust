pub mod trowbridge_reitz;
pub mod trowbridge_reitz_vndf;

#[cfg(test)]
pub mod test {
	use super::*;
	use crate::statistics::{spherical_sampling::*, *};
	use rand::{rngs::ThreadRng, thread_rng, Rng};

	#[test]
	fn trowbridge_reitz_h() {
		let alpha: Float = thread_rng().gen();
		let pdf = |outgoing: Vec3| trowbridge_reitz::pdf_h(outgoing, alpha);
		let sample = |rng: &mut ThreadRng| trowbridge_reitz::sample_h(alpha, rng);
		test_spherical_pdf("trowbrige reitz h", &pdf, &sample, false);
	}

	#[test]
	fn trowbridge_reitz() {
		let mut rng = thread_rng();
		let incoming: Vec3 = generate_wi(&mut rng);
		let alpha: Float = rng.gen();
		let pdf = |outgoing: Vec3| {
			trowbridge_reitz::pdf_outgoing(alpha, incoming, outgoing, Vec3::new(0.0, 0.0, 1.0))
		};
		let sample = |rng: &mut ThreadRng| trowbridge_reitz::sample(alpha, incoming, rng);
		test_spherical_pdf("trowbrige reitz h", &pdf, &sample, false);
	}
}
