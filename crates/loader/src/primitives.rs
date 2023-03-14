use crate::Properties;
use crate::*;
use implementations::sphere::Sphere;
use implementations::*;

use rt_core::Scatter;

impl<M: Scatter> Load for Sphere<'_, M> {
	fn load(props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let mat: region::RegionRes<M> = props
			.scatter("material")
			.unwrap_or_else(|| props.default_scatter());
		let radius = props.float("radius").unwrap_or(1.0);
		let centre = match props.vec3("centre") {
			Some(c) => c,
			None => {
				return Err(LoadErr::MissingRequired(
					"expected centre on sphere, found nothing".to_string(),
				))
			}
		};

		Ok((
			None,
			Self::new(centre, radius, unsafe { &*(&*mat as *const _) }),
		))
	}
}

impl<M: Scatter> Load for AllPrimitives<'_, M> {
	fn load(props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let kind = match props.text("type") {
			Some(k) => k,
			None => return Err(LoadErr::MissingRequiredVariantType),
		};

		Ok(match kind {
			"sphere" => {
				let x = Sphere::load(props)?;
				(x.0, Self::Sphere(x.1))
			}
			_ => todo!(),
			/*o => {
				return Err(LoadErr::MissingRequired(format!(
					"required a known value for material type, found '{o}'"
				)))
			}*/
		})
	}
}

// TODO LOAD FOR TRIANGLE & MESH

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sphere() {
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
)
primitive (
	type sphere
	material ground
	centre 0 -1000 0
	radius 1000
)";
		let data = parser::from_str(file).unwrap();
		let textures = load_textures::<AllTextures>(&data, &lookup).unwrap();

		region_insert_with_lookup(&mut region, textures, |n, t| lookup.texture_insert(n, t));

		let materials = load_materials::<AllMaterials<AllTextures>>(&data, &lookup).unwrap();

		region_insert_with_lookup(&mut region, materials, |n, t| lookup.scatter_insert(n, t));

		load_primitives::<AllPrimitives<AllMaterials<AllTextures>>>(&data, &lookup).unwrap();
	}
}
