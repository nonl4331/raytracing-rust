use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};
use rayon::prelude::*;
use rt_core::{vec::*, Float};
use statrs::function::gamma::*;
use std::{
	f64::{consts::*, INFINITY},
	{fs::File, io::Write},
};

pub mod int;

const SAMPLES: usize = 10_000_000;
const THETA_RES: usize = 80;
const PHI_RES: usize = 2 * THETA_RES;
const CHI2_THRESHOLD: Float = 0.01;
const CHI_TESTS: usize = 1;

use int::*;

fn chi_squared_term(a: Float, b: Float) -> Float {
	if a < (SAMPLES / 100000) as Float && b == 0.0 {
		return 0.0;
	}
	let val = a - b;
	val * val / b
}

pub fn to_vec(sin_theta: Float, cos_theta: Float, phi: Float) -> Vec3 {
	Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
}

fn chi2_probability(dof: f64, distance: f64) -> f64 {
	assert!(
		(gamma_lr(dof * 0.5, distance * 0.5) + gamma_ur(dof * 0.5, distance * 0.5) - 1.0).abs()
			< 0.0001
	);
	gamma_ur(dof * 0.5, distance * 0.5)
}

pub fn integrate_frequency_table<F>(
	pdf: &F,
	wo: Vec3,
	theta_res: usize,
	phi_res: usize,
) -> Vec<Float>
where
	F: Fn(Vec3, Vec3) -> Float,
{
	let theta_step = PI as Float / theta_res as Float;
	let phi_step = TAU as Float / phi_res as Float;
	let pdf = |phi: Float, a, b| {
		adaptive_simpsons(
			|theta| {
				pdf(
					wo,
					Vec3::new(
						phi.cos() * theta.sin(),
						phi.sin() * theta.sin(),
						theta.cos(),
					),
				) * theta.sin()
			},
			a,
			b,
		)
	};

	let mut vec = Vec::new();
	for theta_i in 0..theta_res {
		for phi_i in 0..phi_res {
			let a = adaptive_simpsons(
				|phi| {
					pdf(
						phi,
						theta_i as Float * theta_step,
						(theta_i + 1) as Float * theta_step,
					)
				},
				phi_i as Float * phi_step,
				(phi_i + 1) as Float * phi_step,
			);
			vec.push(a);
		}
	}
	vec
}

pub fn samped_frequency_distribution<F>(
	function: &F,
	wo: Vec3,
	theta_res: usize,
	phi_res: usize,
	sample_count: usize,
) -> Vec<Float>
where
	F: Fn(Vec3) -> Vec3,
{
	let mut freq = vec![0.0; theta_res * phi_res];
	let theta_step = PI as Float / theta_res as Float;
	let phi_step = TAU as Float / phi_res as Float;

	for _ in 0..sample_count {
		let sampled = function(wo);
		let theta = sampled.z.acos();
		let mut phi = (sampled.y).atan2(sampled.x);

		if phi < 0.0 {
			phi += TAU as Float;
		}

		let theta_bin = ((theta / theta_step) as usize).max(0).min(theta_res - 1);
		let phi_bin = ((phi / phi_step) as usize).max(0).min(phi_res - 1);

		freq[theta_bin * phi_res + phi_bin] += 1.0;
	}

	freq
}

pub fn random_float() -> Float {
	let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
	rng.gen()
}

fn generate_wo() -> Vec3 {
	let cos_theta = random_float();
	let phi = TAU as Float * random_float();

	to_vec((1.0 - cos_theta * cos_theta).sqrt(), cos_theta, phi)
}

pub fn chi_squared(
	freq_table: Vec<Float>,
	expected_freq_table: Vec<Float>,
	samples: usize,
) -> (usize, Float) {
	assert_eq!(freq_table.len(), expected_freq_table.len());

	let mut values = expected_freq_table
		.into_par_iter()
		.zip(freq_table.into_par_iter())
		.collect::<Vec<(Float, Float)>>();

	values.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

	let mut df = 0;

	let mut expected_pooled = 0.0;
	let mut actual_pooled = 0.0;

	let mut chi_squared = 0.0;

	for (expected, actual) in values {
		if expected == 0.0 {
			if actual > (samples / 100_000) as Float {
				chi_squared += INFINITY as Float;
			}
		} else if expected < 5.0 || (expected_pooled > 0.0 && expected_pooled < 5.0) {
			expected_pooled += expected;
			actual_pooled += actual;
		} else {
			let diff = actual - expected;
			chi_squared += diff * diff / expected;
			df += 1;
		}
	}

	if actual_pooled > 0.0 || expected_pooled > 0.0 {
		let diff = actual_pooled - expected_pooled;
		chi_squared += diff * diff / expected_pooled;
		df += 1;
	}

	df -= 1;

	(df, chi_squared)
}

fn dump_tables(
	wo: Vec3,
	freq_table: &[Float],
	expected_freq_table: &[Float],
	theta_res: usize,
	phi_res: usize,
) {
	let enumerate = |file: &mut File, func: fn(Float, Float) -> Float| {
		(0..theta_res * phi_res).for_each(|index| {
			file.write_all(
				format!("{}", func(freq_table[index], expected_freq_table[index])).as_bytes(),
			)
			.unwrap();

			if index % phi_res + 1 != phi_res {
				file.write_all(b", ").unwrap();
			} else if index / phi_res + 1 != theta_res {
				file.write_all(b"; ").unwrap();
			}
		});
	};

	let mut file = File::create("chi_test.m").unwrap();

	file.write_all(format!("% wo = {wo}\nfrequencies = [ ").as_bytes())
		.unwrap();
	enumerate(&mut file, |o, _| o);

	file.write_all(b" ];\nexpected_frequencies = [ ").unwrap();
	enumerate(&mut file, |_, e| e);

	file.write_all(b" ];\nchi_terms = [ ").unwrap();
	enumerate(&mut file, chi_squared_term);

	file.write_all(
		b" ];
colormap(jet);
clf;
subplot(3,1,1);
imagesc([0, 360], [0, 180], frequencies);
axis equal;
title('observed frequencies');
subplot(3,1,2);
imagesc([0, 360], [0, 180], chi_terms);
axis equal;
title('chi terms');
subplot(3,1,3);
imagesc([0, 360], [0, 180], expected_frequencies);
axis equal;
title('expected frequencies');",
	)
	.unwrap();
}

pub fn bsdf_testing<S, P>(sample: &S, pdf: &P) -> bool
where
	S: Fn(Vec3) -> Vec3,
	P: Fn(Vec3, Vec3) -> Float,
{
	for i in 0..CHI_TESTS {
		let wo = generate_wo();
		let freq_table = samped_frequency_distribution(&sample, wo, THETA_RES, PHI_RES, SAMPLES);
		let expected_freq_table: Vec<Float> =
			integrate_frequency_table(pdf, wo, THETA_RES, PHI_RES)
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
