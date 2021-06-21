use crate::image::bvh::BVH;
use crate::image::scene::HittablesType;
use crate::image::sky::Sky;
use crate::image::tracing::Hit;
use std::sync::Arc;

use crate::image::material::MaterialTrait;
use crate::image::tracing::HittableTrait;
use ultraviolet::vec::DVec3;

pub type Colour = DVec3;

pub struct Ray {
    pub origin: DVec3,
    pub direction: DVec3,
    pub d_inverse: DVec3,
    pub hittables: HittablesType,
    pub bvh: Arc<BVH>,
    pub sky: Sky,
    pub hit: Option<Hit>,
    pub time: f64,
}

const MAX_DEPTH: u32 = 50;

impl Ray {
    pub fn new(
        origin: DVec3,
        mut direction: DVec3,
        time: f64,
        sky: Sky,
        hittables: HittablesType,
        bvh: Arc<BVH>,
    ) -> Self {
        direction.normalize();

        Ray {
            origin,
            direction,
            d_inverse: DVec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
            hittables,
            bvh,
            time,
            sky,
            hit: None,
        }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + self.direction * t
    }

    fn check_hit(&mut self) {
        let candidates = self.bvh.get_intersection_candidates(&self);

        for object_index in candidates {
            let object = &self.hittables[object_index as usize];

            // check for hit
            if let Some(current_hit) = object.get_int(&self) {
                // make sure ray is going forwards
                if current_hit.t > 0.001 {
                    // check if hit already exists
                    if let Some(last_hit) = &self.hit {
                        // check if t value is close to 0 than previous hit
                        if current_hit.t < last_hit.t {
                            self.hit = Some(current_hit);
                        }
                        continue;
                    }

                    // if hit doesn't exist set current hit to hit
                    self.hit = Some(current_hit);
                }
            }
        }
    }

    pub fn get_colour(&mut self, depth: u32) -> Colour {
        // stop generating new bounce rays after MAX_DEPTH
        if depth >= MAX_DEPTH {
            return Colour::one();
        }

        // check for intersection with any of the objects in scene
        self.check_hit();

        if let Some(hit) = &self.hit {
            return hit.material.colour(hit.uv, hit.point)
                * hit.material.scatter_ray(self, hit, depth);
        }

        self.sky.get_colour(&self)
    }
}
