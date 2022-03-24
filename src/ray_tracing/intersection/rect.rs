use crate::ray_tracing::{
	intersection::{aarect::aarect_intersection, SurfaceIntersection},
	material::Scatter,
	primitives::{Axis, Rect},
	ray::Ray,
};
use crate::utility::{math::rotate_around_point, vec::Vec3};

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
	let center_point = match rect.rotation_point {
		Some(val) => val,
		None => {
			(Axis::point_from_2d(&rect.aarect.max, &rect.aarect.axis, rect.aarect.k)
				+ Axis::point_from_2d(&rect.aarect.min, &rect.aarect.axis, rect.aarect.k))
				/ 2.0
		}
	};

	let mut rotated_origin = ray.origin;
	let mut rotated_direction = ray.direction;

	rotate_around_point(
		&mut rotated_origin,
		center_point,
		rect.sin_rotations,
		rect.cos_rotations,
	);
	rotate_around_point(
		&mut rotated_direction,
		Vec3::zero(),
		rect.sin_rotations,
		rect.cos_rotations,
	);

	let rotated_ray = Ray::new(rotated_origin, rotated_direction, 0.0);

	let mut intersection = match aarect_intersection(&rect.aarect, &rotated_ray) {
		Some(int) => int,
		None => return None,
	};

	rotate_around_point(
		&mut intersection.hit.point,
		center_point,
		-rect.sin_rotations,
		rect.cos_rotations,
	);
	rotate_around_point(
		&mut intersection.hit.normal,
		Vec3::zero(),
		-rect.sin_rotations,
		rect.cos_rotations,
	);

	intersection.hit.normal.normalise();

	Some(intersection)
}
