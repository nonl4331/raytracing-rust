use crate::ray_tracing::{
    intersection::{PrimitiveTrait, SurfaceIntersection},
    primitives::AARect,
    ray::Ray,
};
use crate::utility::{math::Float, vec::Vec3};

const EPSILON: Float = 0.00000003;
const AARECT_INTERSECTION: AARectIntersection = AARectIntersection::One;

enum AARectIntersection {
    One,
}

pub fn aarect_intersection(aarect: &AARect, ray: &Ray) -> Option<SurfaceIntersection> {
    match AARECT_INTERSECTION {
        AARectIntersection::One => aarect_intersection_one(aarect, ray),
    }
}

fn aarect_intersection_one(aarect: &AARect, ray: &Ray) -> Option<SurfaceIntersection> {
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
        Some(SurfaceIntersection::new(
            t,
            point + EPSILON * aarect.axis.return_point_with_axis(Vec3::one()),
            Vec3::zero(),
            aarect
                .axis
                .return_point_with_axis(-1.0 * ray.direction)
                .normalised(),
            aarect.get_uv(point),
            true,
            &aarect.material,
        ))
    } else {
        None
    }
}
