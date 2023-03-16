use crate::Properties;
use crate::*;
use implementations::emissive::Emit;
use implementations::*;

impl<T: Texture> Load for AllMaterials<'_, T> {
	fn load(props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let kind = match props.text("type") {
			Some(k) => k,
			None => return Err(LoadErr::MissingRequiredVariantType),
		};

		Ok(match kind {
			"emissive" => {
				let x = Emit::load(props)?;
				(x.0, Self::Emit(x.1))
			}
			"lambertian" => {
				let x = Lambertian::load(props)?;
				(x.0, Self::Lambertian(x.1))
			}
			"reflect" => {
				let x = Reflect::load(props)?;
				(x.0, Self::Reflect(x.1))
			}
			"refract" => {
				let x = Refract::load(props)?;
				(x.0, Self::Refract(x.1))
			}
			"trowbridge_reitz" => {
				let x = TrowbridgeReitz::load(props)?;
				(x.0, Self::TrowbridgeReitz(x.1))
			}
			o => {
				return Err(LoadErr::MissingRequired(format!(
					"required a known value for material type, found '{o}'"
				)))
			}
		})
	}
}

impl<T: Texture> Load for Lambertian<'_, T> {
	fn load(mut props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let tex = props
			.texture("texture")
			.unwrap_or_else(|| props.default_texture());
		let albedo = props.float("albedo").unwrap_or(0.5);

		let name = props.name();

		Ok((name, Self::new(unsafe { &*(&*tex as *const _) }, albedo)))
	}
}

impl<T: Texture> Load for Emit<'_, T> {
	fn load(mut props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let tex = props
			.texture("texture")
			.unwrap_or_else(|| props.default_texture());
		let strength = props.float("strength").unwrap_or(1.5);

		let name = props.name();

		Ok((name, Self::new(unsafe { &*(&*tex as *const _) }, strength)))
	}
}

impl<T: Texture> Load for Reflect<'_, T> {
	fn load(mut props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let tex = props
			.texture("texture")
			.unwrap_or_else(|| props.default_texture());
		let fuzz = props.float("fuzz").unwrap_or(0.1);

		let name = props.name();

		Ok((name, Self::new(unsafe { &*(&*tex as *const _) }, fuzz)))
	}
}

impl<T: Texture> Load for Refract<'_, T> {
	fn load(mut props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let tex = props
			.texture("texture")
			.unwrap_or_else(|| props.default_texture());
		let eta = props.float("eta").unwrap_or(1.5);

		let name = props.name();

		Ok((name, Self::new(unsafe { &*(&*tex as *const _) }, eta)))
	}
}

impl<T: Texture> Load for TrowbridgeReitz<'_, T> {
	fn load(mut props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let tex = props
			.texture("texture")
			.unwrap_or_else(|| props.default_texture());
		let alpha = props.float("alpha").unwrap_or(0.5);
		let ior = props.vec3("ior").unwrap_or(Vec3::one());
		let metallic = props.float("metallic").unwrap_or(0.0);

		let name = props.name();

		Ok((
			name,
			Self::new(unsafe { &*(&*tex as *const _) }, alpha, ior, metallic),
		))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn lambertian() {
		let mut region = Region::new();
		let mut lookup = Lookup::new();
		let file = "
texture grey (
	type solid
	colour 0.5
)
material ground (
	type lambertian
	texture grey
	albedo 0.5
)";
		let data = parser::from_str(file).unwrap();
		let textures = load_textures::<AllTextures>(&data, &lookup).unwrap();
		region_insert_with_lookup(&mut region, textures, |n, t| lookup.texture_insert(n, t));
		let _ = load_materials::<AllMaterials<AllTextures>>(&data, &lookup).unwrap();
	}
}
