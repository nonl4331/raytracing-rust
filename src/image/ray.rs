use crate::image::scene::HittablesType;
use crate::image::tracing::Hit;

use crate::image::hittables::MaterialTrait;
use crate::image::tracing::HittableTrait;
use ultraviolet::vec::DVec3;

pub type Color = DVec3;

pub struct Ray {
    pub origin: DVec3,
    pub direction: DVec3,
    pub hittables: HittablesType,
    pub hit: Option<Hit>,
}

const MAX_DEPTH: u32 = 50;

impl Ray {
    pub fn new(origin: DVec3, mut direction: DVec3, hittables: HittablesType) -> Self {
        direction.normalize();
        Ray {
            origin,
            direction,
            hittables,
            hit: None,
        }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + self.direction * t
    }

    fn check_hit(&mut self) {
        let hittables = self.hittables.read().unwrap();
        for object in &*hittables {
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

        // if no intersection return "sky" color (which is lerp from white to blue)
        let t: f64 = 0.5 * (self.direction.y + 1.0);
        (1.0 - t) * Color::one() + t * Color::new(0.5, 0.7, 1.0)
    }
}
