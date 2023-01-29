use crate::{
	aabb::{AABound, AABB},
	primitives::Axis,
	rt_core::*,
	utility::{check_side, gamma},
};
use rand::{thread_rng, Rng};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Triangle<'a, M: Scatter> {
	pub points: [Vec3; 3],
	pub normals: [Vec3; 3],
	pub material: &'a M,
}

impl<'a, M> Triangle<'a, M>
where
	M: Scatter,
{
	pub fn new(points: [Vec3; 3], normals: [Vec3; 3], material: &'a M) -> Self {
		Triangle {
			points,
			normals,
			material,
		}
	}
}

#[derive(Debug)]
pub struct MeshTriangle<'a, M: Scatter> {
	pub point_indices: [usize; 3],
	pub normal_indices: [usize; 3],
	pub material: &'a M,
	pub mesh: Arc<MeshData>,
}

impl<'a, M> MeshTriangle<'a, M>
where
	M: Scatter,
{
	pub fn new(
		point_indices: [usize; 3],
		normal_indices: [usize; 3],
		material: &'a M,
		mesh: &Arc<MeshData>,
	) -> Self {
		MeshTriangle {
			point_indices,
			normal_indices,
			material,
			mesh: mesh.clone(),
		}
	}
}

#[derive(Debug)]
pub struct MeshData {
	pub vertices: Vec<Vec3>,
	pub normals: Vec<Vec3>,
}

impl MeshData {
	pub fn new(vertices: Vec<Vec3>, normals: Vec<Vec3>) -> Self {
		MeshData { vertices, normals }
	}
}

pub trait TriangleTrait<'a, M: Scatter> {
	fn get_point(&self, index: usize) -> Vec3;
	fn get_normal(&self, index: usize) -> Vec3;
	fn get_material(&self) -> &'a M;
}

impl<'a, M> TriangleTrait<'a, M> for Triangle<'a, M>
where
	M: Scatter,
{
	fn get_point(&self, index: usize) -> Vec3 {
		self.points[index]
	}
	fn get_normal(&self, index: usize) -> Vec3 {
		self.normals[index]
	}
	fn get_material(&self) -> &'a M {
		&self.material
	}
}

impl<'a, M> TriangleTrait<'a, M> for MeshTriangle<'a, M>
where
	M: Scatter,
{
	fn get_point(&self, index: usize) -> Vec3 {
		self.mesh.vertices[self.point_indices[index]]
	}
	fn get_normal(&self, index: usize) -> Vec3 {
		self.mesh.normals[self.normal_indices[index]]
	}
	fn get_material(&self) -> &'a M {
		&self.material
	}
}

pub fn triangle_intersection<'a, T: TriangleTrait<'a, M>, M: Scatter>(
	triangle: &'a T,
	ray: &Ray,
) -> Option<SurfaceIntersection<'a, M>> {
	let mut p0t = triangle.get_point(0) - ray.origin;
	let mut p1t = triangle.get_point(1) - ray.origin;
	let mut p2t = triangle.get_point(2) - ray.origin;

	let max_axis = Axis::get_max_abs_axis(&ray.direction);
	Axis::swap_z(&mut p0t, &max_axis);
	Axis::swap_z(&mut p1t, &max_axis);
	Axis::swap_z(&mut p2t, &max_axis);

	p0t.x += ray.shear.x * p0t.z;
	p0t.y += ray.shear.y * p0t.z;
	p1t.x += ray.shear.x * p1t.z;
	p1t.y += ray.shear.y * p1t.z;
	p2t.x += ray.shear.x * p2t.z;
	p2t.y += ray.shear.y * p2t.z;

	let mut e0 = p1t.x * p2t.y - p1t.y * p2t.x;
	let mut e1 = p2t.x * p0t.y - p2t.y * p0t.x;
	let mut e2 = p0t.x * p1t.y - p0t.y * p1t.x;
	if e0 == 0.0 || e1 == 0.0 || e2 == 0.0 {
		e0 = (p1t.x as f64 * p2t.y as f64 - p1t.y as f64 * p2t.x as f64) as Float;
		e1 = (p2t.x as f64 * p0t.y as f64 - p2t.y as f64 * p0t.x as f64) as Float;
		e2 = (p0t.x as f64 * p1t.y as f64 - p0t.y as f64 * p1t.x as f64) as Float;
	}

	if (e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0) {
		return None;
	}

	let det = e0 + e1 + e2;
	if det == 0.0 {
		return None;
	}

	p0t *= ray.shear.z;
	p1t *= ray.shear.z;
	p2t *= ray.shear.z;

	let t_scaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;
	if (det < 0.0 && t_scaled >= 0.0) || (det > 0.0 && t_scaled <= 0.0) {
		return None;
	}

	let inv_det = 1.0 / det;

	let b0 = e0 * inv_det;
	let b1 = e1 * inv_det;
	let b2 = e2 * inv_det;

	let t = inv_det * t_scaled;

	let max_z_t = Vec3::new(p0t.z.abs(), p1t.z.abs(), p2t.z.abs()).component_max();
	let delta_z = gamma(3) * max_z_t;

	let max_x_t = Vec3::new(p0t.x.abs(), p1t.x.abs(), p2t.x.abs()).component_max();
	let max_y_t = Vec3::new(p0t.y.abs(), p1t.y.abs(), p2t.y.abs()).component_max();
	let delta_x = gamma(5) * (max_x_t + max_z_t);
	let delta_y = gamma(5) * (max_y_t + max_z_t);

	let delta_e = 2.0 * (gamma(2) * max_x_t * max_y_t + delta_y * max_x_t + delta_x * max_y_t);

	let max_e = Vec3::new(e0.abs(), e1.abs(), e2.abs()).component_max();

	let delta_t =
		3.0 * (gamma(3) * max_e * max_z_t + delta_e * max_z_t + delta_z * max_e) * inv_det.abs();

	if t < delta_t {
		return None;
	}

	let uv = b0 * Vec2::new(0.0, 0.0) + b1 * Vec2::new(1.0, 0.0) + b2 * Vec2::new(1.0, 1.0);

	let mut normal =
		b0 * triangle.get_normal(0) + b1 * triangle.get_normal(1) + b2 * triangle.get_normal(2);

	let out = check_side(&mut normal, &ray.direction);

	let x_abs_sum = (b0 * triangle.get_point(0).x).abs()
		+ (b1 * triangle.get_point(1).x).abs()
		+ (b2 * triangle.get_point(2).x).abs();
	let y_abs_sum = (b0 * triangle.get_point(0).y).abs()
		+ (b1 * triangle.get_point(1).y).abs()
		+ (b2 * triangle.get_point(2).y).abs();
	let z_abs_sum = (b0 * triangle.get_point(0).z).abs()
		+ (b1 * triangle.get_point(1).z).abs()
		+ (b2 * triangle.get_point(2).z).abs();

	let point_error = gamma(7) * Vec3::new(x_abs_sum, y_abs_sum, z_abs_sum)
		+ gamma(6)
			* Vec3::new(
				b2 * triangle.get_point(2).x,
				b2 * triangle.get_point(2).y,
				b2 * triangle.get_point(2).z,
			);

	let point =
		b0 * triangle.get_point(0) + b1 * triangle.get_point(1) + b2 * triangle.get_point(2);

	Some(SurfaceIntersection::new(
		t,
		point,
		point_error,
		normal,
		Some(uv),
		out,
		triangle.get_material(),
	))
}

impl<'a, M> Primitive for Triangle<'a, M>
where
	M: Scatter,
{
	type Material = M;
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		triangle_intersection(self, ray)
	}

	fn area(&self) -> Float {
		0.5 * (self.points[1] - self.points[0])
			.cross(self.points[2] - self.points[0])
			.mag()
	}
	fn sample_visible_from_point(&self, in_point: Vec3) -> Vec3 {
		let mut rng = thread_rng();
		let uv = rng.gen::<Float>().sqrt();
		let uv = (1.0 - uv, uv * rng.gen::<Float>());

		let point =
			uv.0 * self.points[0] + uv.1 * self.points[1] + (1.0 - uv.0 - uv.1) * self.points[2];

		(point - in_point).normalised()
	}
	fn scattering_pdf(&self, hit_point: Vec3, wi: Vec3, sampled_hit: &Hit) -> Float {
		(sampled_hit.point - hit_point).mag_sq() / (sampled_hit.normal.dot(wi).abs() * self.area())
	}
	fn material_is_light(&self) -> bool {
		self.material.is_light()
	}
}

impl<'a, M> Primitive for MeshTriangle<'a, M>
where
	M: Scatter,
{
	type Material = M;
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		triangle_intersection(self, ray)
	}
	fn area(&self) -> Float {
		0.5 * (self.mesh.vertices[self.point_indices[1]]
			- self.mesh.vertices[self.point_indices[0]])
			.cross(
				self.mesh.vertices[self.point_indices[2]]
					- self.mesh.vertices[self.point_indices[0]],
			)
			.mag()
	}
	fn sample_visible_from_point(&self, in_point: Vec3) -> Vec3 {
		let mut rng = thread_rng();
		let uv = rng.gen::<Float>().sqrt();
		let uv = (1.0 - uv, uv * rng.gen::<Float>().sqrt());

		let point = uv.0 * self.mesh.vertices[self.point_indices[0]]
			+ uv.1 * self.mesh.vertices[self.point_indices[1]]
			+ (1.0 - uv.0 - uv.1) * self.mesh.vertices[self.point_indices[2]];

		(point - in_point).normalised()
	}
	fn scattering_pdf(&self, hit_point: Vec3, wi: Vec3, sampled_hit: &Hit) -> Float {
		(sampled_hit.point - hit_point).mag_sq() / (wi.dot(sampled_hit.normal).abs() * self.area())
	}
	fn material_is_light(&self) -> bool {
		self.material.is_light()
	}
}
impl<'a, M: Scatter> AABound for Triangle<'a, M> {
	fn get_aabb(&self) -> AABB {
		AABB::new(
			self.points[0].min_by_component(self.points[1].min_by_component(self.points[2])),
			self.points[0].max_by_component(self.points[1].max_by_component(self.points[2])),
		)
	}
}

impl<'a, M: Scatter> AABound for MeshTriangle<'a, M> {
	fn get_aabb(&self) -> AABB {
		let points = [
			self.mesh.vertices[self.point_indices[0]],
			self.mesh.vertices[self.point_indices[1]],
			self.mesh.vertices[self.point_indices[2]],
		];

		AABB::new(
			points[0].min_by_component(points[1].min_by_component(points[2])),
			points[0].max_by_component(points[1].max_by_component(points[2])),
		)
	}
}
