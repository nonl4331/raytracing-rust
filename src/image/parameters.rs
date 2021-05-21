use crate::image::generate;
use crate::image::scene::Scene;
use std::process;

pub fn process_args(args: Vec<String>) -> Option<Scene> {
    let mut ret = None;
    for arg_i in (0..(args.len() / 2)).map(|i| i * 2 + 1) {
        match args.get(arg_i) {
            Some(arg) => match &arg[..] {
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
                    ret = Some(get_scene(&args, arg_i + 1));
                }
                "--scene" => {
                    ret = Some(get_scene(&args, arg_i + 1));
                }
                _ => {}
            },
            None => {}
        }
    }
    ret
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
    println!("-S [index](m), --scene [index](m)");
    println!("\t Renders scene. Adding a \"m\" will enable motion (only works on scenes with motion blur)");
}

fn get_list() {
    println!("-------------------");
    println!("1(m): Marbles");
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
}

fn get_info(args: &Vec<String>, index: usize) {
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
            _ => {
                println!("{} is not a valid scene index!", string);
                println!("Please specify a valid for scene!");
                println!("Do -L or--list to view scenes or do -H or --help for more information.");
                process::exit(0);
            }
        },
    }
}

fn get_scene(args: &Vec<String>, index: usize) -> Scene {
    match args.get(index) {
        None => {
            println!("Please specify a value for scene!");
            println!("Do -H or --help for more information.");
            process::exit(0);
        }
        Some(string) => match &string[..] {
            "1" => return generate::scene_one(false),
            "1m" => return generate::scene_one(true),
            "2" => return generate::scene_two(),
            "3" => return generate::scene_three(),
            "4" => return generate::scene_four(),
            _ => {
                println!("{} is not a valid scene index!", string);
                println!("Please specify a valid for scene!");
                println!("Do -L or--list to view scenes or do -H or --help for more information.");
                process::exit(0);
            }
        },
    }
}
