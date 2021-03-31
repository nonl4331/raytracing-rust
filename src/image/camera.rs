use crate::image::ray::Ray;
use ultraviolet::vec::Vec3;

pub struct Camera {
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub aspect_ratio: f32,
    pub focal_length: f32,
    pub origin: Vec3,
    pub vertical: Vec3,
    pub horizontal: Vec3,
    pub lower_left: Vec3,
    pub pixel_samples: u32,
}

impl Camera {
    pub fn new(
        viewport_height: f32,
        aspect_ratio: f32,
        focal_length: f32,
        origin: Vec3,
        pixel_samples: u32,
    ) -> Self {
        let viewport_width = aspect_ratio * viewport_height;
        let vertical = Vec3::new(0.0, viewport_height, 0.0);
        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        let lower_left =
            origin - horizontal / 2.0 - vertical / 2.0 + Vec3::new(0.0, 0.0, focal_length);
        Camera {
            viewport_width,
            viewport_height,
            aspect_ratio,
            focal_length,
            origin,
            vertical,
            horizontal,
            lower_left,
            pixel_samples,
        }
    }

    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left
                + self.horizontal * Vec3::new(u, 0.0, 0.0)
                + self.vertical * Vec3::new(0.0, v, 0.0)
                - self.origin,
        )
    }
}
