use crate::image::scene::{Diffuse, Material, SCENE};
use crate::image::tracing::{Hit, Hittable};
use ultraviolet::vec::Vec3;

pub type Color = Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub hit: Option<Hit>,
}

const MAX_DEPTH: u32 = 50;

impl Ray {
    pub fn new(origin: Vec3, mut direction: Vec3) -> Self {
        direction.normalize();
        Ray {
            origin,
            direction,
            hit: None,
        }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    fn check_hit(&mut self, object: &dyn Hittable) {
        // check for hit
        if let Some(current_hit) = object.get_int(&self) {
            // check current hit in infront of camera
            if current_hit.t > 0.0 && current_hit.out {
                // check if hit already exists
                if let Some(last_hit) = &self.hit {
                    // check if t value is close to 0 than previous hit
                    if current_hit.t < last_hit.t {
                        self.hit = Some(current_hit);
                    }
                    return;
                }

                // if hit doesn't exist set current hit to hit
                self.hit = Some(current_hit);
            }
        }
    }

    pub fn get_color(&mut self, depth: u32) -> Color {
        // stop generating new bounce rays after MAX_DEPTH
        if depth >= MAX_DEPTH {
            return Color::new(0.0, 0.0, 0.0);
        }

        // check for intersection with any of the objects in scene
        for &object in &SCENE {
            self.check_hit(object);
        }

        let diffuse_mat = Diffuse { absorption: 0.5 };

        if let Some(hit) = &self.hit {
            return diffuse_mat.scatter_ray(hit, depth);
        }

        // if no intersection return "sky" color (which is lerp from white to blue)
        let t: f32 = 0.5 * (self.direction.y + 1.0);
        (1.0 - t) * Color::one() + t * Color::new(0.5, 0.7, 1.0)
    }
}
