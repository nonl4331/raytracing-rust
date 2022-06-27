mod acceleration;
mod cameras;
mod materials;
mod primitives;
mod samplers;
mod textures;
mod utility;

pub use acceleration::*;
pub use cameras::*;
pub use materials::*;
pub use primitives::*;
pub use proc::*;
pub use samplers::*;
pub use textures::*;

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		let result = 2 + 2;
		assert_eq!(result, 4);
	}
}
