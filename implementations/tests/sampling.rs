use rand::rngs::ThreadRng;
use rt_core::*;
use statistics::spherical_sampling::test_spherical_pdf;

mod common;

#[test]
fn sky_sampling() {
	const SAMPLE_WIDTH: usize = 30;
	const SAMPLE_HEIGHT: usize = 50;

	let tex = std::sync::Arc::new(implementations::AllTextures::Lerp(
		implementations::Lerp::new(Vec3::zero(), Vec3::one()),
	));

	let sky = implementations::Sky::new(&tex, (SAMPLE_WIDTH, SAMPLE_HEIGHT));

	let pdf = |outgoing: Vec3| sky.pdf(outgoing);
	let sample = |_: &mut ThreadRng| sky.sample();
	test_spherical_pdf("lerp sky sampling", &pdf, &sample, false);
}
