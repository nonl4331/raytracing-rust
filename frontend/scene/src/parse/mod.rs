use crate::parse::{
	camera::CameraLoadError, materials::MaterialLoadError, primitives::PrimitiveLoadError,
	sky::SkyLoadError, textures::TextureLoadError,
};
use std::io;
use std::path::Path;
use toml::Value;

pub mod camera;
pub mod materials;
pub mod primitives;
pub mod sky;
pub mod textures;

#[macro_export]
macro_rules! try_into {
	($type:ident, $into_type:ident, $error:ident, $constructor:expr, $( $x:ident ), *) => {
		impl TryInto<$into_type> for $type {
			type Error = $error;

			fn try_into(self) -> Result<$into_type, Self::Error> {
				$(
					let $x = match self.$x {
						Some(v) => v,
						None => return Err($error::MissingField),
					};
				)*

				Ok($constructor)
			}
		}
	};
}

pub trait Load {
	type LoadType;
	fn derive_from(&mut self, other: Self);
	fn load(&mut self, _: &Self::LoadType) -> Result<(), Error> {
		Ok(())
	}
	fn get_definition(&self) -> Option<String>;
	fn get_plural<'a>() -> &'a str;
}

#[derive(Debug)]
pub enum Error {
	IO(io::Error),
	Parse(toml::de::Error),
	TextureLoad(TextureLoadError),
	MaterialLoad(MaterialLoadError),
	PrimitiveLoad(PrimitiveLoadError),
	SkyLoad(SkyLoadError),
	CameraLoad(CameraLoadError),
	NoValue(String),
	WrongType(String),
}

macro_rules! from {
	($error:path, $variant:ident) => {
		impl From<$error> for Error {
			fn from(error: $error) -> Self {
				Error::$variant(error)
			}
		}
	};
}

from!(io::Error, IO);
from!(toml::de::Error, Parse);
from!(TextureLoadError, TextureLoad);
from!(MaterialLoadError, MaterialLoad);
from!(PrimitiveLoadError, PrimitiveLoad);
from!(SkyLoadError, SkyLoad);
from!(CameraLoadError, CameraLoad);

pub fn parse_value_from_file(filepath: &Path) -> Result<Value, Error> {
	let data = std::fs::read_to_string(filepath)?;

	Ok(data.parse::<Value>()?)
}

pub fn parse_items<'de, O, L, T>(data: Value, names: &[String], other: &O) -> Result<Vec<T>, Error>
where
	L: Load + serde::Deserialize<'de> + Load<LoadType = O> + TryInto<T>,
	Error: From<<L as TryInto<T>>::Error>,
{
	let mut loaded_names: Vec<String> = Vec::new();
	let value = match data.get(L::get_plural()) {
		Some(value) => value,
		None => return Err(Error::NoValue(L::get_plural().to_string())),
	};

	match &value {
		Value::Array(vec_values) => {
			for value in vec_values {
				let name = value.clone().try_into()?;
				loaded_names.push(name);
			}
		}
		_ => return Err(Error::WrongType(L::get_plural().to_string())),
	};

	for name in names.iter() {
		if !loaded_names.contains(name) {
			return Err(Error::NoValue(name.to_owned()));
		}
	}

	let mut loads = Vec::new();
	for string_name in names.iter() {
		let value = match data.get(string_name) {
			Some(value) => value,
			None => return Err(Error::NoValue(string_name.clone())),
		};
		match value {
			val @ Value::Table(_) => {
				loads.push(parse_item::<O, L>(val.clone(), other)?);
			}
			_ => return Err(Error::WrongType(string_name.clone())),
		}
	}

	let items = loads
		.into_iter()
		.map(|l| l.try_into())
		.collect::<Result<Vec<T>, _>>()?;

	Ok(items)
}

pub fn parse_item<'de, O, L>(data: Value, other: &O) -> Result<L, Error>
where
	L: Load + serde::Deserialize<'de> + Load<LoadType = O>,
{
	let mut load: L = data.try_into()?;
	let definition_data: Option<L> = match load.get_definition() {
		Some(ref def) => {
			let data = std::fs::read_to_string(def)?.parse::<Value>()?;
			Some(parse_item(data, other)?)
		}
		_ => None,
	};

	if let Some(data) = definition_data {
		load.derive_from(data)
	};

	load.load(other)?;

	Ok(load)
}

#[cfg(test)]
mod tests {

	pub const EXAMPLE_TOML: &str = r#"
# alternative format (must be up top or they will be a subtable)
texture_two = { type = "image", path = "./textures/rock.png" }
texture_four = { type = "solid_colour", colour = [0.5, 0.5, 0.5] }
texture_three = { definition = "./other.toml" } # can define stuff externally
material_two = { definition = "./other.toml" }

# these are pub
scenes = ["scene_one"]
materials = ["material_one"]
primitives = ["sphere_one"]
textures = ["texture_four", "texture_one"]
skies = ["sky_one"]
cameras = ["camera_one"]

# An example scene
[scene_one.materials]
default = "m1"
mat1 = "material_one"
mat2 = "material_two"

[scene_one.primitives]
sphere_one = { type = "sphere", radius = 0.5, position = [10.5, 1.5, -3.5], mat = "mat2" }

[camera_one]
type = "simple"
origin = [0.0, -1.0, 1.0]
lookat = [0.0, 0.0, 0.0]
vup = [0.0, 0.0, 1.0]
fov = 36.0
focus_dist = 10.0
aperture = 2.0
aspect_ratio = 1.777777777

[material_one]
type = "lambertian"
absorbtion = 0.5
texture = "texture_one"

[sky_one]
texture = "texture_one"
sample_res = [ 20, 40 ]

[texture_one]
type = "lerp"
colour_one = [ 0.0, 0.0, 0.0 ]
colour_two = [ 1.0, 1.0, 1.0 ]

[sphere_one]
type = "sphere"
radius = 0.5
position = [0.0, 0.0, 0.5]
material = "material_one"

[a_triangle]
type = "triangle"
points = [ [0.0, 0.0, 0.0], [0.0, 0.5, 0.0], [0.5, 0.5, 0.0] ]
normals = [ [0.0, 0.0, 1.0] ]
mat = "material_two"

[some_mesh]
type = "triangle_mesh"
data = "./models/dragon"
mat = "material_two""#;
}
