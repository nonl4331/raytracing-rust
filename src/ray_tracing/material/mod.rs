use crate::{
	ray_tracing::{
		intersection::{offset_ray, Hit},
		texture::TextureTrait,
		Ray,
	},
	utility::{random_float, random_unit_vector, vec::Vec3, Float},
};
use enum_dispatch::enum_dispatch;
use std::sync::Arc;

pub use cook_torrence::CookTorrence;
pub use lambertian::Lambertian;

mod cook_torrence;
mod lambertian;

#[enum_dispatch(Scatter)]
pub enum MaterialEnum<T: TextureTrait> {
	Lambertian(Lambertian<T>),
	Reflect(Reflect<T>),
	Refract(Refract<T>),
	Emit(Emit<T>),
	CookTorrence(CookTorrence<T>),
}

#[enum_dispatch]
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
	fn get_emission(&self, _hit: &Hit, _wo: Vec3) -> Vec3 {
		Vec3::zero()
	}
}

pub struct Reflect<T: TextureTrait> {
	pub texture: Arc<T>,
	pub fuzz: Float,
}

impl<T> Reflect<T>
where
	T: TextureTrait,
{
	pub fn new(texture: &Arc<T>, fuzz: Float) -> Self {
		Reflect {
			texture: texture.clone(),
			fuzz,
		}
	}
}

pub struct Refract<T: TextureTrait> {
	pub texture: Arc<T>,
	pub eta: Float,
}

impl<T> Refract<T>
where
	T: TextureTrait,
{
	pub fn new(texture: &Arc<T>, eta: Float) -> Self {
		Refract {
			texture: texture.clone(),
			eta,
		}
	}
}

pub struct Emit<T> {
	pub texture: Arc<T>,
	pub strength: Float,
}

impl<T> Emit<T>
where
	T: TextureTrait,
{
	pub fn new(texture: &Arc<T>, strength: Float) -> Self {
		Emit {
			texture: texture.clone(),
			strength,
		}
	}
}

impl<T> Scatter for Reflect<T>
where
	T: TextureTrait,
{
	fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
		let mut direction = ray.direction;
		direction.reflect(hit.normal);
		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		*ray = Ray::new(
			point,
			direction + self.fuzz * random_unit_vector(),
			ray.time,
		);
		false
	}
	fn scattering_albedo(&self, hit: &Hit, wo: Vec3, _wi: Vec3) -> Vec3 {
		let point = offset_ray(hit.point, hit.normal, hit.error, false);
		self.texture.colour_value(wo, point)
	}
	fn is_delta(&self) -> bool {
		true
	}
}

impl<T> Scatter for Refract<T>
where
	T: TextureTrait,
{
	fn scatter_ray(&self, ray: &mut Ray, hit: &Hit) -> bool {
		let mut eta_fraction = 1.0 / self.eta;
		if !hit.out {
			eta_fraction = self.eta;
		}

		let cos_theta = ((-1.0 * ray.direction).dot(hit.normal)).min(1.0);

		let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
		let cannot_refract = eta_fraction * sin_theta > 1.0;
		let f0 = (1.0 - eta_fraction) / (1.0 + eta_fraction);
		let f0 = f0 * f0 * Vec3::one();
		if cannot_refract || fresnel(cos_theta, f0).x > random_float() {
			let ref_mat = Reflect::new(&self.texture.clone(), 0.0);
			return ref_mat.scatter_ray(ray, hit);
		}

		let perp = eta_fraction * (ray.direction + cos_theta * hit.normal);
		let para = -1.0 * (1.0 - perp.mag_sq()).abs().sqrt() * hit.normal;
		let direction = perp + para;
		let point = offset_ray(hit.point, hit.normal, hit.error, false);
		*ray = Ray::new(point, direction, ray.time);
		false
	}
	fn scattering_albedo(&self, hit: &Hit, wo: Vec3, _wi: Vec3) -> Vec3 {
		let point = offset_ray(hit.point, hit.normal, hit.error, false);
		self.texture.colour_value(wo, point)
	}
	fn is_delta(&self) -> bool {
		true
	}
}

impl<T> Scatter for Emit<T>
where
	T: TextureTrait,
{
	fn get_emission(&self, hit: &Hit, _wo: Vec3) -> Vec3 {
		let point = offset_ray(hit.point, hit.normal, hit.error, true);
		//TODO
		self.strength * self.texture.colour_value(Vec3::zero(), point)
	}
	fn is_light(&self) -> bool {
		true
	}
	fn scatter_ray(&self, _: &mut Ray, _: &Hit) -> bool {
		true
	}
}

fn fresnel(cos: Float, f0: Vec3) -> Vec3 {
	f0 + (1.0 - f0) * (1.0 - cos).powf(5.0)
}
