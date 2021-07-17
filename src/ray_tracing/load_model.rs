use crate::ray_tracing::{
    material::{Diffuse, Material},
    primitives::{Triangle, TriangleMesh},
    ray::Colour,
    texture::{SolidColour, Texture},
    tracing::Primitive,
};

use std::sync::Arc;

use tobj;

use ultraviolet::Vec3;

pub fn load_model(filepath: &str) -> Primitive {
    let model = tobj::load_obj(
        filepath,
        &tobj::LoadOptions {
            ignore_points: true,
            ignore_lines: true,
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    );

    assert!(model.is_ok());

    let (mut models, _) = model.unwrap();

    let mesh = &mut models[0].mesh;

    assert!(mesh.indices.len() % 3 == 0);

    let num_triangles = mesh.indices.len() / 3;

    let indices: Vec<u32> = mesh.indices.drain(..).collect();

    let points: Vec<Vec3> = mesh
        .positions
        .chunks(3)
        .map(|point| Vec3::new(point[0], point[1], point[2]))
        .collect();

    let mut mesh = Vec::new();

    let tex = Texture::SolidColour(SolidColour {
        colour: Colour {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        },
    });

    let material = Arc::new(Material::Diffuse(Diffuse::new(tex, 0.5)));

    let mut min = Vec3::zero();
    let mut max = Vec3::one();

    for i in 0..num_triangles {
        let triangle_points = [
            points[indices[i * 3] as usize],
            points[indices[i * 3 + 1] as usize],
            points[indices[i * 3 + 2] as usize],
        ];
        if i != 0 {
            min = min.min_by_component(
                triangle_points[0]
                    .min_by_component(triangle_points[1].min_by_component(triangle_points[2])),
            );
            max = max.max_by_component(
                triangle_points[0]
                    .max_by_component(triangle_points[1].max_by_component(triangle_points[2])),
            );
        } else {
            min = triangle_points[0]
                .min_by_component(triangle_points[1].min_by_component(triangle_points[2]));
            max = triangle_points[0]
                .max_by_component(triangle_points[1].max_by_component(triangle_points[2]));
        }
        mesh.push(Triangle::new_from_arc(
            triangle_points,
            None,
            material.clone(),
        ))
    }

    Primitive::TriangleMesh(TriangleMesh::new(min, max, mesh, material))
}
