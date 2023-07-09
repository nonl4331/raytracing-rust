use crate::statistics::*;
use rayon::prelude::*;
use statrs::function::gamma::gamma_ur;

use std::cmp::Ordering::*;
pub fn chi2_probability(dof: f64, distance: f64) -> f64 {
	match distance.partial_cmp(&0.0).unwrap() {
		Less => panic!("distance < 0.0"),
		Equal => 1.0,
		Greater => {
			if distance.is_infinite() {
				0.0
			} else {
				gamma_ur(dof * 0.5, distance * 0.5)
			}
		}
	}
}

pub fn chi_squared(
	freq_table: &[Float],
	expected_freq_table: &[Float],
	samples: usize,
) -> (usize, Float) {
	assert_eq!(freq_table.len(), expected_freq_table.len());

	let mut values = expected_freq_table
		.into_par_iter()
		.zip(freq_table.into_par_iter())
		.collect::<Vec<(&Float, &Float)>>();

	values.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());

	let mut df = 0;

	let mut expected_pooled = 0.0;
	let mut actual_pooled = 0.0;

	let mut chi_squared = 0.0;

	for (expected, actual) in values {
		if *expected == 0.0 {
			if *actual > (samples / 100_000) as Float {
				chi_squared += INFINITY as Float;
			}
		} else if expected_pooled > 5.0 {
			// prevent df = 0 when all values are less than 5
			let diff = actual_pooled - expected_pooled;
			chi_squared += diff * diff / expected_pooled;
			df += 1;
		} else if *expected < 5.0 || (expected_pooled > 0.0 && expected_pooled < 5.0) {
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

#[cfg(test)]
pub mod test {}

#[cfg(test)]
pub use test::*;
