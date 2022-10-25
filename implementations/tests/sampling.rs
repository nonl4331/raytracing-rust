use crate::common::*;
use implementations::distribution::CDF2D;

use rt_core::*;

mod common;

fn test_bxdf<B>(bxdf: B, bxdf_name: String) -> bool
where
	B: Scatter + Sync,
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
		let freq_table = samped_frequency_distribution(&sample, wo, PHI_RES, THETA_RES, SAMPLES);
		let expected_freq_table: Vec<Float> =
			integrate_frequency_table(&pdf, wo, PHI_RES, THETA_RES)
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
				PHI_RES,
				THETA_RES,
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

fn generate_random_pdf(length: usize) -> Vec<Float> {
	let mut vec = Vec::with_capacity(length);

	for _ in 0..length {
		vec.push(random_float());
	}
	let sum: Float = vec.iter().sum();
	vec.into_iter().map(|v| v / sum).collect()
}

#[test]
fn trowbridge_reitz() {
	let bxdf = implementations::trowbridge_reitz::TrowbridgeReitz::new(
		&std::sync::Arc::new(implementations::SolidColour::new(Vec3::new(0.0, 0.0, 0.0))),
		0.3,
		3.2 * Vec3::one(),
		1.0,
	);

	assert!(test_bxdf(bxdf, "trowbridge_reitz".to_string()))
}

#[test]
#[ignore]
fn lambertian() {
	let bxdf = implementations::lambertian::Lambertian::new(
		&std::sync::Arc::new(implementations::SolidColour::new(Vec3::new(0.0, 0.0, 0.0))),
		0.3,
	);

	assert!(test_bxdf(bxdf, "lambertain".to_string()))
}

#[test]
fn discrete_2d_pdf() {
	let pdf = implementations::generate_pdf(
		&*std::sync::Arc::new(implementations::AllTextures::Lerp(
			implementations::Lerp::new(Vec3::zero(), Vec3::one()),
		)),
		(50, 50),
	);
	assert!(pdf.into_iter().sum::<Float>() - 1.0 < 0.01);
}

#[test]
fn discrete_2d() {
	const SAMPLE_WIDTH: usize = 50;
	const SAMPLE_HEIGHT: usize = 50;

	let pdf = generate_random_pdf(SAMPLE_WIDTH * SAMPLE_HEIGHT);

	let cdf = implementations::distribution::CDF2D::from_pdf(&pdf, SAMPLE_WIDTH);

	let samples = 10_000_000;

	let sample = || cdf.sample();

	let expected_freq_table: Vec<Float> = pdf.into_iter().map(|v| v * samples as Float).collect();

	for i in 0..CHI_TESTS {
		let freq_table =
			samped_frequency_distribution_uv(&sample, SAMPLE_WIDTH, SAMPLE_HEIGHT, samples);

		if i == 0 {
			dump_tables(
				Vec3::zero(),
				&freq_table
					.iter()
					.map(|&v| v as Float)
					.collect::<Vec<Float>>(),
				&expected_freq_table,
				SAMPLE_WIDTH,
				SAMPLE_HEIGHT,
				"2DCDF",
			);
		}
		let (df, chi_squared) = chi_squared(freq_table, expected_freq_table.clone(), samples);
		let p = chi2_probability(df as f64, chi_squared as f64);
		let threshold = 1.0 - (1.0 - CHI2_THRESHOLD).powf(1.0 / CHI_TESTS as Float);
		if p < threshold as f64 || p.is_infinite() {
			panic!("p: {p}");
		}
	}
}

#[test]
fn transform_test() {
	const SAMPLE_WIDTH: usize = 50;
	const SAMPLE_HEIGHT: usize = 50;

	let tex = std::sync::Arc::new(implementations::AllTextures::Lerp(
		implementations::Lerp::new(Vec3::zero(), Vec3::one()),
	));

	let sky = implementations::Sky::new(&tex, (SAMPLE_WIDTH, SAMPLE_HEIGHT));

	let expected_pdf = sky.pdf.clone();

	let pdf = (0..(SAMPLE_HEIGHT * SAMPLE_WIDTH))
		.into_iter()
		.map(|i| (i % SAMPLE_WIDTH, i / SAMPLE_WIDTH))
		.map(|(u, v)| {
			let phi = ((u as Float + 0.5) / SAMPLE_WIDTH as Float) * 2.0 * PI;
			let theta = ((v as Float + 0.5) / SAMPLE_HEIGHT as Float) * PI;

			let vec = Vec3::from_spherical(theta.sin(), theta.cos(), phi.sin(), phi.cos());

			sky.pdf(vec) * 2.0 * PI * PI * theta.sin()
		});

	let samples = 1_000_000;

	let pdf: Vec<Float> = pdf.map(|v| v * samples as Float).collect();
	let expected_pdf: Vec<Float> = expected_pdf
		.into_iter()
		.map(|v| v * samples as Float)
		.collect();

	dump_tables(
		Vec3::zero(),
		&pdf,
		&expected_pdf,
		SAMPLE_WIDTH,
		SAMPLE_HEIGHT,
		"transform UV",
	);
	let (df, chi_squared) = chi_squared(pdf, expected_pdf, SAMPLE_HEIGHT * SAMPLE_WIDTH);
	let p = chi2_probability(df as f64, chi_squared as f64);
	let threshold = CHI2_THRESHOLD;
	if p < threshold as f64 || p.is_infinite() {
		panic!("p: {p}");
	}
}

#[test]
fn sky_sampling_uv() {
	const SAMPLE_WIDTH: usize = 50;
	const SAMPLE_HEIGHT: usize = 50;

	let tex = std::sync::Arc::new(implementations::AllTextures::Lerp(
		implementations::Lerp::new(Vec3::zero(), Vec3::one()),
	));

	let sky = implementations::Sky::new(&tex, (50, 50));

	let pdf = sky.pdf;

	let cdf = CDF2D::from_pdf(&pdf, SAMPLE_WIDTH);

	let samples = 10_000_000;

	let sample = || cdf.sample();

	let expected_freq_table: Vec<Float> = pdf.into_iter().map(|v| v * samples as Float).collect();

	for i in 0..CHI_TESTS {
		let freq_table =
			samped_frequency_distribution_uv(&sample, SAMPLE_WIDTH, SAMPLE_HEIGHT, samples);

		if i == 0 {
			dump_tables(
				Vec3::zero(),
				&freq_table
					.iter()
					.map(|&v| v as Float)
					.collect::<Vec<Float>>(),
				&expected_freq_table,
				SAMPLE_WIDTH,
				SAMPLE_HEIGHT,
				"Sky Sampling UV",
			);
		}
		let (df, chi_squared) = chi_squared(freq_table, expected_freq_table.clone(), samples);
		let p = chi2_probability(df as f64, chi_squared as f64);
		let threshold = 1.0 - (1.0 - CHI2_THRESHOLD).powf(1.0 / CHI_TESTS as Float);
		if p < threshold as f64 || p.is_infinite() {
			panic!("p: {p}");
		}
	}
}

#[test]
pub fn sky_sampling() {
	let tex = std::sync::Arc::new(implementations::AllTextures::Lerp(
		implementations::Lerp::new(Vec3::zero(), Vec3::one()),
	));

	const SAMPLE_HEIGHT: usize = 50;
	const SAMPLE_WIDTH: usize = 100;

	let sky = implementations::Sky::new(&tex, (SAMPLE_WIDTH, SAMPLE_HEIGHT));

	let samples = 1_000_000;

	let sample = |_wo: Vec3| sky.sample();

	let pdf = |_: Vec3, wi: Vec3| sky.pdf(wi) * (SAMPLE_WIDTH * SAMPLE_HEIGHT) as Float;

	let wo = generate_wo();
	let freq_table = samped_frequency_distribution(&sample, wo, PHI_RES, THETA_RES, samples);

	let expected_freq_table: Vec<Float> = integrate_frequency_table(&pdf, wo, PHI_RES, THETA_RES)
		.into_iter()
		.map(|x| x * samples as Float)
		.collect();

	dump_tables(
		wo,
		&freq_table
			.iter()
			.map(|&v| v as Float)
			.collect::<Vec<Float>>(),
		&expected_freq_table,
		PHI_RES,
		THETA_RES,
		"Sky Sampling",
	);

	let (df, chi_squared) = chi_squared(freq_table, expected_freq_table, samples);
	let p = chi2_probability(df as f64, chi_squared as f64);
	let threshold = 1.0 - (1.0 - CHI2_THRESHOLD).powf(1.0 / CHI_TESTS as Float);
	if p < threshold as f64 {
		panic!("Failed to reach pass threshold {p} < {threshold}");
	} else if p.is_infinite() {
		panic!("Failed to reach pass threshold p = inf");
	}
}
