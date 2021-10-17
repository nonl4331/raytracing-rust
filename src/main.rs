use crate::image::parameters;

use std::env;

mod acceleration;

mod image;

mod ray_tracing;

mod utility;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some((scene, parameters)) = parameters::process_args(args) {
        scene.generate_image_threaded(parameters);
    }
}
