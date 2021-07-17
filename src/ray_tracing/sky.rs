use crate::ray_tracing::ray::{Colour, Ray};
use crate::ray_tracing::texture::Texture;
use crate::ray_tracing::texture::TextureTrait;
use core::f32::consts::PI;
use ultraviolet::Vec2;
use ultraviolet::Vec3;

pub struct Sky {
    texture: Option<Texture>,
}

impl Sky {
    pub fn new(texture: Option<Texture>) -> Self {
        Sky { texture }
    }

    pub fn get_colour(&self, ray: &Ray) -> Colour {
        match &self.texture {
            Some(texture) => {
                let direction = ray.direction.normalized();
                if texture.requires_uv() {
                    let phi = (-1.0 * direction.z).atan2(direction.x) + PI;
                    let theta = (direction.y).acos();

                    return texture
                        .colour_value(Some(Vec2::new(phi / (2.0 * PI), theta / PI)), Vec3::zero());
                }
                return texture.colour_value(None, Vec3::zero());
            }
            None => Colour::zero(),
        }
    }
}
