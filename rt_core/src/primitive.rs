use crate::{Float, Ray, Scatter, Vec2, Vec3};
use std::sync::Arc;

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

pub trait Primitive<M: Scatter> {
	fn get_int(&self, _: &Ray) -> Option<SurfaceIntersection<M>> {
		unimplemented!()
	}
	fn does_int(&self, ray: &Ray) -> bool {
		self.get_int(ray).is_some()
	}
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

#[derive(Copy, Clone, Debug)]
pub struct Aabb {
	pub min: Vec3,
	pub max: Vec3,
}

impl Aabb {
	pub fn new(min: Vec3, max: Vec3) -> Self {
		if (min.x >= max.x || min.y >= max.y || min.z >= max.z) && (min != max) {
			panic!("Maximum value in AABB must be greater or equal to minimum!");
		}
		Aabb { min, max }
	}

	pub fn does_int(&self, ray: &Ray) -> bool {
		let t1 = (self.min.x - ray.origin.x) * ray.d_inverse.x;
		let t2 = (self.max.x - ray.origin.x) * ray.d_inverse.x;

		let tmin = t1.min(t2);
		let tmax = t1.max(t2);

		let t1 = (self.min.y - ray.origin.y) * ray.d_inverse.y;
		let t2 = (self.max.y - ray.origin.y) * ray.d_inverse.y;

		let tmin = tmin.max(t1.min(t2));
		let tmax = tmax.min(t1.max(t2));
		let t1 = (self.min.z - ray.origin.z) * ray.d_inverse.z;
		let t2 = (self.max.z - ray.origin.z) * ray.d_inverse.z;

		let tmin = tmin.max(t1.min(t2));
		let tmax = tmax.min(t1.max(t2));

		tmax > tmin.max(0.0)
	}

	pub fn merge(aabb: &mut Option<Self>, second: Self) {
		match aabb {
			Some(inner) => {
				inner.min = inner.min.min_by_component(second.min);
				inner.max = inner.max.max_by_component(second.max);
			}
			None => *aabb = Some(second),
		}
	}

	pub fn extend_contains(aabb: &mut Option<Self>, point: Vec3) {
		match aabb {
			Some(inner) => {
				inner.min = inner.min.min_by_component(point);
				inner.max = inner.max.max_by_component(point);
			}
			None => *aabb = Some(Aabb::new(point, point)),
		}
	}

	pub fn get_extent(&self) -> Vec3 {
		self.max - self.min
	}

	pub fn surface_area(&self) -> Float {
		let extent = self.get_extent();
		2.0 * (extent.x * extent.y + extent.x * extent.z + extent.y * extent.z) as Float
	}
}
