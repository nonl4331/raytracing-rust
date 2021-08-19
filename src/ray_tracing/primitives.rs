use crate::math::Float;
use crate::ray_tracing::{material::Material, tracing::PrimitiveTrait};

use std::sync::Arc;

use ultraviolet::{Vec2, Vec3};

pub enum Primitive {
    Sphere(Sphere),
    AARect(AARect),
    AACuboid(AACuboid),
    Triangle(Triangle),
    TriangleMesh(TriangleMesh),
    None,
}

#[derive(Clone, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn get_axis_value(&self, point: Vec3) -> Float {
        match self {
            Axis::X => point.x,
            Axis::Y => point.y,
            Axis::Z => point.z,
        }
    }

    pub fn point_without_axis(&self, point: Vec3) -> Vec2 {
        match self {
            Axis::X => Vec2::new(point.y, point.z),
            Axis::Y => Vec2::new(point.x, point.z),
            Axis::Z => Vec2::new(point.x, point.y),
        }
    }
    pub fn return_point_with_axis(&self, dir: Vec3) -> Vec3 {
        match self {
            Axis::X => Vec3::new(dir.x, 0.0, 0.0),
            Axis::Y => Vec3::new(0.0, dir.y, 0.0),
            Axis::Z => Vec3::new(0.0, 0.0, dir.z),
        }
    }

    pub fn get_max_axis(vec: &Vec3) -> Self {
        if vec.x > vec.y && vec.x > vec.z {
            Axis::X
        } else if vec.y > vec.z {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    pub fn get_max_abs_axis(vec: &Vec3) -> Self {
        if vec.x.abs() > vec.y.abs() && vec.x.abs() > vec.z.abs() {
            Axis::X
        } else if vec.y.abs() > vec.z.abs() {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    pub fn swap_z(vec: &mut Vec3, axis: &Self) {
        match axis {
            Axis::X => {
                std::mem::swap(&mut vec.x, &mut vec.z);
            }
            Axis::Y => {
                std::mem::swap(&mut vec.x, &mut vec.z);
            }
            _ => {}
        }
    }

    pub fn point_from_2d(vec: &Vec2, axis: &Axis, axis_value: Float) -> Vec3 {
        match axis {
            Axis::X => Vec3::new(axis_value, vec.x, vec.y),
            Axis::Y => Vec3::new(vec.x, axis_value, vec.y),
            Axis::Z => Vec3::new(vec.x, vec.y, axis_value),
        }
    }
}

#[derive(Clone)]
pub struct AARect {
    pub min: Vec2,
    pub max: Vec2,
    pub k: Float,
    pub axis: Axis,
    pub material: Arc<Material>,
}

impl AARect {
    pub fn new(point_one: Vec2, point_two: Vec2, k: Float, axis: Axis, material: Material) -> Self {
        if point_one == point_two {
            panic!("AARect called with two of the same point!");
        }
        let min = point_one.min_by_component(point_two);
        let max = point_one.max_by_component(point_two);
        AARect {
            min,
            max,
            k,
            axis,
            material: Arc::new(material),
        }
    }
    pub fn new_with_arc(
        min: Vec2,
        max: Vec2,
        k: Float,
        axis: Axis,
        material: &Arc<Material>,
    ) -> Self {
        AARect {
            min,
            max,
            k,
            axis,
            material: material.clone(),
        }
    }
}

pub struct AACuboid {
    pub min: Vec3,
    pub max: Vec3,
    pub rects: [AARect; 6],
    pub material: Arc<Material>,
}

impl AACuboid {
    pub fn new(point_one: Vec3, point_two: Vec3, material: Material) -> Self {
        if point_one == point_two {
            panic!("AACuboid called with two of the same point!");
        }
        let min = point_one.min_by_component(point_two);
        let max = point_one.max_by_component(point_two);

        let arc = Arc::new(material);
        let rects = [
            AARect::new_with_arc(
                Axis::X.point_without_axis(min),
                Axis::X.point_without_axis(max),
                min.x,
                Axis::X,
                &arc,
            ),
            AARect::new_with_arc(
                Axis::X.point_without_axis(min),
                Axis::X.point_without_axis(max),
                max.x,
                Axis::X,
                &arc,
            ),
            AARect::new_with_arc(
                Axis::Y.point_without_axis(min),
                Axis::Y.point_without_axis(max),
                min.y,
                Axis::Y,
                &arc,
            ),
            AARect::new_with_arc(
                Axis::Y.point_without_axis(min),
                Axis::Y.point_without_axis(max),
                max.y,
                Axis::Y,
                &arc,
            ),
            AARect::new_with_arc(
                Axis::Z.point_without_axis(min),
                Axis::Z.point_without_axis(max),
                min.z,
                Axis::Z,
                &arc,
            ),
            AARect::new_with_arc(
                Axis::Z.point_without_axis(min),
                Axis::Z.point_without_axis(max),
                max.z,
                Axis::Z,
                &arc,
            ),
        ];
        AACuboid {
            min,
            max,
            rects,
            material: arc.clone(),
        }
    }
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: Float,
    pub material: Arc<Material>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: Float, material: Material) -> Self {
        Sphere {
            center,
            radius,
            material: Arc::new(material),
        }
    }
}

#[derive(Clone)]
pub struct Triangle {
    pub points: [Vec3; 3],
    pub normals: [Vec3; 3],
    pub material: Arc<Material>,
}

impl Triangle {
    pub fn new_from_arc(points: [Vec3; 3], normals: [Vec3; 3], material: Arc<Material>) -> Self {
        Triangle {
            points,
            normals,
            material,
        }
    }
}

pub struct TriangleMesh {
    pub mesh: Vec<Triangle>,
    pub min: Vec3,
    pub max: Vec3,
    pub material: Arc<Material>,
}

impl TriangleMesh {
    pub fn new(mesh: Vec<Triangle>, material: Arc<Material>) -> Self {
        let mut min = None;
        let mut max = None;

        for triangle in &mesh {
            let aabb = triangle.get_aabb().unwrap();
            match (min, max) {
                (None, None) => {
                    min = Some(aabb.min);
                    max = Some(aabb.max);
                }
                (_, _) => {
                    min = Some(min.unwrap().min_by_component(aabb.min));
                    max = Some(max.unwrap().max_by_component(aabb.max))
                }
            }
        }

        TriangleMesh {
            mesh,
            min: min.unwrap(),
            max: max.unwrap(),
            material,
        }
    }
}
