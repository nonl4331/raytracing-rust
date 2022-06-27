use crate::{
	primitives::{aacubiod::AACuboid, rect::Rect, AARect, Axis},
	utility::rotate_around_point,
};
use rt_core::{Aabb, Float, Intersect, Primitive, Ray, Scatter, SurfaceIntersection, Vec3};
use std::sync::Arc;

pub struct Cuboid<M: Scatter> {
	pub rects: [Rect<M>; 6],
	pub min: Vec3,
	pub max: Vec3,
	pub center: Vec3,
	pub material: Arc<M>,
}

impl<M> Cuboid<M>
where
	M: Scatter,
{
	pub fn new(aacuboid: AACuboid<M>, rotations: Vec3, _testing: &Arc<M>) -> Self {
		let cos_rotations = Vec3::new(rotations.x.cos(), rotations.y.cos(), rotations.z.cos());
		let sin_rotations = Vec3::new(rotations.x.sin(), rotations.y.sin(), rotations.z.sin());

		let material = aacuboid.material.clone();

		let min = aacuboid.min;
		let max = aacuboid.max;

		let center = (min + max) / 2.0;

		let rects = [
			Rect::new(
				AARect::new(
					Axis::X.point_without_axis(min),
					Axis::X.point_without_axis(max),
					min.x,
					Axis::X,
					&material,
				),
				rotations,
				Some(min),
			),
			Rect::new(
				AARect::new(
					Axis::X.point_without_axis(min),
					Axis::X.point_without_axis(max),
					max.x,
					Axis::X,
					&material,
				),
				rotations,
				Some(min),
			),
			Rect::new(
				AARect::new(
					Axis::Y.point_without_axis(min),
					Axis::Y.point_without_axis(max),
					min.y,
					Axis::Y,
					&material,
				),
				rotations,
				Some(min),
			),
			Rect::new(
				AARect::new(
					Axis::Y.point_without_axis(min),
					Axis::Y.point_without_axis(max),
					max.y,
					Axis::Y,
					&material,
				),
				rotations,
				Some(min),
			),
			Rect::new(
				AARect::new(
					Axis::Z.point_without_axis(min),
					Axis::Z.point_without_axis(max),
					min.z,
					Axis::Z,
					&material,
				),
				rotations,
				Some(min),
			),
			Rect::new(
				AARect::new(
					Axis::Z.point_without_axis(min),
					Axis::Z.point_without_axis(max),
					max.z,
					Axis::Z,
					&material,
				),
				rotations,
				Some(min),
			),
		];

		let mut point_one = min;
		let mut point_two = max;
		let mut point_three = Vec3::new(min.x, min.y, max.z);
		let mut point_four = Vec3::new(min.x, max.y, max.z);
		let mut point_five = Vec3::new(min.x, max.y, min.z);
		let mut point_six = Vec3::new(max.x, max.y, min.z);
		let mut point_seven = Vec3::new(max.x, min.y, min.z);
		let mut point_eight = Vec3::new(max.x, min.y, max.z);

		rotate_around_point(&mut point_one, max, sin_rotations, cos_rotations);
		rotate_around_point(&mut point_two, max, sin_rotations, cos_rotations);
		rotate_around_point(&mut point_three, max, sin_rotations, cos_rotations);
		rotate_around_point(&mut point_four, max, sin_rotations, cos_rotations);
		rotate_around_point(&mut point_five, max, sin_rotations, cos_rotations);
		rotate_around_point(&mut point_six, max, sin_rotations, cos_rotations);
		rotate_around_point(&mut point_seven, max, sin_rotations, cos_rotations);
		rotate_around_point(&mut point_eight, max, sin_rotations, cos_rotations);

		let max = point_one
			.max_by_component(point_two)
			.max_by_component(point_three)
			.max_by_component(point_four)
			.max_by_component(point_five)
			.max_by_component(point_six)
			.max_by_component(point_seven)
			.max_by_component(point_eight);
		let min = point_one
			.min_by_component(point_two)
			.min_by_component(point_three)
			.min_by_component(point_four)
			.min_by_component(point_five)
			.min_by_component(point_six)
			.min_by_component(point_seven)
			.min_by_component(point_eight);

		let len = max - min;
		let max = max + len * 0.02;
		let min = min - len * 0.02;

		Cuboid {
			rects,
			max,
			min,
			center,
			material,
		}
	}
}

fn cuboid_intersection<M: Scatter>(
	cuboid: &Cuboid<M>,
	ray: &Ray,
) -> Option<SurfaceIntersection<M>> {
	let mut hit: Option<SurfaceIntersection<M>> = None;
	for side in cuboid.rects.iter() {
		if let Some(current_hit) = side.get_int(ray) {
			if current_hit.hit.t > 0.0 {
				if hit.is_some() {
					if current_hit.hit.t < hit.as_ref().unwrap().hit.t {
						hit = Some(current_hit);
					}
					continue;
				}

				hit = Some(current_hit);
			}
		}
	}
	hit
}

impl<M> Intersect<M> for Cuboid<M>
where
	M: Scatter,
{
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		cuboid_intersection(self, ray)
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

impl<M> Primitive<M> for Cuboid<M>
where
	M: Scatter,
{
	fn get_aabb(&self) -> Option<Aabb> {
		Some(Aabb::new(self.min - Vec3::one(), self.max + Vec3::one()))
	}
	fn area(&self) -> Float {
		todo!()
		//(self.max.x - self.min.x) * (self.max.y - self.min.y) * (self.max.z - self.min.z)
	}
	fn material_is_light(&self) -> bool {
		self.material.is_light()
	}
}
