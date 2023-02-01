use crate::{Float, Hit, Ray, Vec3};

// wo (and ray.direction in scatter_ray) points towards the surface and wi away by convention
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
	fn scattering_pdf(&self, _hit: &Hit, _wo: Vec3, _wi: Vec3) -> Float {
		0.0
	}
	fn eval(&self, _hit: &Hit, _wo: Vec3, _wi: Vec3) -> Vec3;
	fn eval_over_scattering_pdf(&self, hit: &Hit, wo: Vec3, wi: Vec3) -> Vec3 {
		self.eval(hit, wo, wi) / self.scattering_pdf(hit, wo, wi)
	}
	fn get_emission(&self, _hit: &Hit, _wo: Vec3) -> Vec3 {
		Vec3::zero()
	}
}
