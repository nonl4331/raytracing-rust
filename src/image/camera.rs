use crate::image::sky::Sky;

use ultraviolet::vec::DVec3;

#[derive(Clone)]
pub struct Camera {
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub aspect_ratio: f64,
    pub origin: DVec3,
    pub vertical: DVec3,
    pub horizontal: DVec3,
    pub u: DVec3,
    pub v: DVec3,
    pub lower_left: DVec3,
    pub lens_radius: f64,
    pub sky: Sky,
}

impl Camera {
    pub fn new(
        origin: DVec3,
        lookat: DVec3,
        vup: DVec3,
        fov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
        sky: Sky,
    ) -> Self {
        let viewport_width = 2.0 * (fov.to_radians() / 2.0).tan();
        let viewport_height = viewport_width / aspect_ratio;

        let w = (origin - lookat).normalized();
        let u = w.cross(vup).normalized();
        let v = u.cross(w);

        let horizontal = focus_dist * u * viewport_width;
        let vertical = focus_dist * v * viewport_height;

        let lower_left = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        Camera {
            viewport_width,
            viewport_height,
            aspect_ratio,
            origin,
            vertical,
            horizontal,
            u,
            v,
            lower_left,
            lens_radius: aperture / 2.0,
            sky,
        }
    }
}
