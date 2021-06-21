use ultraviolet::DVec3;

use ultraviolet::DVec2;

use crate::image::ray::Colour;

use image::GenericImageView;

pub enum Texture {
    CheckeredTexture(CheckeredTexture),
    SolidColour(SolidColour),
    ImageTexture(ImageTexture),
}

pub trait TextureTrait {
    fn colour_value(&self, _: Option<DVec2>, _: DVec3) -> Colour {
        Colour::new(1.0, 1.0, 1.0)
    }
    fn requires_uv(&self) -> bool {
        false
    }
}

impl TextureTrait for Texture {
    fn colour_value(&self, uv: Option<DVec2>, point: DVec3) -> Colour {
        match self {
            Texture::CheckeredTexture(texture) => texture.colour_value(uv, point),
            Texture::SolidColour(texture) => texture.colour_value(uv, point),
            Texture::ImageTexture(texture) => texture.colour_value(uv, point),
        }
    }
    fn requires_uv(&self) -> bool {
        match self {
            Texture::CheckeredTexture(_) => false,
            Texture::SolidColour(_) => false,
            Texture::ImageTexture(_) => true,
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
    fn colour_value(&self, _: Option<DVec2>, point: DVec3) -> Colour {
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

pub struct SolidColour {
    pub colour: Colour,
}

impl SolidColour {
    pub fn new(colour: Colour) -> Self {
        SolidColour { colour }
    }
}

impl TextureTrait for SolidColour {
    fn colour_value(&self, _: Option<DVec2>, _: DVec3) -> Colour {
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
            .map(|val| *val as f64 / 255.999)
            .collect::<Vec<f64>>()
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
    fn colour_value(&self, uv: Option<DVec2>, _: DVec3) -> Colour {
        let uv = uv.unwrap();
        let x_pixel = (self.dim.0 as f64 * uv.x) as usize;
        let y_pixel = (self.dim.1 as f64 * uv.y) as usize;

        // + 1 to get width in pixels
        let index = y_pixel * (self.dim.0 + 1) + x_pixel;
        self.data[index]
    }
    fn requires_uv(&self) -> bool {
        true
    }
}
