use crate::{Float, Hit, Ray, Vec3};

pub trait Scatter {
	fn scatter_ray(&self, _ray: &mut Ray, _hit: &Hit) -> bool {
		true
	}
	fn requires_uv(&self) -> bool {
		false
	}
	fn is_light(&self) -> bool {
		false
	}
	fn ls_chance(&self) -> Float {
		0.0
	}
	fn is_delta(&self) -> bool {
		false
	}
	fn scattering_pdf(&self, _: Vec3, _: Vec3, _: Vec3) -> Float {
		0.0
	}
	fn scattering_albedo(&self, _hit: &Hit, _wo: Vec3, _wi: Vec3) -> Vec3 {
		Vec3::one()
	}
	fn eval(&self, _hit: &Hit, _wo: Vec3, _wi: Vec3) -> Vec3 {
		Vec3::one()
	}
	fn get_emission(&self, _hit: &Hit, _wo: Vec3) -> Vec3 {
		Vec3::zero()
	}
}
