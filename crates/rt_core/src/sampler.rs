use crate::{Float, Ray, Scatter, SurfaceIntersection, Vec3};

pub trait NoHit<M: Scatter>: Sync {
	fn get_colour(&self, ray: &Ray) -> Vec3;
	fn pdf(&self, _: Vec3) -> Float {
		unimplemented!()
	}
	fn can_sample(&self) -> bool {
		false
	}
	fn sample(&self) -> Vec3 {
		unimplemented!()
	}
	fn get_si(&self, _: &Ray) -> SurfaceIntersection<M> {
		unimplemented!()
	}
}
