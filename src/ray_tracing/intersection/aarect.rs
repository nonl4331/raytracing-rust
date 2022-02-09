use crate::ray_tracing::{
    intersection::{Primitive, SurfaceIntersection},
    material::Scatter,
    primitives::AARect,
    ray::Ray,
};
use crate::utility::{math::Float, vec::Vec3};

#[cfg(all(feature = "f64"))]
const EPSILON: Float = 5.58E-17;

#[cfg(not(feature = "f64"))]
const EPSILON: Float = 3.0E-8;

const AARECT_INTERSECTION: AARectIntersection = AARectIntersection::One;

enum AARectIntersection {
    One,
}

pub fn aarect_intersection<M: Scatter>(
    aarect: &AARect<M>,
    ray: &Ray,
) -> Option<SurfaceIntersection<M>> {
    match AARECT_INTERSECTION {
        AARectIntersection::One => aarect_intersection_one(aarect, ray),
    }
}

fn aarect_intersection_one<M: Scatter>(
    aarect: &AARect<M>,
    ray: &Ray,
) -> Option<SurfaceIntersection<M>> {
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
            point,
            EPSILON * aarect.axis.return_point_with_axis(Vec3::one()),
            aarect
                .axis
                .return_point_with_axis(-ray.direction)
                .normalised(),
            aarect.get_uv(point),
            true,
            &aarect.material,
        ))
    } else {
        None
    }
}
