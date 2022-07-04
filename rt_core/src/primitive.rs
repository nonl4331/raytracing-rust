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
	fn get_int(&self, _: &Ray) -> Option<SurfaceIntersection<M>>;
	fn does_int(&self, ray: &Ray) -> bool {
		self.get_int(ray).is_some()
	}
	fn get_uv(&self, _: Vec3) -> Option<Vec2> {
		None
	}
	fn get_sample(&self) -> Vec3 {
		unimplemented!()
	}
	fn sample_visible_from_point(&self, _point: Vec3) -> Vec3 {
		unimplemented!()
	}
	fn area(&self) -> Float;
	fn scattering_pdf(&self, _hit: &Hit, _wo: Vec3, _light_point: Vec3) -> Float {
		1.0 / self.area()
	}
	fn material_is_light(&self) -> bool {
		false
	}
}
