use crate::{power_heuristic, AccelerationStructure, Float, Hit, NoHit, Primitive, Scatter, Vec3};
use rand::{prelude::SliceRandom, rngs::SmallRng, thread_rng, Rng, SeedableRng};

const RUSSIAN_ROULETTE_THRESHOLD: u32 = 3;
const MAX_DEPTH: u32 = 50;

pub type Colour = Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
	pub origin: Vec3,
	pub direction: Vec3,
	pub d_inverse: Vec3,
	pub shear: Vec3,
	pub time: Float,
}

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

	fn sample_lights_test<A: AccelerationStructure<P, M>, P: Primitive<M>, M: Scatter, S: NoHit>(
		bvh: &A,
		hit: &Hit,
		_sky: &S,
		_mat: &M,
		_wo: Vec3,
	) -> Option<(Vec3, Vec3, Float)> {
		//l_pos, le, l_pdf
		let light_index = match bvh
			.get_samplable()
			.choose(&mut SmallRng::from_rng(thread_rng()).unwrap())
		{
			Some(&index) => index,
			None => return None,
		};

		let samplable = bvh.get_object(light_index).unwrap();

		let sampled_wi = samplable.sample_visible_from_point(hit.point);

		if let Some(sampled_si) = bvh.check_hit_index(
			&Ray::new(hit.point + 0.0001 * hit.normal, sampled_wi, 0.0),
			light_index,
		) {
			let sampled_hit = &sampled_si.hit;

			let sampled_pdf = samplable.scattering_pdf(hit.point, sampled_wi, sampled_hit);

			if sampled_pdf > 0.0 {
				let li = sampled_si.material.get_emission(sampled_hit, sampled_wi);

				let num_lights = bvh.get_samplable().len() as Float;

				Some((sampled_si.hit.point, li, sampled_pdf / num_lights))
			} else {
				None
			}
		} else {
			None
		}
	}

	fn sample_material_test<
		A: AccelerationStructure<P, M>,
		P: Primitive<M>,
		M: Scatter,
		S: NoHit,
	>(
		_bvh: &A,
		hit: &Hit,
		_sky: &S,
		mat: &M,
		ray: &mut Ray,
	) -> Option<(Vec3, Float)> {
		let wo = ray.direction;
		if mat.scatter_ray(ray, hit) {
			None
		} else {
			Some((ray.direction, mat.scattering_pdf(hit, wo, ray.direction)))
		}
	}

	pub fn get_colour<A: AccelerationStructure<P, M>, P: Primitive<M>, M: Scatter, S: NoHit>(
		ray: &mut Ray,
		sky: &S,
		bvh: &A,
	) -> (Colour, u64) {
		let (mut throughput, mut output) = (Colour::one(), Colour::zero());
		let mut wo;
		let mut hit;
		let mut mat;
		let mut ray_count = 0;

		// depth 0
		if let Some((surface_intersection, _index)) = bvh.check_hit(ray) {
			(hit, mat) = (surface_intersection.hit, surface_intersection.material);

			wo = ray.direction;

			let emission = mat.get_emission(&hit, wo);

			let exit = mat.scatter_ray(&mut ray.clone(), &hit);

			output += emission;

			if exit {
				return (output, ray_count);
			}
		} else {
			output += sky.get_colour(ray);
			return (output, ray_count);
		}

		let mut depth = 1;
		while depth < MAX_DEPTH {
			if !mat.is_delta() {
				// light sampling
				if let Some((l_pos, le, l_pdf)) = Ray::sample_lights_test(bvh, &hit, sky, &mat, wo)
				{
					let l_wi = (l_pos - hit.point).normalised();
					let m_pdf = mat.scattering_pdf(&hit, wo, l_wi);
					let mis_weight = power_heuristic(l_pdf, m_pdf);

					output += throughput * mat.eval(&hit, wo, l_wi) * mis_weight * le / l_pdf;
				}
				ray_count += 1;
			}

			// material sample and bounce
			let (m_wi, m_pdf) = match Ray::sample_material_test(bvh, &hit, sky, &mat, ray) {
				Some((m_wi, m_pdf)) => (m_wi, m_pdf),
				None => break,
			};

			throughput *= if mat.is_delta() {
				mat.eval(&hit, wo, m_wi)
			} else {
				mat.eval_over_scattering_pdf(&hit, wo, m_wi)
			};

			let (surface_intersection, index) = match bvh.check_hit(ray) {
				Some((surface_intersection, index)) => (surface_intersection, index),
				None => {
					output += throughput * sky.get_colour(ray);
					break;
				} // no sky sampling support yet
			};

			if surface_intersection.material.get_emission(&hit, wo) != Vec3::zero() {
				let le = surface_intersection.material.get_emission(&hit, wo);

				if mat.is_delta() {
					output += throughput * le;
				} else {
					let light_pdf = if bvh.get_samplable().contains(&index) {
						bvh.get_object(index).unwrap().scattering_pdf(
							hit.point,
							m_wi,
							&surface_intersection.hit,
						) / bvh.get_samplable().len() as Float
					} else {
						0.0
					};
					let mis_weight = power_heuristic(m_pdf, light_pdf);

					output += throughput * mis_weight * le;
				}

				if surface_intersection.material.is_light() {
					break;
				}
			}

			hit = surface_intersection.hit;
			mat = surface_intersection.material;
			wo = m_wi;

			depth += 1;
		}
		if output.contains_nan() || !output.is_finite() {
			return (Vec3::zero(), ray_count);
		}
		(output, ray_count)
	}

	pub fn get_colour_naive<
		A: AccelerationStructure<P, M>,
		P: Primitive<M>,
		M: Scatter,
		S: NoHit,
	>(
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
					output += throughput * emission;
					break;
				}

				if !mat.is_delta() {
					throughput *= mat.eval_over_scattering_pdf(hit, wo, ray.direction);
				} else {
					throughput *= mat.eval(hit, wo, ray.direction);
				}

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
