use ultraviolet::DVec3;

use ultraviolet::DVec2;

use crate::image::ray::Color;

use image::GenericImageView;

pub enum Texture {
    CheckeredTexture(CheckeredTexture),
    SolidColor(SolidColor),
    ImageTexture(ImageTexture),
}

pub trait TextureTrait {
    fn color_value(&self, _: Option<DVec2>, _: DVec3) -> Color {
        Color::new(1.0, 1.0, 1.0)
    }
    fn requires_uv(&self) -> bool {
        false
    }
}

impl TextureTrait for Texture {
    fn color_value(&self, uv: Option<DVec2>, point: DVec3) -> Color {
        match self {
            Texture::CheckeredTexture(texture) => texture.color_value(uv, point),
            Texture::SolidColor(texture) => texture.color_value(uv, point),
            Texture::ImageTexture(texture) => texture.color_value(uv, point),
        }
    }
    fn requires_uv(&self) -> bool {
        match self {
            Texture::CheckeredTexture(_) => false,
            Texture::SolidColor(_) => false,
            Texture::ImageTexture(_) => true,
        }
    }
}

pub struct CheckeredTexture {
    primary_color: Color,
    secondary_color: Color,
}

impl CheckeredTexture {
    pub fn new(primary_color: Color, secondary_color: Color) -> Self {
        CheckeredTexture {
            primary_color,
            secondary_color,
        }
    }
}

impl TextureTrait for CheckeredTexture {
    fn color_value(&self, _: Option<DVec2>, point: DVec3) -> Color {
        let sign = (10.0 * point.x).sin() * (10.0 * point.y).sin() * (10.0 * point.z).sin();
        if sign > 0.0 {
            self.primary_color
        } else {
            self.secondary_color
        }
    }
    fn requires_uv(&self) -> bool {
        false
    }
}

pub struct SolidColor {
    pub color: Color,
}

impl SolidColor {
    pub fn new(color: Color) -> Self {
        SolidColor { color }
    }
}

impl TextureTrait for SolidColor {
    fn color_value(&self, _: Option<DVec2>, _: DVec3) -> Color {
        self.color
    }
    fn requires_uv(&self) -> bool {
        false
    }
}

pub struct ImageTexture {
    pub data: Vec<Color>,
    pub dim: (usize, usize),
}

impl ImageTexture {
    pub fn new(filepath: &str) -> Self {
        // open image and get dimensions
        let img = image::open(filepath).unwrap();

        // make sure image in non-zero
        let dim = img.dimensions();
        assert!(dim.0 != 0 && dim.1 != 0);

        // - 1 to prevent indices out of range in color_value
        let dim = ((dim.0 - 1) as usize, (dim.1 - 1) as usize);

        // get raw pixel data as Vec<u16> then convert to Vec<Color>
        let mut data: Vec<Color> = Vec::new();
        for col in (img.to_rgb8().to_vec())
            .to_vec()
            .iter()
            .map(|val| *val as f64 / 255.999)
            .collect::<Vec<f64>>()
            .chunks(3)
        {
            data.push(Color::new(
                *col.get(0).unwrap(),
                *col.get(1).unwrap(),
                *col.get(2).unwrap(),
            ));
        }

        Self { data, dim }
    }
}

impl TextureTrait for ImageTexture {
    fn color_value(&self, uv: Option<DVec2>, _: DVec3) -> Color {
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
