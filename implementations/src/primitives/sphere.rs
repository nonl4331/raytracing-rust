use crate::{
	aabb::{AABound, AABB},
	utility::{coord::Coordinate, random_float},
};
use rt_core::{Float, Hit, Primitive, Ray, Scatter, SurfaceIntersection, Vec2, Vec3, EPSILON, PI};
use std::sync::Arc;

#[derive(Debug)]
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

#[allow(clippy::suspicious_operation_groupings)]
impl<M> Primitive<M> for Sphere<M>
where
	M: Scatter,
{
	fn get_int(&self, ray: &Ray) -> Option<SurfaceIntersection<M>> {
		let dir = ray.direction;
		let center = self.center;
		let radius = self.radius;
		let orig = ray.origin;

		// simplified terms for algorithm below
		let deltap = center - orig;
		let ddp = dir.dot(deltap);
		let deltapdot = deltap.dot(deltap);

		let remedy_term = deltap - ddp * dir;
		let discriminant = radius * radius - remedy_term.dot(remedy_term);

		// check if any solutions exist
		if discriminant > 0.0 {
			// the square root of the discriminant
			let sqrt_val = discriminant.sqrt();

			// Get intermediate q value based on ddp sign
			let q = if ddp > 0.0 {
				ddp + sqrt_val
			} else {
				ddp - sqrt_val
			};

			// Get two solutions of quadratic formula
			let mut t0 = q;
			let mut t1 = (deltapdot - radius * radius) / q;

			// Make sure t1 > t0 (for sorting purposes)
			if t1 < t0 {
				std::mem::swap(&mut t0, &mut t1);
			};

			// Get smallest t value that is above 0
			let t = if t0 > 0.0 {
				t0
			} else {
				if t1 <= 0.0 {
					return None;
				}
				t1
			};

			// Get point at "t"
			let point = ray.at(t);

			// Get normal from intersection point
			let mut normal = (point - center) / radius;

			// Make sure normal faces outward and make note of what side of the object the ray is on
			let mut out = true;
			if normal.dot(dir) > 0.0 {
				out = false;
				normal = -normal;
			}

			// fill in details about intersection point
			Some(SurfaceIntersection::new(
				t,
				point,
				EPSILON * Vec3::one(),
				normal,
				self.get_uv(point),
				out,
				&self.material,
			))
		} else {
			None
		}
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
	fn get_sample(&self) -> Vec3 {
		let z = 1.0 - 2.0 * random_float();
		let a = (1.0 - z * z).max(0.0).sqrt();
		let b = 2.0 * PI * random_float();
		self.center + self.radius * Vec3::new(a * b.cos(), a * b.sin(), z)
	}
	fn sample_visible_from_point(&self, in_point: Vec3) -> Vec3 {
		let distance_sq = (in_point - self.center).mag_sq();
		let point = if distance_sq <= self.radius * self.radius {
			self.get_sample()
		} else {
			let distance = distance_sq.sqrt();
			let sin_theta_max_sq = self.radius * self.radius / distance_sq;
			let cost_theta_max = (1.0 - sin_theta_max_sq).max(0.0).sqrt();
			let r1 = random_float();
			let cos_theta = (1.0 - r1) + r1 * cost_theta_max;
			let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
			let phi = 2.0 * random_float() * PI;

			// calculate alpha
			let ds = distance * cos_theta
				- (self.radius * self.radius - distance_sq * sin_theta * sin_theta)
					.max(0.0)
					.sqrt();
			let cos_alpha = (distance_sq + self.radius * self.radius - ds * ds)
				/ (2.0 * distance * self.radius);
			let sin_alpha = (1.0 - cos_alpha * cos_alpha).max(0.0).sqrt();

			// get sphere point
			let coord_system = Coordinate::new_from_z((in_point - self.center).normalised());
			let mut vec = Vec3::new(sin_alpha * phi.cos(), sin_alpha * phi.sin(), cos_alpha);
			coord_system.vec_to_coordinate(&mut vec);

			self.center + self.radius * vec
		};

		(point - in_point).normalised()
	}
	fn scattering_pdf(&self, hit: &Hit, wo: Vec3, light_point: Vec3) -> Float {
		let rsq = self.radius * self.radius;
		let dsq = (hit.point - self.center).mag_sq();
		if dsq <= rsq {
			return (light_point - hit.point).mag_sq() / (wo.dot(-hit.normal).abs() * self.area());
		}
		let sin_theta_max_sq = rsq / dsq;
		let cos_theta_max = (1.0 - sin_theta_max_sq).max(0.0).sqrt();

		1.0 / (2.0 * PI * (1.0 - cos_theta_max))
	}
	fn area(&self) -> Float {
		4.0 * PI * self.radius * self.radius
	}
	fn material_is_light(&self) -> bool {
		self.material.is_light()
	}
}

impl<M: Scatter> AABound for Sphere<M> {
	fn get_aabb(&self) -> Option<AABB> {
		Some(AABB::new(
			self.center - self.radius * Vec3::one(),
			self.center + self.radius * Vec3::one(),
		))
	}
}
