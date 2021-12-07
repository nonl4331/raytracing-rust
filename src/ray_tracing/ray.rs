use crate::acceleration::bvh::Bvh;

use crate::ray_tracing::{
    intersection::{Hit, PrimitiveTrait},
    material::{Material, MaterialTrait},
    primitives::Axis,
    sky::Sky,
};
use crate::utility::{math::Float, vec::Vec3};
use std::sync::Arc;

pub type Colour = Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub d_inverse: Vec3,
    pub shear: Vec3,
    pub time: Float,
}

const MAX_DEPTH: u32 = 50;

impl Ray {
    pub fn new(origin: Vec3, mut direction: Vec3, time: Float) -> Self {
        direction.normalise();

        let max_axis = Axis::get_max_abs_axis(&direction);
        let mut swaped_dir = direction;
        Axis::swap_z(&mut swaped_dir, &max_axis);
        let shear_x = -swaped_dir.x / swaped_dir.z;
        let shear_y = -swaped_dir.y / swaped_dir.z;
        let shear_z = 1.0 / swaped_dir.z;

        Ray {
            origin,
            direction,
            d_inverse: Vec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
            shear: Vec3::new(shear_x, shear_y, shear_z),
            time,
        }
    }

    pub fn at(&self, t: Float) -> Vec3 {
        self.origin + self.direction * t
    }

    fn check_hit<P: PrimitiveTrait>(
        &mut self,
        bvh: &Arc<Bvh>,
        primitives: &Arc<Vec<P>>,
    ) -> Option<(Hit, Arc<Material>)> {
        let offset_lens = bvh.get_intersection_candidates(self);

        let mut hit: Option<(Hit, Arc<Material>)> = None;

        for offset_len in offset_lens {
            let offset = offset_len.0;
            let len = offset_len.1;
            for object in &primitives[offset..offset + len] {
                // check for hit
                if let Some(current_hit) = object.get_int(self) {
                    // make sure ray is going forwards
                    if current_hit.0.t > 0.0 {
                        // check if hit already exists
                        if let Some(last_hit) = &hit {
                            // check if t value is close to 0 than previous hit
                            if current_hit.0.t < last_hit.0.t {
                                hit = Some(current_hit);
                            }
                            continue;
                        }

                        // if hit doesn't exist set current hit to hit
                        hit = Some(current_hit);
                    }
                }
            }
        }
        hit
    }

    pub fn get_colour<P: PrimitiveTrait>(
        ray: &mut Ray,
        sky: Arc<Sky>,
        bvh: Arc<Bvh>,
        primitives: Arc<Vec<P>>,
    ) -> (Colour, u64) {
        let mut colour = Colour::one();
        let mut depth = 0;
        let mut ray_count = 0;

        // stop generating new bounce rays after MAX_DEPTH
        while depth < MAX_DEPTH {
            // check for intersection with any of the objects in the scene
            let hit = ray.check_hit(&bvh, &primitives);

            ray_count += 1;

            if hit.is_some() {
                let (hit, mat) = hit.unwrap();

                let (colour_multiplier, exit) = mat.scatter_ray(ray, &hit);

                colour *= colour_multiplier;

                if exit {
                    return (colour, ray_count);
                }
                depth += 1;
            } else {
                return (colour * sky.get_colour(ray), ray_count);
            }
        }
        (colour, ray_count)
    }
}
