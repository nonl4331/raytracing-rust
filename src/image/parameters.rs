use crate::bvh::split::SplitType;
use crate::math::Float;

use crate::image::{generate, scene::Scene};

use std::process;

const SAMPLES_DEFAULT: u32 = 30;
const WIDTH_DEFAULT: u32 = 800;
const HEIGHT_DEFAULT: u32 = 600;
const BVH_DEFAULT: SplitType = SplitType::Middle;
const FILENAME_DEFAULT: &str = "out.png";

pub struct Parameters {
    pub samples: u32,
    pub width: u32,
    pub height: u32,
    pub filename: String,
}

impl Parameters {
    pub fn new(
        samples: Option<u32>,
        width: Option<u32>,
        height: Option<u32>,
        filename: Option<String>,
    ) -> Self {
        Parameters {
            samples: samples.unwrap_or(SAMPLES_DEFAULT),
            width: width.unwrap_or(WIDTH_DEFAULT),
            height: height.unwrap_or(HEIGHT_DEFAULT),
            filename: filename.unwrap_or_else(|| FILENAME_DEFAULT.to_string()),
        }
    }
}

pub fn process_args(args: Vec<String>) -> Option<(Scene, Parameters)> {
    let mut scene_index = None;
    let mut samples = None;
    let mut width = None;
    let mut height = None;
    let mut filename = None;
    let mut bvh_type = None;

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
                "-L" => {
                    get_list();
                }
                "--list" => {
                    get_list();
                }
                "-I" => {
                    get_info(&args, arg_i + 1);
                }
                "--info" => {
                    get_info(&args, arg_i + 1);
                }
                "-S" => {
                    scene_index = Some(arg_i + 1);
                }
                "--scene" => {
                    scene_index = Some(arg_i + 1);
                }
                "-N" => {
                    samples = Some(get_samples(&args, arg_i + 1));
                }
                "--samples" => {
                    samples = Some(get_samples(&args, arg_i + 1));
                }
                "-X" => {
                    width = Some(get_dimension(&args, arg_i + 1));
                }
                "--width" => {
                    width = Some(get_dimension(&args, arg_i + 1));
                }
                "-Y" => {
                    height = Some(get_dimension(&args, arg_i + 1));
                }
                "--height" => {
                    height = Some(get_dimension(&args, arg_i + 1));
                }
                "-B" => {
                    bvh_type = Some(get_bvh_type(&args, arg_i + 1));
                }
                "--bvh" => {
                    bvh_type = Some(get_bvh_type(&args, arg_i + 1));
                }
                "-O" => {
                    filename = Some(get_filename(&args, arg_i + 1));
                }
                "--output" => {
                    filename = Some(get_filename(&args, arg_i + 1));
                }
                _ => {}
            }
        }
    }
    match scene_index {
        Some(scene_index) => {
            let aspect_ratio =
                width.unwrap_or(WIDTH_DEFAULT) as Float / height.unwrap_or(HEIGHT_DEFAULT) as Float;
            let bvh_type = bvh_type.unwrap_or(BVH_DEFAULT);
            let scene = get_scene(&args, scene_index, bvh_type, aspect_ratio);

            let parameters = Parameters::new(samples, width, height, filename);
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
    println!("\t Set samples per pixel");
    println!("-X [pixels], --width [pixels]");
    println!("\t Sets width of image");
    println!("-Y [pixels], --height [pixels]");
    println!("\t Sets height of image");
    println!("-B [split_type], --bvh [split_type]");
    println!("\t Sets split type for BVH.");
    println!("\t supported split types: \"equal\", \"middle\"");
    println!("-O [filename], --output [filename]");
    println!("\t filename of output with supported file extension.");
    println!("\t supported file extensions: \"png\", \"jpeg\"");
}

fn get_list() {
    println!("-------------------");
    println!("1: Marbles");
    println!("-------------------");
    println!("Objects: 4-125");
    println!("Sky: Yes");
    println!("Motion Blur: Yes");
    println!("-------------------");
    println!("2: Name goes here");
    println!("-------------------");
    println!("Objects: TODO");
    println!("Sky: Yes");
    println!("Motion Blur: No");
    println!("-------------------");
    println!("3: Name goes here");
    println!("-------------------");
    println!("Objects: TODO");
    println!("Sky: Yes");
    println!("Motion Blur: No");
    println!("-------------------");
    println!("4: Overshadowed");
    println!("-------------------");
    println!("Objects: 3");
    println!("Sky: No");
    println!("Motion Blur: No");
    println!("-------------------");
    println!("5: WIP");
    println!("-------------------");
    println!("Objects: 3");
    println!("Sky: Yes");
    println!("Motion Blur: No");
    println!("-------------------");
    println!("-------------------");
    println!("5: Model Loading (WIP)");
    println!("-------------------");
    println!("Objects: 2");
    println!("Sky: Yes");
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
        Some(string) => match &string[..] {
            "1" => {
                println!("1(m): Marbles");
                println!("Objects: 4-125");
                println!("Sky: Yes");
                println!("Motion Blur: Yes");
            }
            "1m" => {
                println!("1(m): Marbles");
                println!("Objects: 4-125");
                println!("Sky: Yes");
                println!("Motion Blur: Yes");
            }
            "2" => {
                println!("2: TODO");
                println!("Objects: TODO");
                println!("Sky: Yes");
                println!("Motion Blur: No");
            }
            "3" => {
                println!("3: TODO");
                println!("Objects: TODO");
                println!("Sky: Yes");
                println!("Motion Blur: No");
            }
            "4" => {
                println!("4: Overshadowed");
                println!("Objects: 3");
                println!("Sky: No");
                println!("Motion Blur: No");
            }
            "5" => {
                println!("5: WIP");
                println!("Objects: 3");
                println!("Sky: Yes");
                println!("Motion Blur: No");
            }
            "6" => {
                println!("6: Model Loading (WIP)");
                println!("Objects: 2");
                println!("Sky: Yes");
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

fn get_scene(args: &[String], index: usize, bvh_type: SplitType, aspect_ratio: Float) -> Scene {
    match args.get(index) {
        None => {
            println!("Please specify a value for scene!");
            println!("Do -H or --help for more information.");
            process::exit(0);
        }
        Some(string) => match &string[..] {
            "1" => generate::scene_one(bvh_type, aspect_ratio),
            "2" => generate::scene_two(bvh_type, aspect_ratio),
            "3" => generate::scene_three(bvh_type, aspect_ratio),
            "4" => generate::scene_four(bvh_type, aspect_ratio),
            "5" => generate::scene_five(bvh_type, aspect_ratio),
            "6" => generate::scene_six(bvh_type, aspect_ratio),
            _ => {
                println!("{} is not a valid scene index!", string);
                println!("Please specify a valid for scene!");
                println!("Do -L or--list to view scenes or do -H or --help for more information.");
                process::exit(0);
            }
        },
    }
}

fn get_filename(args: &[String], index: usize) -> String {
    match args.get(index) {
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
    }
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

fn get_samples(args: &[String], index: usize) -> u32 {
    match args.get(index) {
        Some(string) => match string.parse::<u32>() {
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

fn get_dimension(args: &[String], index: usize) -> u32 {
    match args.get(index) {
        Some(string) => match string.parse::<u32>() {
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
