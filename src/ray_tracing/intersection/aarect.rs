use crate::ray_tracing::{
    intersection::{Hit, PrimitiveTrait},
    material::Material,
    primitives::AARect,
    ray::Ray,
};
use crate::utility::{math::Float, vec::Vec3};
use std::sync::Arc;

const EPSILON: Float = 0.00000003;
const AARECT_INTERSECTION: AARectIntersection = AARectIntersection::One;

enum AARectIntersection {
    One,
}

pub fn aarect_intersection(aarect: &AARect, ray: &Ray) -> Option<(Hit, Arc<Material>)> {
    match AARECT_INTERSECTION {
        AARectIntersection::One => aarect_intersection_one(aarect, ray),
    }
}

fn aarect_intersection_one(aarect: &AARect, ray: &Ray) -> Option<(Hit, Arc<Material>)> {
    let t = (aarect.k - aarect.axis.get_axis_value(ray.origin))
        / aarect.axis.get_axis_value(ray.direction);
    let point = ray.at(t);
    let point_2d = aarect.axis.point_without_axis(point);

    // x & y are not the x & y axis but rather the two axis that are not self.axis
    if point_2d.x > aarect.min.x
        && point_2d.x < aarect.max.x
        && point_2d.y > aarect.min.y
        && point_2d.y < aarect.max.y
    {
        Some((
            Hit {
                t,
                point: point + EPSILON * aarect.axis.return_point_with_axis(Vec3::one()),
                error: Vec3::zero(),
                normal: aarect
                    .axis
                    .return_point_with_axis(-1.0 * ray.direction)
                    .normalised(),
                uv: aarect.get_uv(point),
                out: true,
            },
            aarect.material.clone(),
        ))
    } else {
        None
    }
}
