use ultraviolet::vec::Vec3;

pub struct Camera {
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub aspect_ratio: f32,
    pub origin: Vec3,
    pub vertical: Vec3,
    pub horizontal: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub lower_left: Vec3,
    pub lens_radius: f32,
}

impl Camera {
    pub fn new(
        origin: Vec3,
        lookat: Vec3,
        vup: Vec3,
        fov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_dist: f32,
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
        }
    }
}
