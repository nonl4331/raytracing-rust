use chrono::Local;
use implementations::{aabb::AABound, rt_core::*, split::SplitType, Axis, Bvh};
use std::{
	io::{stdout, Write},
	process,
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

pub fn create_bvh_with_info<P: Primitive<M> + AABound, M: Scatter>(
	primitives: Vec<P>,
	bvh_type: SplitType,
) -> Arc<Bvh<P, M>> {
	let time = Local::now();

	println!("\n{} - Bvh construction started at", time.format("%X"));

	let start = Instant::now();
	let bvh = Arc::new(Bvh::new(primitives, bvh_type));
	let end = Instant::now();
	let duration = end.checked_duration_since(start).unwrap();

	println!("\tBvh construction finished in: {}ms", duration.as_millis());
	println!("\tNumber of BVH nodes: {}\n", bvh.number_nodes());

	bvh
}

pub fn get_progress_output(samples_completed: u64, total_samples: u64) {
	progress_bar(samples_completed as f64 / total_samples as f64);

	print!(" ({samples_completed}/{total_samples}) samples");

	stdout().flush().unwrap();
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

pub fn print_final_statistics(start: Instant, ray_count: u64, samples: Option<u64>) {
	let end = Instant::now();
	let duration = end.checked_duration_since(start).unwrap();
	let time = Local::now();
	println!(
		"\u{001b}[2K\r{} - Finised rendering image",
		time.format("%X")
	);
	println!("\tRender Time: {}", get_readable_duration(duration));
	println!("\tRays: {ray_count}");
	match samples {
		Some(samples) => println!("\tSamples: {samples}"),
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
	println!("\tWidth: {width}");
	println!("\tHeight: {height}");
	match samples {
		Some(samples) => println!("\tSamples per pixel: {samples}\n"),
		None => println!(),
	}
	Instant::now()
}

pub fn rotate_around_point(
	point: &mut Vec3,
	center_point: Vec3,
	sin_angles: Vec3,
	cos_angles: Vec3,
) {
	*point -= center_point;

	rotate_around_axis(point, Axis::X, sin_angles.x, cos_angles.x);
	rotate_around_axis(point, Axis::Y, sin_angles.y, cos_angles.y);
	rotate_around_axis(point, Axis::Z, sin_angles.z, cos_angles.z);

	*point += center_point;
}

pub fn rotate_around_axis(point: &mut Vec3, axis: Axis, sin: Float, cos: Float) {
	match axis {
		Axis::X => {
			let old_y = point.y;
			point.y = cos * point.y - sin * point.z;
			point.z = sin * old_y + cos * point.z;
		}
		Axis::Y => {
			let old_x = point.x;
			point.x = cos * point.x - sin * point.z;
			point.z = sin * old_x + cos * point.z;
		}
		Axis::Z => {
			let old_y = point.y;
			point.y = cos * point.y - sin * point.x;
			point.x = sin * old_y + cos * point.x;
		}
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	use implementations::rt_core::PI;

	#[test]
	fn rotation() {
		let center_point = Vec3::new(1.0, 0.0, 0.0);

		let mut point = Vec3::new(2.0, 0.0, 0.0);

		let angles = Vec3::new(0.0, 45.0 * PI / 180.0, 0.0);

		let cos_angles = Vec3::new(angles.x.cos(), angles.y.cos(), angles.z.cos());
		let sin_angles = Vec3::new(angles.x.sin(), angles.y.sin(), angles.z.sin());

		rotate_around_point(&mut point, center_point, sin_angles, cos_angles);

		assert!((point - Vec3::new(1.707107, 0.0, 0.7071069)).abs().mag() < 0.000001);
	}
}
