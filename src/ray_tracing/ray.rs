use crate::acceleration::bvh::Bvh;
use crate::math::Float;

use crate::image::scene::PrimitivesType;

use crate::ray_tracing::{
    material::MaterialTrait,
    primitives::Axis,
    sky::Sky,
    tracing::{Hit, PrimitiveTrait},
};

use std::sync::Arc;

use ultraviolet::vec::Vec3;

pub type Colour = Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub d_inverse: Vec3,
    pub shear: Vec3,
    pub hit: Option<Hit>,
    pub time: Float,
}

const MAX_DEPTH: u32 = 50;

impl Ray {
    pub fn new(origin: Vec3, mut direction: Vec3, time: Float) -> Self {
        direction.normalize();

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
            hit: None,
        }
    }

    pub fn at(&self, t: Float) -> Vec3 {
        self.origin + self.direction * t
    }

    fn check_hit(&mut self, bvh: &Arc<Bvh>, primitives: &PrimitivesType) {
        let offset_lens = bvh.get_intersection_candidates(self);

        for offset_len in offset_lens {
            let offset = offset_len.0;
            let len = offset_len.1;
            for object in &primitives[offset..offset + len] {
                // check for hit
                if let Some(current_hit) = object.get_int(self, object.is_brdf()) {
                    // make sure ray is going forwards
                    if current_hit.t > 0.0 {
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
    }

    pub fn get_colour(
        ray: &mut Ray,
        sky: Arc<Sky>,
        bvh: Arc<Bvh>,
        primitives: PrimitivesType,
    ) -> (Colour, u64) {
        let mut colour = Colour::one();
        let mut depth = 0;
        let mut ray_count = 0;

        // stop generating new bounce rays after MAX_DEPTH
        while depth < MAX_DEPTH {
            // check for intersection with any of the objects in the scene
            ray.check_hit(&bvh, &primitives);

            ray_count += 1;

            if ray.hit.is_some() {
                let hit = ray.hit.take().unwrap();

                // scatter_ray can only change ray direction, multiply colour by a factor or exit
                let (multiplier, exit) = hit.material.scatter_ray(ray, &hit);

                colour *= hit.material.colour(hit.uv, hit.point) * multiplier;

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
