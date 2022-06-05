use crate::ray_tracing::{
	intersection::{Intersect, SurfaceIntersection},
	material::Scatter,
	primitives::Cuboid,
	Ray,
};

const CUBOID_INTERSECTION: CuboidIntersection = CuboidIntersection::One;

enum CuboidIntersection {
	One,
}

pub fn cuboid_intersection<M: Scatter>(
	cuboid: &Cuboid<M>,
	ray: &Ray,
) -> Option<SurfaceIntersection<M>> {
	match CUBOID_INTERSECTION {
		CuboidIntersection::One => cuboid_intersection_one(cuboid, ray),
	}
}

fn cuboid_intersection_one<M: Scatter>(
	cuboid: &Cuboid<M>,
	ray: &Ray,
) -> Option<SurfaceIntersection<M>> {
	let mut hit: Option<SurfaceIntersection<M>> = None;
	for side in cuboid.rects.iter() {
		if let Some(current_hit) = side.get_int(ray) {
			if current_hit.hit.t > 0.0 {
				if hit.is_some() {
					if current_hit.hit.t < hit.as_ref().unwrap().hit.t {
						hit = Some(current_hit);
					}
					continue;
				}

				hit = Some(current_hit);
			}
		}
	}
	hit
}
