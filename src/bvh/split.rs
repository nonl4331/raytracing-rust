use crate::bvh::{aabb::AABB, bvh::PrimitiveInfo};

use crate::ray_tracing::primitives::Axis;

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
        center_bounds: &AABB,
        axis: &Axis,
        primitives_info: &mut [PrimitiveInfo],
    ) -> usize;
}

impl Split for SplitType {
    fn split(
        &self,
        start: usize,
        end: usize,
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
                unimplemented!()
            }
            SplitType::HLBVH => {
                unimplemented!()
            }
        }
    }
}
