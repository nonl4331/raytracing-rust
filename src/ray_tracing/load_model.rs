use crate::ray_tracing::{
    material::{Diffuse, Material},
    primitives::{Triangle, TriangleMesh},
    ray::Colour,
    texture::{SolidColour, Texture},
    tracing::Primitive,
};

use std::sync::Arc;

use ultraviolet::Vec3;

use wavefront_obj;

pub fn load_model(filepath: &str) -> Vec<Primitive> {
    let model = wavefront_obj::obj::parse(&std::fs::read_to_string(filepath).unwrap());

    let model = model.unwrap();

    let tex = Texture::SolidColour(SolidColour {
        colour: Colour {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        },
    });

    let material = Arc::new(Material::Diffuse(Diffuse::new(tex, 0.5)));

    let mut primitives = Vec::new();

    for object in model.objects {
        let mut mesh = Vec::new();
        let mut min = None;
        let mut max = None;
        let vertices = &object.vertices;

        for geometric_object in object.geometry {
            for shape in geometric_object.shapes {
                match shape.primitive {
                    wavefront_obj::obj::Primitive::Triangle(i1, i2, i3) => {
                        let points = [
                            vertex_to_vec3(vertices[i1.0 as usize]),
                            vertex_to_vec3(vertices[i2.0 as usize]),
                            vertex_to_vec3(vertices[i3.0 as usize]),
                        ];
                        let triangle = Triangle::new_from_arc(points, None, material.clone());
                        match (min, max) {
                            (None, None) => {
                                min = Some(triangle.aabb.min);
                                max = Some(triangle.aabb.max);
                            }
                            (_, _) => {
                                min = Some(min.unwrap().min_by_component(triangle.aabb.min));
                                max = Some(max.unwrap().max_by_component(triangle.aabb.max))
                            }
                        }
                        mesh.push(triangle)
                    }
                    _ => {}
                }
            }
        }

        if mesh.len() != 0 {
            primitives.push(Primitive::TriangleMesh(TriangleMesh::new(
                min.unwrap(),
                max.unwrap(),
                mesh,
                material.clone(),
            )));
        }
    }
    primitives
}

fn vertex_to_vec3(vertex: wavefront_obj::obj::Vertex) -> Vec3 {
    Vec3::new(vertex.x as f32, vertex.y as f32, vertex.z as f32)
}
