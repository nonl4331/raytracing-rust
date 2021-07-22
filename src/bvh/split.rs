use crate::bvh::bvh::PrimitiveInfo;

use crate::bvh::aabb::AABB;

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
                let (left, right): (Vec<PrimitiveInfo>, Vec<PrimitiveInfo>) = primitives_info
                    .drain(..)
                    .partition(|part| axis.get_axis_value(part.center) < point_mid);
                *primitives_info = left;
                let mid_index = primitives_info.len();
                primitives_info.extend(right);
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
            _ => {
                unimplemented!()
            }
        }
    }
}
