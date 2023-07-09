use proc::Scatter;
use rt_core::{Float, Hit, Ray, Scatter, Vec3};

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
pub enum AllMaterials<'a, T: Texture> {
	Emit(Emit<'a, T>),
	Lambertian(Lambertian<'a, T>),
	TrowbridgeReitz(TrowbridgeReitz<'a, T>),
	Reflect(Reflect<'a, T>),
	Refract(Refract<'a, T>),
}
