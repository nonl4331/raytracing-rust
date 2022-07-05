use crate::generate::SceneType;
use chrono::Local;
use implementations::split::SplitType;
use rt_core::{Float, RenderMethod, RenderOptions};
use std::process;

pub struct Parameters {
	pub render_options: RenderOptions,
	pub gui: bool,
	pub filename: Option<String>,
}

impl Parameters {
	pub fn new(render_options: RenderOptions, gui: bool, filename: Option<String>) -> Self {
		Self {
			render_options,
			gui,
			filename,
		}
	}
}

pub fn line_break() {
	println!("--------------------------------");
}

macro_rules! scene {
	($scene_name:ident, $bvh_type:expr, $aspect_ratio:expr) => {{
		line_break();
		let time = Local::now();
		println!("{} - Scene Generation started", time.format("%X"));
		crate::generate::$scene_name($bvh_type, $aspect_ratio)
	}};
	($scene_name:ident, $bvh_type:expr, $aspect_ratio:expr, $seed:expr) => {{
		line_break();
		let time = Local::now();
		println!("{} - Scene Generation started", time.format("%X"));
		crate::generate::$scene_name($bvh_type, $aspect_ratio, $seed)
	}};
}

pub fn process_args(args: Vec<String>) -> Option<(SceneType, Parameters)> {
	let mut scene_index = None;
	let mut samples = None;
	let mut width = None;
	let mut height = None;
	let mut filename = None;
	let mut bvh_type = None;
	let mut seed = None;
	let mut render_method = None;
	let mut gui = true;

	if args.len() == 1 {
		println!("No arguments specified defaulting to help.");
		display_help();
		process::exit(0);
	}

	for arg_i in (0..(args.len() / 2)).map(|i| i * 2 + 1) {
		if let Some(arg) = args.get(arg_i) {
			match &arg[..] {
				"-H" => {
					display_help();
					process::exit(0);
				}
				"--help" => {
					display_help();
					process::exit(0);
				}
				"-G" | "--gui" => match &args[arg_i + 1].to_lowercase()[..] {
					"false" => {
						gui = false;
					}
					"true" => {}
					_ => {
						println!("Invalid option {} for --gui", args[arg_i + 1]);
					}
				},
				"-L" | "--list" => {
					get_list();
				}
				"-I" | "--info" => {
					get_info(&args, arg_i + 1);
				}
				"-S" | "--scene" => {
					scene_index = Some(arg_i + 1);
				}
				"-N" | "--samples" => {
					samples = Some(get_samples(&args, arg_i + 1));
				}
				"-X" | "--width" => {
					width = Some(get_dimension(&args, arg_i + 1));
				}
				"-Y" | "--height" => {
					height = Some(get_dimension(&args, arg_i + 1));
				}
				"-B" | "--bvh" => {
					bvh_type = Some(get_bvh_type(&args, arg_i + 1));
				}
				"-O" | "--output" => {
					filename = get_filename(&args, arg_i + 1);
				}
				"-R" | "--render_type" => {
					render_method = get_render_method(&args, arg_i + 1);
				}
				"-J" | "--seed" => {
					seed = Some(get_seed(&args, arg_i + 1));
				}
				_ => {}
			}
		}
	}
	match scene_index {
		Some(scene_index) => {
			let mut render_options: RenderOptions = Default::default();
			render_options.samples_per_pixel = samples.unwrap_or(render_options.samples_per_pixel);
			render_options.width = width.unwrap_or(render_options.width);
			render_options.height = height.unwrap_or(render_options.height);
			render_options.render_method = render_method.unwrap_or(render_options.render_method);

			let aspect_ratio = render_options.width as Float / render_options.height as Float;
			let bvh_type = bvh_type.unwrap_or_default();
			let scene = get_scene(&args, scene_index, bvh_type, aspect_ratio, seed);

			let parameters = Parameters::new(render_options, gui, filename);
			Some((scene, parameters))
		}
		None => None,
	}
}

fn display_help() {
	println!("Usage: cpu_raytracer [OPTION...]");
	println!("A headless CPU raytracer!\n");
	println!("Arguments:");
	println!("-H, --help");
	println!("\t Displays help.");
	println!("-L, --list");
	println!("\t Lists all valid scenes.");
	println!("-I [index], --info [index]");
	println!("\t Prints info for scene");
	println!("-S [index], --scene [index]");
	println!("\t Renders scene");
	println!("-N [samples], --samples [samples]");
	println!("\t Set samples per pixel. (Note 0 -> u64::MAX)");
	println!("-X [pixels], --width [pixels]");
	println!("\t Sets width of image");
	println!("-Y [pixels], --height [pixels]");
	println!("\t Sets height of image");
	println!("-B [split_type], --bvh [split_type]");
	println!("\t Sets split type for BVH.");
	println!("\t supported split types: \"equal\", \"middle\"");
	println!("-O [filename], --output [filename]");
	println!("\t Filename of output with supported file extension. If not specified no image will be output.");
	println!("\t supported file extensions: \"png\", \"jpeg\"");
	println!("-J [seed], --seed [seed]");
	println!("-G [true/false] --gui [true/false]");
	println!("\t Show render preview while rendering");
	println!("-R [render_method], --render_method [render_method]");
	println!("\t Possible options: mis, naive");
	println!("Seed for scene generation (if supported).")
}

fn get_list() {
	println!("-------------------");
	println!("1: Marbles");
	println!("-------------------");
	println!("Objects: 4-125");
	println!("Sky: Yes");
	println!("Motion Blur: Yes");
	println!("-------------------");
	println!("-------------------");
	println!("4: Overshadowed");
	println!("-------------------");
	println!("Objects: 3");
	println!("Sky: No");
	println!("Motion Blur: No");
	println!("-------------------");
	println!("-------------------");
	println!("6: Glass Dragon");
	println!("-------------------");
	println!("Objects: 3");
	println!("Sky: No");
	println!("Motion Blur: No");
	println!("-------------------");
	println!("-------------------");
	println!("8: Cornell Box");
	println!("-------------------");
	println!("Objects: 6");
	println!("Sky: No");
	println!("Motion Blur: No");
	println!("-------------------");
}

fn get_info(args: &[String], index: usize) {
	match args.get(index) {
		None => {
			println!("Please specify a value for scene!");
			println!("Do -H or --help for more information.");
			process::exit(0);
		}
		Some(string) => match &string.to_ascii_lowercase()[..] {
			"marbles" => {
				println!("Marbles");
				println!("Objects: 4-125");
				println!("Sky: Yes");
				println!("Motion Blur: Yes");
			}
			"overshadowed" => {
				println!("Overshadowed");
				println!("Objects: 3");
				println!("Sky: No");
				println!("Motion Blur: No");
			}
			"dragon" => {
				println!("Glass Dragon");
				println!("Objects: 3");
				println!("Sky: No");
				println!("Motion Blur: No");
			}
			"cornell" => {
				println!("Cornell Box");
				println!("Objects: 6");
				println!("Sky: No");
				println!("Motion Blur: No");
			}
			"furnace" => {
				println!("Furnace Test");
				println!("Objects: 6");
				println!("Sky: No");
				println!("Motion Blur: No");
			}
			_ => {
				println!("{} is not a valid scene index!", string);
				println!("Please specify a valid for scene!");
				println!("Do -L or--list to view scenes or do -H or --help for more information.");
				process::exit(0);
			}
		},
	}
}

fn get_scene(
	args: &[String],
	index: usize,
	bvh_type: SplitType,
	aspect_ratio: Float,
	seed: Option<String>,
) -> SceneType {
	match args.get(index) {
		None => {
			println!("Please specify a value for scene!");
			println!("Do -H or --help for more information.");
			process::exit(0);
		}

		Some(string) => match &string[..] {
			"classic" => {
				scene!(classic, bvh_type, aspect_ratio, seed)
			}
			"overshadowed" => {
				scene!(overshadowed, bvh_type, aspect_ratio)
			}
			"cornell" => {
				scene!(cornell, bvh_type, aspect_ratio)
			}
			"furnace" => {
				scene!(furnace, bvh_type, aspect_ratio)
			}
			_ => {
				println!("{} is not a valid scene index!", string);
				println!("Please specify a valid for scene!");
				println!("Do -L or--list to view scenes or do -H or --help for more information.");
				process::exit(0);
			}
		},
	}
}

fn get_seed(args: &[String], index: usize) -> String {
	match args.get(index) {
		Some(string) => string.to_string(),
		None => {
			println!("Please specify a value for seed!");
			println!("Do -H or --help for more information.");
			process::exit(0);
		}
	}
}

fn get_render_method(args: &[String], index: usize) -> Option<RenderMethod> {
	Some(match args.get(index) {
		Some(string) => match &string.to_ascii_lowercase()[..] {
			"naive" => RenderMethod::Naive,
			"mis" => RenderMethod::MIS,
			_val => {
				println!("Unsupported render method: {}", _val);
				println!("Do -H or --help for more information.");
				process::exit(0);
			}
		},
		None => {
			println!("Please specify a valid render method!");
			println!("Do -H or --help for more information.");
			process::exit(0);
		}
	})
}

fn get_filename(args: &[String], index: usize) -> Option<String> {
	Some(match args.get(index) {
		Some(string) => {
			let split_vec: Vec<&str> = string.split('.').collect();
			if split_vec.len() < 2 {
				println!("Please specify a valid extension!");
				println!("Do -H or --help for more information.");
				process::exit(0);
			}

			match split_vec[split_vec.len() - 1] {
				"jpeg" => string.to_string(),
				"png" => string.to_string(),
				_ => {
					println!(
						"Unsupported file extension: {}",
						split_vec[split_vec.len() - 1]
					);
					println!("Do -H or --help for more information.");
					process::exit(0);
				}
			}
		}
		None => {
			println!("Please specify a valid filename!");
			println!("Do -H or --help for more information.");
			process::exit(0);
		}
	})
}

fn get_bvh_type(args: &[String], index: usize) -> SplitType {
	match args.get(index) {
		Some(string) => match &string.to_lowercase()[..] {
			"equal" => SplitType::EqualCounts,
			"middle" => SplitType::Middle,
			"sah" => SplitType::Sah,
			_ => {
				println!("{} is not a valid value for BVH type!", string);
				println!("Please specify a valid value for BVH type!");
				println!("Do -H or --help for more information.");
				process::exit(0);
			}
		},
		None => {
			println!("Please specify a value for BVH type!");
			println!("Do -H or --help for more information.");
			process::exit(0);
		}
	}
}

fn get_samples(args: &[String], index: usize) -> u64 {
	match args.get(index) {
		Some(string) => match string.parse::<u64>() {
			Ok(parsed) => match parsed {
				0 => {
					println!("Samples must be non zero positive integer.");
					println!("Please specify a valid value for height!");
					println!("Do -H or --help for more information.");
					process::exit(0);
				}
				_ => parsed,
			},
			Err(_) => {
				println!("{} is not a valid value for samples!", string);
				println!("Please specify a valid value for height!");
				println!("Do -H or --help for more information.");
				process::exit(0);
			}
		},
		None => {
			println!("Please specify a valid value for samples!");
			println!("Do -H or --help for more information.");
			process::exit(0);
		}
	}
}

fn get_dimension(args: &[String], index: usize) -> u64 {
	match args.get(index) {
		Some(string) => match string.parse::<u64>() {
			Ok(parsed) => match parsed {
				0 => {
					println!("Height must be non zero positive integer.");
					println!("Please specify a valid value for height!");
					println!("Do -H or --help for more information.");
					process::exit(0);
				}
				_ => parsed,
			},
			Err(_) => {
				println!("{} is not a valid value for height!", string);
				println!("Please specify a valid value for height!");
				println!("Do -H or --help for more information.");
				process::exit(0);
			}
		},
		None => {
			println!("Please specify a valid value for height!");
			println!("Do -H or --help for more information.");
			process::exit(0);
		}
	}
}
