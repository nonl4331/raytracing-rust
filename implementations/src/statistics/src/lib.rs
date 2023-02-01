pub use rt_core::{self, *};

pub mod bxdfs;
pub mod chi_squared;
pub mod distributions;
pub mod integrators;
pub mod spherical_sampling;

pub mod utility {

	use rayon::prelude::*;
	use std::ops::*;

	pub fn distribute_samples_over_threads<T, F>(samples: u64, f: &F) -> Vec<T>
	where
		T: Add + Send,
		F: Fn(u64) -> Vec<T> + Sync,
		Vec<T>: FromIterator<<T as Add>::Output>,
	{
		let thread_count = num_cpus::get();
		let mut samples_per_thread = vec![samples / thread_count as u64; thread_count];
		let diff = ((samples / thread_count as u64) * thread_count as u64) - samples;
		let last = samples_per_thread.len() - 1;
		samples_per_thread[last] += diff;

		samples_per_thread
			.into_par_iter()
			.map(f)
			.reduce_with(|a, b| {
				a.into_iter()
					.zip(b.into_iter())
					.map(|(a, b)| a + b)
					.collect()
			})
			.unwrap()
	}

	use rt_core::Float;

	pub fn recursively_binary_average<T: Add + Mul>(mut values: Vec<T>) -> T
	where
		<T as Add>::Output: Mul<Float>,
		T: Copy,
		Vec<T>: FromIterator<<<T as Add>::Output as Mul<Float>>::Output>,
	{
		let mut len = values.len();
		if len & (len - 1) != 0 && len != 0 {
			panic!("values.len() is not a power of 2");
		}
		while len != 1 {
			len /= 2;

			let (a, b) = values.split_at(len);

			values = a
				.iter()
				.zip(b.iter())
				.map(|(&a, &b)| (a + b) * 0.5)
				.collect();
		}

		values[0]
	}

	#[cfg(test)]
	#[test]
	fn binary_average() {
		assert!(
			(recursively_binary_average::<Float>(vec![1.0, 3.0, 7.0, 1.0]) - 3.0).abs() < 0.0000001
		);
	}
}
