use crate::rt_core::{Float, Hit, Ray, Scatter, Vec3};
use proc::Scatter;

pub mod emissive;
pub mod lambertian;
pub mod reflect;
pub mod refract;
pub mod trowbridge_reitz;

pub use crate::{
	materials::{
		emissive::Emit, lambertian::Lambertian, reflect::Reflect, refract::Refract,
		trowbridge_reitz::TrowbridgeReitz,
	},
	textures::Texture,
};

#[derive(Scatter, Debug, Clone)]
pub enum AllMaterials<T: Texture> {
	Emit(Emit<T>),
	Lambertian(Lambertian<T>),
	TrowbridgeReitz(TrowbridgeReitz<T>),
	Reflect(Reflect<T>),
	Refract(Refract<T>),
}
