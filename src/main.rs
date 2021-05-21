use crate::image::parameters;
use std::env;

mod image;

fn main() {
    let args: Vec<String> = env::args().collect();
    match parameters::process_args(args) {
        Some(scene) => scene.generate_image_sample_threaded("out.png", 1920, 30),
        None => {}
    }
}
