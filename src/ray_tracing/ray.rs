use crate::acceleration::bvh::Bvh;

use crate::ray_tracing::{
	intersection::{Hit, Primitive, SurfaceIntersection},
	material::Scatter,
	primitives::Axis,
	sky::Sky,
};
use crate::utility::math::power_heuristic;
use crate::utility::{
	math::{random_float, Float},
	vec::Vec3,
};

const RUSSIAN_ROULETTE_THRESHOLD: u32 = 3;

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

		let light_t = match bvh.primitives[light_index].get_int(&self) {
			Some(hit) => {
				if hit.hit.t > 0.0 {
					hit.hit.t
				} else {
					return None;
				}
			}
			None => return None,
		};

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
					if current_hit.hit.t > 0.0 && current_hit.hit.t < light_t {
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
	) -> (Vec3, Option<Vec3>, Vec3) {
		let light = &bvh.primitives[light_index];
		let (light_point, dir, _normal) = light.sample_visible_from_point(hit.point);

		let ray = Ray::new(hit.point, dir, 0.0);

		let li = match ray.get_light_int(light_index, bvh) {
			Some(int) => Some(int.material.get_emission(hit)),
			None => return (Vec3::zero(), None, Vec3::zero()),
		};

		(dir, li, light_point)
	}

	fn get_light_contribution<P: Primitive<M>, M: Scatter>(
		old_dir: Vec3,
		hit: &Hit,
		surface_intersection: &SurfaceIntersection<M>,
		bvh: &Bvh<P, M>,
	) -> Vec3 {
		let mut direct_lighting = Vec3::zero();

		let mat = &surface_intersection.material;
		let light_obj = &bvh.primitives[bvh.lights[0]];

		// sample light
		let (light_dir, light_colour, light_point) = Ray::sample_light(&hit, bvh.lights[0], bvh);

		let pdf_light = light_obj.scattering_pdf(&hit, light_dir, light_point);
		if !(pdf_light == 0.0 || light_colour.is_none()) {
			let light_colour = light_colour.unwrap();

			let scattering_pdf = mat.scattering_pdf(hit.point, light_dir, hit.normal);

			let weight = power_heuristic(pdf_light, scattering_pdf);

			if light_colour != Vec3::zero() {
				direct_lighting += light_colour
					* mat.scattering_albedo(&hit, old_dir, light_dir)
					* scattering_pdf * weight
					/ pdf_light;
			}
		}

		// sample bxdf
		let mut ray = Ray::new(surface_intersection.hit.point, old_dir, 0.0);
		mat.scatter_ray(&mut ray, &surface_intersection.hit);

		// check light intersection & get colour
		let (int_point, li) = match ray.get_light_int(bvh.lights[0], bvh) {
			Some(int) => (int.hit.point, int.material.get_emission(hit)),
			None => return direct_lighting,
		};

		// calculate pdfs
		let scattering_pdf = mat.scattering_pdf(hit.point, ray.direction, hit.normal);
		if scattering_pdf != 0.0 {
			let light_pdf = light_obj.scattering_pdf(&hit, ray.direction, int_point);
			if light_pdf != 0.0 {
				let weight = power_heuristic(scattering_pdf, light_pdf);

				direct_lighting +=
					li * mat.scattering_albedo(&hit, old_dir, ray.direction) * weight;
			}
		}

		direct_lighting
	}

	pub fn get_colour<P: Primitive<M>, M: Scatter>(
		ray: &mut Ray,
		sky: &Sky,
		bvh: &Bvh<P, M>,
	) -> (Colour, u64) {
		let (mut throughput, mut output) = (Colour::one(), Colour::zero());
		let mut depth = 0;
		let mut ray_count = 0;

		while depth < MAX_DEPTH {
			let hit_info = ray.check_hit(&bvh);

			ray_count += 1;

			if let Some((surface_intersection, _index)) = hit_info {
				let (hit, mat) = (&surface_intersection.hit, &surface_intersection.material);

				let old_dir = ray.direction;

				let emission = mat.get_emission(&hit);

				let exit = mat.scatter_ray(ray, &hit);

				if depth == 0 {
					output += throughput * emission;
				}

				if exit {
					break;
				}

				//add light contribution
				ray_count += 1;
				output += throughput
					* Ray::get_light_contribution(old_dir, &hit, &surface_intersection, bvh);

				// add bxdf contribution
				throughput *= mat.scattering_albedo(&hit, old_dir, ray.direction);

				// russian roulette
				if depth > RUSSIAN_ROULETTE_THRESHOLD {
					let p = throughput.component_max();
					if random_float() > p {
						break;
					}
					throughput /= p;
				}

				depth += 1;
			} else {
				output += throughput * sky.get_colour(ray);
				break;
			}
		}
		if output.contains_nan() {
			return (Vec3::zero(), ray_count);
		}
		(output, ray_count)
	}
}
