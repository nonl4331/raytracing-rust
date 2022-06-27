use crate::primitives::Axis;
use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};
use rt_core::{
	Aabb, Float, Hit, Intersect, Primitive, Ray, Scatter, SurfaceIntersection, Vec2, Vec3, EPSILON,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AARect<M: Scatter> {
	pub min: Vec2,
	pub max: Vec2,
	pub k: Float,
	pub axis: Axis,
	pub material: Arc<M>,
}

impl<M> AARect<M>
where
	M: Scatter,
{
	pub fn new(point_one: Vec2, point_two: Vec2, k: Float, axis: Axis, material: &Arc<M>) -> Self {
		if point_one == point_two {
			panic!("AARect called with two of the same point!");
		}
		let min = point_one.min_by_component(point_two);
		let max = point_one.max_by_component(point_two);
		AARect {
			min,
			max,
			k,
			axis,
			material: material.clone(),
		}
	}
}

pub fn aarect_intersection<M: Scatter>(
	aarect: &AARect<M>,
	ray: &Ray,
) -> Option<SurfaceIntersection<M>> {
	let t = (aarect.k - aarect.axis.get_axis_value(ray.origin))
		/ aarect.axis.get_axis_value(ray.direction);
	let point = ray.at(t);
	let point_2d = aarect.axis.point_without_axis(point);

	// x & y are not the x & y axis but rather the two axis that are not self.axis
	if point_2d.x > aarect.min.x
		&& point_2d.x < aarect.max.x
		&& point_2d.y > aarect.min.y
		&& point_2d.y < aarect.max.y
	{
		Some(SurfaceIntersection::new(
			t,
			point,
			EPSILON * aarect.axis.return_point_with_axis(Vec3::one()),
			aarect
				.axis
				.return_point_with_axis(-ray.direction)
				.normalised(),
			aarect.get_uv(point),
			true,
			&aarect.material,
		))
	} else {
		None
	}
}

impl<M> Intersect<M> for AARect<M>
where
	M: Scatter,
{
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		aarect_intersection(self, ray)
	}

	fn does_int(&self, ray: &Ray) -> bool {
		let t = (self.k - self.axis.get_axis_value(ray.origin))
			/ self.axis.get_axis_value(ray.direction);
		let point = ray.at(t);
		let point_2d = self.axis.point_without_axis(point);

		// x & y are not the x & y axis but rather the two axis that are not self.axis
		point_2d.x > self.min.x
			&& point_2d.x < self.max.x
			&& point_2d.y > self.min.y
			&& point_2d.y < self.max.y
	}
}

impl<M> Primitive<M> for AARect<M>
where
	M: Scatter,
{
	fn get_uv(&self, point: Vec3) -> Option<Vec2> {
		if self.material.requires_uv() {
			let pwa = self.axis.point_without_axis(point);
			return Some(Vec2::new(
				(pwa.x - self.min.x) / self.max.x,
				(pwa.y - self.min.y) / self.max.y,
			));
		}
		None
	}
	fn get_aabb(&self) -> Option<Aabb> {
		Some(Aabb::new(
			Axis::point_from_2d(&self.min, &self.axis, self.k - 0.0001),
			Axis::point_from_2d(&self.max, &self.axis, self.k + 0.0001),
		))
	}
	fn sample_visible_from_point(&self, in_point: Vec3) -> (Vec3, Vec3, Vec3) {
		let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
		let point = Vec2::new(
			rng.gen_range(self.min.x..self.max.x),
			rng.gen_range(self.min.y..self.max.y),
		);
		let point = Axis::point_from_2d(&point, &self.axis, self.k);
		let dir = (point - in_point).normalised();
		let norm = self.axis.return_point_with_axis(-dir).normalised();
		let point = point - 0.0001 * norm;

		(point, dir, norm)
	}
	fn scattering_pdf(&self, hit: &Hit, _: Vec3, light_point: Vec3) -> Float {
		(light_point - hit.point).mag_sq()
			/ ((hit.point - light_point).normalised().y.abs() * self.area())
	}
	fn area(&self) -> Float {
		let a = self.max - self.min;
		a.x * a.y
	}
	fn material_is_light(&self) -> bool {
		self.material.is_light()
	}
}
