use crate::image::ray::Color;

use crate::image::ray::Ray;

#[derive(Copy, Clone)]
pub struct Sky {
    color: Option<Color>,
}

impl Sky {
    pub fn new(color: Option<Color>) -> Self {
        Sky { color }
    }

    pub fn get_color(&self, ray: &Ray) -> Color {
        match self.color {
            Some(color) => {
                let t: f64 = 0.5 * (ray.direction.y + 1.0);
                (1.0 - t) * Color::one() + t * color
            }
            None => Color::zero(),
        }
    }
}
