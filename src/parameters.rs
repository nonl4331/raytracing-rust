use crate::{scene::Scene, Float};
use clap::Parser;

use implementations::{split::SplitType, *};
use region::Region;

type MaterialType<'a> = AllMaterials<'a, AllTextures>;
type PrimitiveType<'a> = AllPrimitives<'a, MaterialType<'a>>;
type SkyType<'a> = Sky<'a, AllTextures, MaterialType<'a>>;
type BvhType<'a> = Bvh<PrimitiveType<'a>, MaterialType<'a>, SkyType<'a>>;
pub type SceneType<'a> =
	Scene<MaterialType<'a>, PrimitiveType<'a>, SimpleCamera, SkyType<'a>, BvhType<'a>>;

pub struct Parameters {
	pub render_options: RenderOptions,
	pub gui: bool,
	pub filename: Option<String>,
}

#[derive(Parser, Debug)]
#[command(about, long_about=None)]
#[command(name = "Pathtracer")]
#[command(about = "An experimental pathtracer written in Rust")]
struct Cli {
	#[arg(short, long, default_value_t = false)]
	gui: bool,
	#[arg(short, long, default_value_t = 128)]
	samples: u64,
	#[arg(short = 'x', long, default_value_t = 1920)]
	width: u64,
	#[arg(short = 'y', long, default_value_t = 1080)]
	height: u64,
	#[arg(short, long)]
	filepath: String,
	#[arg(short, long,value_enum, default_value_t = SplitType::Sah)]
	bvh_type: SplitType,
	#[arg(short, long,value_enum, default_value_t = RenderMethod::MIS)]
	render_method: RenderMethod,
	#[arg(short, long)]
	output: Option<String>,
	#[arg(long, default_value_t = 2.2)]
	gamma: Float,
}

pub fn process_args() -> Option<(SceneType<'static>, Parameters)> {
	let cli = Cli::parse();

	let mut region = Region::new();
	let (primitives, camera, sky) = match loader::load_file_full::<
		AllTextures,
		MaterialType,
		PrimitiveType,
		SimpleCamera,
		SkyType,
	>(&mut region, &cli.filepath)
	{
		Ok(a) => a,
		Err(e) => panic!("{e:?}"),
	};

	let bvh = Bvh::new(primitives, sky, cli.bvh_type);

	let scene = Scene::new(bvh, camera, region);

	let render_ops = RenderOptions {
		width: cli.width,
		height: cli.height,
		samples_per_pixel: cli.samples,
		render_method: cli.render_method,
		gamma: cli.gamma,
	};
	let params = Parameters {
		render_options: render_ops,
		gui: cli.gui,
		filename: cli.output,
	};
	Some((scene, params))
}
