use crate::ray_tracing::material::Scatter;
use crate::utility::{
    math::{rotate_around_point, Float},
    vec::{Vec2, Vec3},
};
use std::sync::Arc;

pub enum PrimitiveEnum<M: Scatter> {
    Sphere(Sphere<M>),
    AARect(AARect<M>),
    Rect(Rect<M>),
    AACuboid(AACuboid<M>),
    Cuboid(Cuboid<M>),
    Triangle(Triangle<M>),
    MeshTriangle(MeshTriangle<M>),
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
pub struct AARect<M: Scatter> {
    pub min: Vec2,
    pub max: Vec2,
    pub k: Float,
    pub axis: Axis,
    pub material: Arc<M>,
}

impl<M> AARect<M>
where
    M: Scatter,
{
    pub fn new(point_one: Vec2, point_two: Vec2, k: Float, axis: Axis, material: &Arc<M>) -> Self {
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
            material: material.clone(),
        }
    }
}

pub struct AACuboid<M: Scatter> {
    pub min: Vec3,
    pub max: Vec3,
    pub rects: [AARect<M>; 6],
    pub material: Arc<M>,
}

impl<M> AACuboid<M>
where
    M: Scatter,
{
    pub fn new(point_one: Vec3, point_two: Vec3, material: &Arc<M>) -> Self {
        if point_one == point_two {
            panic!("AACuboid called with two of the same point!");
        }
        let min = point_one.min_by_component(point_two);
        let max = point_one.max_by_component(point_two);

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
        AACuboid {
            min,
            max,
            rects,
            material: material.clone(),
        }
    }
}

pub struct Sphere<M: Scatter> {
    pub center: Vec3,
    pub radius: Float,
    pub material: Arc<M>,
}

impl<M> Sphere<M>
where
    M: Scatter,
{
    pub fn new(center: Vec3, radius: Float, material: &Arc<M>) -> Self {
        Sphere {
            center,
            radius,
            material: material.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Triangle<M: Scatter> {
    pub points: [Vec3; 3],
    pub normals: [Vec3; 3],
    pub material: Arc<M>,
}

impl<M> Triangle<M>
where
    M: Scatter,
{
    pub fn new(points: [Vec3; 3], normals: [Vec3; 3], material: &Arc<M>) -> Self {
        Triangle {
            points,
            normals,
            material: material.clone(),
        }
    }
}

pub struct MeshTriangle<M: Scatter> {
    pub point_indices: [usize; 3],
    pub normal_indices: [usize; 3],
    pub material: Arc<M>,
    pub mesh: Arc<MeshData<M>>,
}

impl<M> MeshTriangle<M>
where
    M: Scatter,
{
    pub fn new(
        point_indices: [usize; 3],
        normal_indices: [usize; 3],
        material: &Arc<M>,
        mesh: &Arc<MeshData<M>>,
    ) -> Self {
        MeshTriangle {
            point_indices,
            normal_indices,
            material: material.clone(),
            mesh: mesh.clone(),
        }
    }
}

pub struct MeshData<M: Scatter> {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub material: Arc<M>,
}

impl<M> MeshData<M>
where
    M: Scatter,
{
    pub fn new(vertices: Vec<Vec3>, normals: Vec<Vec3>, material: &Arc<M>) -> Self {
        MeshData {
            vertices,
            normals,
            material: material.clone(),
        }
    }
}

pub struct Rect<M: Scatter> {
    pub aarect: AARect<M>,
    pub cos_rotations: Vec3,
    pub sin_rotations: Vec3,
    pub rotation_point: Option<Vec3>,
}

impl<M> Rect<M>
where
    M: Scatter,
{
    pub fn new(aarect: AARect<M>, rotations: Vec3, rotation_point: Option<Vec3>) -> Self {
        let cos_rotations = Vec3::new(rotations.x.cos(), rotations.y.cos(), rotations.z.cos());
        let sin_rotations = Vec3::new(rotations.x.sin(), rotations.y.sin(), rotations.z.sin());

        Rect {
            aarect,
            cos_rotations,
            sin_rotations,
            rotation_point,
        }
    }
}

pub struct Cuboid<M: Scatter> {
    pub rects: [Rect<M>; 6],
    pub min: Vec3,
    pub max: Vec3,
    pub center: Vec3,
    pub material: Arc<M>,
}

impl<M> Cuboid<M>
where
    M: Scatter,
{
    pub fn new(aacuboid: AACuboid<M>, rotations: Vec3, _testing: &Arc<M>) -> Self {
        let cos_rotations = Vec3::new(rotations.x.cos(), rotations.y.cos(), rotations.z.cos());
        let sin_rotations = Vec3::new(rotations.x.sin(), rotations.y.sin(), rotations.z.sin());

        let material = aacuboid.material.clone();

        let min = aacuboid.min;
        let max = aacuboid.max;

        let center = (min + max) / 2.0;

        let rects = [
            Rect::new(
                AARect::new(
                    Axis::X.point_without_axis(min),
                    Axis::X.point_without_axis(max),
                    min.x,
                    Axis::X,
                    &material,
                ),
                rotations,
                Some(center),
            ),
            Rect::new(
                AARect::new(
                    Axis::X.point_without_axis(min),
                    Axis::X.point_without_axis(max),
                    max.x,
                    Axis::X,
                    &material,
                ),
                rotations,
                Some(center),
            ),
            Rect::new(
                AARect::new(
                    Axis::Y.point_without_axis(min),
                    Axis::Y.point_without_axis(max),
                    min.y,
                    Axis::Y,
                    &material,
                ),
                rotations,
                Some(center),
            ),
            Rect::new(
                AARect::new(
                    Axis::Y.point_without_axis(min),
                    Axis::Y.point_without_axis(max),
                    max.y,
                    Axis::Y,
                    &material,
                ),
                rotations,
                Some(center),
            ),
            Rect::new(
                AARect::new(
                    Axis::Z.point_without_axis(min),
                    Axis::Z.point_without_axis(max),
                    min.z,
                    Axis::Z,
                    &material,
                ),
                rotations,
                Some(center),
            ),
            Rect::new(
                AARect::new(
                    Axis::Z.point_without_axis(min),
                    Axis::Z.point_without_axis(max),
                    max.z,
                    Axis::Z,
                    &material,
                ),
                rotations,
                Some(center),
            ),
        ];

        let mut point_one = min;
        let mut point_two = max;
        let mut point_three = Vec3::new(min.x, min.y, max.z);
        let mut point_four = Vec3::new(min.x, max.y, max.z);
        let mut point_five = Vec3::new(min.x, max.y, min.z);
        let mut point_six = Vec3::new(max.x, max.y, min.z);
        let mut point_seven = Vec3::new(max.x, min.y, min.z);
        let mut point_eight = Vec3::new(max.x, min.y, max.z);

        rotate_around_point(&mut point_one, center, sin_rotations, cos_rotations);
        rotate_around_point(&mut point_two, center, sin_rotations, cos_rotations);
        rotate_around_point(&mut point_three, center, sin_rotations, cos_rotations);
        rotate_around_point(&mut point_four, center, sin_rotations, cos_rotations);
        rotate_around_point(&mut point_five, center, sin_rotations, cos_rotations);
        rotate_around_point(&mut point_six, center, sin_rotations, cos_rotations);
        rotate_around_point(&mut point_seven, center, sin_rotations, cos_rotations);
        rotate_around_point(&mut point_eight, center, sin_rotations, cos_rotations);

        let max = point_one
            .max_by_component(point_two)
            .max_by_component(point_three)
            .max_by_component(point_four)
            .max_by_component(point_five)
            .max_by_component(point_six)
            .max_by_component(point_seven)
            .max_by_component(point_eight);
        let min = point_one
            .min_by_component(point_two)
            .min_by_component(point_three)
            .min_by_component(point_four)
            .min_by_component(point_five)
            .min_by_component(point_six)
            .min_by_component(point_seven)
            .min_by_component(point_eight);

        Cuboid {
            rects,
            max,
            min,
            center,
            material,
        }
    }
}
