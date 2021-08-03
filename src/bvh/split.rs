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
        primitives_info: &mut Vec<PrimitiveInfo>,
    ) -> usize;
}

impl Split for SplitType {
    fn split(
        &self,
        start: usize,
        end: usize,
        center_bounds: &AABB,
        axis: &Axis,
        primitives_info: &mut Vec<PrimitiveInfo>,
    ) -> usize {
        match self {
            SplitType::Middle => {
                let point_mid = 0.5
                    * (axis.get_axis_value(center_bounds.min)
                        + axis.get_axis_value(center_bounds.max));

                let (mut left, mut right): (Vec<PrimitiveInfo>, Vec<PrimitiveInfo>) =
                    primitives_info
                        .drain(start..end)
                        .partition(|part| axis.get_axis_value(part.center) < point_mid);

                let mid_index = left.len() + start;

                let left_len = left.len();
                for (index, element) in left.drain(..).enumerate() {
                    primitives_info.insert(start + index, element);
                }

                for (index, element) in right.drain(..).enumerate() {
                    primitives_info.insert(start + index + left_len, element);
                }

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
