extern crate cpu_raytracer;
extern crate utility;

use cpu_raytracer::{Float, SamplerProgress};
use std::env;
//use utility::save_u8_to_image_ppm;

use utility::{
	get_progress_output, line_break, parameters, print_final_statistics, print_render_start,
	save_u8_to_image,
};

fn main() {
	let args: Vec<String> = env::args().collect();

	if let Some((scene, parameters)) = parameters::process_args(args) {
		let (width, height, samples, filename) = (
			parameters.width,
			parameters.height,
			parameters.samples,
			parameters.filename.clone(),
		);

		let start = print_render_start(width, height, Some(samples));

		let mut image = Some(SamplerProgress::new(width * height, 3));
		let progress_bar_output =
			|sp: &mut Option<SamplerProgress>, previous: &SamplerProgress, i: u64| {
				if let Some(sp) = sp {
					sp.samples_completed += 1;
					sp.rays_shot += previous.rays_shot;

					sp.current_image
						.iter_mut()
						.zip(previous.current_image.iter())
						.for_each(|(pres, acc)| {
							*pres += (acc - *pres) / i as Float; // since copies first buffer when i=1
						});

					get_progress_output(sp.samples_completed, parameters.samples);
				}
			};

		scene.generate_image_threaded(
			width,
			height,
			samples,
			Some(progress_bar_output),
			&mut image,
		);

		let output = image.unwrap();

		let ray_count = output.rays_shot;

		print_final_statistics(start, ray_count, None);
		line_break();

		let output: Vec<u8> = output
			.current_image
			.iter()
			.map(|val| (val.sqrt() * 255.999) as u8)
			.collect();

		match filename {
			Some(filename) => {
				save_u8_to_image(width, height, output.clone(), filename.clone(), false);
				/*save_u8_to_image_ppm(
					width,
					height,
					output,
					filename[0..(filename.len() - 3)].to_owned() + "ppm",
				)*/
			}
			None => {}
		}
	}
}
