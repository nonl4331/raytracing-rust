use crate::image::scene::HittablesType;
use crate::image::sky::Sky;
use crate::image::tracing::Hit;

use crate::image::material::MaterialTrait;
use crate::image::tracing::HittableTrait;
use ultraviolet::vec::DVec3;

pub type Color = DVec3;

pub struct Ray {
    pub origin: DVec3,
    pub direction: DVec3,
    pub hittables: HittablesType,
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
    ) -> Self {
        direction.normalize();
        Ray {
            origin,
            direction,
            hittables,
            time,
            sky,
            hit: None,
        }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + self.direction * t
    }

    fn check_hit(&mut self) {
        for object in self.hittables.iter() {
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

    pub fn get_color(&mut self, depth: u32) -> Color {
        // stop generating new bounce rays after MAX_DEPTH
        if depth >= MAX_DEPTH {
            return Color::one();
        }

        // check for intersection with any of the objects in scene
        self.check_hit();

        if let Some(hit) = &self.hit {
            return hit.material.color() * hit.material.scatter_ray(self, hit, depth);
        }

        self.sky.get_color(&self)
    }
}
