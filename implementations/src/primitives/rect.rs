use crate::{
	primitives::{aarect_intersection, AARect, Axis},
	utility::rotate_around_point,
};
use rt_core::{Aabb, Float, Intersect, Primitive, Ray, Scatter, SurfaceIntersection, Vec2, Vec3};

pub struct Rect<M: Scatter> {
	pub aarect: AARect<M>,
	pub cos_rotations: Vec3,
	pub sin_rotations: Vec3,
	pub rotation_point: Option<Vec3>,
}

impl<M> Rect<M>
where
	M: Scatter,
{
	pub fn new(aarect: AARect<M>, rotations: Vec3, rotation_point: Option<Vec3>) -> Self {
		let cos_rotations = Vec3::new(rotations.x.cos(), rotations.y.cos(), rotations.z.cos());
		let sin_rotations = Vec3::new(rotations.x.sin(), rotations.y.sin(), rotations.z.sin());

		Rect {
			aarect,
			cos_rotations,
			sin_rotations,
			rotation_point,
		}
	}
}

fn rect_intersection<M: Scatter>(rect: &Rect<M>, ray: &Ray) -> Option<SurfaceIntersection<M>> {
	let center_point = match rect.rotation_point {
		Some(val) => val,
		None => {
			(Axis::point_from_2d(&rect.aarect.max, &rect.aarect.axis, rect.aarect.k)
				+ Axis::point_from_2d(&rect.aarect.min, &rect.aarect.axis, rect.aarect.k))
				/ 2.0
		}
	};

	let mut rotated_origin = ray.origin;
	let mut rotated_direction = ray.direction;

	rotate_around_point(
		&mut rotated_origin,
		center_point,
		rect.sin_rotations,
		rect.cos_rotations,
	);
	rotate_around_point(
		&mut rotated_direction,
		Vec3::zero(),
		rect.sin_rotations,
		rect.cos_rotations,
	);

	let rotated_ray = Ray::new(rotated_origin, rotated_direction, 0.0);

	let mut intersection = match aarect_intersection(&rect.aarect, &rotated_ray) {
		Some(int) => int,
		None => return None,
	};

	rotate_around_point(
		&mut intersection.hit.point,
		center_point,
		-rect.sin_rotations,
		rect.cos_rotations,
	);
	rotate_around_point(
		&mut intersection.hit.normal,
		Vec3::zero(),
		-rect.sin_rotations,
		rect.cos_rotations,
	);

	intersection.hit.normal.normalise();

	Some(intersection)
}

impl<M> Intersect<M> for Rect<M>
where
	M: Scatter,
{
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		rect_intersection(self, ray)
	}

	fn does_int(&self, ray: &Ray) -> bool {
		let t = (self.aarect.k - self.aarect.axis.get_axis_value(ray.origin))
			/ self.aarect.axis.get_axis_value(ray.direction);
		let point = ray.at(t);
		let point_2d = self.aarect.axis.point_without_axis(point);

		// x & y are not the x & y axis but rather the two axis that are not self.axis
		point_2d.x > self.aarect.min.x
			&& point_2d.x < self.aarect.max.x
			&& point_2d.y > self.aarect.min.y
			&& point_2d.y < self.aarect.max.y
	}
}

impl<M> Primitive<M> for Rect<M>
where
	M: Scatter,
{
	fn get_uv(&self, point: Vec3) -> Option<Vec2> {
		if self.aarect.material.requires_uv() {
			let pwa = self.aarect.axis.point_without_axis(point);
			return Some(Vec2::new(
				(pwa.x - self.aarect.min.x) / self.aarect.max.x,
				(pwa.y - self.aarect.min.y) / self.aarect.max.y,
			));
		}
		None
	}
	fn get_aabb(&self) -> Option<Aabb> {
		let max = Axis::point_from_2d(&self.aarect.max, &self.aarect.axis, self.aarect.k);
		let min = Axis::point_from_2d(&self.aarect.min, &self.aarect.axis, self.aarect.k);

		let center_point = (max + min) / 2.0;

		let mut point_a = max;
		let mut point_b = min;

		rotate_around_point(
			&mut point_a,
			center_point,
			self.sin_rotations,
			self.cos_rotations,
		);
		rotate_around_point(
			&mut point_b,
			center_point,
			self.sin_rotations,
			self.cos_rotations,
		);

		let mut max = point_a.max_by_component(point_b).max_by_component(max);
		let mut min = point_a.min_by_component(point_b).min_by_component(min);

		max += self
			.aarect
			.axis
			.return_point_with_axis(Vec3::one() * 0.0001);
		min -= self
			.aarect
			.axis
			.return_point_with_axis(Vec3::one() * 0.0001);

		Some(Aabb::new(min, max))
	}

	fn area(&self) -> Float {
		(self.aarect.max.x - self.aarect.min.x) * (self.aarect.max.y - self.aarect.min.y)
	}
	fn material_is_light(&self) -> bool {
		self.aarect.material.is_light()
	}
}
