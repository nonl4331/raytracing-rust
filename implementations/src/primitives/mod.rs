use crate::{
	aabb::{AABound, AABB},
	primitives::{
		sphere::Sphere,
		triangle::{MeshTriangle, Triangle},
	},
	rt_core::*,
};
use proc::Primitive;

pub mod sphere;
pub mod triangle;

#[derive(Primitive, Debug)]
pub enum AllPrimitives<M: Scatter> {
	Sphere(Sphere<M>),
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
