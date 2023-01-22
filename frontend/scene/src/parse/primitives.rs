use crate::implementations::{sphere::Sphere, AllMaterials, AllPrimitives, AllTextures};
use crate::parse::*;
use crate::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use toml::Value;

type Material = AllMaterials<AllTextures>;

pub fn parse_primitives(
	data: Value,
	primitive_names: &[String],
	materials_map: &mut HashMap<String, Arc<Material>>,
) -> Result<Vec<AllPrimitives<Material>>, Error> {
	parse_items::<
		HashMap<String, Arc<AllMaterials<AllTextures>>>,
		PrimitiveLoad,
		AllPrimitives<AllMaterials<AllTextures>>,
	>(data, primitive_names, materials_map)
}

#[derive(Deserialize, Debug)]
pub struct PrimitiveLoad {
	definition: Option<String>,
	#[serde(rename = "type")]
	primitive_type: Option<String>,
	material: Option<String>,
	#[serde(skip)]
	material_instance: Option<Arc<AllMaterials<AllTextures>>>,
	position: Option<[Float; 3]>,
	radius: Option<Float>,
}

impl Load for PrimitiveLoad {
	type LoadType = HashMap<String, Arc<AllMaterials<AllTextures>>>;
	fn load(&mut self, materials_map: &mut Self::LoadType) -> Result<(), Error> {
		let mat_name = match &self.material {
			Some(mat_name) => mat_name,
			None => return Err(PrimitiveLoadError::MissingField.into()),
		};

		match materials_map.get(mat_name) {
			Some(mat) => self.material_instance = Some(mat.clone()),
			None => return Err(PrimitiveLoadError::MaterialNotFound.into()),
		}

		Ok(())
	}
	fn derive_from(&mut self, other: Self) {
		if self.primitive_type.is_none() {
			self.primitive_type = other.primitive_type;
		}
		if self.material.is_none() {
			self.material = other.material;
		}
		if self.radius.is_none() {
			self.radius = other.radius;
		}
	}
	fn get_definition(&self) -> Option<String> {
		self.definition.clone()
	}
	fn get_plural<'a>() -> &'a str {
		"primitives"
	}
}

#[derive(Debug)]
pub enum PrimitiveLoadError {
	MissingField,
	InvalidType,
	MaterialNotFound,
}

type SphereInstance = Sphere<Material>;

try_into!(
	PrimitiveLoad,
	SphereInstance,
	PrimitiveLoadError,
	Sphere::new(position.into(), radius, &material_instance),
	position,
	radius,
	material_instance
);

impl TryInto<AllPrimitives<Material>> for PrimitiveLoad {
	type Error = PrimitiveLoadError;

	fn try_into(self) -> Result<AllPrimitives<Material>, Self::Error> {
		match self.primitive_type.clone() {
			Some(m_type) => match &m_type[..] {
				"sphere" => Ok(AllPrimitives::Sphere(self.try_into()?)),
				_ => Err(PrimitiveLoadError::InvalidType),
			},
			_ => Err(PrimitiveLoadError::MissingField),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::parse::materials::parse_materials;
	use crate::parse::tests::EXAMPLE_TOML;
	use crate::parse::textures::parse_textures;

	#[test]
	pub fn parse_primitive() {
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
		let mut materials: HashMap<String, Arc<Material>> = HashMap::from_iter(
			material_names
				.into_iter()
				.zip(materials.into_iter().map(Arc::new)),
		);

		let _ = parse_primitives(value, &["sphere_one".to_string()], &mut materials).unwrap();
	}
}
