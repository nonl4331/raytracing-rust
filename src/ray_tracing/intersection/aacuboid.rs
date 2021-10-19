use crate::ray_tracing::{
    primitives::AACuboid,
    ray::Ray,
    tracing::{Hit, Intersection},
};

const AACUBOID_INTERSECTION: AACuboidIntersection = AACuboidIntersection::One;

enum AACuboidIntersection {
    One,
}

pub fn aacuboid_intersection(aacuboid: &AACuboid, ray: &Ray) -> Option<Hit> {
    match AACUBOID_INTERSECTION {
        AACuboidIntersection::One => aacuboid_intersection_one(aacuboid, ray),
    }
}

fn aacuboid_intersection_one(aacuboid: &AACuboid, ray: &Ray) -> Option<Hit> {
    let mut hit: Option<Hit> = None;
    for side in aacuboid.rects.iter() {
        if let Some(current_hit) = side.get_int(ray) {
            // make sure ray is going forwards
            if current_hit.t > 0.0 {
                // check if hit already exists
                if hit.is_some() {
                    // check if t value is close to 0 than previous hit
                    if current_hit.t < hit.as_ref().unwrap().t {
                        hit = Some(current_hit);
                    }
                    continue;
                }

                // if hit doesn't exist set current hit to hit
                hit = Some(current_hit);
            }
        }
    }
    hit
}
