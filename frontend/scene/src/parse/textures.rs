use crate::parse::*;
use implementations::{AllTextures, CheckeredTexture, ImageTexture, Lerp, Perlin, SolidColour};
use serde::Deserialize;
use std::path::PathBuf;

use toml::Value;

use crate::*;

pub fn parse_textures(data: Value, texture_names: &[String]) -> Result<Vec<AllTextures>, Error> {
	parse_items::<(), TextureLoad, AllTextures>(data, texture_names, &mut ())
}

#[derive(Deserialize, Debug)]
pub struct TextureLoad {
	definition: Option<String>,
	#[serde(rename = "type")]
	tex_type: Option<String>,
	colour: Option<[Float; 3]>,
	colour_one: Option<[Float; 3]>,
	colour_two: Option<[Float; 3]>,
	path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum TextureLoadError {
	MissingField,
	InvalidType,
}

impl Load for TextureLoad {
	type LoadType = ();
	fn derive_from(&mut self, other: Self) {
		if self.tex_type.is_none() {
			self.tex_type = other.tex_type;
		}
		if self.colour.is_none() {
			self.colour = other.colour;
		}
		if self.colour_one.is_none() {
			self.colour_one = other.colour_one;
		}
		if self.colour_two.is_none() {
			self.colour_two = other.colour_two;
		}
		if self.path.is_none() {
			self.path = other.path;
		}
	}
	fn get_definition(&self) -> Option<String> {
		self.definition.clone()
	}
	fn get_plural<'a>() -> &'a str {
		"textures"
	}
}

try_into!(
	TextureLoad,
	SolidColour,
	TextureLoadError,
	SolidColour::new(colour.into()),
	colour
);

try_into!(
	TextureLoad,
	ImageTexture,
	TextureLoadError,
	ImageTexture::new(&path),
	path
);

try_into!(
	TextureLoad,
	CheckeredTexture,
	TextureLoadError,
	CheckeredTexture::new(colour_one.into(), colour_two.into()),
	colour_one,
	colour_two
);

try_into!(
	TextureLoad,
	Lerp,
	TextureLoadError,
	Lerp::new(colour_one.into(), colour_two.into()),
	colour_one,
	colour_two
);

try_into!(TextureLoad, Perlin, TextureLoadError, Perlin::new(),);

impl TryInto<AllTextures> for TextureLoad {
	type Error = TextureLoadError;

	fn try_into(self) -> Result<AllTextures, Self::Error> {
		match self.tex_type.clone() {
			Some(t_type) => match &t_type[..] {
				"solid_colour" => Ok(AllTextures::SolidColour(self.try_into()?)),
				"lerp" => Ok(AllTextures::Lerp(self.try_into()?)),
				"perlin" => Ok(AllTextures::Perlin(Box::new(self.try_into()?))),
				"checkered" => Ok(AllTextures::CheckeredTexture(self.try_into()?)),
				"image" => Ok(AllTextures::ImageTexture(self.try_into()?)),
				_ => Err(TextureLoadError::InvalidType),
			},
			_ => Err(TextureLoadError::MissingField),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::parse::tests::EXAMPLE_TOML;

	#[test]
	pub fn parse_texture() {
		let value = EXAMPLE_TOML.parse::<Value>().unwrap();
		let _ = parse_textures(
			value,
			&["texture_four".to_string(), "texture_one".to_string()],
		)
		.unwrap();
	}
}
