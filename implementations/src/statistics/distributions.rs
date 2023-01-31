use rt_core::Float;

use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub struct Distribution1D {
	pub pdf: Vec<Float>,
	pub cdf: Vec<Float>,
}

impl Distribution1D {
	pub fn new(values: &[Float]) -> Self {
		if values.is_empty() {
			panic!("Empty pdf passed to Distribution1D::from_pdf!");
		}

		let n = values.len();

		let mut intervals = vec![0.0];

		for i in 1..=n {
			let last = intervals[i - 1];
			intervals.push(last + values[i - 1] as Float);
		}

		let c = intervals[n];
		for (_, value) in intervals.iter_mut().enumerate() {
			if c != 0.0 {
				*value /= c as Float;
			}
		}

		let mut pdf = Vec::new();
		let mut last = 0.0;
		for value in &intervals[1..] {
			pdf.push(value - last);
			last = *value;
		}

		Self {
			pdf,
			cdf: intervals,
		}
	}

	pub fn sample_naive<R: Rng>(&self, rng: &mut R) -> usize {
		let threshold = rng.gen();

		self.cdf.iter().position(|v| v >= &threshold).unwrap() - 1
	}
	pub fn sample<R: Rng>(&self, rng: &mut R) -> usize {
		let num = rng.gen();

		let pred = |i| self.cdf[i] <= num;

		{
			let mut first = 0;
			let mut len = self.cdf.len();
			while len > 0 {
				let half = len >> 1;
				let middle = first + half;

				if pred(middle) {
					first = middle + 1;
					len -= half + 1;
				} else {
					len = half;
				}
			}
			(first - 1).clamp(0, self.cdf.len() - 2)
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Distribution2D {
	pub x_distributions: Vec<Distribution1D>,
	pub y_distribution: Distribution1D,
	pub dim: (usize, usize),
}

impl Distribution2D {
	pub fn new(values: &[Float], width: usize) -> Self {
		assert!(values.len() % width == 0 && !values.is_empty());
		let mut y_values = Vec::new();
		let mut x_distributions = Vec::new();
		for vec_x in values.chunks_exact(width) {
			x_distributions.push(Distribution1D::new(vec_x));
			let row_sum: Float = vec_x.iter().sum();
			y_values.push(row_sum);
		}
		let y_distribution = Distribution1D::new(&y_values);

		Self {
			x_distributions,
			y_distribution,
			dim: (width, values.len() / width),
		}
	}
	pub fn sample<R: Rng>(&self, rng: &mut R) -> (usize, usize) {
		let v = self.y_distribution.sample(rng);
		let u = self.x_distributions[v].sample(rng);
		(u, v)
	}
	pub fn pdf(&self, u: Float, v: Float) -> Float {
		let u = ((self.dim.0 as Float * u) as usize).clamp(0, self.dim.0 - 1);
		let v = ((self.dim.1 as Float * v) as usize).clamp(0, self.dim.1 - 1);

		self.y_distribution.pdf[v] * self.x_distributions[v].pdf[u]
	}
	pub fn dim(&self) -> (usize, usize) {
		self.dim
	}
}

#[cfg(test)]
mod tests {
	use crate::statistics::{chi_squared::*, distributions::*, utility::*};
	use rand::thread_rng;
	use rayon::prelude::*;

	macro_rules! random_1d {
        ($len:expr) => {{
            const SAMPLES: usize = 100_000;
            const SAMPLE_LEN: usize = 100;
            const NUMBER_TESTS: usize = 10;
            const CHI_SQUARED_THRESHOLD: Float = 0.01;
            const BATCH_EXPONENT: usize = 6;
			const BATCHES: usize = 2usize.pow(BATCH_EXPONENT as u32);

            let mut rng = thread_rng();

            let values: Vec<Float> = (0..SAMPLE_LEN)
                .into_iter()
                .map(|_| rng.gen_range(0.0..100.0))
                .collect();

            let cdf = Distribution1D::new(&values);

            let expected_values: Vec<Float> = cdf.pdf.iter().map(|v| v * SAMPLES as Float).collect();

            let func = |samples| {
                const SAMPLE_LEN_MINUS_ONE: usize = SAMPLE_LEN - 1;
                let mut rng = thread_rng();
                let mut sampled_values = vec![0u64; SAMPLE_LEN];
                for _ in 0..samples {
                    match cdf.sample(&mut rng) {
                        index @ 0..=SAMPLE_LEN_MINUS_ONE => {
                            sampled_values[index] += 1;
                        }
                        _ => unreachable!(),
                    }
                }
                sampled_values
            };

            // Šidák correction
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
				let sampled_values: Vec<Float> = (0..SAMPLE_LEN)
					.into_par_iter()
					.map(|i| {
						recursively_binary_average((0..BATCHES).map(|j| sampled_vecs[j][i]).collect())
					})
					.collect();

                let (df, chi_squared) = chi_squared(&sampled_values, &expected_values, SAMPLES);

                let p_value = chi2_probability(df as f64, chi_squared as f64);
                if p_value < threshold as f64 {
                    panic!("LERP: recieved p value of {p_value} with {SAMPLES} samples on test {i}/{NUMBER_TESTS}")
                }
            }}
        };
    }

	#[test]
	fn random_1d_small() {
		random_1d!(50)
	}

	#[test]
	fn random_1d_medium() {
		random_1d!(500)
	}
	#[test]
	fn random_1d_large() {
		random_1d!(5000)
	}

	macro_rules! random_2d {
        ($x:expr, $y:expr) => { {
        const SAMPLES: usize = 100_000;
        const X_RES_MINUS_ONE: usize = $x - 1;
        const Y_RES_MINUS_ONE: usize = $y - 1;
        const SAMPLES_RES: (usize, usize) = (X_RES_MINUS_ONE + 1, Y_RES_MINUS_ONE + 1);
        const SAMPLE_LEN: usize = SAMPLES_RES.0 * SAMPLES_RES.1;
        const NUMBER_TESTS: usize = 10;
        const CHI_SQUARED_THRESHOLD: Float = 0.01;
        const BATCH_EXPONENT: usize = 6;
		const BATCHES: usize = 2usize.pow(BATCH_EXPONENT as u32);

        //use rand_core::SeedableRng;

        //let mut rng = SmallRng::seed_from_u64(321321);
        let mut rng = thread_rng();

        let values: Vec<Float> = (0..SAMPLE_LEN)
            .into_iter()
            .map(|_| rng.gen_range(0.0..100.0))
            .collect();

        let dist = Distribution2D::new(&values, SAMPLES_RES.0);

        let mut expected_values: Vec<Float> = Vec::new();
        for y in 0..SAMPLES_RES.1 {
            for x in 0..SAMPLES_RES.0 {
                expected_values.push(
                    dist.y_distribution.pdf[y] * dist.x_distributions[y].pdf[x] * SAMPLES as Float,
                );
            }
        }

        // dist.pdf.iter().map(|v| v * SAMPLES as Float).collect();

        let func = |samples| {
            const SAMPLE_LEN_MINUS_ONE: usize = SAMPLE_LEN - 1;
            let mut rng = thread_rng();
            let mut sampled_values = vec![0u64; SAMPLE_LEN];
            for _ in 0..samples {
                match dist.sample(&mut rng) {
                    indices @ (0..=X_RES_MINUS_ONE, 0..=Y_RES_MINUS_ONE)
                        if indices.0 + (X_RES_MINUS_ONE + 1) * indices.1 < SAMPLE_LEN =>
                    {
                        sampled_values[indices.0 + indices.1 * (X_RES_MINUS_ONE + 1)] += 1;
                    }
                    _ => unreachable!(),
                }
            }
            sampled_values
        };

        // Šidák correction
        let threshold = 1.0 - (1.0 - CHI_SQUARED_THRESHOLD).powf(1.0 / NUMBER_TESTS as Float);

        for i in 0..NUMBER_TESTS {
            let sampled_vecs: Vec<Vec<Float>> = (0..BATCHES)
				.into_iter()
				.map(|_| {
					distribute_samples_over_threads(SAMPLES as u64, &func)
						.into_iter()
						.map(|v| v as Float)
						.collect()
				})
				.collect();
			let sampled_values: Vec<Float> = (0..SAMPLE_LEN)
				.into_par_iter()
				.map(|i| {
					recursively_binary_average(
						(0..BATCHES)
							.into_iter()
							.map(|j| sampled_vecs[j][i])
							.collect(),
					)
				})
				.collect();

            let (df, chi_squared) = chi_squared(&sampled_values, &expected_values, SAMPLES);

            let p_value = chi2_probability(df as f64, chi_squared as f64);
            if p_value < threshold as f64 {
                panic!("LERP: recieved p value of {p_value} with {SAMPLES} samples on test {i}/{NUMBER_TESTS}")
            }
        }
        }};
    }

	#[test]
	fn random_2d_small() {
		random_2d!(3, 3)
	}

	#[test]
	fn random_2d_medium() {
		random_2d!(30, 60)
	}

	#[test]
	fn random_2d_large() {
		random_2d!(800, 1200)
	}
}
