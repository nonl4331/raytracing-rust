use crate::ray_tracing::{
    intersection::{aarect::aarect_intersection, SurfaceIntersection},
    material::Scatter,
    primitives::{Axis, Rect},
    ray::Ray,
};
use crate::utility::math::Float;

const EPSILON: Float = 0.00000003;
const RECT_INTERSECTION: RectIntersection = RectIntersection::One;

enum RectIntersection {
    One,
}

pub fn rect_intersection<M: Scatter>(rect: &Rect<M>, ray: &Ray) -> Option<SurfaceIntersection<M>> {
    match RECT_INTERSECTION {
        RectIntersection::One => rect_intersection_one(rect, ray),
    }
}

fn rect_intersection_one<M: Scatter>(rect: &Rect<M>, ray: &Ray) -> Option<SurfaceIntersection<M>> {
    let center_point = (Axis::point_from_2d(&rect.aarect.max, &rect.aarect.axis, rect.aarect.k)
        + Axis::point_from_2d(&rect.aarect.min, &rect.aarect.axis, rect.aarect.k))
        / 2.0;
    let mut rotated_origin = ray.origin - center_point;
    let mut rotated_direction = ray.direction;

    // rotate around x
    rotated_origin.y =
        rect.cos_rotations.x * rotated_origin.y - rect.sin_rotations.x * rotated_origin.z;
    rotated_origin.z =
        rect.sin_rotations.x * rotated_origin.y + rect.cos_rotations.x * rotated_origin.z;
    rotated_direction.y =
        rect.cos_rotations.x * rotated_direction.y - rect.sin_rotations.x * rotated_direction.z;
    rotated_direction.z =
        rect.sin_rotations.x * rotated_direction.y + rect.cos_rotations.x * rotated_direction.z;

    // rotate around y
    rotated_origin.x =
        rect.cos_rotations.y * rotated_origin.x - rect.sin_rotations.y * rotated_origin.z;
    rotated_origin.z =
        rect.sin_rotations.y * rotated_origin.x + rect.cos_rotations.y * rotated_origin.z;
    rotated_direction.x =
        rect.cos_rotations.y * rotated_direction.x - rect.sin_rotations.y * rotated_direction.z;
    rotated_direction.z =
        rect.sin_rotations.y * rotated_direction.x + rect.cos_rotations.y * rotated_direction.z;

    // rotate around z
    rotated_origin.x =
        rect.cos_rotations.z * rotated_origin.x - rect.sin_rotations.z * rotated_origin.y;
    rotated_origin.y =
        rect.sin_rotations.z * rotated_origin.x + rect.cos_rotations.z * rotated_origin.y;
    rotated_direction.x =
        rect.cos_rotations.z * rotated_direction.x - rect.sin_rotations.z * rotated_direction.y;
    rotated_direction.y =
        rect.sin_rotations.z * rotated_direction.x + rect.cos_rotations.z * rotated_direction.y;

    rotated_origin += center_point;

    let rotated_ray = Ray::new(rotated_origin, rotated_direction, 0.0);

    let mut intersection = match aarect_intersection(&rect.aarect, &rotated_ray) {
        Some(int) => int,
        None => return None,
    };

    intersection.hit.point -= center_point;

    // inverse rotate around x
    intersection.hit.point.y = rect.cos_rotations.x * intersection.hit.point.y
        + rect.sin_rotations.x * intersection.hit.point.z;
    intersection.hit.point.z = -rect.sin_rotations.x * intersection.hit.point.y
        + rect.cos_rotations.x * intersection.hit.point.z;
    intersection.hit.normal.y = rect.cos_rotations.x * intersection.hit.normal.y
        + rect.sin_rotations.x * intersection.hit.normal.z;
    intersection.hit.normal.z = -rect.sin_rotations.x * intersection.hit.normal.y
        + rect.cos_rotations.x * intersection.hit.normal.z;

    // inverse rotate around y
    intersection.hit.point.x = rect.cos_rotations.y * intersection.hit.point.x
        + rect.sin_rotations.y * intersection.hit.point.z;
    intersection.hit.point.z = -rect.sin_rotations.y * intersection.hit.point.x
        + rect.cos_rotations.y * intersection.hit.point.z;
    intersection.hit.normal.x = rect.cos_rotations.y * intersection.hit.normal.x
        + rect.sin_rotations.y * intersection.hit.normal.z;
    intersection.hit.normal.z = -rect.sin_rotations.y * intersection.hit.normal.x
        + rect.cos_rotations.y * intersection.hit.normal.z;

    // inverse rotate around z
    intersection.hit.point.x = rect.cos_rotations.z * intersection.hit.point.x
        + rect.sin_rotations.z * intersection.hit.point.y;
    intersection.hit.point.y = -rect.sin_rotations.z * intersection.hit.point.x
        + rect.cos_rotations.z * intersection.hit.point.y;
    intersection.hit.normal.x = rect.cos_rotations.z * intersection.hit.normal.x
        + rect.sin_rotations.z * intersection.hit.normal.y;
    intersection.hit.normal.y = -rect.sin_rotations.z * intersection.hit.normal.x
        + rect.cos_rotations.z * intersection.hit.normal.y;

    intersection.hit.point += center_point;

    Some(intersection)
}
