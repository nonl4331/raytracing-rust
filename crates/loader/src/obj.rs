use crate::Float;
use crate::Properties;
use crate::Scatter;
use crate::Vec3;
use implementations::{
	triangle::{MeshData, MeshTriangle},
	AllPrimitives,
};
use std::sync::Arc;

pub fn load_obj<'a, M: Scatter>(filepath: &str, props: Properties) -> Vec<AllPrimitives<'a, M>> {
	let model = wavefront_obj::obj::parse(&std::fs::read_to_string(filepath).unwrap()).unwrap();

	let mut primitives: Vec<AllPrimitives<'a, M>> = Vec::new();

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

					let mat: region::RegionRes<M> = props
						.lookup_material(
							geometric_object
								.material_name
								.as_ref()
								.unwrap_or(&"default".to_owned()),
						)
						.unwrap_or_else(|| props.default_scatter());

					let triangle: AllPrimitives<'a, M> =
						AllPrimitives::MeshTriangle(MeshTriangle::new(
							[i1.0, i2.0, i3.0],
							[i1.2.unwrap(), i2.2.unwrap(), i3.2.unwrap()],
							unsafe { &*(&*mat as *const _) },
							mesh_data.clone(),
						));

					primitives.push(triangle)
				}
			}
		}
		std::mem::forget(mesh_data);
	}
	primitives
}

fn vertex_to_vec3(vertex: wavefront_obj::obj::Vertex) -> Vec3 {
	Vec3::new(vertex.x as Float, vertex.y as Float, vertex.z as Float)
}
