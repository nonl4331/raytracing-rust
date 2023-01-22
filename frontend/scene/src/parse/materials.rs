use crate::{parse::*, *};
use implementations::{AllMaterials, AllTextures, Lambertian};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

use toml::Value;

use super::parse_items;

pub fn parse_materials(
	data: Value,
	material_names: &[String],
	textures_map: &mut HashMap<String, Arc<AllTextures>>,
) -> Result<Vec<AllMaterials<AllTextures>>, Error> {
	parse_items::<HashMap<String, Arc<AllTextures>>, MaterialLoad, AllMaterials<AllTextures>>(
		data,
		material_names,
		textures_map,
	)
}

#[derive(Deserialize, Debug)]
pub struct MaterialLoad {
	definition: Option<String>,
	#[serde(rename = "type")]
	mat_type: Option<String>,
	texture: Option<String>,
	#[serde(skip)]
	texture_instance: Option<Arc<AllTextures>>,
	absorbtion: Option<Float>,
}

impl Load for MaterialLoad {
	type LoadType = HashMap<String, Arc<AllTextures>>;
	fn derive_from(&mut self, other: Self) {
		if self.mat_type.is_none() {
			self.mat_type = other.mat_type;
		}
		if self.texture.is_none() {
			self.texture = other.texture;
		}
		if self.absorbtion.is_none() {
			self.absorbtion = other.absorbtion;
		}
	}
	fn load(&mut self, textures_map: &mut Self::LoadType) -> Result<(), Error> {
		let tex_name = match &self.texture {
			Some(tex_name) => tex_name,
			None => return Err(MaterialLoadError::MissingField.into()),
		};

		match textures_map.get(tex_name) {
			Some(tex) => self.texture_instance = Some(tex.clone()),
			None => return Err(MaterialLoadError::TextureNotFound.into()),
		};

		Ok(())
	}
	fn get_definition(&self) -> Option<String> {
		self.definition.clone()
	}
	fn get_plural<'a>() -> &'a str {
		"materials"
	}
}

#[derive(Debug)]
pub enum MaterialLoadError {
	MissingField,
	InvalidType,
	TextureNotFound,
}

type Lambert = Lambertian<AllTextures>;

try_into!(
	MaterialLoad,
	Lambert,
	MaterialLoadError,
	Lambertian::new(&texture_instance, absorbtion),
	texture_instance,
	absorbtion
);

impl TryInto<AllMaterials<AllTextures>> for MaterialLoad {
	type Error = MaterialLoadError;

	fn try_into(self) -> Result<AllMaterials<AllTextures>, Self::Error> {
		match self.mat_type.clone() {
			Some(m_type) => match &m_type[..] {
				"lambertian" => Ok(AllMaterials::Lambertian(self.try_into()?)),
				_ => Err(MaterialLoadError::InvalidType),
			},
			_ => Err(MaterialLoadError::MissingField),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::parse::tests::EXAMPLE_TOML;
	use crate::parse::textures::parse_textures;

	#[test]
	pub fn parse_material() {
		let value = EXAMPLE_TOML.parse::<Value>().unwrap();
		let texture_names = vec!["texture_four".to_string(), "texture_one".to_string()];
		let textures = parse_textures(value.clone(), &texture_names).unwrap();

		let mut textures: HashMap<String, Arc<AllTextures>> = HashMap::from_iter(
			texture_names
				.into_iter()
				.zip(textures.into_iter().map(Arc::new)),
		);

		let _ = parse_materials(value, &["material_one".to_string()], &mut textures).unwrap();
	}
}
