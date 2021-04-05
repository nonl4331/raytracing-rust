use crate::image::ray::Ray;
use crate::image::scene::HittablesType;

use ultraviolet::vec::DVec3;

pub struct Camera {
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub aspect_ratio: f64,
    pub focal_length: f64,
    pub origin: DVec3,
    pub vertical: DVec3,
    pub horizontal: DVec3,
    pub lower_left: DVec3,
    pub hittables: HittablesType,
}

impl Camera {
    pub fn new(
        viewport_height: f64,
        aspect_ratio: f64,
        focal_length: f64,
        hittables: HittablesType,
    ) -> Self {
        let viewport_width = aspect_ratio * viewport_height;
        let vertical = DVec3::new(0.0, viewport_height, 0.0);
        let horizontal = DVec3::new(viewport_width, 0.0, 0.0);
        let lower_left = DVec3::new(0.0, 0.0, focal_length) - horizontal / 2.0 - vertical / 2.0;
        Camera {
            viewport_width,
            viewport_height,
            aspect_ratio,
            focal_length,
            origin: DVec3::zero(),
            vertical,
            horizontal,
            lower_left,
            hittables,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left
                + self.horizontal * DVec3::new(u, 0.0, 0.0)
                + self.vertical * DVec3::new(0.0, v, 0.0)
                - self.origin,
            self.hittables.clone(),
        )
    }
}
