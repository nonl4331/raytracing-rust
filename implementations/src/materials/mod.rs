use proc::Scatter;
use rt_core::{Float, Hit, Ray, Scatter, Vec3};

pub mod emissive;
pub mod lambertain;
pub mod reflect;
pub mod refract;

pub use crate::{
	materials::{emissive::Emit, lambertain::Lambertian, reflect::Reflect, refract::Refract},
	textures::Texture,
};

#[derive(Scatter, Debug)]
pub enum AllMaterials<T: Texture> {
	Emit(Emit<T>),
	Lambertian(Lambertian<T>),
	Reflect(Reflect<T>),
	Refract(Refract<T>),
}
