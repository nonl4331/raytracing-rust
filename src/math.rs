use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};

use ultraviolet::vec::Vec3;

pub fn random_unit_vector() -> Vec3 {
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    let (mut x, mut y, mut z) = (1.0, 1.0, 1.0);
    while x * x + y * y + z * z > 1.0 {
        x = rng.gen_range(-1.0..1.0);
        y = rng.gen_range(-1.0..1.0);
        z = rng.gen_range(-1.0..1.0);
    }

    Vec3::new(x, y, z).normalized()
}

pub fn random_in_unit_disk() -> Vec3 {
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    let mut point = Vec3::new(1.0, 1.0, 0.0);

    while point.mag_sq() >= 1.0 {
        point.x = rng.gen_range(-1.0..1.0);
        point.y = rng.gen_range(-1.0..1.0);
    }
    point
}

pub fn random_f32() -> f32 {
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    rng.gen()
}

pub fn near_zero(vec: Vec3) -> bool {
    let s = 0.001;
    vec.x.abs() < s && vec.y.abs() < s && vec.z.abs() < s
}

pub fn next_float(mut float: f32) -> f32 {
    if float.is_infinite() && float > 0.0 {
        return float;
    }

    if float == -0.0 {
        float = 0.0
    }

    f32::from_bits(if float >= 0.0 {
        f32::to_bits(float) + 1
    } else {
        f32::to_bits(float) - 1
    })
}

pub fn previous_float(mut float: f32) -> f32 {
    if float.is_infinite() && float < 0.0 {
        return float;
    }

    if float == 0.0 {
        float = -0.0
    }

    f32::from_bits(if float <= 0.0 {
        f32::to_bits(float) + 1
    } else {
        f32::to_bits(float) - 1
    })
}
