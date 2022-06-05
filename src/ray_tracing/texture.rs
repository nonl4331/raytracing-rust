use crate::{
	ray_tracing::Colour,
	utility::{
		vec::{Vec2, Vec3},
		Float,
	},
};
use enum_dispatch::enum_dispatch;
use image::GenericImageView;
use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};

#[cfg(all(feature = "f64"))]
use std::f64::consts::PI;

#[cfg(not(feature = "f64"))]
use std::f32::consts::PI;

const PERLIN_RVECS: usize = 256;

#[enum_dispatch(TextureTrait)]
pub enum TextureEnum {
	CheckeredTexture(CheckeredTexture),
	SolidColour(SolidColour),
	ImageTexture(ImageTexture),
	Lerp(Lerp),
	Perlin(Box<Perlin>),
}

#[enum_dispatch]
pub trait TextureTrait {
	fn colour_value(&self, _: Vec3, _: Vec3) -> Colour {
		Colour::new(1.0, 1.0, 1.0)
	}
	fn requires_uv(&self) -> bool {
		false
	}
}

pub struct CheckeredTexture {
	primary_colour: Colour,
	secondary_colour: Colour,
}

impl CheckeredTexture {
	pub fn new(primary_colour: Colour, secondary_colour: Colour) -> Self {
		CheckeredTexture {
			primary_colour,
			secondary_colour,
		}
	}
}

impl TextureTrait for CheckeredTexture {
	fn colour_value(&self, _: Vec3, point: Vec3) -> Colour {
		let sign = (10.0 * point.x).sin() * (10.0 * point.y).sin() * (10.0 * point.z).sin();
		if sign > 0.0 {
			self.primary_colour
		} else {
			self.secondary_colour
		}
	}
	fn requires_uv(&self) -> bool {
		false
	}
}

pub struct Perlin {
	ran_vecs: [Vec3; PERLIN_RVECS],
	perm_x: [u32; PERLIN_RVECS],
	perm_y: [u32; PERLIN_RVECS],
	perm_z: [u32; PERLIN_RVECS],
}

impl Perlin {
	pub fn new() -> Self {
		let mut rng = SmallRng::from_rng(thread_rng()).unwrap();

		let mut ran_vecs: [Vec3; PERLIN_RVECS] = [Vec3::one(); PERLIN_RVECS];
		for ran_vec in &mut ran_vecs {
			*ran_vec = rng.gen_range(-1.0..1.0) * Vec3::one();
		}

		let perm_x = Self::generate_perm();
		let perm_y = Self::generate_perm();
		let perm_z = Self::generate_perm();

		Perlin {
			ran_vecs,
			perm_x,
			perm_y,
			perm_z,
		}
	}

	pub fn noise(&self, point: Vec3) -> Float {
		let u = point.x - point.x.floor();

		let v = point.y - point.y.floor();

		let w = point.z - point.z.floor();

		let i = point.x.floor() as i32;
		let j = point.y.floor() as i32;
		let k = point.z.floor() as i32;
		let mut c: [Vec3; 8] = [Vec3::one(); 8];

		for index in 0..8 {
			let di = (index / 4) as i32;
			let dj = ((index / 2) % 2) as i32;
			let dk = (index % 2) as i32;
			c[index] = self.ran_vecs[(self.perm_x[((i + di) & 255) as usize]
				^ self.perm_y[((j + dj) & 255) as usize]
				^ self.perm_z[((k + dk) & 255) as usize]) as usize];
		}

		Perlin::trilinear_lerp(c, u, v, w)
	}

	fn generate_perm() -> [u32; PERLIN_RVECS] {
		let mut perm: [u32; PERLIN_RVECS] = [0; PERLIN_RVECS];
		for (i, perm) in perm.iter_mut().enumerate() {
			*perm = i as u32;
		}
		Self::permute(&mut perm);
		perm
	}

	fn permute(perm: &mut [u32; 256]) {
		let mut rng = rand::rngs::SmallRng::from_rng(rand::thread_rng()).unwrap();

		for i in (1..256).rev() {
			let target = rng.gen_range(0..i);
			perm[0..256].swap(i, target);
		}
	}

	fn trilinear_lerp(c: [Vec3; 8], u: Float, v: Float, w: Float) -> Float {
		let uu = u * u * (3.0 - 2.0 * u);
		let vv = v * v * (3.0 - 2.0 * v);
		let ww = w * w * (3.0 - 2.0 * w);

		let mut value = 0.0;
		for index in 0..8 {
			let i = index / 4;
			let j = (index / 2) % 2;
			let k = index % 2;
			let weight = Vec3::new(u - i as Float, v - j as Float, w - k as Float);
			value += (i as Float * uu + (1.0 - i as Float) * (1.0 - uu))
				* (j as Float * vv + (1.0 - j as Float) * (1.0 - vv))
				* (k as Float * ww + (1.0 - k as Float) * (1.0 - ww))
				* c[i * 4 + j * 2 + k * 1].dot(weight);
		}
		value
	}
}

impl TextureTrait for Box<Perlin> {
	fn colour_value(&self, _: Vec3, point: Vec3) -> Colour {
		0.5 * Colour::one() * (1.0 + self.noise(point))
	}

	fn requires_uv(&self) -> bool {
		false
	}
}

pub struct SolidColour {
	pub colour: Colour,
}

impl SolidColour {
	pub fn new(colour: Colour) -> Self {
		SolidColour { colour }
	}
}

impl TextureTrait for SolidColour {
	fn colour_value(&self, _: Vec3, _: Vec3) -> Colour {
		self.colour
	}
	fn requires_uv(&self) -> bool {
		false
	}
}

pub struct ImageTexture {
	pub data: Vec<Colour>,
	pub dim: (usize, usize),
}

impl ImageTexture {
	pub fn new(filepath: &str) -> Self {
		// open image and get dimensions
		let img = image::open(filepath).unwrap();

		// make sure image in non-zero
		let dim = img.dimensions();
		assert!(dim.0 != 0 && dim.1 != 0);

		// - 1 to prevent indices out of range in colour_value
		let dim = ((dim.0 - 1) as usize, (dim.1 - 1) as usize);

		// get raw pixel data as Vec<u16> then convert to Vec<Colour>
		let mut data: Vec<Colour> = Vec::new();
		for col in (img.to_rgb8().to_vec())
			.to_vec()
			.iter()
			.map(|val| *val as Float / 255.999)
			.collect::<Vec<Float>>()
			.chunks(3)
		{
			data.push(Colour::new(
				*col.get(0).unwrap(),
				*col.get(1).unwrap(),
				*col.get(2).unwrap(),
			));
		}

		Self { data, dim }
	}
}

impl TextureTrait for ImageTexture {
	fn colour_value(&self, direction: Vec3, _: Vec3) -> Colour {
		let phi = direction.z.atan2(direction.x) + PI;
		let theta = direction.y.acos();
		let uv = Vec2::new(phi / (2.0 * PI), theta / PI);
		let x_pixel = (self.dim.0 as Float * uv.x) as usize;
		let y_pixel = (self.dim.1 as Float * uv.y) as usize;

		// + 1 to get width in pixels
		let index = y_pixel * (self.dim.0 + 1) + x_pixel;
		self.data[index]
	}
	fn requires_uv(&self) -> bool {
		true
	}
}

pub struct Lerp {
	pub colour_one: Colour,
	pub colour_two: Colour,
}

impl Lerp {
	pub fn new(colour_one: Colour, colour_two: Colour) -> Self {
		Lerp {
			colour_one,
			colour_two,
		}
	}
}

impl TextureTrait for Lerp {
	fn colour_value(&self, direction: Vec3, _: Vec3) -> Colour {
		let t = direction.y * 0.5 + 0.5;
		self.colour_one * t + self.colour_two * (1.0 - t)
	}
	fn requires_uv(&self) -> bool {
		true
	}
}
