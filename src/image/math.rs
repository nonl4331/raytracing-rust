use rand::Rng;
use ultraviolet::vec::Vec3;

pub fn random_unit_vector() -> Vec3 {
    let mut rng = rand::thread_rng();
    let (mut x, mut y, mut z) = (1.0, 1.0, 1.0);
    while x * x + y * y + z * z > 1.0 {
        x = rng.gen_range(-1.0..1.0);
        y = rng.gen_range(-1.0..1.0);
        z = rng.gen_range(-1.0..1.0);
    }

    Vec3::new(x, y, z).normalized()
}
