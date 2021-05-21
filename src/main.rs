use crate::image::parameters;
use std::env;

mod image;

fn main() {
    let args: Vec<String> = env::args().collect();
    let parameters = parameters::process_args(args);
    match parameters {
        Some(parameters) => {
            parameters.scene.generate_image_sample_threaded(
                &parameters.filename,
                parameters.width,
                parameters.samples,
            );
        }
        None => {}
    }
}
