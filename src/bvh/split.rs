use crate::bvh::{aabb::AABB, bvh::PrimitiveInfo};
use crate::math::Float;

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
        start: usize,
        end: usize,
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
        start: usize,
        end: usize,
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
                let (mut left, mut right) = (0, len - 1);
                let mid_index: usize;

                loop {
                    while left < len
                        && axis.get_axis_value(primitives_info[left].center) < point_mid
                    {
                        left += 1;
                    }
                    while right > 0
                        && axis.get_axis_value(primitives_info[right].center) >= point_mid
                    {
                        right -= 1;
                    }
                    if left >= right {
                        mid_index = left;
                        break;
                    }
                    primitives_info.swap(left, right);
                }

                let mid_index = mid_index + start;

                if mid_index == start || mid_index == end {
                    primitives_info[start..end].sort_by(|a, b| {
                        axis.get_axis_value(a.center)
                            .partial_cmp(&axis.get_axis_value(b.center))
                            .unwrap()
                    });
                }
                mid_index
            }
            SplitType::EqualCounts => {
                let point_mid = (start + end) / 2;
                primitives_info[start..end].sort_by(|a, b| {
                    axis.get_axis_value(a.center)
                        .partial_cmp(&axis.get_axis_value(b.center))
                        .unwrap()
                });
                point_mid
            }
            SplitType::SAH => {
                if end - start <= 4 {
                    let point_mid = (start + end) / 2;
                    primitives_info[start..end].sort_by(|a, b| {
                        axis.get_axis_value(a.center)
                            .partial_cmp(&axis.get_axis_value(b.center))
                            .unwrap()
                    });
                    return point_mid;
                }

                let mut buckets: [BucketInfo; NUM_BUCKETS] = [BucketInfo::new(); NUM_BUCKETS];

                let max_val = &axis.get_axis_value(center_bounds.max);
                let min_val = &axis.get_axis_value(center_bounds.min);

                let centroid_extent = max_val - min_val;

                for i in start..end {
                    let axis_val = &axis.get_axis_value(primitives_info[i].center);

                    let mut b: usize =
                        (NUM_BUCKETS as Float * (axis_val - min_val) / centroid_extent) as usize;

                    if b == NUM_BUCKETS {
                        b -= 1;
                    }
                    buckets[b].count += 1;
                    AABB::merge(
                        &mut buckets[b].bounds,
                        AABB::new(primitives_info[i].min, primitives_info[i].max),
                    );
                }

                let mut costs: [Float; NUM_BUCKETS - 1] = [0.0; NUM_BUCKETS - 1];
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
                        * (count_left as Float * left_sa + count_right as Float * right_sa)
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

                if end - start > MAX_IN_NODE || min_cost < (end - start) as f32 {
                    // TODO
                    let len = primitives_info.len();
                    let (mut left, mut right) = (0, len - 1);
                    let mid_index: usize;

                    loop {
                        let axis_val = &axis.get_axis_value(primitives_info[left].center);
                        let mut b: usize = (NUM_BUCKETS as Float * (axis_val - min_val)
                            / centroid_extent) as usize;
                        if b == NUM_BUCKETS {
                            b -= 1;
                        }

                        while left < len && b <= min_cost_index {
                            left += 1;
                            let axis_val = &axis.get_axis_value(primitives_info[left].center);
                            b = (NUM_BUCKETS as Float * (axis_val - min_val) / centroid_extent)
                                as usize;
                            if b == NUM_BUCKETS {
                                b -= 1;
                            }
                        }

                        let axis_val = &axis.get_axis_value(primitives_info[right].center);
                        let mut b: usize = (NUM_BUCKETS as Float * (axis_val - min_val)
                            / centroid_extent) as usize;
                        if b == NUM_BUCKETS {
                            b -= 1;
                        }

                        while right > 0 && b > min_cost_index {
                            right -= 1;
                            let axis_val = &axis.get_axis_value(primitives_info[right].center);
                            b = (NUM_BUCKETS as Float * (axis_val - min_val) / centroid_extent)
                                as usize;
                            if b == NUM_BUCKETS {
                                b -= 1;
                            }
                        }

                        if left >= right {
                            mid_index = left;
                            break;
                        }
                        primitives_info.swap(left, right);
                    }
                    mid_index + start
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
