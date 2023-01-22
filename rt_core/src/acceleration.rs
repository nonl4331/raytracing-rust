use crate::{Primitive, Ray, Scatter, SurfaceIntersection};

pub trait AccelerationStructure<P, M>
where
	P: Primitive,
	M: Scatter,
{
	fn get_intersection_candidates(&self, ray: &Ray) -> Vec<(usize, usize)>;

	fn check_hit_index(&self, ray: &Ray, object_index: usize) -> Option<SurfaceIntersection<M>>;

	fn check_hit(&self, ray: &Ray) -> Option<(SurfaceIntersection<M>, usize)>;

	fn get_samplable(&self) -> &[usize] {
		unimplemented!()
	}

	fn get_object(&self, _index: usize) -> Option<&P> {
		unimplemented!()
	}
}
