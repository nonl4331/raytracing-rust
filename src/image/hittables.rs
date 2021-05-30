use ultraviolet::{DVec2, DVec3};

use std::sync::Arc;

use crate::image::aabb::AABB;

use crate::image::material::Material;

use rand::rngs::SmallRng;
use rand::thread_rng;

use rand::{Rng, SeedableRng};

pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn get_axis_value(&self, point: DVec3) -> f64 {
        match self {
            Axis::X => point.x,
            Axis::Y => point.y,
            Axis::Z => point.z,
        }
    }

    pub fn point_without_axis(&self, point: DVec3) -> DVec2 {
        match self {
            Axis::X => DVec2::new(point.y, point.z),
            Axis::Y => DVec2::new(point.x, point.z),
            Axis::Z => DVec2::new(point.x, point.y),
        }
    }
    pub fn return_point_with_axis(&self, dir: DVec3) -> DVec3 {
        match self {
            Axis::X => DVec3::new(dir.x, 0.0, 0.0),
            Axis::Y => DVec3::new(0.0, dir.y, 0.0),
            Axis::Z => DVec3::new(0.0, 0.0, dir.z),
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

pub struct AARect {
    pub min: DVec2,
    pub max: DVec2,
    pub k: f64,
    pub axis: Axis,
    pub aabb: AABB,
    pub material: Arc<Material>,
}

pub struct AABox {
    pub min: DVec3,
    pub max: DVec3,
    pub rects: [AARect; 6],
    pub aabb: AABB,
    pub material: Arc<Material>,
}

impl AARect {
    pub fn new(min: DVec2, max: DVec2, k: f64, axis: Axis, material: Material) -> Self {
        let kvec = k * axis.return_point_with_axis(DVec3::one());
        AARect {
            min,
            max,
            k,
            axis,
            aabb: AABB::new(kvec - 0.0001 * DVec3::one(), kvec + 0.0001 * DVec3::one()),
            material: Arc::new(material),
        }
    }
}

impl AABox {
    pub fn new(min: DVec3, max: DVec3, material: Material) -> Self {
        let rects = [
            AARect::new(
                Axis::X.point_without_axis(min),
                Axis::X.point_without_axis(max),
                min.x,
                Axis::X,
                material,
            ),
            AARect::new(
                Axis::X.point_without_axis(min),
                Axis::X.point_without_axis(max),
                max.x,
                Axis::X,
                material,
            ),
            AARect::new(
                Axis::Y.point_without_axis(min),
                Axis::Y.point_without_axis(max),
                min.y,
                Axis::Y,
                material,
            ),
            AARect::new(
                Axis::Y.point_without_axis(min),
                Axis::Y.point_without_axis(max),
                max.y,
                Axis::Y,
                material,
            ),
            AARect::new(
                Axis::Z.point_without_axis(min),
                Axis::Z.point_without_axis(max),
                min.z,
                Axis::Z,
                material,
            ),
            AARect::new(
                Axis::Z.point_without_axis(min),
                Axis::Z.point_without_axis(max),
                max.z,
                Axis::Z,
                material,
            ),
        ];
        AABox {
            min,
            max,
            rects,
            aabb: AABB::new(min, max),
            material: Arc::new(material),
        }
    }
}

pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
    pub aabb: AABB,
    pub material: Arc<Material>,
}

impl Sphere {
    pub fn new(center: DVec3, radius: f64, material: Material) -> Self {
        Sphere {
            center,
            radius,
            aabb: AABB::new(
                center - radius * DVec3::one(),
                center + radius * DVec3::one(),
            ),
            material: Arc::new(material),
        }
    }
}

pub struct MovingSphere {
    pub start_pos: DVec3,
    pub end_pos: DVec3,
    pub radius: f64,
    pub aabb: AABB,
    pub material: Arc<Material>,
}

impl MovingSphere {
    pub fn new(start_pos: DVec3, end_pos: DVec3, radius: f64, material: Material) -> Self {
        let start_aabb = AABB::new(
            start_pos - radius * DVec3::one(),
            start_pos + radius * DVec3::one(),
        );
        let end_aabb = AABB::new(
            end_pos - radius * DVec3::one(),
            end_pos + radius * DVec3::one(),
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
