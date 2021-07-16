use crate::ray_tracing::ray::Ray;

use ultraviolet::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        if min.x >= max.x || min.y >= max.y || min.z >= max.z {
            panic!("Maximum value in AABB must be strictly greater than minimum!");
        }
        AABB { min, max }
    }
    pub fn new_contains(boxes: &Vec<AABB>) -> Self {
        if boxes.len() == 0 {
            panic!("AABB::new_contains() was called with an empty vector!");
        }
        let mut min = Vec3::new(std::f32::INFINITY, std::f32::INFINITY, std::f32::INFINITY);
        let mut max = Vec3::new(
            std::f32::NEG_INFINITY,
            std::f32::NEG_INFINITY,
            std::f32::NEG_INFINITY,
        );
        for bb in boxes {
            min = Vec3::new(
                min.x.min(bb.min.x),
                min.y.min(bb.min.y),
                min.z.min(bb.min.z),
            );
            max = Vec3::new(
                max.x.max(bb.max.x),
                max.y.max(bb.max.y),
                max.z.max(bb.max.z),
            );
        }
        AABB::new(min, max)
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

        return tmax > tmin.max(0.0);
    }
}
