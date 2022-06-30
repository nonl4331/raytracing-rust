use rt_core::{Float, Ray, Vec3};

pub trait AABound {
	fn get_aabb(&self) -> Option<AABB>;
}

#[derive(Copy, Clone, Debug)]
pub struct AABB {
	pub min: Vec3,
	pub max: Vec3,
}

impl AABB {
	pub fn new(min: Vec3, max: Vec3) -> Self {
		if (min.x >= max.x || min.y >= max.y || min.z >= max.z) && (min != max) {
			panic!("Maximum value in AABB must be greater or equal to minimum!");
		}
		AABB { min, max }
	}

	pub fn does_int(&self, ray: &Ray) -> bool {
		let t1 = (self.min.x - ray.origin.x) * ray.d_inverse.x;
		let t2 = (self.max.x - ray.origin.x) * ray.d_inverse.x;

		let tmin = t1.min(t2);
		let tmax = t1.max(t2);

		let t1 = (self.min.y - ray.origin.y) * ray.d_inverse.y;
		let t2 = (self.max.y - ray.origin.y) * ray.d_inverse.y;

		let tmin = tmin.max(t1.min(t2));
		let tmax = tmax.min(t1.max(t2));
		let t1 = (self.min.z - ray.origin.z) * ray.d_inverse.z;
		let t2 = (self.max.z - ray.origin.z) * ray.d_inverse.z;

		let tmin = tmin.max(t1.min(t2));
		let tmax = tmax.min(t1.max(t2));

		tmax > tmin.max(0.0)
	}

	pub fn merge(aabb: &mut Option<Self>, second: Self) {
		match aabb {
			Some(inner) => {
				inner.min = inner.min.min_by_component(second.min);
				inner.max = inner.max.max_by_component(second.max);
			}
			None => *aabb = Some(second),
		}
	}

	pub fn extend_contains(aabb: &mut Option<Self>, point: Vec3) {
		match aabb {
			Some(inner) => {
				inner.min = inner.min.min_by_component(point);
				inner.max = inner.max.max_by_component(point);
			}
			None => *aabb = Some(AABB::new(point, point)),
		}
	}

	pub fn get_extent(&self) -> Vec3 {
		self.max - self.min
	}

	pub fn surface_area(&self) -> Float {
		let extent = self.get_extent();
		2.0 * (extent.x * extent.y + extent.x * extent.z + extent.y * extent.z) as Float
	}
}
