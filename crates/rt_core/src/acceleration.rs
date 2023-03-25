use crate::*;

pub trait AccelerationStructure: Sync {
	type Object: Primitive;
	type Material: Scatter;
	type Sky: NoHit<Self::Material>;
	fn get_intersection_candidates(&self, ray: &Ray) -> Vec<(usize, usize)>;

	fn check_hit_index(
		&self,
		ray: &Ray,
		object_index: usize,
	) -> Option<SurfaceIntersection<Self::Material>>;

	fn check_hit(&self, ray: &Ray) -> (SurfaceIntersection<Self::Material>, usize);

	fn get_samplable(&self) -> &[usize] {
		unimplemented!()
	}

	fn get_object(&self, _index: usize) -> Option<&Self::Object> {
		unimplemented!()
	}
	fn get_pdf_from_index(
		&self,
		last_hit: &Hit,
		light_hit: &Hit,
		sampled_dir: Vec3,
		index: usize,
	) -> Float;
	fn sky(&self) -> &Self::Sky;
}
