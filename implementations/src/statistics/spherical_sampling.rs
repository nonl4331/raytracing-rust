use crate::statistics::chi_squared::*;
use crate::statistics::integrators::integrate_solid_angle;
use crate::statistics::utility::*;
use crate::statistics::*;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use rand::Rng;
use rayon::prelude::*;

pub fn cosine_hemisphere_sampling<R: Rng>(rng: &mut R) -> Vec3 {
	let cos_theta = (1.0 - rng.gen::<Float>()).sqrt();
	let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
	let phi = 2.0 * PI * rng.gen::<Float>();
	Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
}

pub fn cosine_hemisphere_pdf(wo: Vec3) -> Float {
	wo.z.max(0.0) / PI
}

pub fn specular_sampling<R: Rng>(n: Float, rng: &mut R) -> Vec3 {
	let a = rng.gen::<Float>().powf(1.0 / (n + 1.0));
	let term = (1.0 - a * a).sqrt();
	let phi = 2.0 * PI * rng.gen::<Float>();
	Vec3::new(term * phi.cos(), term * phi.sin(), a)
}

pub fn random_unit_vector<R: Rng>(rng: &mut R) -> Vec3 {
	let (mut x, mut y, mut z) = (1.0, 1.0, 1.0);
	while x * x + y * y + z * z > 1.0 {
		x = rng.gen_range(-1.0..1.0);
		y = rng.gen_range(-1.0..1.0);
		z = rng.gen_range(-1.0..1.0);
	}

	Vec3::new(x, y, z).normalised()
}

pub fn test_spherical_pdf<P, S>(name: &str, pdf: &P, sample: &S, hemisphere: bool)
where
	P: Fn(Vec3) -> Float,
	S: Fn(&mut ThreadRng) -> Vec3 + Send + Sync,
{
	const THETA_RES: usize = 80;
	const PHI_RES: usize = 160;
	const SAMPLES: usize = 100_000;
	const SAMPLE_LEN: usize = THETA_RES * PHI_RES;
	const NUMBER_TESTS: usize = 10;
	const CHI_SQUARED_THRESHOLD: Float = 0.01;
	const BATCH_EXPONENT: usize = 6;
	const BATCHES: usize = 2usize.pow(BATCH_EXPONENT as u32);

	let mut expected_values = Vec::new();

	let theta_step = if hemisphere { FRAC_PI_2 } else { PI } / THETA_RES as Float;
	let phi_step = TAU / PHI_RES as Float;
	for phi_i in 0..PHI_RES {
		for theta_i in 0..THETA_RES {
			let theta_start = theta_i as Float * theta_step;
			let phi_start = phi_i as Float * phi_step;
			expected_values.push(integrate_solid_angle(
				pdf,
				theta_start,
				theta_start + theta_step,
				phi_start,
				phi_start + phi_step,
			));
		}
	}

	let pdf_sum = expected_values.iter().sum::<Float>();
	if (pdf_sum - 1.0).abs() > 0.001 {
		panic!("reference pdf doesn't integrate to 1: {pdf_sum}");
	}

	let mut expected_values: Vec<Float> = expected_values
		.into_iter()
		.map(|v| v * SAMPLES as Float)
		.collect();

	let func = |samples| {
		const SAMPLE_LEN_MINUS_ONE: usize = SAMPLE_LEN - 1;
		let mut rng = thread_rng();
		let mut sampled_values = vec![0u64; SAMPLE_LEN];
		for _ in 0..samples {
			let wo = sample(&mut rng);

			let sin_theta = (1.0 - wo.z * wo.z).sqrt();
			if sin_theta < 0.0 {
				panic!("sin_theta ({sin_theta}) < 0.0");
			}
			let theta = wo.z.acos();
			let mut phi = (wo.y).atan2(wo.x);
			if phi < 0.0 {
				phi += 2.0 * PI;
			}
			let theta_i = theta / theta_step;
			let phi_i = phi / phi_step;
			let index = (phi_i as usize * THETA_RES + theta_i as usize).min(SAMPLE_LEN_MINUS_ONE);

			sampled_values[index] += 1;
		}
		sampled_values
	};

	let threshold = 1.0 - (1.0 - CHI_SQUARED_THRESHOLD).powf(1.0 / NUMBER_TESTS as Float);

	for i in 0..NUMBER_TESTS {
		let sampled_vecs: Vec<Vec<Float>> = (0..BATCHES)
			.map(|_| {
				distribute_samples_over_threads(SAMPLES as u64, &func)
					.into_iter()
					.map(|v| v as Float)
					.collect()
			})
			.collect();
		let mut sampled_values: Vec<Float> = (0..SAMPLE_LEN)
			.into_par_iter()
			.map(|i| {
				recursively_binary_average::<Float>(
					(0..BATCHES).map(|j| sampled_vecs[j][i]).collect(),
				)
			})
			.collect();

		let (df, chi_squared) = chi_squared(&sampled_values, &expected_values, SAMPLES);

		let p_value = chi2_probability(df as f64, chi_squared as f64);
		if p_value < threshold as f64 {
			let expected_abs_max = expected_values
				.iter()
				.max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
				.unwrap()
				.abs();

			let sampled_abs_max = sampled_values
				.iter()
				.max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
				.unwrap()
				.abs();

			let get_colour = |value: Float, max_abs_value: Float| -> Vec3 {
				if value > 0.0 {
					let t = value / max_abs_value;

					t * Vec3::new(1.0, 0.0, 0.0)
				} else if value == 0.0 {
					Vec3::new(0.0, 1.0, 0.0)
				} else {
					Vec3::new(0.0, 0.0, 1.0)
				}
			};

			let transpose = |vec: &mut Vec<Float>| {
				*vec = (0..(PHI_RES * THETA_RES))
					.map(|i| {
						let y = i % PHI_RES;
						let x = i / PHI_RES;

						vec[y * THETA_RES + x]
					})
					.collect::<Vec<Float>>();
			};

			transpose(&mut expected_values);
			transpose(&mut sampled_values);

			let mut image = expected_values
				.into_iter()
				.map(|v| get_colour(v, expected_abs_max))
				.collect::<Vec<Vec3>>();
			image.extend(
				(0..PHI_RES)
					.map(|_| Vec3::new(0.12, 0.95, 0.95))
					.collect::<Vec<Vec3>>(),
			);
			image.extend(
				sampled_values
					.into_iter()
					.map(|v| get_colour(v, sampled_abs_max))
					.collect::<Vec<Vec3>>(),
			);
			let image = image
				.into_iter()
				.flat_map(|v| [v.x, v.y, v.z])
				.map(|v| (v * 256.0).clamp(0.0, 255.0) as u8)
				.collect::<Vec<u8>>();

			image::save_buffer(
				format!("{name}_failed_output_test_{i}.png"),
				&image,
				PHI_RES.try_into().unwrap(),
				(THETA_RES * 2 + 1).try_into().unwrap(),
				image::ColorType::Rgb8,
			)
			.unwrap();

			panic!("{name}: recieved p value of {p_value} with {SAMPLES} samples averaged over {BATCHES} batches on test {i}/{NUMBER_TESTS}")
		}
	}
}

#[cfg(test)]
pub mod test {
	use super::*;
	use rand::Rng;

	pub fn to_vec(sin_theta: Float, cos_theta: Float, phi: Float) -> Vec3 {
		Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
	}

	pub fn generate_wi<R: Rng>(rng: &mut R) -> Vec3 {
		let cos_theta: Float = rng.gen();
		let phi = TAU as Float * rng.gen::<Float>();

		to_vec((1.0 - cos_theta * cos_theta).sqrt(), cos_theta, phi)
	}

	#[test]
	fn cosine_hemisphere() {
		test_spherical_pdf(
			"cosine hemisphere sampling",
			&cosine_hemisphere_pdf,
			&cosine_hemisphere_sampling,
			true,
		);
	}
}

#[cfg(test)]
pub use test::*;
