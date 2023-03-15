use fern::colors::{Color, ColoredLevelConfig};

use std::process;
use std::time::Instant;

use std::io::Write;
use std::time::Duration;

pub fn create_logger() {
	let colors = ColoredLevelConfig::new()
		.error(Color::Red)
		.warn(Color::Yellow)
		.info(Color::Cyan)
		.debug(Color::Magenta);

	fern::Dispatch::new()
		.format(move |out, message, record| {
			out.finish(format_args!(
				"{} {} [{}] {}",
				chrono::Local::now().format("%H:%M:%S"),
				colors.color(record.level()),
				record.target(),
				message
			))
		})
		.level(log::LevelFilter::Info)
		.level_for("winit", log::LevelFilter::Warn)
		.chain(std::io::stderr())
		.apply()
		.unwrap();
}

pub fn get_readable_duration(duration: Duration) -> String {
	let days = duration.as_secs() / 86400;

	let days_string = match days {
		0 => "".to_string(),
		1 => format!("{days} day, "),
		_ => format!("{days} days, "),
	};

	let hours = (duration.as_secs() - days * 86400) / 3600;
	let hours_string = match hours {
		0 => "".to_string(),
		1 => format!("{hours} hour, "),
		_ => format!("{hours} hours, "),
	};

	let minutes = (duration.as_secs() - days * 86400 - hours * 3600) / 60;
	let minutes_string = match minutes {
		0 => "".to_string(),
		1 => format!("{minutes} minute, "),
		_ => format!("{minutes} minutes, "),
	};

	let seconds = duration.as_secs() % 60;
	let seconds_string = match seconds {
		0 => "~0 seconds".to_string(),
		1 => format!("{seconds} second"),
		_ => format!("{seconds} seconds"),
	};
	days_string + &hours_string + &minutes_string + &seconds_string
}

pub fn save_u8_to_image(width: u64, height: u64, image: Vec<u8>, filename: String, alpha: bool) {
	let split = filename.split('.').collect::<Vec<_>>();
	if split.len() != 2 {
		println!("Invalid filename: {filename}");
		process::exit(0);
	}

	let extension = split[1];

	match extension {
		"png" | "jpg" | "jpeg" | "exr" | "tiff" => {
			image::save_buffer(
				filename,
				&image,
				width.try_into().unwrap(),
				height.try_into().unwrap(),
				if alpha {
					image::ColorType::Rgba8
				} else {
					image::ColorType::Rgb8
				},
			)
			.unwrap();
		}
		"ppm" => {
			let mut data = format!("P3\n{width} {height}\n255\n").as_bytes().to_owned();

			image.iter().enumerate().for_each(|(i, &v)| {
				if i % 3 == 0 {
					data.extend_from_slice(format!("{v}\n").as_bytes())
				} else {
					data.extend_from_slice(format!("{v} ").as_bytes())
				}
			});

			let mut file = std::fs::File::create(filename).unwrap();
			file.write_all(&data).unwrap();
		}
		_ => {
			println!("Unknown filetype: .{extension}");
		}
	}
}

pub fn print_final_statistics(start: Instant, ray_count: u64, samples: u64) {
	let end = Instant::now();
	let duration = end.checked_duration_since(start).unwrap();

	log::info!(
			"Finished rendering:\n\tSamples:\t{samples}\n\tTime taken:\t{}\n\tRays shot:\t{ray_count} @ {:.2} Mray/s",
			get_readable_duration(duration),
			(ray_count as f64 / duration.as_secs_f64()) / 1000000.0,
		)
}

pub fn print_render_start(width: u64, height: u64, samples: Option<u64>) -> Instant {
	match samples {
		Some(samples) => log::info!(
			"Render started:\n\tWidth:\t\t{width}\n\tHeight:\t\t{height}\n\tSamples:\t{samples}"
		),
		None => {
			log::info!("Render started:\n\tWidth:\t\t{width}\n\tHeight:\t\t{height}\n\tSamples:\t∞")
		}
	}
	Instant::now()
}
