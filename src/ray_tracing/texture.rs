use crate::math::Float;
use crate::ray_tracing::ray::Colour;

use image::GenericImageView;

use ultraviolet::{Vec2, Vec3};

use rand::{Rng, SeedableRng};

pub enum Texture {
    CheckeredTexture(CheckeredTexture),
    SolidColour(SolidColour),
    ImageTexture(ImageTexture),
    Lerp(Lerp),
    Perlin(Perlin),
}

pub trait TextureTrait {
    fn colour_value(&self, _: Option<Vec2>, _: Vec3) -> Colour {
        Colour::new(1.0, 1.0, 1.0)
    }
    fn requires_uv(&self) -> bool {
        false
    }
}

impl TextureTrait for Texture {
    fn colour_value(&self, uv: Option<Vec2>, point: Vec3) -> Colour {
        match self {
            Texture::CheckeredTexture(texture) => texture.colour_value(uv, point),
            Texture::SolidColour(texture) => texture.colour_value(uv, point),
            Texture::ImageTexture(texture) => texture.colour_value(uv, point),
            Texture::Lerp(texture) => texture.colour_value(uv, point),
            Texture::Perlin(texture) => texture.colour_value(uv, point),
        }
    }
    fn requires_uv(&self) -> bool {
        match self {
            Texture::CheckeredTexture(_) => false,
            Texture::SolidColour(_) => false,
            Texture::ImageTexture(_) => true,
            Texture::Lerp(_) => true,
            Texture::Perlin(_) => false,
        }
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
    fn colour_value(&self, _: Option<Vec2>, point: Vec3) -> Colour {
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
    ran_float: [Float; 256],
    perm_x: [u32; 256],
    perm_y: [u32; 256],
    perm_z: [u32; 256],
}

impl Perlin {
    pub fn new() -> Self {
        let mut ran_float: [Float; 256] = [0.0; 256];
        for i in 0..256 {
            ran_float[i] = crate::math::random_float();
        }
        let perm_x = Self::generate_perm();
        let perm_y = Self::generate_perm();
        let perm_z = Self::generate_perm();

        Perlin {
            ran_float,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    pub fn noise(&self, point: Vec3) -> f32 {
        let i = ((4.0 * point.x) as u32 & 255) as usize;
        let j = ((4.0 * point.y) as u32 & 255) as usize;
        let k = ((4.0 * point.z) as u32 & 255) as usize;

        self.ran_float[(self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]) as usize]
    }

    fn generate_perm() -> [u32; 256] {
        let mut perm: [u32; 256] = [0; 256];
        for i in 0..256 {
            perm[i] = i as u32;
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
}

impl TextureTrait for Perlin {
    fn colour_value(&self, _: Option<Vec2>, point: Vec3) -> Colour {
        Colour::one() * self.noise(point)
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
    fn colour_value(&self, _: Option<Vec2>, _: Vec3) -> Colour {
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
    fn colour_value(&self, uv: Option<Vec2>, _: Vec3) -> Colour {
        let uv = uv.unwrap();
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
    fn colour_value(&self, uv: Option<Vec2>, _: Vec3) -> Colour {
        let uv = uv.unwrap();
        self.colour_one * uv.y + self.colour_two * (1.0 - uv.y)
    }
    fn requires_uv(&self) -> bool {
        true
    }
}
