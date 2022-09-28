use proc::Scatter;
use rt_core::{Float, Hit, Ray, Scatter, Vec3};

pub mod emissive;
pub mod lambertian;
pub mod phong;
pub mod reflect;
pub mod refract;
pub mod trowbridge_reitz;

pub use crate::{
	materials::{
		emissive::Emit, lambertian::Lambertian, phong::Phong, reflect::Reflect, refract::Refract,
		trowbridge_reitz::TrowbridgeReitz,
	},
	textures::Texture,
};

#[derive(Scatter, Debug)]
pub enum AllMaterials<T: Texture> {
	Emit(Emit<T>),
	Lambertian(Lambertian<T>),
	Phong(Phong<T>),
	TrowbridgeReitz(TrowbridgeReitz<T>),
	Reflect(Reflect<T>),
	Refract(Refract<T>),
}
