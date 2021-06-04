use ultraviolet::DVec3;

use ultraviolet::DVec2;

use crate::image::ray::Color;

#[derive(Clone, Copy)]
pub enum Texture {
    CheckeredTexture(CheckeredTexture),
    SolidColor(SolidColor),
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
        }
    }
    fn requires_uv(&self) -> bool {
        match self {
            Texture::CheckeredTexture(_) => false,
            Texture::SolidColor(_) => false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct CheckeredTexture {
    primary_color: Color,
    secondary_color: Color,
}

#[derive(Clone, Copy)]
pub struct SolidColor {
    pub color: Color,
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
