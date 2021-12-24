mod generate;
mod parameters;
mod utility;

use crate::utility::get_readable_duration;
use crate::utility::line_break;
use crate::utility::{get_progress_output, save_u8_to_image};
use chrono::Local;
use cpu_raytracer::SamplerProgress;
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
        let progress_bar_output = |sp: &SamplerProgress| {
            get_progress_output(&parameters, sp);
        };
        let output =
            scene.generate_image_threaded(width, height, samples, Some(progress_bar_output));
        let end = Instant::now();
        let duration = end.checked_duration_since(start).unwrap();

        let ray_count = output.rays_shot;

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

        let output: Vec<u8> = output
            .current_image
            .iter()
            .map(|val| (val.sqrt() * 255.0) as u8)
            .collect();

        save_u8_to_image(width, height, output, filename);
    }
}
