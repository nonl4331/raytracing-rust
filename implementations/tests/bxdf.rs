use crate::common::*;
use rt_core::*;

mod common;

fn test_bxdf<B>(bxdf: B, bxdf_name: String) -> bool
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
	for i in 0..CHI_TESTS {
		let wo = generate_wo();
		let freq_table = samped_frequency_distribution(&sample, wo, THETA_RES, PHI_RES, SAMPLES);
		let expected_freq_table: Vec<Float> =
			integrate_frequency_table(&pdf, wo, THETA_RES, PHI_RES)
				.into_iter()
				.map(|x| x * SAMPLES as Float)
				.collect();
		if i == 0 {
			dump_tables(
				wo,
				&freq_table
					.iter()
					.map(|&v| v as Float)
					.collect::<Vec<Float>>(),
				&expected_freq_table,
				THETA_RES,
				PHI_RES,
				&bxdf_name,
			);
		}
		let (df, chi_squared) = chi_squared(freq_table, expected_freq_table, SAMPLES);
		let p = chi2_probability(df as f64, chi_squared as f64);
		let threshold = 1.0 - (1.0 - CHI2_THRESHOLD).powf(1.0 / CHI_TESTS as Float);
		if p < threshold as f64 || p.is_infinite() {
			return false;
		}
	}
	true
}

#[test]
fn trowbridge_reitz() {
	let bxdf = implementations::trowbridge_reitz::TrowbridgeReitz::new(
		&std::sync::Arc::new(implementations::SolidColour::new(Vec3::new(0.0, 0.0, 0.0))),
		0.3,
		3.2 * Vec3::one(),
		1.0,
	);

	assert!(test_bxdf(bxdf, "ggx".to_string()))
}

#[test]
fn lambertian() {
	let bxdf = implementations::lambertian::Lambertian::new(
		&std::sync::Arc::new(implementations::SolidColour::new(Vec3::new(0.0, 0.0, 0.0))),
		0.3,
	);

	assert!(test_bxdf(bxdf, "lambertain".to_string()))
}
