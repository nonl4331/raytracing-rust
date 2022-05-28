pub mod aacuboid;
pub mod aarect;
pub mod cuboid;
pub mod rect;
pub mod sphere;
pub mod triangle;

use crate::acceleration::aabb::Aabb;
use crate::ray_tracing::{
	intersection::{
		aacuboid::aacuboid_intersection, aarect::aarect_intersection, cuboid::cuboid_intersection,
		rect::rect_intersection, sphere::sphere_intersection, triangle::triangle_intersection,
	},
	material::Scatter,
	primitives::{
		AACuboid, AARect, Axis, Cuboid, MeshTriangle, PrimitiveEnum, Rect, Sphere, Triangle,
	},
	ray::Ray,
};
use crate::utility::math::rotate_around_point;
use crate::utility::{
	coord::Coordinate,
	math::{next_float, previous_float, random_float, Float},
	vec::{Vec2, Vec3},
};
use rand::rngs::SmallRng;
use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use std::sync::Arc;

#[cfg(all(feature = "f64"))]
use std::f64::consts::PI;

#[cfg(not(feature = "f64"))]
use std::f32::consts::PI;

pub struct Hit {
	pub t: Float,
	pub point: Vec3,
	pub error: Vec3,
	pub normal: Vec3,
	pub uv: Option<Vec2>,
	pub out: bool,
}

pub struct SurfaceIntersection<M: Scatter> {
	pub hit: Hit,
	pub material: Arc<M>,
}

impl<M> SurfaceIntersection<M>
where
	M: Scatter,
{
	pub fn new(
		t: Float,
		point: Vec3,
		error: Vec3,
		normal: Vec3,
		uv: Option<Vec2>,
		out: bool,
		material: &Arc<M>,
	) -> Self {
		SurfaceIntersection {
			hit: Hit {
				t,
				point,
				error,
				normal,
				uv,
				out,
			},
			material: material.clone(),
		}
	}
}

pub trait Intersect<M: Scatter> {
	fn get_int(&self, _: &Ray) -> Option<SurfaceIntersection<M>> {
		unimplemented!()
	}
	fn does_int(&self, ray: &Ray) -> bool {
		self.get_int(ray).is_some()
	}
}

pub trait Primitive<M>: Intersect<M>
where
	M: Scatter,
{
	fn get_aabb(&self) -> Option<Aabb> {
		unimplemented!()
	}
	fn requires_uv(&self) -> bool {
		false
	}
	fn get_uv(&self, _: Vec3) -> Option<Vec2> {
		None
	}
	fn get_sample(&self) -> Vec3 {
		todo!()
	}
	fn sample_visible_from_point(&self, _: Vec3) -> (Vec3, Vec3, Vec3) {
		todo!()
	}
	fn area(&self) -> Float;
	fn scattering_pdf(&self, _: &Hit, _: Vec3, _: Vec3) -> Float {
		1.0 / self.area()
	}
	fn material_is_light(&self) -> bool {
		false
	}
}

pub fn offset_ray(origin: Vec3, normal: Vec3, error: Vec3, is_brdf: bool) -> Vec3 {
	let offset_val = normal.abs().dot(error);
	let mut offset = offset_val * normal;

	if !is_brdf {
		offset = -offset;
	}

	let mut new_origin = origin + offset;

	if offset.x > 0.0 {
		new_origin.x = next_float(new_origin.x);
	} else {
		new_origin.x = previous_float(new_origin.x);
	}

	if offset.y > 0.0 {
		new_origin.y = next_float(new_origin.y);
	} else {
		new_origin.y = previous_float(new_origin.y);
	}

	if offset.z > 0.0 {
		new_origin.z = next_float(new_origin.z);
	} else {
		new_origin.z = previous_float(new_origin.z);
	}

	new_origin
}

pub fn check_side(normal: &mut Vec3, ray_direction: &Vec3) -> bool {
	if normal.dot(*ray_direction) > 0.0 {
		*normal = -*normal;
		false
	} else {
		true
	}
}

impl<M> Intersect<M> for PrimitiveEnum<M>
where
	M: Scatter,
{
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		match self {
			PrimitiveEnum::Sphere(sphere) => sphere.get_int(ray),
			PrimitiveEnum::AARect(rect) => rect.get_int(ray),
			PrimitiveEnum::Rect(rect) => rect.get_int(ray),
			PrimitiveEnum::AACuboid(aab) => aab.get_int(ray),
			PrimitiveEnum::Cuboid(aab) => aab.get_int(ray),
			PrimitiveEnum::Triangle(triangle) => triangle.get_int(ray),
			PrimitiveEnum::MeshTriangle(triangle) => triangle.get_int(ray),
		}
	}

	fn does_int(&self, ray: &Ray) -> bool {
		match self {
			PrimitiveEnum::Sphere(sphere) => sphere.does_int(ray),
			PrimitiveEnum::AARect(rect) => rect.does_int(ray),
			PrimitiveEnum::Rect(rect) => rect.does_int(ray),
			PrimitiveEnum::AACuboid(aab) => aab.does_int(ray),
			PrimitiveEnum::Cuboid(aab) => aab.does_int(ray),
			PrimitiveEnum::Triangle(triangle) => triangle.does_int(ray),
			PrimitiveEnum::MeshTriangle(triangle) => triangle.does_int(ray),
		}
	}
}

impl<M> Primitive<M> for PrimitiveEnum<M>
where
	M: Scatter,
{
	fn get_aabb(&self) -> Option<Aabb> {
		match self {
			PrimitiveEnum::Sphere(sphere) => sphere.get_aabb(),
			PrimitiveEnum::AARect(rect) => rect.get_aabb(),
			PrimitiveEnum::Rect(rect) => rect.get_aabb(),
			PrimitiveEnum::AACuboid(aab) => aab.get_aabb(),
			PrimitiveEnum::Cuboid(aab) => aab.get_aabb(),
			PrimitiveEnum::Triangle(triangle) => triangle.get_aabb(),
			PrimitiveEnum::MeshTriangle(triangle) => triangle.get_aabb(),
		}
	}
	fn get_uv(&self, point: Vec3) -> Option<Vec2> {
		match self {
			PrimitiveEnum::Sphere(sphere) => sphere.get_uv(point),
			PrimitiveEnum::AARect(rect) => rect.get_uv(point),
			PrimitiveEnum::Rect(rect) => rect.get_uv(point),
			PrimitiveEnum::AACuboid(aab) => aab.get_uv(point),
			PrimitiveEnum::Cuboid(aab) => aab.get_uv(point),
			PrimitiveEnum::Triangle(triangle) => triangle.get_uv(point),
			PrimitiveEnum::MeshTriangle(triangle) => triangle.get_uv(point),
		};
		None
	}
	fn requires_uv(&self) -> bool {
		match self {
			PrimitiveEnum::Sphere(sphere) => (*sphere.material).requires_uv(),
			PrimitiveEnum::AARect(rect) => rect.material.requires_uv(),
			PrimitiveEnum::Rect(rect) => rect.aarect.material.requires_uv(),
			PrimitiveEnum::AACuboid(aab) => aab.material.requires_uv(),
			PrimitiveEnum::Cuboid(aab) => aab.material.requires_uv(),
			PrimitiveEnum::Triangle(triangle) => triangle.material.requires_uv(),
			PrimitiveEnum::MeshTriangle(triangle) => triangle.material.requires_uv(),
		}
	}
	fn get_sample(&self) -> Vec3 {
		match self {
			PrimitiveEnum::Sphere(sphere) => sphere.get_sample(),
			PrimitiveEnum::AARect(rect) => rect.get_sample(),
			PrimitiveEnum::Rect(rect) => rect.get_sample(),
			PrimitiveEnum::AACuboid(aab) => aab.get_sample(),
			PrimitiveEnum::Cuboid(aab) => aab.get_sample(),
			PrimitiveEnum::Triangle(triangle) => triangle.get_sample(),
			PrimitiveEnum::MeshTriangle(triangle) => triangle.get_sample(),
		}
	}
	fn sample_visible_from_point(&self, point: Vec3) -> (Vec3, Vec3, Vec3) {
		match self {
			PrimitiveEnum::Sphere(sphere) => sphere.sample_visible_from_point(point),
			PrimitiveEnum::AARect(rect) => rect.sample_visible_from_point(point),
			PrimitiveEnum::Rect(rect) => rect.sample_visible_from_point(point),
			PrimitiveEnum::AACuboid(aab) => aab.sample_visible_from_point(point),
			PrimitiveEnum::Cuboid(aab) => aab.sample_visible_from_point(point),
			PrimitiveEnum::Triangle(triangle) => triangle.sample_visible_from_point(point),
			PrimitiveEnum::MeshTriangle(triangle) => triangle.sample_visible_from_point(point),
		}
	}
	fn area(&self) -> Float {
		match self {
			PrimitiveEnum::Sphere(sphere) => sphere.area(),
			PrimitiveEnum::AARect(rect) => rect.area(),
			PrimitiveEnum::Rect(rect) => rect.area(),
			PrimitiveEnum::AACuboid(aab) => aab.area(),
			PrimitiveEnum::Cuboid(aab) => aab.area(),
			PrimitiveEnum::Triangle(triangle) => triangle.area(),
			PrimitiveEnum::MeshTriangle(triangle) => triangle.area(),
		}
	}
	fn scattering_pdf(&self, hit: &Hit, out_dir: Vec3, light_point: Vec3) -> Float {
		match self {
			PrimitiveEnum::Sphere(sphere) => sphere.scattering_pdf(hit, out_dir, light_point),
			PrimitiveEnum::AARect(rect) => rect.scattering_pdf(hit, out_dir, light_point),
			PrimitiveEnum::Rect(rect) => rect.scattering_pdf(hit, out_dir, light_point),
			PrimitiveEnum::AACuboid(aab) => aab.scattering_pdf(hit, out_dir, light_point),
			PrimitiveEnum::Cuboid(aab) => aab.scattering_pdf(hit, out_dir, light_point),
			PrimitiveEnum::Triangle(triangle) => triangle.scattering_pdf(hit, out_dir, light_point),
			PrimitiveEnum::MeshTriangle(triangle) => {
				triangle.scattering_pdf(hit, out_dir, light_point)
			}
		}
	}
	fn material_is_light(&self) -> bool {
		match self {
			PrimitiveEnum::Sphere(sphere) => sphere.material.is_light(),
			PrimitiveEnum::AARect(rect) => rect.material.is_light(),
			PrimitiveEnum::Rect(rect) => rect.aarect.material.is_light(),
			PrimitiveEnum::AACuboid(aab) => aab.material.is_light(),
			PrimitiveEnum::Cuboid(aab) => aab.material.is_light(),
			PrimitiveEnum::Triangle(triangle) => triangle.material.is_light(),
			PrimitiveEnum::MeshTriangle(triangle) => triangle.material.is_light(),
		}
	}
}

impl<M> Intersect<M> for Sphere<M>
where
	M: Scatter,
{
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		sphere_intersection(self, ray)
	}
}

#[allow(clippy::suspicious_operation_groupings)]
impl<M> Primitive<M> for Sphere<M>
where
	M: Scatter,
{
	fn get_uv(&self, point: Vec3) -> Option<Vec2> {
		if self.material.requires_uv() {
			let x = (self.center.x - point.x) / self.radius;
			let y = (self.center.y - point.y) / self.radius;
			let z = (self.center.z - point.z) / self.radius;
			let phi = (-1.0 * z).atan2(x) + PI;
			let theta = (-1.0 * y).acos();

			return Some(Vec2::new(phi / (2.0 * PI), theta / PI));
		}
		None
	}
	fn get_aabb(&self) -> Option<Aabb> {
		Some(Aabb::new(
			self.center - self.radius * Vec3::one(),
			self.center + self.radius * Vec3::one(),
		))
	}
	fn get_sample(&self) -> Vec3 {
		let z = 1.0 - 2.0 * random_float();
		let a = (1.0 - z * z).max(0.0).sqrt();
		let b = 2.0 * PI * random_float();
		self.center + self.radius * Vec3::new(a * b.cos(), a * b.sin(), z)
	}
	fn sample_visible_from_point(&self, in_point: Vec3) -> (Vec3, Vec3, Vec3) {
		let distance_sq = (in_point - self.center).mag_sq();
		let point = if distance_sq <= self.radius * self.radius {
			self.get_sample()
		} else {
			let distance = distance_sq.sqrt();
			let sin_theta_max_sq = self.radius * self.radius / distance_sq;
			let cost_theta_max = (1.0 - sin_theta_max_sq).max(0.0).sqrt();
			let r1 = random_float();
			let cos_theta = (1.0 - r1) + r1 * cost_theta_max;
			let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
			let phi = 2.0 * random_float() * PI;

			// calculate alpha
			let ds = distance * cos_theta
				- (self.radius * self.radius - distance_sq * sin_theta * sin_theta)
					.max(0.0)
					.sqrt();
			let cos_alpha = (distance_sq + self.radius * self.radius - ds * ds)
				/ (2.0 * distance * self.radius);
			let sin_alpha = (1.0 - cos_alpha * cos_alpha).max(0.0).sqrt();

			// get sphere point
			let coord_system = Coordinate::new_from_z((in_point - self.center).normalised());
			let mut vec = Vec3::new(sin_alpha * phi.cos(), sin_alpha * phi.sin(), cos_alpha);
			coord_system.vec_to_coordinate(&mut vec);

			self.center + self.radius * vec
		};
		(
			point,
			(point - in_point).normalised(),
			(point - self.center).normalised(),
		)
	}
	fn scattering_pdf(&self, hit: &Hit, out_dir: Vec3, light_point: Vec3) -> Float {
		let rsq = self.radius * self.radius;
		let dsq = (hit.point - self.center).mag_sq();
		if dsq <= rsq {
			return (light_point - hit.point).mag_sq()
				/ (out_dir.dot(-hit.normal).abs() * self.area());
		}
		let sin_theta_max_sq = rsq / dsq;
		let cos_theta_max = (1.0 - sin_theta_max_sq).max(0.0).sqrt();

		1.0 / (2.0 * PI * (1.0 - cos_theta_max))
	}
	fn area(&self) -> Float {
		4.0 * PI * self.radius * self.radius
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
}

impl<M> Intersect<M> for Triangle<M>
where
	M: Scatter,
{
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		triangle_intersection(self, ray)
	}
}

impl<M> Primitive<M> for Triangle<M>
where
	M: Scatter,
{
	fn get_aabb(&self) -> Option<Aabb> {
		Some(Aabb::new(
			self.points[0].min_by_component(self.points[1].min_by_component(self.points[2])),
			self.points[0].max_by_component(self.points[1].max_by_component(self.points[2])),
		))
	}
	fn area(&self) -> Float {
		todo!()
	}
}

impl<M> Intersect<M> for MeshTriangle<M>
where
	M: Scatter,
{
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		triangle_intersection(self, ray)
	}
}

impl<M> Primitive<M> for MeshTriangle<M>
where
	M: Scatter,
{
	fn get_aabb(&self) -> Option<Aabb> {
		let points = [
			(*self.mesh).vertices[self.point_indices[0]],
			(*self.mesh).vertices[self.point_indices[1]],
			(*self.mesh).vertices[self.point_indices[2]],
		];

		Some(Aabb::new(
			points[0].min_by_component(points[1].min_by_component(points[2])),
			points[0].max_by_component(points[1].max_by_component(points[2])),
		))
	}
	fn area(&self) -> Float {
		todo!()
	}
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
}
