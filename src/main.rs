use crate::image::parameters;
use std::env;

mod image;

fn main() {
    let args: Vec<String> = env::args().collect();
    match parameters::process_args(args) {
        Some((scene, parameters)) => {
            scene.generate_image_sample_threaded(parameters);
        }
        None => {}
    }
}
