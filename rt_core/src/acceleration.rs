use crate::{Hit, Primitive, Ray, Scatter, SurfaceIntersection, Vec3};

pub trait PrimitiveSampling<P, M>: AccelerationStructure<M>
where
	P: Primitive<M>,
	M: Scatter,
{
	fn sample_object(&self, hit: &Hit, index: usize) -> (Vec3, Option<Vec3>, Vec3);

	fn get_samplable(&self) -> &[usize];

	fn get_object(&self, index: usize) -> Option<&P>;
}

pub trait AccelerationStructure<M: Scatter> {
	fn get_intersection_candidates(&self, ray: &Ray) -> Vec<(usize, usize)>;

	fn check_hit_index(&self, ray: &Ray, object_index: usize) -> Option<SurfaceIntersection<M>>;

	fn check_hit(&self, ray: &Ray) -> Option<(SurfaceIntersection<M>, usize)>;

	fn number_nodes(&self) -> usize;
}
