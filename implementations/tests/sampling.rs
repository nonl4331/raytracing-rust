use rand::rngs::ThreadRng;
use rt_core::*;
use statistics::spherical_sampling::test_spherical_pdf;

#[test]
fn sky_sampling() {
	const SAMPLE_WIDTH: usize = 30;
	const SAMPLE_HEIGHT: usize = 50;

	let tex = std::sync::Arc::new(implementations::AllTextures::Lerp(
		implementations::Lerp::new(Vec3::zero(), Vec3::one()),
	));

	let sky = implementations::Sky::new(&tex, (SAMPLE_WIDTH, SAMPLE_HEIGHT));

	let pdf = |outgoing: Vec3| sky.pdf(outgoing);
	let sample = |_: &mut ThreadRng| sky.sample();
	test_spherical_pdf("lerp sky sampling", &pdf, &sample, false);
}

use implementations::sphere::Sphere;
use implementations::*;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use std::sync::Arc;

type MaterialType = AllMaterials<AllTextures>;
type PrimitiveType = AllPrimitives<MaterialType>;
type BvhType = Bvh<PrimitiveType, MaterialType>;

pub fn furnace_test(sampler_res: (usize, usize)) -> (Sky<AllTextures>, BvhType) {
	let light_tex = AllTextures::SolidColour(SolidColour::new(Vec3::new(1.0, 1.0, 1.0)));
	let light_mat = Arc::new(AllMaterials::Emit(Emit::new(&Arc::new(light_tex), 1.0)));

	let mat = AllTextures::SolidColour(SolidColour::new(Vec3::new(0.5, 0.5, 0.5)));

	let mat = Arc::new(AllMaterials::Lambertian(Lambertian::new(
		&Arc::new(mat),
		0.5,
	)));

	let hidden_light_tex = AllTextures::SolidColour(SolidColour::new(Vec3::new(1.0, 0.0, 1.0)));
	let hidden_light_mat = Arc::new(AllMaterials::Emit(Emit::new(
		&Arc::new(hidden_light_tex),
		15.0,
	)));

	let primitives = vec![
		AllPrimitives::Sphere(Sphere::new(Vec3::zero(), 0.5, &mat)),
		AllPrimitives::Sphere(Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1000.0, &light_mat)),
		AllPrimitives::Sphere(Sphere::new(
			Vec3::new(0.0, 0.0, -5.0),
			0.45,
			&hidden_light_mat,
		)), // hidden light from above
	];

	let sky_tex = AllTextures::Lerp(Lerp::new(Vec3::zero(), Vec3::new(0.5, 1.0, 0.2)));

	let sky = implementations::Sky::new(&std::sync::Arc::new(sky_tex), sampler_res);

	(sky, Bvh::new(primitives, split::SplitType::Sah))
}

pub fn get_test_scene(m: MaterialType, sampler_res: (usize, usize)) -> (Sky<AllTextures>, BvhType) {
	let light_tex = AllTextures::SolidColour(SolidColour::new(Vec3::new(1.0, 0.0, 0.5)));
	let _light_mat = Arc::new(AllMaterials::Emit(Emit::new(&Arc::new(light_tex), 5.0)));

	let mat = Arc::new(m);

	let primitives = vec![
		AllPrimitives::Triangle(Triangle::new(
			[
				Vec3::new(-0.5, -0.5, 0.0),
				Vec3::new(0.0, 0.5, 0.0),
				Vec3::new(0.5, -0.5, 0.0),
			],
			[Vec3::new(0.0, 0.0, 1.0); 3],
			&mat,
		)),
		//AllPrimitives::Sphere(Sphere::new(Vec3::new(0.0, 3.0, 1.0), 0.3, &light_mat)),
		//AllPrimitives::Sphere(Sphere::new(Vec3::new(0.0, 2.9, 0.9), 0.2, &mat)),
		//AllPrimitives::Sphere(Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1000.0, &light_mat)),
	];

	//let sky_tex = AllTextures::Lerp(Lerp::new(Vec3::zero(), Vec3::one()));
	let sky_tex = AllTextures::SolidColour(implementations::SolidColour::new(rt_core::Vec3::new(
		1.0, 1.0, 1.0, //5.0, 0.0, 2.5,
	))); //rt_core::Vec3::zero()));
	let sky = implementations::Sky::new(&std::sync::Arc::new(sky_tex), sampler_res); //Arc::new(sky_tex), sampler_res);

	(sky, Bvh::new(primitives, split::SplitType::Sah))
}

pub fn bxdf_testing(sampler_res: (usize, usize)) -> (Sky<AllTextures>, BvhType) {
	let mut primitives = Vec::new();

	let triangle = |point_one, point_two, point_three, material| {
		let normal = {
			let a: Vec3 = point_two - point_one;
			let b = point_three - point_one;
			a.cross(b)
		}
		.normalised();

		AllPrimitives::Triangle(Triangle::new(
			[point_one, point_two, point_three],
			[normal; 3],
			material,
		))
	};

	let aarect = |point_one: &Vec2, point_two: &Vec2, axis_value: Float, axis: &Axis, material| {
		let point_three =
			Axis::point_from_2d(&Vec2::new(point_one.x, point_two.y), axis, axis_value);
		let point_four =
			Axis::point_from_2d(&Vec2::new(point_two.x, point_one.y), axis, axis_value);
		let point_one = Axis::point_from_2d(point_one, axis, axis_value);
		let point_two = Axis::point_from_2d(point_two, axis, axis_value);
		vec![
			triangle(point_one, point_two, point_three, material),
			triangle(point_one, point_two, point_four, material),
		]
	};

	let sphere =
		|position, radius, material| AllPrimitives::Sphere(Sphere::new(position, radius, material));

	let grey_tex = AllTextures::SolidColour(SolidColour::new(Vec3::new(0.5, 0.5, 0.5)));

	let diffuse = AllMaterials::Lambertian(Lambertian::new(&Arc::new(grey_tex), 0.5));

	primitives.extend(aarect(
		&Vec2::new(-500.0, -500.0),
		&Vec2::new(500.0, 500.0),
		-40.0,
		&Axis::Z,
		&Arc::new(diffuse),
	));

	let mat = Arc::new(AllMaterials::Emit(Emit::new(
		&Arc::new(AllTextures::SolidColour(SolidColour::new(Vec3::new(
			0.0, 1.0, 0.0,
		)))),
		5.5,
	)));

	let glowy = sphere(Vec3::new(0.0, -100.5, 0.0), 50.0, &mat);

	let glowy_two = sphere(
		Vec3::new(0.0, 0.0, 300.0),
		50.0,
		&Arc::new(AllMaterials::Emit(Emit::new(
			&Arc::new(AllTextures::SolidColour(SolidColour::new(Vec3::new(
				1.0, 1.0, 1.0,
			)))),
			10.5,
		))),
	);

	//let materials = vec![(testing_mat.clone(), "default")];

	/*primitives.extend(crate::load_model::load_model_with_materials(
		"../res/dragon.obj",
		&materials,
	));*/

	primitives.push(glowy);
	primitives.push(glowy_two);

	let image = |filepath| Arc::new(AllTextures::ImageTexture(ImageTexture::new(filepath)));

	let bvh = Bvh::new(primitives, split::SplitType::Sah); //create_bvh_with_info(primitives, bvh_type);

	(
		Sky::new(&image("../res/skymaps/lilienstein.webp"), sampler_res),
		bvh,
	)
}

#[test]
fn mis() {
	let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0), 0.0);

	let (sky, bvh) = bxdf_testing((0, 0));

	let samples = 2u64.pow(23); //5_000_000;
	let mut ref_vals = Vec::new();
	for _ in 0..samples {
		ref_vals.push(Ray::get_colour_naive(&mut ray.clone(), &sky, &bvh).0)
	}

	let ref_val = statistics::utility::recursively_binary_average(ref_vals);

	println!("naive_sampling: {ref_val}");

	let mut vals = Vec::new();
	for _ in 0..samples {
		vals.push(Ray::get_colour(&mut ray.clone(), &sky, &bvh).0)
	}

	let val = statistics::utility::recursively_binary_average(vals);

	println!("mis_sampling: {val}");

	assert!((ref_val - val).mag() < 0.001)
}

#[test]
fn mis_sky_sampling() {
	let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0), 0.0);

	let (sky, bvh) = bxdf_testing((20, 10));

	let samples = 5_000_000;

	let ref_val = (0..samples)
		.into_par_iter()
		.map(|_| Ray::get_colour_naive(&mut ray.clone(), &sky, &bvh).0)
		.reduce_with(std::ops::Add::add)
		.unwrap()
		/ samples as Float;

	let mut sum = Vec3::zero();
	for _ in 0..500 {
		sum += (0..100_000)
			.into_par_iter()
			.map(|_| Ray::get_colour(&mut ray.clone(), &sky, &bvh).0)
			.reduce_with(std::ops::Add::add)
			.unwrap() / 100_000.0 as Float;
	}
	let val = sum / 500.0;

	println!("sky_sampling: {val}");

	assert!((ref_val - val).mag() < 0.001)
}

#[test]
fn naive_furnace_test() {
	let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0), 0.0);

	let (sky, bvh) = furnace_test((0, 0));

	let samples = 5_000_000;

	let ref_val = Vec3::new(0.25, 0.25, 0.25);

	let val = (0..samples)
		.into_par_iter()
		.map(|_| Ray::get_colour_naive(&mut ray.clone(), &sky, &bvh).0)
		.reduce_with(std::ops::Add::add)
		.unwrap()
		/ samples as Float;

	assert!((ref_val - val).mag() < 0.001)
}

#[test]
fn mis_furnace_test() {
	let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0), 0.0);

	let (sky, bvh) = furnace_test((0, 0));

	let samples = 5_000_000;

	let ref_val = Vec3::new(0.25, 0.25, 0.25);

	let val = (0..samples)
		.into_par_iter()
		.map(|_| Ray::get_colour(&mut ray.clone(), &sky, &bvh).0)
		.reduce_with(std::ops::Add::add)
		.unwrap()
		/ samples as Float;

	assert!((ref_val - val).mag() < 0.001)
}

#[test]
fn mis_sky_sampling_furnace_test() {
	let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0), 0.0);

	let (sky, bvh) = furnace_test((10, 10));

	let samples = 5_000_000;

	let ref_val = Vec3::new(0.25, 0.25, 0.25);

	let val = (0..samples)
		.into_par_iter()
		.map(|_| Ray::get_colour(&mut ray.clone(), &sky, &bvh).0)
		.reduce_with(std::ops::Add::add)
		.unwrap()
		/ samples as Float;

	assert!((ref_val - val).mag() < 0.001)
}
