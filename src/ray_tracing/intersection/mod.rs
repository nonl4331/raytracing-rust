pub mod aacuboid;
pub mod aarect;
pub mod sphere;
pub mod triangle;

use crate::acceleration::aabb::Aabb;
use crate::ray_tracing::{
    intersection::{
        aacuboid::aacuboid_intersection, aarect::aarect_intersection, sphere::sphere_intersection,
        triangle::triangle_intersection,
    },
    material::MaterialTrait,
    primitives::{AACuboid, AARect, Axis, MeshTriangle, PrimitiveEnum, Sphere, Triangle},
    ray::Ray,
};
use crate::utility::{
    math::{next_float, previous_float, Float},
    vec::{Vec2, Vec3},
};
use std::sync::Arc;

#[cfg(all(feature = "f64"))]
use std::f64::consts::PI;

#[cfg(not(feature = "f64"))]
use std::f32::consts::PI;

pub struct Hit {
    pub t: Float,
    pub point: Vec3,
    pub error: Vec3,
    pub normal: Vec3,
    pub uv: Option<Vec2>,
    pub out: bool,
}

pub struct SurfaceIntersection<M: MaterialTrait> {
    pub hit: Hit,
    pub material: Arc<M>,
}

impl<M> SurfaceIntersection<M>
where
    M: MaterialTrait,
{
    pub fn new(
        t: Float,
        point: Vec3,
        error: Vec3,
        normal: Vec3,
        uv: Option<Vec2>,
        out: bool,
        material: &Arc<M>,
    ) -> Self {
        SurfaceIntersection {
            hit: Hit {
                t,
                point,
                error,
                normal,
                uv,
                out,
            },
            material: material.clone(),
        }
    }
}

pub trait Intersection<M: MaterialTrait> {
    fn get_int(&self, _: &Ray) -> Option<SurfaceIntersection<M>> {
        unimplemented!()
    }
    fn does_int(&self, ray: &Ray) -> bool {
        self.get_int(ray).is_some()
    }
}

pub trait PrimitiveTrait<M>: Intersection<M>
where
    M: MaterialTrait,
{
    fn get_aabb(&self) -> Option<Aabb> {
        unimplemented!()
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

impl<M> Intersection<M> for PrimitiveEnum<M>
where
    M: MaterialTrait,
{
    fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
        match self {
            PrimitiveEnum::Sphere(sphere) => sphere.get_int(ray),
            PrimitiveEnum::AARect(rect) => rect.get_int(ray),
            PrimitiveEnum::AACuboid(aab) => aab.get_int(ray),
            PrimitiveEnum::Triangle(triangle) => triangle.get_int(ray),
            PrimitiveEnum::MeshTriangle(triangle) => triangle.get_int(ray),
        }
    }

    fn does_int(&self, ray: &Ray) -> bool {
        match self {
            PrimitiveEnum::Sphere(sphere) => sphere.does_int(ray),
            PrimitiveEnum::AARect(rect) => rect.does_int(ray),
            PrimitiveEnum::AACuboid(aab) => aab.does_int(ray),
            PrimitiveEnum::Triangle(triangle) => triangle.does_int(ray),
            PrimitiveEnum::MeshTriangle(triangle) => triangle.does_int(ray),
        }
    }
}

impl<M> PrimitiveTrait<M> for PrimitiveEnum<M>
where
    M: MaterialTrait,
{
    fn get_aabb(&self) -> Option<Aabb> {
        match self {
            PrimitiveEnum::Sphere(sphere) => sphere.get_aabb(),
            PrimitiveEnum::AARect(rect) => rect.get_aabb(),
            PrimitiveEnum::AACuboid(aab) => aab.get_aabb(),
            PrimitiveEnum::Triangle(triangle) => triangle.get_aabb(),
            PrimitiveEnum::MeshTriangle(triangle) => triangle.get_aabb(),
        }
    }
    fn get_uv(&self, point: Vec3) -> Option<Vec2> {
        match self {
            PrimitiveEnum::Sphere(sphere) => sphere.get_uv(point),
            PrimitiveEnum::AARect(rect) => rect.get_uv(point),
            PrimitiveEnum::AACuboid(aab) => aab.get_uv(point),
            PrimitiveEnum::Triangle(triangle) => triangle.get_uv(point),
            PrimitiveEnum::MeshTriangle(triangle) => triangle.get_uv(point),
        };
        None
    }
    fn requires_uv(&self) -> bool {
        match self {
            PrimitiveEnum::Sphere(sphere) => (*sphere.material).requires_uv(),
            PrimitiveEnum::AARect(rect) => rect.material.requires_uv(),
            PrimitiveEnum::AACuboid(aab) => aab.material.requires_uv(),
            PrimitiveEnum::Triangle(triangle) => triangle.material.requires_uv(),
            PrimitiveEnum::MeshTriangle(triangle) => triangle.material.requires_uv(),
        }
    }
}

impl<M> Intersection<M> for Sphere<M>
where
    M: MaterialTrait,
{
    fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
        sphere_intersection(self, ray)
    }
}

#[allow(clippy::suspicious_operation_groupings)]
impl<M> PrimitiveTrait<M> for Sphere<M>
where
    M: MaterialTrait,
{
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

impl<M> Intersection<M> for AARect<M>
where
    M: MaterialTrait,
{
    fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
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

impl<M> PrimitiveTrait<M> for AARect<M>
where
    M: MaterialTrait,
{
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

impl<M> Intersection<M> for AACuboid<M>
where
    M: MaterialTrait,
{
    fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
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

impl<M> PrimitiveTrait<M> for AACuboid<M>
where
    M: MaterialTrait,
{
    fn get_aabb(&self) -> Option<Aabb> {
        Some(Aabb::new(self.min, self.max))
    }
}

impl<M> Intersection<M> for Triangle<M>
where
    M: MaterialTrait,
{
    fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
        triangle_intersection(self, ray)
    }
}

impl<M> PrimitiveTrait<M> for Triangle<M>
where
    M: MaterialTrait,
{
    fn get_aabb(&self) -> Option<Aabb> {
        Some(Aabb::new(
            self.points[0].min_by_component(self.points[1].min_by_component(self.points[2])),
            self.points[0].max_by_component(self.points[1].max_by_component(self.points[2])),
        ))
    }
}

impl<M> Intersection<M> for MeshTriangle<M>
where
    M: MaterialTrait,
{
    fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
        triangle_intersection(self, ray)
    }
}

impl<M> PrimitiveTrait<M> for MeshTriangle<M>
where
    M: MaterialTrait,
{
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
