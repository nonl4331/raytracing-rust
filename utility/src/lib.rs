extern crate chrono;
extern crate cpu_raytracer;
extern crate image;

pub mod generate;
pub mod parameters;

use chrono::Local;
use cpu_raytracer::{
	acceleration::Bvh, material::Scatter, ray_tracing::intersection::Primitive, *,
};
use std::{
	convert::TryInto,
	io::{stdout, Write},
	sync::Arc,
	time::{Duration, Instant},
};

const BAR_LENGTH: u32 = 30;

pub fn progress_bar(percentage: f64) {
	print!("\r[");
	for i in 1..=BAR_LENGTH {
		if percentage >= i as f64 / BAR_LENGTH as f64
			&& percentage < (i + 1) as f64 / BAR_LENGTH as f64
			&& i != BAR_LENGTH
		{
			print!(">");
		} else if percentage >= i as f64 / BAR_LENGTH as f64 {
			print!("=");
		} else {
			print!("-");
		}
	}
	print!("]");
}

pub fn line_break() {
	println!("--------------------------------");
}

pub fn get_readable_duration(duration: Duration) -> String {
	let days = duration.as_secs() / 86400;

	let days_string = match days {
		0 => "".to_string(),
		1 => format!("{} day, ", days),
		_ => format!("{} days, ", days),
	};

	let hours = (duration.as_secs() - days * 86400) / 3600;
	let hours_string = match hours {
		0 => "".to_string(),
		1 => format!("{} hour, ", hours),
		_ => format!("{} hours, ", hours),
	};

	let minutes = (duration.as_secs() - days * 86400 - hours * 3600) / 60;
	let minutes_string = match minutes {
		0 => "".to_string(),
		1 => format!("{} minute, ", minutes),
		_ => format!("{} minutes, ", minutes),
	};

	let seconds = duration.as_secs() % 60;
	let seconds_string = match seconds {
		0 => "~0 seconds".to_string(),
		1 => format!("{} second", seconds),
		_ => format!("{} seconds", seconds),
	};
	days_string + &hours_string + &minutes_string + &seconds_string
}

pub fn save_u8_to_image(width: u64, height: u64, image: Vec<u8>, filename: String, alpha: bool) {
	if alpha {
		image::save_buffer(
			filename,
			&image,
			width.try_into().unwrap(),
			height.try_into().unwrap(),
			image::ColorType::Rgba8,
		)
		.unwrap();
	} else {
		image::save_buffer(
			filename,
			&image,
			width.try_into().unwrap(),
			height.try_into().unwrap(),
			image::ColorType::Rgb8,
		)
		.unwrap();
	}
}

/*pub fn save_u8_to_image_ppm(width: u64, height: u64, image: Vec<u8>, filename: String) {
	let mut data = format!("P3\n{} {}\n255\n", width, height)
		.as_bytes()
		.to_owned();

	image.iter().enumerate().for_each(|(i, &v)| {
		if i % 3 == 0 {
			data.extend_from_slice(format!("{}\n", v).as_bytes())
		} else {
			data.extend_from_slice(format!("{} ", v).as_bytes())
		}
	});

	let mut file = std::fs::File::create(filename).unwrap();
	file.write(&data).unwrap();
}*/

pub fn get_progress_output(samples_completed: u64, total_samples: u64) {
	progress_bar(samples_completed as f64 / total_samples as f64);

	print!(" ({}/{}) samples", samples_completed, total_samples);

	stdout().flush().unwrap();
}

pub fn create_bvh_with_info<P: Primitive<M>, M: Scatter>(
	primitives: Vec<P>,
	bvh_type: SplitType,
) -> Arc<Bvh<P, M>> {
	let time = Local::now();

	println!("\n{} - Bvh construction started at", time.format("%X"));

	let start = Instant::now();
	let bvh = bvh!(primitives, bvh_type);
	let end = Instant::now();
	let duration = end.checked_duration_since(start).unwrap();

	println!("\tBvh construction finished in: {}ms", duration.as_millis());
	println!("\tNumber of BVH nodes: {}\n", bvh.number_nodes());

	bvh
}

pub fn print_final_statistics(start: Instant, ray_count: u64, samples: Option<u64>) {
	let end = Instant::now();
	let duration = end.checked_duration_since(start).unwrap();
	let time = Local::now();
	println!(
		"\u{001b}[2K\r{} - Finised rendering image",
		time.format("%X")
	);
	println!("\tRender Time: {}", get_readable_duration(duration));
	println!("\tRays: {}", ray_count);
	match samples {
		Some(samples) => println!("\tSamples: {}", samples),
		None => {
			println!()
		}
	}

	println!(
		"\tMrays/s: {:.2}",
		(ray_count as f64 / duration.as_secs_f64()) / 1000000.0
	);
}

pub fn print_render_start(width: u64, height: u64, samples: Option<u64>) -> Instant {
	let time = Local::now();
	println!("{} - Render started", time.format("%X"));
	println!("\tWidth: {}", width);
	println!("\tHeight: {}", height);
	match samples {
		Some(samples) => println!("\tSamples per pixel: {}\n", samples),
		None => println!(),
	}
	Instant::now()
}
