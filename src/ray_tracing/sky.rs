use crate::ray_tracing::ray::{Colour, Ray};

#[derive(Copy, Clone)]
pub struct Sky {
    colour: Option<Colour>,
}

impl Sky {
    pub fn new(colour: Option<Colour>) -> Self {
        Sky { colour }
    }

    pub fn get_colour(&self, ray: &Ray) -> Colour {
        match self.colour {
            Some(colour) => {
                let t: f32 = 0.5 * (ray.direction.y + 1.0);
                (1.0 - t) * Colour::one() + t * colour
            }
            None => Colour::zero(),
        }
    }
}
