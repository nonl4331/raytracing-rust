use crate::integrators::*;
use rt_core::*;

pub struct MisIntegrator;

impl Integrator for MisIntegrator {
	fn get_colour<A: AccelerationStructure<Object = P, Material = M>, P: Primitive, M: Scatter>(
		ray: &mut Ray,
		bvh: &A,
	) -> (Vec3, u64) {
		let (mut throughput, mut output) = (Vec3::one(), Vec3::zero());
		let mut ray_count = 0;

		let mut wo;
		let mut hit;
		let mut mat;
		let (surface_intersection, _index) = bvh.check_hit(ray);

		(hit, mat) = (surface_intersection.hit, surface_intersection.material);

		wo = ray.direction;

		let emission = mat.get_emission(&hit, wo);

		let exit = mat.scatter_ray(&mut ray.clone(), &hit);

		output += emission;

		if exit {
			return (output, ray_count);
		}

		let mut depth = 1;

		while depth < MAX_DEPTH {
			// light sampling
			let sample_lights = sample_lights(bvh, &hit);
			ray_count += 1;
			if let Some((l_wi, le, l_pdf)) = sample_lights {
				let m_pdf = mat.scattering_pdf(&hit, wo, l_wi);
				let mis_weight = power_heuristic(l_pdf, m_pdf);
				output += throughput * mat.eval(&hit, wo, l_wi) * mis_weight * le / l_pdf;
			}

			// material sampling and bounce
			let exit = mat.scatter_ray(ray, &hit);
			if exit {
				break;
			}
			let m_wi = ray.direction;

			let (intersection, index) = bvh.check_hit(ray);

			let m_pdf = mat.scattering_pdf(&hit, wo, m_wi);
			let le = intersection.material.get_emission(&hit, m_wi);
			throughput *= mat.eval_over_scattering_pdf(&hit, wo, m_wi);
			if le != Vec3::zero() {
				if (bvh.get_samplable().contains(&index) && !mat.is_delta())
					|| (index == usize::MAX && bvh.sky().can_sample())
				{
					let l_pdf = bvh.get_pdf_from_index(&hit, &intersection.hit, m_wi, index);
					let mis_weight = power_heuristic(m_pdf, l_pdf);
					output += throughput * le * mis_weight;
				} else {
					output += throughput * le;
				}
			}

			if intersection.material.is_light() {
				break;
			}

			if depth > RUSSIAN_ROULETTE_THRESHOLD {
				let p = throughput.component_max();
				let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
				if rng.gen::<Float>() > p {
					break;
				}
				throughput /= p;
			}

			wo = m_wi;
			hit = intersection.hit;
			mat = intersection.material;

			depth += 1;
		}
		if output.contains_nan() || !output.is_finite() {
			return (Vec3::zero(), ray_count);
		}
		(output, ray_count)
	}
}

fn sample_lights<A: AccelerationStructure<Object = P, Material = M>, P: Primitive, M: Scatter>(
	bvh: &A,
	hit: &Hit,
) -> Option<(Vec3, Vec3, Float)> {
	//l_wi, le, l_pdf
	let sky = bvh.sky();
	let samplable_len = bvh.get_samplable().len();
	let sky_can_sample = sky.can_sample();

	let sample_sky = |pdf_multiplier: Float| {
		let l_wi = sky.sample();
		let ray = Ray::new(hit.point + 0.0001 * hit.normal, l_wi, 0.0);

		let (sa, index) = bvh.check_hit(&ray);
		if index == usize::MAX {
			let le = sa.material.get_emission(hit, l_wi);
			let l_pdf = sky.pdf(l_wi);
			return Some((l_wi, le, l_pdf * pdf_multiplier));
		}
		None
	};

	let sample_light = |pdf_multiplier: Float, index: usize| {
		let index = bvh.get_samplable()[index];
		let light = bvh.get_object(index).unwrap();

		let l_wi = light.sample_visible_from_point(hit.point);

		if let Some(si) =
			bvh.check_hit_index(&Ray::new(hit.point + 0.0001 * hit.normal, l_wi, 0.0), index)
		{
			let l_pdf = light.scattering_pdf(hit.point, l_wi, &si.hit);
			if l_pdf > 0.0 {
				let le = si.material.get_emission(&si.hit, l_wi);
				return Some((l_wi, le, l_pdf * pdf_multiplier));
			}
		}
		None
	};

	match (samplable_len, sky_can_sample) {
		(0, false) => None,
		(0, true) => sample_sky(1.0),
		(_, false) => {
			let multipler = 1.0 / samplable_len as Float;
			let light_index = SmallRng::from_rng(thread_rng())
				.unwrap()
				.gen_range(0..samplable_len);
			sample_light(multipler, light_index)
		}
		(_, true) => {
			let multipler = 1.0 / (samplable_len + 1) as Float;
			let light_index = SmallRng::from_rng(thread_rng())
				.unwrap()
				.gen_range(0..=samplable_len);
			if light_index == samplable_len {
				sample_sky(multipler)
			} else {
				sample_light(multipler, light_index)
			}
		}
	}
}
