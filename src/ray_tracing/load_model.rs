use crate::math::Float;
use crate::ray_tracing::{
    material::Material,
    primitives::{Primitive, Triangle, TriangleMesh},
};

use std::sync::Arc;

use ultraviolet::Vec3;

use wavefront_obj;

pub fn load_model(filepath: &str, material: &Arc<Material>) -> Vec<Primitive> {
    let model = wavefront_obj::obj::parse(&std::fs::read_to_string(filepath).unwrap());

    let model = model.unwrap();

    let material = Arc::new(material);

    let mut primitives = Vec::new();

    for object in model.objects {
        let mut mesh = Vec::new();
        let vertices = &object.vertices;
        let normals = &object.normals;

        for geometric_object in object.geometry {
            for shape in geometric_object.shapes {
                match shape.primitive {
                    wavefront_obj::obj::Primitive::Triangle(i1, i2, i3) => {
                        let points = [
                            vertex_to_vec3(vertices[i1.0 as usize]),
                            vertex_to_vec3(vertices[i2.0 as usize]),
                            vertex_to_vec3(vertices[i3.0 as usize]),
                        ];

                        if i1.2.is_none() {
                            panic!("Please export obj file with vertex normals!");
                        }

                        let tri_normals = [
                            vertex_to_vec3(normals[i1.2.unwrap()]),
                            vertex_to_vec3(normals[i2.2.unwrap()]),
                            vertex_to_vec3(normals[i3.2.unwrap()]),
                        ];

                        let triangle = Triangle::new(points, tri_normals, &material);

                        mesh.push(triangle)
                    }
                    _ => {}
                }
            }
        }

        if mesh.len() != 0 {
            primitives.push(Primitive::TriangleMesh(TriangleMesh::new(mesh, &material)));
        }
    }
    primitives
}

fn vertex_to_vec3(vertex: wavefront_obj::obj::Vertex) -> Vec3 {
    Vec3::new(vertex.x as Float, vertex.y as Float, vertex.z as Float)
}
