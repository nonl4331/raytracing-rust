use crate::bvh::{aabb::AABB, bvh::PrimitiveInfo};
use crate::math::Float;
use crate::partition;

use crate::ray_tracing::primitives::Axis;

const NUM_BUCKETS: usize = 12;
const MAX_IN_NODE: usize = 255;

pub enum SplitType {
    SAH,
    HLBVH,
    Middle,
    EqualCounts,
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
                    axis.get_axis_value(primitive_info.center) >= point_mid
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
            SplitType::SAH => {
                let len = primitives_info.len();

                if len <= 4 {
                    return split_equal(axis, primitives_info);
                }

                let mut buckets = [BucketInfo::new(); NUM_BUCKETS];

                let max_val = axis.get_axis_value(center_bounds.max);
                let min_val = axis.get_axis_value(center_bounds.min);

                let centroid_extent = max_val - min_val;

                for primitive_info in primitives_info.iter() {
                    let b = calculate_b(&axis, &primitive_info, min_val, centroid_extent);

                    buckets[b].count += 1;

                    AABB::merge(
                        &mut buckets[b].bounds,
                        AABB::new(primitive_info.min, primitive_info.max),
                    );
                }

                let mut costs = [0.0; NUM_BUCKETS - 1];
                for i in 0..(NUM_BUCKETS - 1) {
                    let mut bounds_left = None;
                    let mut count_left = 0;

                    let mut bounds_right = None;
                    let mut count_right = 0;

                    for j in 0..(i + 1) {
                        match buckets[j].bounds {
                            Some(bounds) => {
                                AABB::merge(&mut bounds_left, bounds);
                                count_left += buckets[j].count;
                            }
                            None => {}
                        }
                    }

                    for j in (i + 1)..NUM_BUCKETS {
                        match buckets[j].bounds {
                            Some(bounds) => {
                                AABB::merge(&mut bounds_right, bounds);
                                count_right += buckets[j].count;
                            }
                            None => {}
                        }
                    }

                    let left_sa = match bounds_left {
                        Some(val) => val.surface_area(),
                        None => 0.0,
                    };

                    let right_sa = match bounds_right {
                        Some(val) => val.surface_area(),
                        None => 0.0,
                    };

                    costs[i] = 0.125
                        + (count_left as Float * left_sa + count_right as Float * right_sa)
                            / bounds.surface_area();
                }

                let mut min_cost = costs[0];
                let mut min_cost_index = 0;

                for i in 1..(NUM_BUCKETS - 1) {
                    if costs[i] < min_cost {
                        min_cost = costs[i];
                        min_cost_index = i;
                    }
                }

                if len > MAX_IN_NODE || min_cost < len as f32 {
                    let closure = |primitive_info: &PrimitiveInfo| -> bool {
                        calculate_b(&axis, &primitive_info, min_val, centroid_extent)
                            <= min_cost_index
                    };
                    partition!(primitives_info, closure)
                } else {
                    0
                }
            }
            SplitType::HLBVH => {
                unimplemented!()
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
