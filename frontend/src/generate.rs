use crate::{utility::create_bvh_with_info, *};
use rand::{distributions::Alphanumeric, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use rand_seeder::Seeder;
use scene::implementations::{
	random_sampler::RandomSampler, rt_core::Float, split::SplitType, AllMaterials, AllPrimitives,
	AllTextures, Bvh, Lambertian,
};
use scene::*;

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

pub fn classic(bvh_type: SplitType, aspect_ratio: Float, seed: Option<String>) -> SceneType {
	let mut primitives = Vec::new();

	let ground = sphere!(0, 0, -1000, 1000, &diffuse!(0.5, 0.5, 0.5, 0.5));

	let sphere_one = sphere!(0, 0, 1, 1, &refract!(&solid_colour!(colour!(1)), 1.5));

	let sphere_two = sphere!(-4, 0, 1, 1, &diffuse!(0.4, 0.2, 0.1, 0.5));

	let sphere_three = sphere!(4, 0, 1, 1, &reflect!(&solid_colour!(0.7, 0.6, 0.5), 0));

	primitives.push(ground);
	primitives.push(sphere_one);
	primitives.push(sphere_two);
	primitives.push(sphere_three);
	let seed = match seed {
		Some(seed) => seed,
		None => get_seed(32),
	};

	println!("\tseed: {seed}");
	let mut rng: SmallRng = Seeder::from(seed).make_rng();

	for a in -11..11 {
		for b in -11..11 {
			let center = position!(
				a as Float + 0.9 * rng.gen::<Float>(),
				b as Float + 0.9 * rng.gen::<Float>(),
				0.2
			);

			if (center - position!(4.0, 0.0, 0.2)).mag() > 0.9 {
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
		position!(13, -3, 2),
		position!(0, 0, 0),
		position!(0, 0, 1),
		29,
		aspect_ratio,
		0.1,
		10
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);

	scene!(camera, sky, random_sampler!(), bvh)
}

pub fn bxdf_testing(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let testing_mat = &std::sync::Arc::new(implementations::AllMaterials::Lambertian(
		Lambertian::new(&solid_colour!(0.93, 0.62, 0.54), 0.5),
	));

	primitives.extend(aarect!(
		-500.0,
		-500.0,
		500.0,
		500.0,
		-40.0,
		&axis!(Z),
		&diffuse!(0.5, 0.5, 0.5, 0.5)
	));

	let glowy = sphere!(
		0,
		-100.5,
		0,
		50,
		&emit!(&solid_colour!(colour!(0, 1, 0)), 5.5)
	);

	let glowy_two = sphere!(
		0,
		0.0,
		300,
		50,
		&emit!(&solid_colour!(colour!(1, 1, 1)), 10.5)
	);

	let materials = vec![(testing_mat.clone(), "default")];

	primitives.extend(crate::load_model::load_model_with_materials(
		"../res/dragon.obj",
		&materials,
	));

	primitives.push(glowy);
	primitives.push(glowy_two);

	let camera = camera!(
		position!(700, -700, 700),
		position!(0, 0, 0),
		position!(0, 0, 1),
		34,
		aspect_ratio,
		0,
		10
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);

	scene!(
		camera,
		sky!(&image!("../res/skymaps/lilienstein.webp")),
		random_sampler!(),
		bvh
	)
}

pub fn furnace(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
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
}

pub fn overshadowed(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
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
	primitives.extend(cube);

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

/*pub fn scene_six(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
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
}*/

pub fn coffee(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let diffuse = &diffuse!(1, 1, 1, 0.9);

	let orange = &std::sync::Arc::new(implementations::AllMaterials::Phong(
		implementations::Phong::new(&solid_colour!(0.592, 0.192, 0.0), 0.2, 0.8, 1250.0),
	));

	let floor = &std::sync::Arc::new(implementations::AllMaterials::Phong(
		implementations::Phong::new(&solid_colour!(colour!(1)), 0.1, 0.9, 1250.0),
	));

	let black = &std::sync::Arc::new(implementations::AllMaterials::Phong(
		implementations::Phong::new(&solid_colour!(colour!(0.006)), 0.1, 0.9, 1250.0),
	));

	let glass = &refract!(&solid_colour!(colour!(1)), 1.5);

	let left_light = &emit!(&solid_colour!(colour!(1)), 3.0 * 1.0);
	let right_light = &emit!(&solid_colour!(colour!(1)), 1.0 * 1.0);
	let left_reflection = &emit!(&solid_colour!(colour!(1)), 0.75 * 1.0);

	let metal = &reflect!(&solid_colour!(colour!(1)), 0);

	let materials = vec![
		(diffuse.clone(), "default"),
		(orange.clone(), "Plastic_Orange"),
		(glass.clone(), "Glass"),
		(left_light.clone(), "Left_Light"),
		(left_reflection.clone(), "Left_Reflection"),
		(right_light.clone(), "Right_Light"),
		(floor.clone(), "Floor"),
		(black.clone(), "Plastic_Black"),
		(metal.clone(), "Metal"),
	];
	primitives.extend(crate::load_model::load_model_with_materials(
		"../../../render_scenes/coffee.obj",
		&materials,
	));

	let sky = sky!();

	let camera = camera!(
		position!(0, 0.17 * 10.0, (2.0386 - 1.0) * 10.0),
		position!(0, 0.150512 * 10.0, 0),
		position!(0, 1, 0),
		40,
		aspect_ratio,
		0,
		0.0075 * 10.0
	);

	let bvh = create_bvh_with_info(primitives, bvh_type);
	scene!(camera, sky, random_sampler!(), bvh)
}

pub fn cornell(bvh_type: SplitType, aspect_ratio: Float) -> SceneType {
	let mut primitives = Vec::new();

	let red = &diffuse!(0.65, 0.05, 0.05, 0.0);
	let white = &diffuse!(0.73, 0.73, 0.73, 0.0);
	let green = &diffuse!(0.12, 0.45, 0.15, 0.0);
	let light = &emit!(&solid_colour!(colour!(1)), 15);

	primitives.extend(aarect!(0.0, 0.0, 555.0, 555.0, 555.0, &axis!(X), green));
	primitives.extend(aarect!(0.0, 0.0, 555.0, 555.0, 0.0, &axis!(X), red));

	primitives.extend(aarect!(0.0, 0.0, 555.0, 555.0, 555.0, &axis!(Y), white));
	primitives.extend(aarect!(0.0, 0.0, 555.0, 555.0, 0.0, &axis!(Y), white));

	primitives.extend(aarect!(0.0, 0.0, 555.0, 555.0, 555.0, &axis!(Z), white));
	primitives.extend(aarect!(213.0, 227.0, 343.0, 332.0, 554.0, &axis!(Y), light));

	primitives.extend(cuboid!(
		265.0,
		0.0,
		295.0,
		430.0,
		330.0,
		460.0,
		rotation!(0, 15, 0, D),
		white
	));

	primitives.extend(cuboid!(
		130.0,
		0.0,
		65.0,
		295.0,
		165.0,
		230.0,
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
