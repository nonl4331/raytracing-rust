use proc::Scatter;
use rt_core::{Float, Hit, Ray, Scatter, Vec3};

pub mod emissive;
pub mod lambertain;
pub mod phong;
pub mod reflect;
pub mod refract;

pub use crate::{
	materials::{
		emissive::Emit, lambertain::Lambertian, phong::Phong, reflect::Reflect, refract::Refract,
	},
	textures::Texture,
};

#[derive(Scatter, Debug)]
pub enum AllMaterials<T: Texture> {
	Emit(Emit<T>),
	Lambertian(Lambertian<T>),
	Phong(Phong<T>),
	Reflect(Reflect<T>),
	Refract(Refract<T>),
}
