use crate::rt_core::*;
use rand::rngs::SmallRng;
use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;

const MAX_DEPTH: u32 = 50;
const RUSSIAN_ROULETTE_THRESHOLD: u32 = 3;

pub mod mis;
pub use mis::*;

pub trait Integrator {
	fn get_colour<A: AccelerationStructure<Object = P, Material = M>, P: Primitive, M: Scatter>(
		ray: &mut Ray,
		bvh: &A,
	) -> (Vec3, u64);
}

pub struct NaiveIntegrator;

impl Integrator for NaiveIntegrator {
	fn get_colour<A: AccelerationStructure<Object = P, Material = M>, P: Primitive, M: Scatter>(
		ray: &mut Ray,
		bvh: &A,
	) -> (Vec3, u64) {
		let (mut throughput, mut output) = (Vec3::one(), Vec3::zero());
		let mut depth = 0;
		let mut ray_count = 0;

		while depth < MAX_DEPTH {
			let hit_info = bvh.check_hit(ray);

			ray_count += 1;

			let (surface_intersection, _index) = hit_info;
			let (hit, mat) = (&surface_intersection.hit, &surface_intersection.material);

			let wo = ray.direction;

			let emission = mat.get_emission(hit, wo);

			let exit = mat.scatter_ray(ray, hit);

			if depth == 0 {
				output += emission;
				if exit {
					break;
				}
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
		}
		if output.contains_nan() || !output.is_finite() {
			return (Vec3::zero(), ray_count);
		}
		(output, ray_count)
	}
}
