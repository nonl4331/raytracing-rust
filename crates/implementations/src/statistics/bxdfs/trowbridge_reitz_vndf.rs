use crate::coord::Coordinate;
use rand::Rng;
use rt_core::*;

pub mod isotropic {
	use super::*;
	pub use crate::bxdfs::trowbridge_reitz::{d, g1, g2};

	pub fn vndf(a: Float, h: Vec3, incoming: Vec3) -> Float {
		if h.z < 0.0 {
			return 0.0;
		}
		g1(a, Vec3::new(0.0, 0.0, 1.0), h, incoming) * incoming.dot(h).max(0.0) * d(a, h.z)
			/ incoming.z
	}

	pub fn sample_vndf<R: Rng>(a: Float, incoming: Vec3, rng: &mut R) -> Vec3 {
		ansiotropic::sample_vndf(a, a, incoming, rng)
	}

	pub fn sample_local<R: Rng>(a: Float, incoming: Vec3, rng: &mut R) -> Vec3 {
		let h = sample_vndf(a, incoming, rng);
		incoming.reflected(h)
	}

	pub fn pdf_local(alpha: Float, incoming: Vec3, outgoing: Vec3) -> Float {
		let mut h = (outgoing + incoming).normalised();
		if h.z < 0.0 {
			h = -h;
		}
		let vndf = vndf(alpha, h, incoming);
		vndf / (4.0 * incoming.dot(h))
	}

	pub fn sample<R: Rng>(a: Float, incoming: Vec3, normal: Vec3, rng: &mut R) -> Vec3 {
		let coord = Coordinate::new_from_z(normal);
		let inverse = coord.create_inverse();
		let h = coord.to_coord(sample_vndf(a, inverse.to_coord(incoming), rng));
		incoming.reflected(h)
	}

	pub fn pdf(alpha: Float, incoming: Vec3, outgoing: Vec3, normal: Vec3) -> Float {
		let inverse = Coordinate::new_from_z(normal).create_inverse();
		let incoming = inverse.to_coord(incoming);
		let outgoing = inverse.to_coord(outgoing);
		let mut h = (outgoing + incoming).normalised();
		if h.z < 0.0 {
			h = -h;
		}
		let vndf = vndf(alpha, h, incoming);
		vndf / (4.0 * incoming.dot(h))
	}
}
pub mod ansiotropic {
	use super::*;

	pub fn d(a_x: Float, a_y: Float, h: Vec3) -> Float {
		let tmp = h.x * h.x / (a_x * a_x) + h.y * h.y / (a_y * a_y) + h.z * h.z;
		1.0 / (PI * a_x * a_y * tmp * tmp)
	}

	pub fn lambda(a_x: Float, a_y: Float, incoming: Vec3) -> Float {
		let tmp = 1.0
			+ (a_x * a_x * incoming.x * incoming.x + a_y * a_y * incoming.y * incoming.y)
				/ (incoming.z * incoming.z);
		0.5 * (tmp.sqrt() - 1.0)
	}

	pub fn g1(a_x: Float, a_y: Float, incoming: Vec3) -> Float {
		1.0 / (1.0 + lambda(a_x, a_y, incoming))
	}

	pub fn vndf(a_x: Float, a_y: Float, h: Vec3, incoming: Vec3) -> Float {
		if h.z < 0.0 {
			return 0.0;
		}
		g1(a_x, a_y, incoming) * incoming.dot(h).max(0.0) * d(a_x, a_y, h) / incoming.z
	}

	pub fn sample_vndf<R: Rng>(a_x: Float, a_y: Float, incoming: Vec3, rng: &mut R) -> Vec3 {
		let v_hemisphere = Vec3::new(a_x * incoming.x, a_y * incoming.y, incoming.z).normalised();

		let len_sq = v_hemisphere.x * v_hemisphere.x + v_hemisphere.y * v_hemisphere.y;

		let basis_two = if len_sq > 0.0 {
			Vec3::new(-v_hemisphere.y, v_hemisphere.x, 0.0) / len_sq.sqrt()
		} else {
			Vec3::new(1.0, 0.0, 0.0)
		};
		let basis_three = v_hemisphere.cross(basis_two);

		let r = rng.gen::<Float>().sqrt();
		let phi = TAU * rng.gen::<Float>();
		let mut t = r * Vec2::new(phi.cos(), phi.sin());
		let s = 0.5 * (1.0 + v_hemisphere.z);
		t.y = (1.0 - s) * (1.0 - t.x * t.x).sqrt() + s * t.y;

		let h_hemisphere = t.x * basis_two
			+ t.y * basis_three
			+ (1.0 - t.x * t.x - t.y * t.y).max(0.0).sqrt() * v_hemisphere;

		Vec3::new(
			a_x * h_hemisphere.x,
			a_y * h_hemisphere.y,
			h_hemisphere.z.max(0.0),
		)
		.normalised()
	}

	pub fn sample_local<R: Rng>(a_x: Float, a_y: Float, incoming: Vec3, rng: &mut R) -> Vec3 {
		let h = sample_vndf(a_x, a_y, incoming, rng);
		incoming.reflected(h)
	}

	pub fn pdf_local(a_x: Float, a_y: Float, incoming: Vec3, outgoing: Vec3) -> Float {
		let mut h = (outgoing + incoming).normalised();
		if h.z < 0.0 {
			h = -h;
		}
		let vndf = vndf(a_x, a_y, h, incoming);
		vndf / (4.0 * incoming.dot(h))
	}

	pub fn sample<R: Rng>(
		a_x: Float,
		a_y: Float,
		incoming: Vec3,
		normal: Vec3,
		rng: &mut R,
	) -> Vec3 {
		let coord = Coordinate::new_from_z(normal);
		let inverse = coord.create_inverse();
		let h = coord.to_coord(sample_vndf(a_x, a_y, inverse.to_coord(incoming), rng));
		incoming.reflected(h)
	}

	pub fn pdf(a_x: Float, a_y: Float, incoming: Vec3, outgoing: Vec3, normal: Vec3) -> Float {
		let inverse = Coordinate::new_from_z(normal).create_inverse();
		let incoming = inverse.to_coord(incoming);
		let outgoing = inverse.to_coord(outgoing);
		let mut h = (outgoing + incoming).normalised();
		if h.z < 0.0 {
			h = -h;
		}
		let vndf = vndf(a_x, a_y, h, incoming);
		vndf / (4.0 * incoming.dot(h))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::statistics::spherical_sampling::*;
	use rand::{rngs::ThreadRng, thread_rng, Rng};

	#[test]
	fn isotropic_h() {
		let mut rng = thread_rng();
		let incoming = -generate_wi(&mut rng);
		let alpha = rng.gen();
		let pdf = |outgoing: Vec3| isotropic::vndf(alpha, outgoing, incoming);
		let sample = |rng: &mut ThreadRng| isotropic::sample_vndf(alpha, incoming, rng);
		test_spherical_pdf("iso_tr_vndf_h", &pdf, &sample, false);
	}

	#[test]
	fn isotropic() {
		let mut rng = thread_rng();
		let incoming = -generate_wi(&mut rng);
		let alpha = rng.gen();
		let pdf = |outgoing: Vec3| isotropic::pdf_local(alpha, incoming, outgoing);
		let sample = |rng: &mut ThreadRng| isotropic::sample_local(alpha, incoming, rng);
		test_spherical_pdf("iso_tr_vndf", &pdf, &sample, false);
	}

	#[test]
	fn isotropic_non_local() {
		let mut rng = thread_rng();
		let normal = random_unit_vector(&mut rng);
		let to_local = Coordinate::new_from_z(normal);
		let incoming = to_local.to_coord(-generate_wi(&mut rng));
		let alpha = rng.gen();
		let pdf = |outgoing: Vec3| isotropic::pdf(alpha, incoming, outgoing, normal);
		let sample = |rng: &mut ThreadRng| isotropic::sample(alpha, incoming, normal, rng);
		test_spherical_pdf("iso_tr_vndf_nl", &pdf, &sample, false);
	}

	#[test]
	fn ansiotropic_h() {
		let mut rng = thread_rng();
		let incoming = -generate_wi(&mut rng);
		let (a_x, a_y) = (rng.gen(), rng.gen());
		let pdf = |outgoing: Vec3| ansiotropic::vndf(a_x, a_y, outgoing, incoming);
		let sample = |rng: &mut ThreadRng| ansiotropic::sample_vndf(a_x, a_y, incoming, rng);
		test_spherical_pdf("ansio_tr_vndf_h", &pdf, &sample, false);
	}

	#[test]
	fn ansiotropic() {
		let mut rng = thread_rng();
		let incoming = -generate_wi(&mut rng);
		let (a_x, a_y) = (rng.gen(), rng.gen());
		let pdf = |outgoing: Vec3| ansiotropic::pdf_local(a_x, a_y, incoming, outgoing);
		let sample = |rng: &mut ThreadRng| ansiotropic::sample_local(a_x, a_y, incoming, rng);
		test_spherical_pdf("ansio_tr_vndf", &pdf, &sample, false);
	}

	#[test]
	fn ansiotropic_non_local() {
		let mut rng = thread_rng();
		let normal = random_unit_vector(&mut rng);
		let to_local = Coordinate::new_from_z(normal);
		let incoming = to_local.to_coord(-generate_wi(&mut rng));
		let (a_x, a_y) = (rng.gen(), rng.gen());
		let pdf = |outgoing: Vec3| ansiotropic::pdf(a_x, a_y, incoming, outgoing, normal);
		let sample = |rng: &mut ThreadRng| ansiotropic::sample(a_x, a_y, incoming, normal, rng);
		test_spherical_pdf("ansio_tr_vndf_nl", &pdf, &sample, false);
	}
}
