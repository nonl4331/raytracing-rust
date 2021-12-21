mod generate;
mod parameters;
mod utility;

use crate::utility::get_readable_duration;
use crate::utility::line_break;
use crate::utility::{get_progress_output, save_u8_to_image};
use chrono::Local;
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some((scene, parameters)) = parameters::process_args(args) {
        let (width, height, samples, filename) = (
            parameters.width,
            parameters.height,
            parameters.samples,
            parameters.filename.clone(),
        );

        let time = Local::now();
        println!("{} - Render started", time.format("%X"));
        println!("\tWidth: {}", width);
        println!("\tHeight: {}", height);
        println!("\tSamples per pixel: {}\n", samples);
        let start = Instant::now();
        let progresses = scene.generate_image_threaded(width, height, samples);
        let output = get_progress_output(&parameters, &progresses);
        let end = Instant::now();
        let duration = end.checked_duration_since(start).unwrap();

        let mut ray_count = 0;
        for progress in &progresses {
            ray_count += progress.read().unwrap().rays_shot;
        }
        let time = Local::now();
        println!(
            "\u{001b}[2K\r{} - Finised rendering image",
            time.format("%X")
        );
        println!("\tRender Time: {}", get_readable_duration(duration));
        println!("\tRays: {}", ray_count);
        println!(
            "\tMrays/s: {:.2}",
            (ray_count as f64 / duration.as_secs_f64()) / 1000000.0
        );
        line_break();

        save_u8_to_image(width, height, output, filename);
    }
}
