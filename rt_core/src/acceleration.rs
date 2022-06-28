use crate::{Hit, Primitive, Ray, Scatter, SurfaceIntersection, Vec3};

pub trait AccelerationStructure<P, M>
where
	P: Primitive<M>,
	M: Scatter,
{
	fn get_intersection_candidates(&self, ray: &Ray) -> Vec<(usize, usize)>;

	fn check_hit_index(&self, ray: &Ray, object_index: usize) -> Option<SurfaceIntersection<M>>;

	fn check_hit(&self, ray: &Ray) -> Option<(SurfaceIntersection<M>, usize)>;

	fn number_nodes(&self) -> usize;

	fn sample_object(&self, _hit: &Hit, _index: usize) -> (Vec3, Option<Vec3>, Vec3) {
		unimplemented!()
	}

	fn get_samplable(&self) -> &[usize] {
		unimplemented!()
	}

	fn get_object(&self, _index: usize) -> Option<&P> {
		unimplemented!()
	}
}