use crate::utility::math::Float;

use crate::ray_tracing::ray::Ray;

use crate::utility::vec::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        if min.x > max.x || min.y > max.y || min.z > max.z {
            panic!("Maximum value in AABB must be greater or equal to minimum!");
        }
        Aabb { min, max }
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
            None => *aabb = Some(Aabb::new(point, point)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn new() {
        let min = -0.5 * Vec3::one();
        let max = 1.5 * Vec3::one();
        Aabb::new(max, min);
    }

    #[test]
    fn merge() {
        let first = Aabb::new(-1.0 * Vec3::one(), Vec3::one());
        let mut second = None;
        Aabb::merge(&mut second, first);
        assert!(first.min == second.unwrap().min && first.max == second.unwrap().max);
    }

    #[test]
    fn extend_contains() {
        let mut first = Some(Aabb::new(-1.0 * Vec3::one(), Vec3::one()));
        Aabb::extend_contains(&mut first, Vec3::new(-1.5, 3.0, 0.1));

        assert!(
            first.unwrap().min == Vec3::new(-1.5, -1.0, -1.0)
                && first.unwrap().max == Vec3::new(1.0, 3.0, 1.0)
        );
    }

    #[test]
    fn ray_intersection() {
        let ray = Ray::new(
            0.5 * Vec3::one(),
            Vec3::new(1.5, 2.5, 1.7).normalised(),
            0.0,
        );

        let aabb = Aabb::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(1.1, 2.0, 2.0));

        assert!(aabb.does_int(&ray));
    }

    #[test]
    fn get_extent() {
        let aabb = Aabb::new(Vec3::new(-1.0, 1.5, 1.0), Vec3::new(1.1, 2.0, 2.0));
        assert_eq!(aabb.get_extent(), Vec3::new(2.1, 0.5, 1.0));
    }
}
