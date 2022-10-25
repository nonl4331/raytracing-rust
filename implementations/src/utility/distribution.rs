use rt_core::Float;

use super::random_float;

pub struct CDF1D {
	pub intervals: Vec<Float>,
}

impl CDF1D {
	pub fn from_pdf(pdf: Vec<Float>) -> Self {
		if pdf.is_empty() {
			panic!("Empty pdf passed to CDF1D::from_pdf!");
		}

		let mut intervals = vec![0.0];
		for val in pdf.iter() {
			let len = intervals.len();
			intervals.push((intervals[len - 1] + val).min(1.0));
		}
		intervals[pdf.len()] = 1.0;
		Self { intervals }
	}
	pub fn sample(&self) -> usize {
		let num = random_float();

		let mut low = 0;
		let mut high = self.intervals.len() - 1;

		let mut i = (low + high) / 2;
		let mut above = num >= self.intervals[i];
		let mut below = self.intervals[i + 1] > num;

		while !(above && below) {
			if above {
				low = i;
			} else {
				high = i;
			}
			i = (low + high) / 2;
			above = num >= self.intervals[i];
			below = self.intervals[i + 1] > num;
		}

		i
	}
}

pub struct CDF2D {
	pub y_cdf: CDF1D,
	x_cdfs: Vec<CDF1D>,
}

impl CDF2D {
	pub fn from_pdf(pdf: &[Float], width: usize) -> Self {
		if pdf.len() % width != 0 {
			panic!("Invalid width passed to CDF2D");
		}
		let height = pdf.len() / width;
		let mut row_sums: Vec<Float> = Vec::new();
		let mut average = 0.0;
		for y in 0..height {
			let mut row_sum = 0.0;
			for x in 0..width {
				let i = width * y + x;
				average += pdf[i];
				row_sum += pdf[i];
			}
			row_sums.push(row_sum);
		}

		let y_pdf: Vec<Float> = row_sums.iter().map(|v| v / average).collect();

		// renormalise due to floating point error
		let sum: Float = y_pdf.iter().sum();
		let y_pdf: Vec<Float> = y_pdf.into_iter().map(|v| v / sum).collect();

		let y_cdf = CDF1D::from_pdf(y_pdf);

		assert!(pdf.len() / width == (y_cdf.intervals.len() - 1));

		let x_cdfs: Vec<CDF1D> = pdf
			.chunks_exact(width)
			.zip(row_sums)
			.map(|(chunk, row_sum)| CDF1D::from_pdf(chunk.iter().map(|v| v / row_sum).collect()))
			.collect();

		Self { y_cdf, x_cdfs }
	}
	pub fn sample(&self) -> (usize, usize) {
		let v = self.y_cdf.sample();
		let u = self.x_cdfs[v].sample();
		(u, v)
	}
}

#[test]
fn cdf1d_sampling() {
	let pdf = [0.1, 0.5, 0.3, 0.1];

	let cdf = CDF1D::from_pdf(pdf.to_vec());

	let samples = 1_000_000;
	let mut bins = [0, 0, 0, 0];

	for _ in 0..samples {
		match cdf.sample() {
			0 => {
				bins[0] += 1;
			}
			1 => {
				bins[1] += 1;
			}
			2 => {
				bins[2] += 1;
			}
			3 => {
				bins[3] += 1;
			}
			_ => unreachable!(),
		}
	}

	bins.into_iter()
		.zip(pdf)
		.for_each(|(v, p)| assert!((v as Float / samples as Float - p).abs() < 0.01));
}

#[test]
fn cdf2d_sampling() {
	use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};

	let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
	let pdf: Vec<usize> = (0..10000).map(|_| rng.gen_range(0..1000)).collect();
	let sum: usize = pdf.iter().sum();
	let pdf: Vec<Float> = pdf.into_iter().map(|v| v as Float / sum as Float).collect();

	let cdf = CDF2D::from_pdf(&pdf, 100);

	let mut bins = [0; 10000];

	let samples = 1_000_000;

	for _ in 0..samples {
		let sample = cdf.sample();
		bins[sample.0 + sample.1 * 100] += 1;

		bins.into_iter()
			.zip(pdf.clone())
			.for_each(|(v, p)| assert!((v as Float / samples as Float - p).abs() < 0.01));
	}
}
