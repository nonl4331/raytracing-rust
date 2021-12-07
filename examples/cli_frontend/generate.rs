extern crate cpu_raytracer;

use cpu_raytracer::{material::MaterialEnum, *};
use rand::{distributions::Alphanumeric, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use rand_seeder::Seeder;

pub fn get_seed(length: usize) -> String {
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(length)
        .collect()
}

pub fn scene_one(
    bvh_type: SplitType,
    aspect_ratio: Float,
    seed: Option<String>,
) -> Scene<PrimitiveEnum<MaterialEnum>, MaterialEnum> {
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
    let mut rng: SmallRng = Seeder::from(seed.clone()).make_rng();

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

    scene!(
        position!(13, 2, -3),
        position!(0, 0, 0),
        position!(0, 1, 0),
        29,
        aspect_ratio,
        0.1,
        10,
        sky,
        bvh_type,
        primitives
    )
}

pub fn scene_two(
    bvh_type: SplitType,
    aspect_ratio: Float,
) -> Scene<PrimitiveEnum<MaterialEnum>, MaterialEnum> {
    let mut primitives = Vec::new();

    println!("\tCook Torrence currently has a low convergence rate!");

    let ground = sphere!(0, -1000, 0, 1000, &diffuse!(&perlin!(), 0.5));

    let sphere_one = sphere!(
        0,
        1,
        0,
        1,
        //R, G, B, alpha, absorbtion, spec_chance, f0
        &cook_torrence!(1.0, 0.86, 0.57, 0.2, 0.0, 1.0, Vec3::new(1.0, 0.86, 0.57))
    );

    let sphere_two = sphere!(2, 2, -1.5, 0.5, &emit!(&solid_colour!(colour!(1)), 100));

    let rect_one = aarect!(
        position!(-5, 0),
        position!(5, 8),
        5,
        axis!(Z),
        &diffuse!(1.0, 1.0, 1.0, 0.8)
    );

    let rect_two = aarect!(
        position!(0, -10),
        position!(8, 5),
        -2,
        axis!(X),
        &diffuse!(1.0, 0.25, 0.25, 0.8)
    );

    primitives.push(ground);
    primitives.push(rect_one);
    primitives.push(rect_two);
    primitives.push(sphere_one);
    primitives.push(sphere_two);

    let sky = sky!(&texture_lerp!(colour!(0.5, 0.7, 1), colour!(1)));

    scene!(
        position!(3, 1, -15),
        position!(0, 1, 0),
        position!(0, 1, 0),
        34,
        aspect_ratio,
        0,
        10,
        sky,
        bvh_type,
        primitives
    )
}

pub fn scene_three(
    bvh_type: SplitType,
    aspect_ratio: Float,
) -> Scene<PrimitiveEnum<MaterialEnum>, MaterialEnum> {
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

    scene!(
        position!(-5, 3, -3),
        position!(0, 1.5, 0),
        position!(0, 1, 0),
        34,
        aspect_ratio,
        0,
        10,
        sky,
        bvh_type,
        primitives
    )
}

pub fn scene_four(
    bvh_type: SplitType,
    aspect_ratio: Float,
) -> Scene<PrimitiveEnum<MaterialEnum>, MaterialEnum> {
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

    scene!(
        position!(-5, 3, -3),
        position!(0, 0, 0),
        position!(0, 1, 0),
        34,
        aspect_ratio,
        0,
        10,
        sky!(),
        bvh_type,
        primitives
    )
}

pub fn scene_five(
    bvh_type: SplitType,
    aspect_ratio: Float,
) -> Scene<PrimitiveEnum<MaterialEnum>, MaterialEnum> {
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

    scene!(
        position!(-5, 4, -3),
        position!(0, 0.5, 0),
        position!(0, 1, 0),
        34,
        aspect_ratio,
        0,
        10,
        sky,
        bvh_type,
        primitives
    )
}

pub fn scene_six(
    bvh_type: SplitType,
    aspect_ratio: Float,
) -> Scene<PrimitiveEnum<MaterialEnum>, MaterialEnum> {
    let mut primitives = Vec::new();

    let ground = sphere!(0, -1000, 0, 1000, &diffuse!(0.5, 0.5, 0.5, 0.5));

    let glowy = sphere!(5, 3.5, 5, 1.5, &emit!(&solid_colour!(colour!(1)), 5));

    primitives.push(ground);
    primitives.push(glowy);
    primitives.extend(model!(
        "res/dragon.obj",
        &refract!(&solid_colour!(1, 1, 1), 1.52)
    ));

    let sky = sky!();

    scene!(
        position!(-20, 20, -25),
        position!(0, 3.5, 0),
        position!(0, 1, 0),
        34,
        aspect_ratio,
        0,
        10,
        sky,
        bvh_type,
        primitives
    )
}
