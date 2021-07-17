use crate::bvh::aabb::AABB;

use crate::ray_tracing::material::Material;

use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};

use std::sync::Arc;

use ultraviolet::{Vec2, Vec3};

#[derive(Clone)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn get_axis_value(&self, point: Vec3) -> f32 {
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

    pub fn random_axis() -> Self {
        let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
        match rng.gen_range(0..3) {
            0 => Axis::X,
            1 => Axis::Y,
            _ => Axis::Z,
        }
    }
}

#[derive(Clone)]
pub struct AARect {
    pub min: Vec2,
    pub max: Vec2,
    pub k: f32,
    pub axis: Axis,
    pub aabb: AABB,
    pub material: Arc<Material>,
}

impl AARect {
    pub fn new(min: Vec2, max: Vec2, k: f32, axis: Axis, material: Material) -> Self {
        let kvec = k * axis.return_point_with_axis(Vec3::one());
        AARect {
            min,
            max,
            k,
            axis,
            aabb: AABB::new(kvec - 0.0001 * Vec3::one(), kvec + 0.0001 * Vec3::one()),
            material: Arc::new(material),
        }
    }
    pub fn new_with_arc(
        min: Vec2,
        max: Vec2,
        k: f32,
        axis: Axis,
        material: &Arc<Material>,
    ) -> Self {
        let kvec = k * axis.return_point_with_axis(Vec3::one());
        AARect {
            min,
            max,
            k,
            axis,
            aabb: AABB::new(kvec - 0.0001 * Vec3::one(), kvec + 0.0001 * Vec3::one()),
            material: material.clone(),
        }
    }
}

pub struct AABox {
    pub min: Vec3,
    pub max: Vec3,
    pub rects: [AARect; 6],
    pub aabb: AABB,
    pub material: Arc<Material>,
}

impl AABox {
    pub fn new(min: Vec3, max: Vec3, material: Material) -> Self {
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
        AABox {
            min,
            max,
            rects,
            aabb: AABB::new(min, max),
            material: arc.clone(),
        }
    }
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub aabb: AABB,
    pub material: Arc<Material>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Material) -> Self {
        Sphere {
            center,
            radius,
            aabb: AABB::new(center - radius * Vec3::one(), center + radius * Vec3::one()),
            material: Arc::new(material),
        }
    }
}

pub struct MovingSphere {
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub radius: f32,
    pub aabb: AABB,
    pub material: Arc<Material>,
}

impl MovingSphere {
    pub fn new(start_pos: Vec3, end_pos: Vec3, radius: f32, material: Material) -> Self {
        let start_aabb = AABB::new(
            start_pos - radius * Vec3::one(),
            start_pos + radius * Vec3::one(),
        );
        let end_aabb = AABB::new(
            end_pos - radius * Vec3::one(),
            end_pos + radius * Vec3::one(),
        );
        MovingSphere {
            start_pos,
            end_pos,
            radius,
            aabb: AABB::new_contains(&vec![start_aabb, end_aabb]),
            material: Arc::new(material),
        }
    }
}

#[derive(Clone)]
pub struct Triangle {
    pub points: [Vec3; 3],
    pub normal: Vec3,
    pub aabb: AABB,
    pub material: Arc<Material>,
}

impl Triangle {
    pub fn new_from_arc(points: [Vec3; 3], normal: Option<Vec3>, material: Arc<Material>) -> Self {
        let normal = match normal {
            Some(normal) => normal,
            None => {
                let a = points[1] - points[0];
                let b = points[2] - points[0];
                a.cross(b)
            }
        };
        let min = points[0].min_by_component(points[1].min_by_component(points[2]))
            - Vec3::new(0.0001, 0.0001, 0.0001);
        let max = points[0].max_by_component(points[1].max_by_component(points[2]))
            + Vec3::new(0.0001, 0.0001, 0.0001);

        Triangle {
            points,
            normal,
            aabb: AABB::new(min, max),
            material,
        }
    }
}

pub struct TriangleMesh {
    pub mesh: Vec<Triangle>,
    pub aabb: AABB,
    pub material: Arc<Material>,
}

impl TriangleMesh {
    pub fn new(min: Vec3, max: Vec3, mesh: Vec<Triangle>, material: Arc<Material>) -> Self {
        TriangleMesh {
            mesh,
            aabb: AABB::new(min, max),
            material,
        }
    }
}
