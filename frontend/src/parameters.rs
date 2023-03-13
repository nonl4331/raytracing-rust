use crate::scene::Scene;
use clap::Parser;

use implementations::{rt_core::*, split::SplitType, *};
use region::Region;

type MaterialType<'a> = AllMaterials<'a, AllTextures>;
type PrimitiveType<'a> = AllPrimitives<'a, MaterialType<'a>>;
type BvhType<'a> = Bvh<PrimitiveType<'a>, MaterialType<'a>>;
pub type SceneType<'a> =
	Scene<MaterialType<'a>, PrimitiveType<'a>, SimpleCamera, Sky<'a, AllTextures>, BvhType<'a>>;

pub struct Parameters {
	pub render_options: RenderOptions,
	pub gui: bool,
	pub filename: Option<String>,
}

#[derive(Parser, Debug)]
#[command(about, long_about=None)]
#[command(name = "Pathtracer")]
#[command(about = "An expiremental pathtracer written in Rust")]
struct Cli {
	#[arg(short, long, default_value_t = false)]
	gui: bool,
	#[arg(short, long, default_value_t = 128)]
	samples: u64,
	#[arg(short, long, default_value_t = 1920)]
	width: u64,
	#[arg(short, long, default_value_t = 1080)]
	height: u64,
	#[arg(short, long)]
	filepath: String,
	#[arg(short, long,value_enum, default_value_t = SplitType::Sah)]
	bvh_type: SplitType,
	#[arg(short, long,value_enum, default_value_t = RenderMethod::MIS)]
	render_method: RenderMethod,
	#[arg(short, long)]
	output: Option<String>,
}

pub fn process_args() -> Option<(SceneType<'static>, Parameters)> {
	let cli = Cli::parse();

	let mut region = Region::new();
	let (primitives, camera, sky) = match loader::load_file_full::<
		AllTextures,
		MaterialType,
		PrimitiveType,
		SimpleCamera,
		Sky<'_, AllTextures>,
	>(&mut region, &cli.filepath)
	{
		Ok(a) => a,
		Err(e) => panic!("{e:?}"),
	};

	let bvh = Bvh::new(primitives, cli.bvh_type);

	let scene = Scene::new(bvh, camera, sky, region);

	let render_ops = RenderOptions {
		width: cli.width,
		height: cli.height,
		samples_per_pixel: cli.samples,
		render_method: cli.render_method,
	};
	let params = Parameters {
		render_options: render_ops,
		gui: cli.gui,
		filename: cli.output,
	};
	Some((scene, params))
}
