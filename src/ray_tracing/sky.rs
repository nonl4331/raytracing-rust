use crate::ray_tracing::{
    ray::{Colour, Ray},
    texture::{TextureEnum, TextureTrait},
};
use crate::utility::vec::{Vec2, Vec3};
use std::sync::Arc;

#[cfg(all(feature = "f64"))]
use std::f64::consts::PI;

#[cfg(not(feature = "f64"))]
use std::f32::consts::PI;

pub struct Sky {
    texture: Option<Arc<TextureEnum>>,
}

impl Sky {
    pub fn new(texture: Option<&Arc<TextureEnum>>) -> Self {
        let texture = texture.cloned();
        Sky { texture }
    }

    pub fn get_colour(&self, ray: &Ray) -> Colour {
        match &self.texture {
            Some(texture) => {
                let direction = ray.direction.normalised();
                if texture.requires_uv() {
                    let phi = (-1.0 * direction.z).atan2(direction.x) + PI;
                    let theta = (direction.y).acos();

                    return texture
                        .colour_value(Some(Vec2::new(phi / (2.0 * PI), theta / PI)), Vec3::zero());
                }
                texture.colour_value(None, Vec3::zero())
            }
            None => Colour::zero(),
        }
    }
}
