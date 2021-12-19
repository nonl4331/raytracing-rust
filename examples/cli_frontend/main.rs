mod generate;
mod parameters;
mod utility;

use crate::utility::{get_progress_output, save_u8_to_image};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some((scene, parameters)) = parameters::process_args(args) {
        let (width, height, samples, filename) = (
            parameters.width,
            parameters.height,
            parameters.samples,
            parameters.filename.clone(),
        );
        let output = get_progress_output(
            &parameters,
            scene.generate_image_threaded(width, height, samples),
        );
        save_u8_to_image(width, height, output, filename);
    }
}
