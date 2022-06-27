use crate::*;

use crate::scene::Scene;
use crate::utility::create_bvh_with_info;
use implementations::random_sampler::RandomSampler;
use implementations::split::SplitType;
use implementations::AllMaterials;
use implementations::AllPrimitives;
use implementations::AllTextures;
use implementations::Bvh;

use rand::distributions::Alphanumeric;
use rand::rngs::SmallRng;
use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_seeder::Seeder;
use rt_core::Float;

type MaterialType = AllMaterials<AllTextures>;
type PrimitiveType = AllPrimitives<MaterialType>;
type BvhType = Bvh<PrimitiveType, MaterialType>;
pub type SceneType = Scene<PrimitiveType, MaterialType, RandomSampler, BvhType, AllTextures>;

pub fn get_seed(length: usize) -> String {
	let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
	std::iter::repeat(())
		.map(|()| rng.sample(Alphanumeric))
		.map(char::from)
		.take(length)
		.collect()
}

/*pub fn scene_one(bvh_type: SplitType, aspect_ratio: Float, _seed: Option<String>) -> SceneType {
	let primitives = Vec::new();
	let sky = Arc::new(Sky::new(None));
	let sampler = Arc::new(RandomSampler {});
	let bvh = create_bvh_with_info(primitives, bvh_type);
	let camera = Arc::new(SimpleCamera::new(
		Vec3::new(0.0, -1.0, 0.0),
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(0.0, 0.0, 1.0),
		60.0,
		aspect_ratio,
		1.0,
		0.0,
	));
	Scene::new(camera, sky, sampler, bvh)
}*/

pub fn scene_one(bvh_type: SplitType, aspect_ratio: Float, seed: Option<String>) -> SceneType {
	let mut primitives = Vec::new();

	let ground = sphere!(0, -1000, 0, 1000, &diffuse!(0.5, 0.5, 0.5, 0.5));

	let sphere_one = sphere!(0, 1, 0, 1, &refract!(&solid_colour!(colour!(1)), 1.5));

	let sphere_two = sphere!(-4, 1, 0, 1, &diffuse!(0.4, 0.2, 0.1, 0.5));

	let sphere_three = sphere!(4, 1, 0, 1, &reflect!(&solid_colour!(0.7, 0.6, 0.5), 0));

	primitives.push(ground);
	primitives.push(sphere_one);
	primitives.push(sphere_two);
	primitives.push(sphere_three);
	let seed = match seed {
		Some(seed) => seed,
		None => get_seed(32),
	};

	println!("\tseed: {}", seed);
	let mut rng: SmallRng = Seeder::from(seed).make_rng();

	for a in -11..11 {
		for b in -11..11 {
			let center = position!(
				a as Float + 0.9 * rng.gen::<Float>(),
				0.2,
				b as Float + 0.9 * rng.gen::<Float>()
			);

			if (center - position!(4.0, 0.2, 0.0)).mag() > 0.9 {
				let choose_material: Float = rng.gen();
				let colour = colour!(rng.gen::<Float>(), rng.gen::<Float>(), rng.gen::<Float>());

				let sphere;

				if choose_material < 0.8 {
					sphere = sphere!(center, 0.2, &diffuse!(&solid_colour!(colour), 0.5));
				} else if choose_material < 0.95 {
					sphere = sphere!(
						center,
						0.2,
						&reflect!(&solid_colour!(colour), rng.gen::<Float>() / 2.0)
					);
				} else {
					sphere = sphere!(center, 0.2, &refract!(&solid_colour!(colour!(1)), 1.5));
				}
				primitives.push(sphere);
			}
		}
	}

	let sky = sky!(&texture_lerp!(colour!(0.5, 0.7, 1), colour!(1)));

	let camera = camera!(
		position!(13, 2, -3),
		position!(0, 0, 0),
		position!(0, 1, 0),
		29,
		aspect_ratio,
		0.1,
		10
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);

	scene!(camera, sky, random_sampler!(), bvh)
}

/*pub fn scene_two(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let mat = diffuse!(1.0, 1.0, 1.0, 0.5);

	let ground = sphere!(0, -100.5, 1, 100, &mat);

	let sphere_one = sphere!(0, 0, 1, 0.5, &mat);

	primitives.push(ground);
	primitives.push(sphere_one);

	let sky = sky!(&texture_lerp!(colour!(0.5, 0.7, 1), colour!(1)));

	let camera = camera!(
		position!(0, 0, 0),
		position!(0, 0, 1),
		position!(0, 1, 0),
		2.116_813_7 * 180.0 / PI,
		aspect_ratio,
		0,
		1
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);

	scene!(camera, sky, random_sampler!(), bvh)
}

pub fn scene_three(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let ground = sphere!(0, -1000, 0, 1000, &diffuse!(0.5, 0.5, 0.5, 0.5));

	let box_left = aacuboid!(-1.6, 1, -0.5, -0.6, 2, 0.5, &diffuse!(1, 0, 0, 0.5));

	let box_middle = aacuboid!(
		-0.5,
		1,
		-0.5,
		0.5,
		2,
		0.5,
		&reflect!(&solid_colour!(1, 1, 1), 0)
	);

	let sphere_middle = sphere!(0, 2.5, 0, 0.3, &reflect!(&solid_colour!(1, 1, 1), 0));

	let box_right = aacuboid!(0.6, 1, -0.5, 1.6, 2, 0.5, &diffuse!(0, 0, 1, 0.5));

	primitives.push(ground);
	primitives.push(box_left);
	primitives.push(box_middle);
	primitives.push(sphere_middle);
	primitives.push(box_right);

	let sky = sky!(&texture_lerp!(colour!(0.5, 0.7, 1), colour!(1)));

	let camera = camera!(
		position!(-5, 3, -3),
		position!(0, 1.5, 0),
		position!(0, 1, 0),
		34,
		aspect_ratio,
		0,
		10
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);

	scene!(camera, sky, random_sampler!(), bvh)
}

pub fn scene_four(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let ground = sphere!(0, -1000, 0, 1000, &diffuse!(0.5, 0.5, 0.5, 0.5));

	let glowy = sphere!(0, 0.5, 0, 0.5, &emit!(&solid_colour!(colour!(1)), 1.5));

	let cube = aacuboid!(
		-0.5,
		0.1,
		-0.5,
		-0.4,
		0.2,
		-0.4,
		&diffuse!(0.5, 0.5, 0.5, 0.5)
	);

	primitives.push(ground);
	primitives.push(glowy);
	primitives.push(cube);

	let camera = camera!(
		position!(-5, 3, -3),
		position!(0, 0.5, 0),
		position!(0, 1, 0),
		34,
		aspect_ratio,
		0,
		10
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);

	scene!(camera, sky!(), random_sampler!(), bvh)
}

pub fn scene_five(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let ground = sphere!(
		0,
		-1000,
		0,
		1000,
		&diffuse!(&checkered!(colour!(0), colour!(0.5)), 0.5)
	);

	let cube = aacuboid!(-0.5, 0.1, -0.5, 1, 0.6, 1, &diffuse!(0.5, 0.5, 0.5, 0.5));

	let earth = sphere!(0, 1.2, 0, 0.5, &diffuse!(&image!("res/earth.png"), 0.5));

	primitives.push(ground);
	primitives.push(cube);
	primitives.push(earth);

	let sky = sky!(&texture_lerp!(colour!(0.5, 0.7, 1), colour!(1)));

	let camera = camera!(
		position!(-5, 4, -3),
		position!(0, 0.5, 0),
		position!(0, 1, 0),
		34,
		aspect_ratio,
		0,
		10
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);

	scene!(camera, sky, random_sampler!(), bvh)
}

pub fn scene_six(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let ground = sphere!(0, -1000, 0, 1000, &diffuse!(0.5, 0.5, 0.5, 0.5));

	let glowy = sphere!(5, 3.5, 5, 1.5, &emit!(&solid_colour!(colour!(1)), 5));

	primitives.push(ground);
	primitives.push(glowy);
	primitives.extend(model!(
		"res/dragon.obj",
		&refract!(&solid_colour!(1, 1, 1), 1.52)
	));

	let camera = camera!(
		position!(-20, 20, -25),
		position!(0, 3.5, 0),
		position!(0, 1, 0),
		34,
		aspect_ratio,
		0,
		10
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);

	scene!(camera, sky!(), random_sampler!(), bvh)
}

pub fn scene_seven(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let ground_mat = &diffuse!(
		&checkered!(
			242.0 / 256.0,
			242.0 / 256.0,
			242.0 / 256.0,
			109.0 / 256.0,
			112.0 / 256.0,
			117.0 / 256.0
		),
		0.5
	);

	let ground = aarect!(-10, -10, 10, 10, -0.3, axis!(Y), ground_mat);

	primitives.push(ground);
	primitives.extend(model!("res/bunny.obj", &diffuse!(0.5, 0.5, 0.5, 0.5)));

	let sky = sky!(&image!("res/sky.png"));

	let camera = camera!(
		position!(-7, 1.5, -7),
		position!(0, 1.5, 0),
		position!(0, 1, 0),
		34,
		aspect_ratio,
		0,
		10
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);

	scene!(camera, sky, random_sampler!(), bvh)
}

pub fn scene_eight(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let red = &diffuse!(0.65, 0.05, 0.05, 0.0);
	let white = &diffuse!(0.73, 0.73, 0.73, 0.0);
	let green = &diffuse!(0.12, 0.45, 0.15, 0.0);
	let light = &emit!(&solid_colour!(colour!(1)), 15);

	primitives.push(aarect!(0, 0, 555, 555, 555, axis!(X), green));
	primitives.push(aarect!(0, 0, 555, 555, 0, axis!(X), red));

	primitives.push(aarect!(0, 0, 555, 555, 555, axis!(Y), white));
	primitives.push(aarect!(0, 0, 555, 555, 0, axis!(Y), white));

	primitives.push(aarect!(0, 0, 555, 555, 555, axis!(Z), white));
	primitives.push(aarect!(213, 227, 343, 332, 554, axis!(Y), light));

	primitives.push(cuboid!(
		265,
		0,
		295,
		430,
		330,
		460,
		rotation!(0, 15, 0, D),
		white
	));

	primitives.push(cuboid!(
		130,
		0,
		65,
		295,
		165,
		230,
		rotation!(0, -18, 0, D),
		white
	));

	let sky = sky!();

	let camera = camera!(
		position!(278, 278, -800),
		position!(278, 278, 0),
		position!(0, 1, 0),
		40,
		aspect_ratio,
		0,
		10
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);

	scene!(camera, sky, random_sampler!(), bvh)
}

pub fn scene_nine(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let inner = &diffuse!(1, 1, 1, 0.9);
	let emit = &emit!(&solid_colour!(colour!(1)), 1);

	primitives.push(sphere!(0, 0, 0, 0.5, inner));
	primitives.push(sphere!(0, 0, 0, 10, emit));

	let sky = sky!();

	let camera = camera!(
		position!(3, 0, 0),
		position!(0, 0, 0),
		position!(0, 1, 0),
		40,
		aspect_ratio,
		0,
		10
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);
	scene!(camera, sky, random_sampler!(), bvh)
}*/
