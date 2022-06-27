use crate::primitives::{AARect, Axis};
use rt_core::{Aabb, Float, Intersect, Primitive, Ray, Scatter, SurfaceIntersection, Vec3};
use std::sync::Arc;

pub struct AACuboid<M: Scatter> {
	pub min: Vec3,
	pub max: Vec3,
	pub rects: [AARect<M>; 6],
	pub material: Arc<M>,
}

impl<M> AACuboid<M>
where
	M: Scatter,
{
	pub fn new(point_one: Vec3, point_two: Vec3, material: &Arc<M>) -> Self {
		if point_one == point_two {
			panic!("AACuboid called with two of the same point!");
		}
		let min = point_one.min_by_component(point_two);
		let max = point_one.max_by_component(point_two);

		let rects = [
			AARect::new(
				Axis::X.point_without_axis(min),
				Axis::X.point_without_axis(max),
				min.x,
				Axis::X,
				material,
			),
			AARect::new(
				Axis::X.point_without_axis(min),
				Axis::X.point_without_axis(max),
				max.x,
				Axis::X,
				material,
			),
			AARect::new(
				Axis::Y.point_without_axis(min),
				Axis::Y.point_without_axis(max),
				min.y,
				Axis::Y,
				material,
			),
			AARect::new(
				Axis::Y.point_without_axis(min),
				Axis::Y.point_without_axis(max),
				max.y,
				Axis::Y,
				material,
			),
			AARect::new(
				Axis::Z.point_without_axis(min),
				Axis::Z.point_without_axis(max),
				min.z,
				Axis::Z,
				material,
			),
			AARect::new(
				Axis::Z.point_without_axis(min),
				Axis::Z.point_without_axis(max),
				max.z,
				Axis::Z,
				material,
			),
		];
		AACuboid {
			min,
			max,
			rects,
			material: material.clone(),
		}
	}
}

fn aacuboid_intersection<M: Scatter>(
	aacuboid: &AACuboid<M>,
	ray: &Ray,
) -> Option<SurfaceIntersection<M>> {
	let mut hit: Option<SurfaceIntersection<M>> = None;
	for side in aacuboid.rects.iter() {
		if let Some(current_hit) = side.get_int(ray) {
			// make sure ray is going forwards
			if current_hit.hit.t > 0.0 {
				// check if hit already exists
				if hit.is_some() {
					// check if t value is close to 0 than previous hit
					if current_hit.hit.t < hit.as_ref().unwrap().hit.t {
						hit = Some(current_hit);
					}
					continue;
				}

				// if hit doesn't exist set current hit to hit
				hit = Some(current_hit);
			}
		}
	}
	hit
}

impl<M> Intersect<M> for AACuboid<M>
where
	M: Scatter,
{
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		aacuboid_intersection(self, ray)
	}

	fn does_int(&self, ray: &Ray) -> bool {
		for side in self.rects.iter() {
			if side.does_int(ray) {
				return true;
			}
		}
		false
	}
}

impl<M> Primitive<M> for AACuboid<M>
where
	M: Scatter,
{
	fn get_aabb(&self) -> Option<Aabb> {
		Some(Aabb::new(self.min, self.max))
	}
	fn area(&self) -> Float {
		(self.max.x - self.min.x) * (self.max.y - self.min.y) * (self.max.z - self.min.z)
	}
	fn material_is_light(&self) -> bool {
		self.material.is_light()
	}
}
