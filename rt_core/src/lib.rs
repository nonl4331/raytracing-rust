pub mod acceleration;
pub mod material;
pub mod primitive;
pub mod ray;
pub mod sampler;
pub mod vec;

pub use acceleration::*;
pub use material::*;
pub use primitive::*;
pub use ray::*;
pub use sampler::*;
pub use vec::*;

#[cfg(all(feature = "f64"))]
pub mod f64_stuff {
	pub type Float = f64;
	pub use std::f64::consts::*;
	pub use std::f64::*;
	pub const EPSILON: Float = 5.58E-8;
}

#[cfg(not(feature = "f64"))]
pub mod f32_stuff {
	pub type Float = f32;
	pub use std::f32::consts::*;
	pub use std::f32::*;
	pub const EPSILON: Float = 3.0E-4;
}

#[cfg(not(feature = "f64"))]
pub use f32_stuff::*;
#[cfg(all(feature = "f64"))]
pub use f64_stuff::*;

#[inline]
pub fn power_heuristic(pdf_a: Float, pdf_b: Float) -> Float {
	let a_sq = pdf_a * pdf_a;
	a_sq / (a_sq + pdf_b * pdf_b)
}
