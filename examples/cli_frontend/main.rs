extern crate cpu_raytracer;

use cpu_raytracer::*;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some((scene, parameters)) = parameters::process_args(args) {
        scene.generate_image_threaded(parameters);
    }
}
