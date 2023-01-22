use crate::parse::parse_item;
use crate::parse::Error;
use crate::parse::Load;
use crate::Scene;
use implementations::AllPrimitives;
use implementations::Bvh;
use implementations::SimpleCamera;
use implementations::Sky;
use toml::Value;

use implementations::AllMaterials;

use implementations::AllTextures;

use crate::*;
use implementations::random_sampler::RandomSampler;
use serde::Deserialize;

type Material = AllMaterials<AllTextures>;
type Primitive = AllPrimitives<Material>;
type BvhInstance = Bvh<Primitive, AllMaterials<AllTextures>>;
type SceneInstance = Scene<Primitive, Material, RandomSampler, BvhInstance, AllTextures>;
type Passthrough = (
	Option<BvhInstance>,
	Option<SimpleCamera>,
	Option<Sky<AllTextures>>,
);

pub fn parse_scene(
	data: Value,
	name: String,
	passthrough_data: &mut Passthrough,
) -> Result<SceneInstance, Error> {
	let data = match data.get(name.clone()) {
		Some(value) => value,
		None => return Err(Error::NoValue(name)),
	};
	let load: SceneLoad = parse_item::<Passthrough, SceneLoad>(data.clone(), passthrough_data)?;
	Ok(load.try_into()?)
}

#[derive(Deserialize)]
pub struct SceneLoad {
	definition: Option<String>,
	camera: Option<String>,
	sky: Option<String>,
	bvh: Option<String>,
	#[serde(skip)]
	bvh_instance: Option<Bvh<Primitive, Material>>,
	#[serde(skip)]
	camera_instance: Option<SimpleCamera>,
	#[serde(skip)]
	sky_instance: Option<Sky<AllTextures>>,
}

#[derive(Debug)]
pub enum SceneLoadError {
	MissingField,
	InvalidType,
	BvhNotFound,
	CameraNotFound,
	SkyNotFound,
	PrimitiveNotFound,
}

impl Load for SceneLoad {
	type LoadType = (
		Option<BvhInstance>,
		Option<SimpleCamera>,
		Option<Sky<AllTextures>>,
	);
	fn load(&mut self, data: &mut Self::LoadType) -> Result<(), Error> {
		self.bvh_instance = data.0.take();
		self.camera_instance = data.1.take();
		self.sky_instance = data.2.take();

		Ok(())
	}
	fn derive_from(&mut self, other: Self) {
		if self.camera.is_none() {
			self.camera = other.camera;
		}
		if self.sky.is_none() {
			self.sky = other.sky;
		}
		if self.bvh.is_none() {
			self.bvh = other.bvh;
		}
	}
	fn get_definition(&self) -> Option<String> {
		self.definition.clone()
	}
	fn get_plural<'a>() -> &'a str {
		"scenes"
	}
}

try_into!(
	SceneLoad,
	SceneInstance,
	SceneLoadError,
	Scene::new(
		Arc::new(camera_instance),
		Arc::new(sky_instance),
		Arc::new(RandomSampler {}),
		Arc::new(bvh_instance),
	),
	bvh_instance,
	camera_instance,
	sky_instance
);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::parse::acceleration_structures::parse_accleration_structure;
	use crate::parse::camera::parse_cameras;
	use crate::parse::materials::parse_materials;
	use crate::parse::primitives::parse_primitives;
	use crate::parse::sky::parse_skies;
	use crate::parse::tests::EXAMPLE_TOML;
	use crate::parse::textures::parse_textures;
	use std::collections::HashMap;

	#[test]
	pub fn scene() {
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

		let bvh =
			parse_accleration_structure(value.clone(), "bvh_one".to_string(), &mut primitives)
				.unwrap();
		let camera = parse_cameras(value.clone(), &["camera_one".to_string()])
			.unwrap()
			.remove(0);
		let sky = parse_skies(value.clone(), &["sky_one".to_string()], &mut textures)
			.unwrap()
			.remove(0);
		let _ = parse_scene(
			value,
			"scene_one".to_string(),
			&mut (Some(bvh), Some(camera), Some(sky)),
		)
		.unwrap();
	}
}
