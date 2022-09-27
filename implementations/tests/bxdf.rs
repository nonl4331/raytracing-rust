mod common;
use common::bsdf_testing;

use rt_core::Hit;
use rt_core::Ray;

use rt_core::{Scatter, Vec3};

fn test_bxdf<B>(bxdf: B) -> bool
where
	B: Scatter,
{
	let hit = Hit {
		t: 0.0,
		error: Vec3::zero(),
		point: Vec3::zero(),
		uv: None,
		normal: Vec3::new(0.0, 0.0, 1.0),
		out: true,
	};

	let sample = |wo: Vec3| {
		let mut ray = Ray::new(Vec3::zero(), wo, 0.0);
		bxdf.scatter_ray(&mut ray, &hit);
		ray.direction
	};

	let pdf = |wo: Vec3, wi: Vec3| bxdf.scattering_pdf(&hit, wo, wi);
	bsdf_testing(&sample, &pdf)
}

#[test] // change name
fn ggx_cook_torrence() {
	let bxdf = implementations::cook_torrence::CookTorrence::new(
		&std::sync::Arc::new(implementations::SolidColour::new(Vec3::new(0.0, 0.0, 0.0))),
		0.3,
		3.2 * Vec3::one(),
		1.0,
	);

	assert!(test_bxdf(bxdf))
}

#[test] // change name
fn lambertian() {
	let bxdf = implementations::lambertain::Lambertian::new(
		&std::sync::Arc::new(implementations::SolidColour::new(Vec3::new(0.0, 0.0, 0.0))),
		0.3,
	);

	assert!(test_bxdf(bxdf))
}
