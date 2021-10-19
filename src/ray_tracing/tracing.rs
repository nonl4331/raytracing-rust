use crate::acceleration::aabb::Aabb;
use crate::utility::math::Float;

use crate::utility::math::{next_float, previous_float};

use crate::ray_tracing::{
    intersection::{
        aacuboid::aacuboid_intersection, aarect::aarect_intersection, sphere::sphere_intersection,
        triangle::triangle_intersection,
    },
    material::{Material, MaterialTrait},
    primitives::{AACuboid, AARect, Axis, MeshTriangle, Primitive, Sphere, Triangle},
    ray::Ray,
};

use std::f32::consts::PI;

use std::sync::Arc;

use crate::utility::vec::{Vec2, Vec3};

pub const EPSILON: Float = 0.00000003;

pub struct Hit {
    pub t: Float,
    pub point: Vec3,
    pub error: Vec3,
    pub normal: Vec3,
    pub uv: Option<Vec2>,
    pub out: bool,
    pub material: Arc<Material>,
}

pub trait Intersection {
    fn get_int(&self, _: &Ray) -> Option<Hit> {
        None
    }
    fn does_int(&self, ray: &Ray) -> bool {
        self.get_int(ray).is_some()
    }
}

pub trait PrimitiveTrait: Intersection {
    fn get_internal(self) -> Vec<Primitive>;
    fn get_aabb(&self) -> Option<Aabb> {
        None
    }
    fn requires_uv(&self) -> bool {
        false
    }
    fn get_uv(&self, _: Vec3) -> Option<Vec2> {
        None
    }
}

pub fn offset_ray(origin: Vec3, normal: Vec3, error: Vec3, is_brdf: bool) -> Vec3 {
    let offset_val = normal.abs().dot(error);
    let mut offset = offset_val * normal;

    if !is_brdf {
        offset = -offset;
    }

    let mut new_origin = origin + offset;

    if offset.x > 0.0 {
        new_origin.x = next_float(new_origin.x);
    } else {
        new_origin.x = previous_float(new_origin.x);
    }

    if offset.y > 0.0 {
        new_origin.y = next_float(new_origin.y);
    } else {
        new_origin.y = previous_float(new_origin.y);
    }

    if offset.z > 0.0 {
        new_origin.z = next_float(new_origin.z);
    } else {
        new_origin.z = previous_float(new_origin.z);
    }

    new_origin
}

pub fn check_side(normal: &mut Vec3, ray_direction: &Vec3) -> bool {
    if normal.dot(*ray_direction) > 0.0 {
        *normal = -*normal;
        false
    } else {
        true
    }
}

impl Intersection for Primitive {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_int(ray),
            Primitive::AARect(rect) => rect.get_int(ray),
            Primitive::AACuboid(aab) => aab.get_int(ray),
            Primitive::Triangle(triangle) => triangle.get_int(ray),
            Primitive::MeshTriangle(triangle) => triangle.get_int(ray),
            Primitive::None => panic!("get_int called on PrimitiveNone"),
        }
    }

    fn does_int(&self, ray: &Ray) -> bool {
        match self {
            Primitive::Sphere(sphere) => sphere.does_int(ray),
            Primitive::AARect(rect) => rect.does_int(ray),
            Primitive::AACuboid(aab) => aab.does_int(ray),
            Primitive::Triangle(triangle) => triangle.does_int(ray),
            Primitive::MeshTriangle(triangle) => triangle.does_int(ray),
            Primitive::None => panic!("does_int called on PrimitiveNone"),
        }
    }
}

impl PrimitiveTrait for Primitive {
    fn get_internal(self) -> Vec<Primitive> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_internal(),
            Primitive::AARect(rect) => rect.get_internal(),
            Primitive::AACuboid(aab) => aab.get_internal(),
            Primitive::Triangle(triangle) => triangle.get_internal(),
            Primitive::MeshTriangle(triangle) => triangle.get_internal(),
            Primitive::None => panic!("get_internal called on PrimitiveNone"),
        }
    }

    fn get_aabb(&self) -> Option<Aabb> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_aabb(),
            Primitive::AARect(rect) => rect.get_aabb(),
            Primitive::AACuboid(aab) => aab.get_aabb(),
            Primitive::Triangle(triangle) => triangle.get_aabb(),
            Primitive::MeshTriangle(triangle) => triangle.get_aabb(),
            Primitive::None => panic!("get_aabb called on PrimitiveNone"),
        }
    }
    fn get_uv(&self, point: Vec3) -> Option<Vec2> {
        match self {
            Primitive::Sphere(sphere) => sphere.get_uv(point),
            Primitive::AARect(rect) => rect.get_uv(point),
            Primitive::AACuboid(aab) => aab.get_uv(point),
            Primitive::Triangle(triangle) => triangle.get_uv(point),
            Primitive::MeshTriangle(triangle) => triangle.get_uv(point),
            Primitive::None => panic!("get_uv called on PrimitiveNone"),
        };
        None
    }
    fn requires_uv(&self) -> bool {
        match self {
            Primitive::Sphere(sphere) => (*sphere.material).requires_uv(),
            Primitive::AARect(rect) => rect.material.requires_uv(),
            Primitive::AACuboid(aab) => aab.material.requires_uv(),
            Primitive::Triangle(triangle) => triangle.material.requires_uv(),
            Primitive::MeshTriangle(triangle) => triangle.material.requires_uv(),
            Primitive::None => panic!("requires_uv called on PrimitiveNone"),
        }
    }
}

impl Intersection for Sphere {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        sphere_intersection(self, ray)
    }
}

#[allow(clippy::suspicious_operation_groupings)]
impl PrimitiveTrait for Sphere {
    fn get_internal(self) -> Vec<Primitive> {
        vec![Primitive::Sphere(self)]
    }
    fn get_uv(&self, point: Vec3) -> Option<Vec2> {
        if self.material.requires_uv() {
            let x = (self.center.x - point.x) / self.radius;
            let y = (self.center.y - point.y) / self.radius;
            let z = (self.center.z - point.z) / self.radius;
            let phi = (-1.0 * z).atan2(x) + PI;
            let theta = (-1.0 * y).acos();

            return Some(Vec2::new(phi / (2.0 * PI), theta / PI));
        }
        None
    }
    fn get_aabb(&self) -> Option<Aabb> {
        Some(Aabb::new(
            self.center - self.radius * Vec3::one(),
            self.center + self.radius * Vec3::one(),
        ))
    }
}

impl Intersection for AARect {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        aarect_intersection(self, ray)
    }

    fn does_int(&self, ray: &Ray) -> bool {
        let t = (self.k - self.axis.get_axis_value(ray.origin))
            / self.axis.get_axis_value(ray.direction);
        let point = ray.at(t);
        let point_2d = self.axis.point_without_axis(point);

        // x & y are not the x & y axis but rather the two axis that are not self.axis
        point_2d.x > self.min.x
            && point_2d.x < self.max.x
            && point_2d.y > self.min.y
            && point_2d.y < self.max.y
    }
}

impl PrimitiveTrait for AARect {
    fn get_internal(self) -> Vec<Primitive> {
        vec![Primitive::AARect(self)]
    }
    fn get_uv(&self, point: Vec3) -> Option<Vec2> {
        if self.material.requires_uv() {
            let pwa = self.axis.point_without_axis(point);
            return Some(Vec2::new(
                (pwa.x - self.min.x) / self.max.x,
                (pwa.y - self.min.y) / self.max.y,
            ));
        }
        None
    }
    fn get_aabb(&self) -> Option<Aabb> {
        Some(Aabb::new(
            Axis::point_from_2d(&self.min, &self.axis, self.k - 0.0001),
            Axis::point_from_2d(&self.max, &self.axis, self.k + 0.0001),
        ))
    }
}

impl Intersection for AACuboid {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        aacuboid_intersection(self, ray)
    }

    fn does_int(&self, ray: &Ray) -> bool {
        for side in self.rects.iter() {
            if side.does_int(ray) {
                return true;
            }
        }
        false
    }
}

impl PrimitiveTrait for AACuboid {
    fn get_internal(mut self) -> Vec<Primitive> {
        self.rects
            .iter_mut()
            .map(|rect| Primitive::AARect(rect.clone()))
            .collect()
    }

    fn get_aabb(&self) -> Option<Aabb> {
        None
    }
}

impl Intersection for Triangle {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        triangle_intersection(self, ray)
    }
}

impl PrimitiveTrait for Triangle {
    fn get_internal(self) -> Vec<Primitive> {
        vec![Primitive::Triangle(self)]
    }

    fn get_aabb(&self) -> Option<Aabb> {
        Some(Aabb::new(
            self.points[0].min_by_component(self.points[1].min_by_component(self.points[2])),
            self.points[0].max_by_component(self.points[1].max_by_component(self.points[2])),
        ))
    }
}

impl Intersection for MeshTriangle {
    fn get_int(&self, ray: &Ray) -> Option<Hit> {
        triangle_intersection(self, ray)
    }
}

impl PrimitiveTrait for MeshTriangle {
    fn get_internal(self) -> Vec<Primitive> {
        vec![Primitive::MeshTriangle(self)]
    }

    fn get_aabb(&self) -> Option<Aabb> {
        let points = [
            (*self.mesh).vertices[self.point_indices[0]],
            (*self.mesh).vertices[self.point_indices[1]],
            (*self.mesh).vertices[self.point_indices[2]],
        ];

        Some(Aabb::new(
            points[0].min_by_component(points[1].min_by_component(points[2])),
            points[0].max_by_component(points[1].max_by_component(points[2])),
        ))
    }
}
