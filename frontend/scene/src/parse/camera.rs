use crate::parse::*;

use serde::Deserialize;

use toml::Value;

use crate::*;

pub fn parse_cameras(data: Value, camera_names: &[String]) -> Result<Vec<SimpleCamera>, Error> {
	parse_items::<(), CameraLoad, SimpleCamera>(data, camera_names, &())
}

#[derive(Debug)]
pub enum CameraLoadError {
	MissingField,
	InvalidType,
}

#[derive(Deserialize, Debug)]
pub struct CameraLoad {
	definition: Option<String>,
	#[serde(rename = "type")]
	camera_type: Option<String>,
	origin: Option<[Float; 3]>,
	lookat: Option<[Float; 3]>,
	vup: Option<[Float; 3]>,
	fov: Option<Float>,
	aspect_ratio: Option<Float>,
	aperture: Option<Float>,
	focus_dist: Option<Float>,
}

impl Load for CameraLoad {
	type LoadType = ();
	fn derive_from(&mut self, other: Self) {
		if self.camera_type.is_none() {
			self.camera_type = other.camera_type;
		}
		if self.origin.is_none() {
			self.origin = other.origin;
		}
		if self.lookat.is_none() {
			self.lookat = other.lookat;
		}
		if self.vup.is_none() {
			self.vup = other.vup;
		}
		if self.fov.is_none() {
			self.fov = other.fov;
		}
		if self.aspect_ratio.is_none() {
			self.aspect_ratio = other.aspect_ratio;
		}
		if self.aperture.is_none() {
			self.aperture = other.aperture;
		}
		if self.focus_dist.is_none() {
			self.focus_dist = other.focus_dist;
		}
	}
	fn get_definition(&self) -> Option<String> {
		self.definition.clone()
	}
	fn get_plural<'a>() -> &'a str {
		"cameras"
	}
}

try_into!(
	CameraLoad,
	SimpleCamera,
	CameraLoadError,
	SimpleCamera::new(
		origin.into(),
		lookat.into(),
		vup.into(),
		fov,
		aspect_ratio,
		aperture,
		focus_dist,
	),
	origin,
	lookat,
	vup,
	fov,
	aspect_ratio,
	aperture,
	focus_dist
);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::parse::tests::EXAMPLE_TOML;

	#[test]
	pub fn parse_texture() {
		let value = EXAMPLE_TOML.parse::<Value>().unwrap();
		let _ = parse_cameras(value, &["camera_one".to_string()]).unwrap();
	}
}
