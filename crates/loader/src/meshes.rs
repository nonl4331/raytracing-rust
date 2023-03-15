use crate::Properties;
use crate::*;
use implementations::triangle::MeshData;
use implementations::triangle::MeshTriangle;
use implementations::*;

impl<M: Scatter> Load for Vec<AllPrimitives<'_, M>> {
	fn load(props: Properties) -> Result<(Option<String>, Self), LoadErr> {
		let kind = match props.text("type") {
			Some(k) => k,
			None => return Err(LoadErr::MissingRequiredVariantType),
		};
		match kind {
			"mesh" => {
				unimplemented!()
			}
			"aacuboid" => cuboid(props),
			o => {
				return Err(LoadErr::MissingRequired(format!(
					"required a known value for mesh type, found '{o}'"
				)))
			}
		}
	}
}

fn cuboid<'a, M: Scatter>(
	props: Properties,
) -> Result<(Option<String>, Vec<AllPrimitives<'a, M>>), LoadErr> {
	let mat: region::RegionRes<M> = props
		.scatter("material")
		.unwrap_or_else(|| props.default_scatter());
	let point_one = match props.vec3("point_one") {
		Some(c) => c,
		None => {
			return Err(LoadErr::MissingRequired(
				"expected point_one on aacubiod, found nothing".to_string(),
			))
		}
	};
	let point_two = match props.vec3("point_two") {
		Some(c) => c,
		None => {
			return Err(LoadErr::MissingRequired(
				"expected point_two on aacubiod, found nothing".to_string(),
			))
		}
	};

	let min = point_one.min_by_component(point_two);
	let max = point_one.max_by_component(point_two);

	let points = vec![
		min,                            // 0
		Vec3::new(max.x, min.y, min.z), // 1
		Vec3::new(max.x, max.y, min.z), // 2
		Vec3::new(min.x, max.y, min.z), // 3
		Vec3::new(min.x, min.y, max.z), // 4
		Vec3::new(max.x, min.y, max.z), // 5
		max,                            // 6
		Vec3::new(min.x, max.y, max.z), // 7
	];

	let normals = vec![
		Vec3::x(),  // 0
		-Vec3::x(), // 1
		Vec3::y(),  // 2
		-Vec3::y(), // 3
		Vec3::z(),  // 4
		-Vec3::z(), // 5
	];

	let mesh_data = std::sync::Arc::new(MeshData::new(points, normals));
	std::mem::forget(mesh_data.clone()); // prevent drop when primitives get moved to region

	macro_rules! mesh_tri {
		($p:expr, $normal:expr) => {
			AllPrimitives::MeshTriangle(MeshTriangle::new(
				$p,
				[$normal; 3],
				unsafe { &*(&*mat as *const _) },
				mesh_data.clone(),
			))
		};
	}

	let triangles = vec![
		mesh_tri!([0, 1, 2], 5),
		mesh_tri!([0, 2, 3], 5),
		mesh_tri!([0, 1, 5], 3),
		mesh_tri!([0, 5, 4], 3),
		mesh_tri!([1, 2, 5], 0),
		mesh_tri!([2, 5, 6], 0),
		mesh_tri!([2, 3, 7], 2),
		mesh_tri!([2, 6, 7], 2),
		mesh_tri!([0, 3, 4], 1),
		mesh_tri!([3, 4, 7], 1),
		mesh_tri!([4, 5, 6], 4),
		mesh_tri!([4, 6, 7], 4),
	];

	Ok((None, triangles))
}
