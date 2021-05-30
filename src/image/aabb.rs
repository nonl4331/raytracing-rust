use ultraviolet::DVec3;

#[derive(Copy, Clone)]
pub struct AABB {
    pub min: DVec3,
    pub max: DVec3,
}

impl AABB {
    pub fn new(min: DVec3, max: DVec3) -> Self {
        if min.x >= max.x || min.y >= max.y || min.z >= max.z {
            panic!("Maximum value in AABB must be strictly greater than minimum!");
        }
        AABB { min, max }
    }
    pub fn new_contains(boxes: &Vec<AABB>) -> Self {
        if boxes.len() != 0 {
            let mut min = DVec3::new(std::f64::INFINITY, std::f64::INFINITY, std::f64::INFINITY);
            let mut max = DVec3::new(
                std::f64::NEG_INFINITY,
                std::f64::NEG_INFINITY,
                std::f64::NEG_INFINITY,
            );
            for bb in boxes {
                min = DVec3::new(
                    min.x.min(bb.min.x),
                    min.y.min(bb.min.y),
                    min.z.min(bb.min.z),
                );
                max = DVec3::new(
                    max.x.min(bb.max.x),
                    max.y.min(bb.max.y),
                    max.z.min(bb.max.z),
                );
            }
            AABB::new(min, max);
        }
        panic!("AABB::new_contains() was called with an empty vector!");
    }
}
