use rand::rngs::SmallRng;
use rand::thread_rng;

use rand::{Rng, SeedableRng};
use ultraviolet::vec::DVec3;

pub fn random_unit_vector() -> DVec3 {
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    let (mut x, mut y, mut z) = (1.0, 1.0, 1.0);
    while x * x + y * y + z * z > 1.0 {
        x = rng.gen_range(-1.0..1.0);
        y = rng.gen_range(-1.0..1.0);
        z = rng.gen_range(-1.0..1.0);
    }

    DVec3::new(x, y, z).normalized()
}

pub fn random_in_unit_disk() -> DVec3 {
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    let mut point = DVec3::new(1.0, 1.0, 0.0);

    while point.mag_sq() >= 1.0 {
        point.x = rng.gen_range(-1.0..1.0);
        point.y = rng.gen_range(-1.0..1.0);
    }
    point
}

pub fn random_f64() -> f64 {
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    rng.gen()
}

pub fn near_zero(vec: DVec3) -> bool {
    let s = 0.00000001;
    vec.x.abs() < s && vec.y.abs() < s && vec.z.abs() < s
}
