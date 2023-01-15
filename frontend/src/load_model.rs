use implementations::{
	rt_core::*,
	triangle::{MeshData, MeshTriangle},
	AllMaterials, AllPrimitives, AllTextures,
};
use std::sync::Arc;

pub fn load_model_with_materials(
	filepath: &str,
	materials: &[(Arc<AllMaterials<AllTextures>>, &str)],
) -> Vec<AllPrimitives<AllMaterials<AllTextures>>> {
	let model = wavefront_obj::obj::parse(&std::fs::read_to_string(filepath).unwrap()).unwrap();

	let mut primitives: Vec<AllPrimitives<AllMaterials<AllTextures>>> = Vec::new();

	for object in model.objects {
		let mesh_data: Arc<MeshData> = Arc::new(MeshData::new(
			object
				.vertices
				.iter()
				.map(|vertex| vertex_to_vec3(*vertex))
				.collect(),
			object
				.normals
				.iter()
				.map(|normal| vertex_to_vec3(*normal))
				.collect(),
		));

		for geometric_object in object.geometry {
			for shape in geometric_object.shapes {
				if let wavefront_obj::obj::Primitive::Triangle(i1, i2, i3) = shape.primitive {
					if i1.2.is_none() {
						panic!("Please export obj file with vertex normals!");
					}

					let mat = get_material(
						materials,
						geometric_object
							.material_name
							.as_ref()
							.unwrap_or(&"default".to_owned()),
					);

					let triangle: AllPrimitives<AllMaterials<AllTextures>> =
						AllPrimitives::MeshTriangle(MeshTriangle::new(
							[i1.0, i2.0, i3.0],
							[i1.2.unwrap(), i2.2.unwrap(), i3.2.unwrap()],
							&mat,
							&mesh_data,
						));

					primitives.push(triangle)
				}
			}
		}
	}
	primitives
}

fn vertex_to_vec3(vertex: wavefront_obj::obj::Vertex) -> Vec3 {
	Vec3::new(vertex.x as Float, vertex.y as Float, vertex.z as Float)
}

fn get_material(
	materials: &[(Arc<AllMaterials<AllTextures>>, &str)],
	name: &str,
) -> Arc<AllMaterials<AllTextures>> {
	let mat = materials.iter().find(|&v| v.1 == name);

	match mat {
		Some(mat) => mat.0.clone(),
		None => {
			let mat = materials.iter().find(|&v| v.1 == "default");
			if let Some(mat) = mat {
				return mat.0.clone();
			}
			panic!("{name} material not found");
		}
	}
}
