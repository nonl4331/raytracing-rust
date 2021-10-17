use crate::math::Float;
use crate::ray_tracing::{
    material::Material,
    primitives::{MeshData, MeshTriangle, Primitive},
};

use std::sync::Arc;

use crate::utility::vec::Vec3;

pub fn load_model(filepath: &str, material: &Arc<Material>) -> Vec<Primitive> {
    let model = wavefront_obj::obj::parse(&std::fs::read_to_string(filepath).unwrap());

    let model = model.unwrap();

    let material = Arc::new(material);

    let mut primitives = Vec::new();

    for object in model.objects {
        let mesh_data = Arc::new(MeshData::new(
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
            &material,
        ));

        for geometric_object in object.geometry {
            for shape in geometric_object.shapes {
                if let wavefront_obj::obj::Primitive::Triangle(i1, i2, i3) = shape.primitive {
                    if i1.2.is_none() {
                        panic!("Please export obj file with vertex normals!");
                    }

                    let triangle = Primitive::MeshTriangle(MeshTriangle::new(
                        [i1.0, i2.0, i3.0],
                        [i1.2.unwrap(), i2.2.unwrap(), i3.2.unwrap()],
                        &material,
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
