use crate::{aabb::AABB, acceleration::PrimitiveInfo, Axis};
use rt_core::Float;

const NUM_BUCKETS: usize = 12;
const MAX_IN_NODE: usize = 255;

#[macro_export]
macro_rules! partition {
	($array:expr, $closure:expr) => {{
		let len = $array.len();
		let (mut left, mut right) = (0, len - 1);
		let mid_index: usize;

		loop {
			while left < len && $closure(&$array[left]) {
				left += 1;
			}

			while right > 0 && !($closure(&$array[right])) {
				right -= 1;
			}

			if left >= right {
				mid_index = left;
				break;
			}
			$array.swap(left, right);
		}
		mid_index
	}};
}

pub enum SplitType {
	Sah,
	Middle,
	EqualCounts,
}

impl Default for SplitType {
	fn default() -> Self {
		SplitType::Sah
	}
}

pub trait Split {
	fn split(
		&self,
		bounds: &AABB,
		center_bounds: &AABB,
		axis: &Axis,
		primitives_info: &mut [PrimitiveInfo],
	) -> usize;
}

#[derive(Copy, Clone)]
pub struct BucketInfo {
	count: u32,
	bounds: Option<AABB>,
}

impl Default for BucketInfo {
	fn default() -> Self {
		Self::new()
	}
}

impl BucketInfo {
	pub fn new() -> Self {
		BucketInfo {
			count: 0,
			bounds: None,
		}
	}
}

impl Split for SplitType {
	fn split(
		&self,
		bounds: &AABB,
		center_bounds: &AABB,
		axis: &Axis,
		primitives_info: &mut [PrimitiveInfo],
	) -> usize {
		match self {
			SplitType::Middle => {
				let point_mid = 0.5
					* (axis.get_axis_value(center_bounds.min)
						+ axis.get_axis_value(center_bounds.max));

				let len = primitives_info.len();

				let closure = |primitive_info: &PrimitiveInfo| -> bool {
					axis.get_axis_value(primitive_info.center) < point_mid
				};
				let mid_index = partition!(primitives_info, closure);

				if mid_index == 0 || mid_index == (len - 1) {
					primitives_info[0..len].sort_by(|a, b| {
						axis.get_axis_value(a.center)
							.partial_cmp(&axis.get_axis_value(b.center))
							.unwrap()
					});
				}
				mid_index
			}
			SplitType::EqualCounts => split_equal(axis, primitives_info),
			SplitType::Sah => {
				let len = primitives_info.len();

				if len <= 4 {
					return split_equal(axis, primitives_info);
				}

				let mut buckets = [BucketInfo::new(); NUM_BUCKETS];

				let max_val = axis.get_axis_value(center_bounds.max);
				let min_val = axis.get_axis_value(center_bounds.min);

				let centroid_extent = max_val - min_val;

				for primitive_info in primitives_info.iter() {
					let b = calculate_b(axis, primitive_info, min_val, centroid_extent);

					buckets[b].count += 1;

					AABB::merge(
						&mut buckets[b].bounds,
						AABB::new(primitive_info.min, primitive_info.max),
					);
				}

				let mut costs = [0.0; NUM_BUCKETS - 1];
				for (i, cost) in costs.iter_mut().enumerate().take(NUM_BUCKETS - 1) {
					let (mut bounds_left, mut bounds_right) = (None, None);
					let (mut count_left, mut count_right) = (0, 0);

					for bucket in buckets.iter().take(i + 1) {
						if bucket.bounds.is_some() {
							AABB::merge(&mut bounds_left, bucket.bounds.unwrap());
							count_left += bucket.count;
						}
					}

					for bucket in buckets.iter().take(NUM_BUCKETS).skip(i + 1) {
						if bucket.bounds.is_some() {
							AABB::merge(&mut bounds_right, bucket.bounds.unwrap());
							count_right += bucket.count;
						}
					}

					let left_sa = bounds_left
						.map(|bounds: AABB| bounds.surface_area())
						.unwrap_or(0.0);

					let right_sa = bounds_right
						.map(|bounds: AABB| bounds.surface_area())
						.unwrap_or(0.0);

					*cost = 0.125
						+ (count_left as Float * left_sa + count_right as Float * right_sa)
							/ bounds.surface_area();
				}

				let mut min_cost = costs[0];
				let mut min_cost_index = 0;

				for (i, cost) in costs.iter().enumerate().take(NUM_BUCKETS - 1).skip(1) {
					if cost < &min_cost {
						min_cost = *cost;
						min_cost_index = i;
					}
				}

				if len > MAX_IN_NODE || min_cost < len as Float {
					let closure = |primitive_info: &PrimitiveInfo| -> bool {
						calculate_b(axis, primitive_info, min_val, centroid_extent)
							<= min_cost_index
					};
					return partition!(primitives_info, closure);
				}
				0
			}
		}
	}
}

fn calculate_b(axis: &Axis, primitive_info: &PrimitiveInfo, min: Float, extent: Float) -> usize {
	let absolute_value = axis.get_axis_value(primitive_info.center);

	let mut b = (NUM_BUCKETS as Float * (absolute_value - min) / extent) as usize;

	if b == NUM_BUCKETS {
		b -= 1;
	}

	b
}

fn split_equal(axis: &Axis, primitives_info: &mut [PrimitiveInfo]) -> usize {
	let len = primitives_info.len();
	let point_mid = len / 2;
	primitives_info[0..len].sort_by(|a, b| {
		axis.get_axis_value(a.center)
			.partial_cmp(&axis.get_axis_value(b.center))
			.unwrap()
	});
	point_mid
}
