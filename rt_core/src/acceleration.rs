use crate::{Primitive, Ray, Scatter, SurfaceIntersection};

pub trait AccelerationStructure {
	type Object: Primitive;
	type Material: Scatter;
	fn get_intersection_candidates(&self, ray: &Ray) -> Vec<(usize, usize)>;

	fn check_hit_index(
		&self,
		ray: &Ray,
		object_index: usize,
	) -> Option<SurfaceIntersection<Self::Material>>;

	fn check_hit(&self, ray: &Ray) -> Option<(SurfaceIntersection<Self::Material>, usize)>;

	fn get_samplable(&self) -> &[usize] {
		unimplemented!()
	}

	fn get_object(&self, _index: usize) -> Option<&Self::Object> {
		unimplemented!()
	}
}
