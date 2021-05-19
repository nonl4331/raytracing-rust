use ultraviolet::{DVec2, DVec3};

use std::sync::Arc;

use crate::image::material::Material;

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
}

pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
    pub material: Arc<Material>,
}

pub struct AARect {
    pub min: DVec2,
    pub max: DVec2,
    pub k: f64,
    pub axis: Axis,
    pub material: Arc<Material>,
}

pub struct AABox {
    pub min: DVec3,
    pub max: DVec3,
    pub rects: [AARect; 6],
    pub material: Arc<Material>,
}

impl AARect {
    pub fn new(min: DVec2, max: DVec2, k: f64, axis: Axis, material: Material) -> Self {
        AARect {
            min,
            max,
            k,
            axis,
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
            material: Arc::new(material),
        }
    }
}

impl Sphere {
    pub fn new(center: DVec3, radius: f64, material: Material) -> Self {
        Sphere {
            center,
            radius,
            material: Arc::new(material),
        }
    }
}

pub struct MovingSphere {
    pub start_pos: DVec3,
    pub end_pos: DVec3,
    pub radius: f64,
    pub material: Arc<Material>,
}

impl MovingSphere {
    pub fn new(start_pos: DVec3, end_pos: DVec3, radius: f64, material: Material) -> Self {
        MovingSphere {
            start_pos,
            end_pos,
            radius,
            material: Arc::new(material),
        }
    }
}
