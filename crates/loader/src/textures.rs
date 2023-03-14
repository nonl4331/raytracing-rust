use crate::*;

use implementations::*;

impl Load for AllTextures {
	fn load(props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let kind = match props.text("type") {
			Some(k) => k,
			None => return Err(LoadErr::MissingRequiredVariantType),
		};

		Ok(match kind {
			"checkered" => {
				let x = CheckeredTexture::load(props)?;
				(x.0, Self::CheckeredTexture(x.1))
			}
			"solid" => {
				let x = SolidColour::load(props)?;
				(x.0, Self::SolidColour(x.1))
			}
			"image" => {
				let x = ImageTexture::load(props)?;
				(x.0, Self::ImageTexture(x.1))
			}
			"lerp" => {
				let x = Lerp::load(props)?;
				(x.0, Self::Lerp(x.1))
			}
			"perlin" => {
				let x = Perlin::load(props)?;
				(x.0, Self::Perlin(Box::new(x.1)))
			}
			o => {
				return Err(LoadErr::MissingRequired(format!(
					"required a known value for texture type, found '{o}'"
				)))
			}
		})
	}
}

impl Load for CheckeredTexture {
	fn load(mut props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let primary = props.vec3("primary").unwrap_or(Vec3::one());
		let secondary = props.vec3("secondary").unwrap_or(Vec3::zero());
		let name = props.name();
		Ok((name, Self::new(primary, secondary)))
	}
}

impl Load for ImageTexture {
	fn load(mut props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let name = props.name();
		let filename = match props.text("filename") {
			Some(f) => f,
			None => return Err(LoadErr::MissingRequired("filename".to_string())),
		};
		Ok((name, Self::new(&filename)))
	}
}

impl Load for Perlin {
	fn load(mut props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let name = props.name();
		Ok((name, Self::new()))
	}
}

impl Load for Lerp {
	fn load(mut props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let primary = props.vec3("primary").unwrap_or(Vec3::one());
		let secondary = props.vec3("secondary").unwrap_or(Vec3::zero());
		let name = props.name();
		Ok((name, Self::new(primary, secondary)))
	}
}

impl Load for SolidColour {
	fn load(mut props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let colour = props.vec3("colour").unwrap_or(0.5 * Vec3::one());
		let name = props.name();
		Ok((name, Self::new(colour)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn coloured_texture() {
		let lookup = Lookup::new();
		let thing = "texture grey (
	type solid
	colour 0.5
)";
		let a = parser::from_str(thing).unwrap();
		let props = Properties::new(&lookup, &a[0]);
		let b = <AllTextures as Load>::load(props).unwrap();
		println!("{b:?}");
	}

	#[test]
	fn checkered_texture() {
		let lookup = Lookup::new();
		let thing = "texture checkered (
	type checkered
	primary 0.5 0.5 0.0
	secondary 0.0
)";
		let a = parser::from_str(thing).unwrap();
		let props = Properties::new(&lookup, &a[0]);
		let b = <AllTextures as Load>::load(props).unwrap();
		println!("{b:?}");
	}
}
