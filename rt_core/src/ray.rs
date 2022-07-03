use crate::{
	power_heuristic, AccelerationStructure, Float, Hit, NoHit, Primitive, Scatter,
	SurfaceIntersection, Vec3,
};
use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};

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

		let max_axis =
			if direction.x.abs() > direction.y.abs() && direction.x.abs() > direction.z.abs() {
				0
			} else if direction.y.abs() > direction.z.abs() {
				1
			} else {
				2
			};

		let mut swaped_dir = direction;
		match max_axis {
			0 => {
				std::mem::swap(&mut swaped_dir.x, &mut swaped_dir.z);
			}
			1 => {
				std::mem::swap(&mut swaped_dir.x, &mut swaped_dir.z);
			}
			_ => {}
		}
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

	fn get_light_contribution<A: AccelerationStructure<P, M>, P: Primitive<M>, M: Scatter>(
		wo: Vec3,
		hit: &Hit,
		surface_intersection: &SurfaceIntersection<M>,
		bvh: &A,
	) -> Vec3 {
		let mut direct_lighting = Vec3::zero();

		let mat = &surface_intersection.material;

		if mat.is_delta() {
			return direct_lighting;
		}

		let lights = bvh.get_samplable();
		let num_lights = lights.len();
		let light_index = if num_lights == 0 {
			return direct_lighting;
		} else {
			let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
			lights[rng.gen_range(0..num_lights)]
		};

		let light_obj = bvh.get_object(light_index).unwrap();

		// sample light
		let (wi, light_colour, light_point) = bvh.sample_object(hit, light_index);

		let pdf_light = light_obj.scattering_pdf(hit, wi, light_point);
		if let Some(light_colour) = light_colour {
			if pdf_light != 0.0 {
				let scattering_pdf = mat.scattering_pdf(hit.point, wi, hit.normal);

				let weight = power_heuristic(pdf_light, scattering_pdf);

				let f = mat.eval(hit, wo, wi);

				if light_colour != Vec3::zero() {
					direct_lighting += light_colour * f * weight / pdf_light;
				}
			}
		}

		// sample bxdf
		let mut ray = Ray::new(surface_intersection.hit.point, wo, 0.0);
		mat.scatter_ray(&mut ray, &surface_intersection.hit);

		// check light intersection & get colour
		let (int_point, li) = match bvh.check_hit_index(&ray, light_index) {
			Some(int) => (int.hit.point, int.material.get_emission(hit, wo)),
			None => return direct_lighting,
		};

		// calculate pdfs
		let scattering_pdf = mat.scattering_pdf(hit.point, ray.direction, hit.normal);
		if scattering_pdf != 0.0 {
			let light_pdf = light_obj.scattering_pdf(hit, ray.direction, int_point);
			if light_pdf != 0.0 {
				let weight = power_heuristic(scattering_pdf, light_pdf);

				direct_lighting += li * weight * mat.eval(hit, wo, ray.direction) / scattering_pdf;
			}
		}

		direct_lighting
	}

	pub fn get_colour<A: AccelerationStructure<P, M>, P: Primitive<M>, M: Scatter, S: NoHit>(
		ray: &mut Ray,
		sky: &S,
		bvh: &A,
	) -> (Colour, u64) {
		let (mut throughput, mut output) = (Colour::one(), Colour::zero());
		let mut depth = 0;
		let mut ray_count = 0;

		while depth < MAX_DEPTH {
			let hit_info = bvh.check_hit(ray);

			ray_count += 1;

			if let Some((surface_intersection, _index)) = hit_info {
				let (hit, mat) = (&surface_intersection.hit, &surface_intersection.material);

				let wo = ray.direction;

				let emission = mat.get_emission(hit, wo);

				let exit = mat.scatter_ray(ray, hit);

				if depth == 0 {
					output += throughput * emission;
				}

				if exit {
					break;
				}

				//add light contribution
				ray_count += 1;
				output +=
					throughput * Ray::get_light_contribution(wo, hit, &surface_intersection, bvh);

				// add bxdf contribution
				if !mat.is_delta() {
					throughput *= mat.eval(hit, wo, ray.direction)
						/ mat.scattering_pdf(wo, ray.direction, hit.normal);
				} else {
					throughput *= mat.eval(hit, wo, ray.direction);
				}

				// russian roulette
				if depth > RUSSIAN_ROULETTE_THRESHOLD {
					let p = throughput.component_max();
					let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
					if rng.gen::<Float>() > p {
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
		if output.contains_nan() || !output.is_finite() {
			return (Vec3::zero(), ray_count);
		}
		(output, ray_count)
	}
}
