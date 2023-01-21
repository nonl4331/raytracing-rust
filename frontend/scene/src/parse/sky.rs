use crate::parse::*;
use implementations::AllTextures;
use serde::Deserialize;
use std::collections::HashMap;

use toml::Value;

use crate::*;

pub fn parse_skies(
	data: Value,
	sky_names: &[String],
	textures_map: &HashMap<String, Arc<AllTextures>>,
) -> Result<Vec<Sky<AllTextures>>, Error> {
	parse_items::<HashMap<String, Arc<AllTextures>>, SkyLoad, Sky<AllTextures>>(
		data,
		sky_names,
		textures_map,
	)
}

#[derive(Deserialize, Debug)]
pub struct SkyLoad {
	definition: Option<String>,
	texture: Option<String>,
	#[serde(skip)]
	texture_instance: Option<Arc<AllTextures>>,
	sample_res: Option<[usize; 2]>,
}

impl Load for SkyLoad {
	type LoadType = HashMap<String, Arc<AllTextures>>;
	fn derive_from(&mut self, other: Self) {
		if self.texture.is_none() {
			self.texture = other.texture;
		}
		if self.sample_res.is_none() {
			self.sample_res = other.sample_res;
		}
	}
	fn load(&mut self, textures_map: &Self::LoadType) -> Result<(), Error> {
		let tex_name = match &self.texture {
			Some(tex_name) => tex_name,
			None => return Err(MaterialLoadError::MissingField.into()),
		};

		match textures_map.get(tex_name) {
			Some(tex) => self.texture_instance = Some(tex.clone()),
			None => return Err(MaterialLoadError::TextureNotFound.into()),
		}

		Ok(())
	}
	fn get_definition(&self) -> Option<String> {
		self.definition.clone()
	}
	fn get_plural<'a>() -> &'a str {
		"skies"
	}
}

type SkyInstance = Sky<AllTextures>;

try_into!(
	SkyLoad,
	SkyInstance,
	SkyLoadError,
	Sky::new(&texture_instance, (sample_res[0], sample_res[1])),
	texture_instance,
	sample_res
);

#[derive(Debug)]
pub enum SkyLoadError {
	MissingField,
	InvalidType,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::parse::tests::EXAMPLE_TOML;
	use crate::parse::textures::parse_textures;

	#[test]
	pub fn parse_sky() {
		let value = EXAMPLE_TOML.parse::<Value>().unwrap();
		let texture_names = vec!["texture_four".to_string(), "texture_one".to_string()];
		let textures = parse_textures(value.clone(), &texture_names).unwrap();

		let textures: HashMap<String, Arc<AllTextures>> = HashMap::from_iter(
			texture_names
				.into_iter()
				.zip(textures.into_iter().map(Arc::new)),
		);

		let _ = parse_skies(value, &["sky_one".to_string()], &textures).unwrap();
	}
}
