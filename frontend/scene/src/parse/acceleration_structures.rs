use crate::parse::parse_item;
use crate::parse::Error;
use crate::parse::Load;
use implementations::AllMaterials;
use implementations::AllPrimitives;
use implementations::Bvh;
use toml::Value;

use implementations::split::SplitType;
use implementations::AllTextures;

use crate::*;
use serde::Deserialize;

type PrimitiveInstance = AllPrimitives<AllMaterials<AllTextures>>;
type BvhInstance = Bvh<PrimitiveInstance, AllMaterials<AllTextures>>;

pub fn parse_accleration_structure(
	data: Value,
	name: String,
	primitives_map: &mut Vec<PrimitiveInstance>,
) -> Result<BvhInstance, Error> {
	let data = match data.get(name.clone()) {
		Some(value) => value,
		None => return Err(Error::NoValue(name)),
	};
	Ok(
		parse_item::<Vec<PrimitiveInstance>, AccelerationStructureLoad>(
			data.clone(),
			primitives_map,
		)?
		.try_into()?,
	)
}

#[derive(Deserialize, Debug)]
pub struct AccelerationStructureLoad {
	definition: Option<String>,
	#[serde(rename = "type")]
	split_type: Option<String>,
	#[serde(skip)]
	primitive_instances: Vec<PrimitiveInstance>,
}

#[derive(Debug)]
pub enum AccelerationStructureLoadError {
	MissingField,
	InvalidType,
	PrimitiveNotFound,
}

impl Load for AccelerationStructureLoad {
	type LoadType = Vec<PrimitiveInstance>;
	fn load(&mut self, primitives_vec: &mut Self::LoadType) -> Result<(), Error> {
		self.primitive_instances = primitives_vec.drain(..).collect();

		Ok(())
	}
	fn derive_from(&mut self, other: Self) {
		if self.split_type.is_none() {
			self.split_type = other.split_type;
		}
	}
	fn get_definition(&self) -> Option<String> {
		self.definition.clone()
	}
	fn get_plural<'a>() -> &'a str {
		unreachable!()
	}
}

impl TryInto<BvhInstance> for AccelerationStructureLoad {
	type Error = AccelerationStructureLoadError;

	fn try_into(self) -> Result<BvhInstance, Self::Error> {
		let split_type = match self.split_type {
			Some(v) => v,
			None => return Err(AccelerationStructureLoadError::MissingField),
		};

		Ok(Bvh::new(self.primitive_instances, {
			match &split_type[..] {
				"sah" => SplitType::Sah,
				"middle" => SplitType::Middle,
				"equal_counts" => SplitType::EqualCounts,
				_ => return Err(AccelerationStructureLoadError::InvalidType),
			}
		}))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::parse::materials::parse_materials;
	use crate::parse::primitives::parse_primitives;
	use crate::parse::tests::EXAMPLE_TOML;
	use crate::parse::textures::parse_textures;
	use std::collections::HashMap;

	#[test]
	pub fn parse_bvh() {
		let value = EXAMPLE_TOML.parse::<Value>().unwrap();
		let texture_names = vec!["texture_four".to_string(), "texture_one".to_string()];
		let textures = parse_textures(value.clone(), &texture_names).unwrap();

		let mut textures: HashMap<String, Arc<AllTextures>> = HashMap::from_iter(
			texture_names
				.into_iter()
				.zip(textures.into_iter().map(Arc::new)),
		);

		let material_names = vec!["material_one".to_string()];
		let materials = parse_materials(value.clone(), &material_names, &mut textures).unwrap();
		let mut materials: HashMap<String, Arc<AllMaterials<AllTextures>>> = HashMap::from_iter(
			material_names
				.into_iter()
				.zip(materials.into_iter().map(Arc::new)),
		);

		let mut primitives =
			parse_primitives(value.clone(), &["sphere_one".to_string()], &mut materials).unwrap();

		let _ = parse_accleration_structure(value, "bvh_one".to_string(), &mut primitives).unwrap();
	}
}
