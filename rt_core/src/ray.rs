use crate::{
	power_heuristic, AccelerationStructure, Float, Hit, NoHit, Primitive, Scatter,
	SurfaceIntersection, Vec3,
};
use rand::{prelude::SliceRandom, rngs::SmallRng, thread_rng, Rng, SeedableRng};

const RUSSIAN_ROULETTE_THRESHOLD: u32 = 3;
const MAX_DEPTH: u32 = 50;

pub type Colour = Vec3;

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

	fn sample_light<A: AccelerationStructure<P, M>, P: Primitive<M>, M: Scatter>(
		bvh: &A,
		hit: &Hit,
		mat: &M,
		wo: Vec3,
	) -> Vec3 {
		let light_index = match bvh
			.get_samplable()
			.choose(&mut SmallRng::from_rng(thread_rng()).unwrap())
		{
			Some(&index) => index,
			None => return Vec3::zero(),
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

				let f = mat.eval(hit, wo, sampled_wi);

				let num_lights = bvh.get_samplable().len() as Float;

				let scattering_pdf = mat.scattering_pdf(hit, wo, sampled_wi);

				let weight = power_heuristic(sampled_pdf, scattering_pdf);

				return li * f * num_lights * weight / sampled_pdf;
			}
		}
		Vec3::zero()
	}

	fn sample_light_mis<A: AccelerationStructure<P, M>, P: Primitive<M>, M: Scatter>(
		bvh: &A,
		hit: &Hit,
		mat: &M,
		wo: Vec3,
		index: usize,
		wi: Vec3,
		new_si: &SurfaceIntersection<M>,
	) -> Vec3 {
		let mut output = Vec3::zero();
		let samplable = bvh.get_object(index).unwrap();

		let sampled_wi = samplable.sample_visible_from_point(hit.point);

		if let Some(sampled_si) = bvh.check_hit_index(
			&Ray::new(hit.point + 0.0001 * hit.normal, sampled_wi, 0.0),
			index,
		) {
			let sampled_hit = &sampled_si.hit;

			let sampled_pdf = samplable.scattering_pdf(hit.point, sampled_wi, sampled_hit);

			if sampled_pdf > 0.0 {
				let li = new_si.material.get_emission(sampled_hit, sampled_wi);

				let f = mat.eval(hit, wo, sampled_wi);

				let scattering_pdf = mat.scattering_pdf(hit, wo, sampled_wi);

				let weight = power_heuristic(sampled_pdf, scattering_pdf);

				output += li * f * weight / sampled_pdf;
			}
		}

		let scattering_pdf = mat.scattering_pdf(hit, wo, wi);
		if scattering_pdf != 0.0 {
			let li = new_si.material.get_emission(&new_si.hit, wi);

			let f = mat.eval(hit, wo, wi);

			let sampling_pdf = samplable.scattering_pdf(hit.point, wi, &new_si.hit);

			let weight = power_heuristic(scattering_pdf, sampling_pdf);

			output += li * f * weight / scattering_pdf;
		}

		output
	}

	pub fn get_colour<A: AccelerationStructure<P, M>, P: Primitive<M>, M: Scatter, S: NoHit>(
		ray: &mut Ray,
		sky: &S,
		bvh: &A,
	) -> (Colour, u64) {
		let mut output;
		let mut throughput = Colour::one();
		let mut ray_count = 1;

		if let Some((surface_intersection, _index)) = bvh.check_hit(ray) {
			let (mut hit, mut mat) = (surface_intersection.hit, surface_intersection.material);
			let mut wo = ray.direction;
			output = mat.get_emission(&hit, wo);

			let mut exit = mat.scatter_ray(ray, &hit);

			if !exit {
				for depth in 1..MAX_DEPTH {
					exit = mat.scatter_ray(ray, &hit);

					if exit {
						ray_count += depth as u64;
						break;
					}

					let wi = ray.direction;

					if let Some((new_si, new_index)) = bvh.check_hit(ray) {
						if mat.is_delta() {
							throughput *= mat.eval(&hit, wo, wi);
							output += throughput * new_si.material.get_emission(&hit, wo);
						} else if bvh.get_samplable().contains(&new_index) {
							output += throughput
								* Self::sample_light_mis(
									bvh, &hit, &mat, wo, new_index, wi, &new_si,
								);
							ray_count += 1;
							throughput *= mat.eval(&hit, wo, wi) / mat.scattering_pdf(&hit, wo, wi);
						} else {
							output += throughput * Self::sample_light(bvh, &hit, &mat, wo);
							ray_count += 1;
							throughput *= mat.eval(&hit, wo, wi) / mat.scattering_pdf(&hit, wo, wi);
						}

						mat = new_si.material;
						hit = new_si.hit;
						wo = wi;
					} else {
						if mat.is_delta() {
							throughput *= mat.eval(&hit, wo, wi);
						} else {
							output += throughput * Self::sample_light(bvh, &hit, &mat, wo);
							ray_count += 1;
							throughput *= mat.eval(&hit, wo, wi) / mat.scattering_pdf(&hit, wo, wi);
						}

						output += throughput * sky.get_colour(ray);
						ray_count += depth as u64;
						break;
					}

					if depth > RUSSIAN_ROULETTE_THRESHOLD {
						let p = throughput.component_max();
						let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
						if rng.gen::<Float>() > p {
							ray_count += depth as u64;
							break;
						}
						throughput /= p;
					}
				}
			} else {
				output = mat.get_emission(&hit, wo);
			}
		} else {
			output = sky.get_colour(ray);
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
					throughput *= mat.eval(hit, wo, ray.direction)
						/ mat.scattering_pdf(hit, wo, ray.direction);
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
