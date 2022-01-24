use crate::acceleration::bvh::Bvh;

use crate::ray_tracing::{
    intersection::{Hit, Primitive, SurfaceIntersection},
    material::Scatter,
    primitives::Axis,
    sky::Sky,
};
use crate::utility::{math::Float, vec::Vec3};

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

    fn check_hit<P: Primitive<M>, M: Scatter>(
        &mut self,
        bvh: &Bvh<P, M>,
    ) -> Option<(SurfaceIntersection<M>, usize)> {
        let offset_lens = bvh.get_intersection_candidates(self);

        let mut hit: Option<(SurfaceIntersection<M>, usize)> = None;

        for offset_len in offset_lens {
            let offset = offset_len.0;
            let len = offset_len.1;
            for index in offset..(offset + len) {
                let object = &bvh.primitives[index];
                // check for hit
                if let Some(current_hit) = object.get_int(self) {
                    // make sure ray is going forwards
                    if current_hit.hit.t > 0.0 {
                        // check if hit already exists
                        if let Some((last_hit, _)) = &hit {
                            // check if t value is close to 0 than previous hit
                            if current_hit.hit.t < last_hit.hit.t {
                                hit = Some((current_hit, index));
                            }
                            continue;
                        }

                        // if hit doesn't exist set current hit to hit
                        hit = Some((current_hit, index));
                    }
                }
            }
        }
        hit
    }

    pub fn get_light_int<P: Primitive<M>, M: Scatter>(
        &self,
        light_index: usize,
        bvh: &Bvh<P, M>,
    ) -> Option<SurfaceIntersection<M>> {
        let light = &bvh.primitives[light_index];

        let offset_lens = bvh.get_intersection_candidates(&self);

        // check if object blocking
        for offset_len in offset_lens {
            let offset = offset_len.0;
            let len = offset_len.1;
            for index in offset..(offset + len) {
                if index == light_index {
                    continue;
                }
                let tobject = &bvh.primitives[index];
                // check for hit
                if let Some(current_hit) = tobject.get_int(&self) {
                    // make sure ray is going forwards
                    if current_hit.hit.t > 0.0 {
                        return None;
                    }
                }
            }
        }
        light.get_int(&self)
    }

    pub fn sample_light<P: Primitive<M>, M: Scatter>(
        hit: &Hit,
        light_index: usize,
        bvh: &Bvh<P, M>,
    ) -> (Vec3, Vec3, Vec3) {
        let light = &bvh.primitives[light_index];
        let (light_point, dir, _normal) = light.sample_visible_from_point(hit.point);

        let ray = Ray::new(hit.point, dir, 0.0);

        let li = match ray.get_light_int(light_index, bvh) {
            Some(int) => int.material.get_emission(hit),
            None => return (Vec3::zero(), Vec3::zero(), Vec3::zero()),
        };

        (dir, li, light_point)
    }

    pub fn get_colour<P: Primitive<M>, M: Scatter>(
        ray: &mut Ray,
        sky: &Sky,
        bvh: &Bvh<P, M>,
    ) -> (Colour, u64) {
        let (mut bxdf_contrib, mut light_contrib) = (Colour::one(), Colour::zero());
        let mut depth = 0;
        let mut ray_count = 0;

        while depth < MAX_DEPTH {
            let hit_info = ray.check_hit(&bvh);

            ray_count += 1;

            if let Some((surface_intersection, _index)) = hit_info {
                let (hit, mat) = (surface_intersection.hit, surface_intersection.material);

                let old_dir = ray.direction;

                let emission = mat.get_emission(&hit);

                let (pdf_scattering, exit) = mat.scatter_ray(ray, &hit);

                if exit {
                    bxdf_contrib *= emission;
                    return (bxdf_contrib + light_contrib, ray_count);
                } else {
                    let pdf_light; // = 0.0;

                    light_contrib += bxdf_contrib * emission;

                    //add light contribution
                    let (light_dir, light_colour, light_point) =
                        Ray::sample_light(&hit, bvh.lights[0], bvh);
                    pdf_light =
                        bvh.primitives[bvh.lights[0]].scattering_pdf(&hit, light_dir, light_point);
                    if pdf_light > 0.0 && light_colour != Vec3::zero() && !exit {
                        let lc = light_colour
                            * mat.scattering_pdf(hit.point, light_dir, hit.normal)
                            * 0.0 // power_heuristic(pdf_light, pdf_scattering)
                            * light_dir.dot(hit.normal).abs()
                            / pdf_light;
                        light_contrib += bxdf_contrib * lc;
                    }

                    // add bxdf contribution
                    let bc = mat.scattering_albedo(&hit, old_dir, ray.direction)
                        * mat.scattering_pdf(hit.point, ray.direction, hit.normal)
                        * ray.direction.dot(hit.normal).abs()
                        * 1.0
                        / pdf_scattering;

                    assert_eq!(
                        mat.scattering_pdf(hit.point, ray.direction, hit.normal),
                        pdf_scattering
                    );

                    bxdf_contrib *= bc;
                }

                depth += 1;
            } else {
                return (
                    (bxdf_contrib + light_contrib) * sky.get_colour(ray),
                    ray_count,
                );
            }
        }
        (bxdf_contrib + light_contrib, ray_count)
    }
}
