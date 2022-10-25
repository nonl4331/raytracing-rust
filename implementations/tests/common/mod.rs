use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};
use rayon::prelude::*;
use rt_core::{vec::*, Float};
use statrs::function::gamma::*;
use std::{
	cmp::Ordering::*,
	f64::{consts::*, INFINITY},
	{fs::File, io::Write},
};

pub mod int;

pub const SAMPLES: usize = 10_000_000;
pub const THETA_RES: usize = 80;
pub const PHI_RES: usize = 2 * THETA_RES;
pub const CHI2_THRESHOLD: Float = 0.01;
pub const CHI_TESTS: usize = 1;

use int::*;

pub fn to_vec(sin_theta: Float, cos_theta: Float, phi: Float) -> Vec3 {
	Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
}

pub fn chi2_probability(dof: f64, distance: f64) -> f64 {
	match distance.partial_cmp(&0.0).unwrap() {
		Less => panic!("distance < 0.0"),
		Equal => 1.0,
		Greater => gamma_ur(dof * 0.5, distance * 0.5),
	}
}

pub fn integrate_frequency_table<F>(
	pdf: &F,
	wo: Vec3,
	phi_res: usize,
	theta_res: usize,
) -> Vec<Float>
where
	F: Fn(Vec3, Vec3) -> Float + Sync,
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

	(0..(theta_res * phi_res))
		.into_par_iter()
		.map(|i| {
			let phi_i = i % phi_res;
			let theta_i = i / phi_res;
			adaptive_simpsons(
				|phi| {
					pdf(
						phi,
						theta_i as Float * theta_step,
						(theta_i + 1) as Float * theta_step,
					)
				},
				phi_i as Float * phi_step,
				(phi_i + 1) as Float * phi_step,
			)
		})
		.collect()
}

pub fn samped_frequency_distribution_uv<F>(
	function: &F,
	u_res: usize,
	v_res: usize,
	sample_count: usize,
) -> Vec<Float>
where
	F: Fn() -> (usize, usize) + std::marker::Sync,
{
	let mut freq = vec![vec![0.0; u_res * v_res]; 16];

	freq.par_iter_mut().for_each(|x| {
		for _ in 0..(sample_count / 16) {
			let sample = function();
			x[sample.0 + sample.1 * u_res] += 1.0;
		}
	});

	freq.into_iter()
		.fold(vec![0.0; u_res * v_res], |mut sum, val| {
			sum.iter_mut().zip(val).for_each(|(s, v)| *s += v);
			sum
		})

	/*for _ in 0..sample_count {
		let sample = function();
		freq[sample.0 + sample.1 * u_res] += 1.0;
	}

	freq*/
}

pub fn samped_frequency_distribution<F>(
	function: &F,
	wo: Vec3,
	phi_res: usize,
	theta_res: usize,
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

pub fn generate_wo() -> Vec3 {
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
		} else if expected_pooled > 5.0 {
			// prevent df = 0 when all values are less than 5
			let diff = actual_pooled - expected_pooled;
			chi_squared += diff * diff / expected_pooled;
			df += 1;
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

pub fn dump_tables(
	wo: Vec3,
	freq_table: &[Float],
	expected_freq_table: &[Float],
	x: usize,
	y: usize,
	bxdf_name: &str,
) {
	fn chi_squared_term(a: Float, b: Float) -> Float {
		if a < (SAMPLES / 100000) as Float && b == 0.0 {
			return 0.0;
		}
		let val = a - b;
		val * val / b
	}

	let enumerate = |file: &mut File, func: fn(Float, Float) -> Float| {
		(0..(x * y)).for_each(|index| {
			file.write_all(
				format!("{}", func(freq_table[index], expected_freq_table[index])).as_bytes(),
			)
			.unwrap();

			if index % x + 1 != x {
				file.write_all(b", ").unwrap();
			} else if index / x + 1 != y {
				file.write_all(b"; ").unwrap();
			}
		});
	};

	let time = chrono::Local::now();

	let mut file = File::create(format!(
		"chi_test_{bxdf_name}@{}.m",
		time.format("%Y-%m-%d:%H:%M")
	))
	.unwrap();

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
