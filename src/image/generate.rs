use crate::{
    aacuboid, aarect, axis, checkered, colour, diffuse, emit, image, model, position, reflect,
    refract, scene, sky, solid_colour, sphere, texture_lerp,
};

use crate::bvh::split::SplitType;

use crate::image::scene::Scene;

use crate::math;

use crate::ray_tracing::{
    load_model::load_model,
    material::*,
    primitives::{AACuboid, AARect, Axis, Primitive, Sphere},
    ray::Colour,
    sky::Sky,
    texture::{CheckeredTexture, ImageTexture, Lerp, SolidColour, Texture},
};

use std::sync::Arc;

use ultraviolet::{Vec2, Vec3};

pub fn scene_one(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground = sphere!(0, -1000, 0, 1000, diffuse!(0.5, 0.5, 0.5, 0.5));

    let sphere_one = sphere!(0, 1, 0, 1, refract!(&solid_colour!(colour!(1)), 1.5));

    let sphere_two = sphere!(-4, 1, 0, 1, diffuse!(0.4, 0.2, 0.1, 0.5));

    let sphere_three = sphere!(4, 1, 0, 1, reflect!(&solid_colour!(0.7, 0.6, 0.5), 0));

    primitives.push(ground);
    primitives.push(sphere_one);
    primitives.push(sphere_two);
    primitives.push(sphere_three);

    use math::random_f32;

    for a in -11..11 {
        for b in -11..11 {
            let center = position!(
                a as f32 + 0.9 * random_f32(),
                0.2,
                b as f32 + 0.9 * random_f32()
            );

            if (center - position!(4.0, 0.2, 0.0)).mag() > 0.9 {
                let choose_material = random_f32();
                let colour = colour!(random_f32(), random_f32(), random_f32());

                let sphere;

                if choose_material < 0.8 {
                    sphere = sphere!(center, 0.2, diffuse!(&solid_colour!(colour), 0.5));
                } else if choose_material < 0.95 {
                    sphere = sphere!(
                        center,
                        0.2,
                        reflect!(&solid_colour!(colour), random_f32() / 2.0)
                    );
                } else {
                    sphere = sphere!(center, 0.2, refract!(&solid_colour!(colour!(1)), 1.5));
                }
                primitives.push(sphere);
            }
        }
    }

    let sky = sky!(texture_lerp!(colour!(0.5, 0.7, 1), colour!(1)));

    scene!(
        position!(13, 2, -3),
        position!(0, 1, 0.1),
        position!(0, 1, 0),
        34,
        aspect_ratio,
        0.1,
        10,
        sky,
        bvh_type,
        primitives
    )
}

pub fn scene_two(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground = sphere!(0, -1000, 0, 1000, diffuse!(0.5, 0.5, 0.5, 0.5));

    let sphere_two = sphere!(-1.5, 0.5, 1.5, 0.5, diffuse!(0, 1, 0, 0.5));

    let sphere_three = sphere!(0, 1.5, 0, 0.5, diffuse!(1, 1, 0, 0.5));

    let sphere_four = sphere!(-1, 1.5, 0, 0.5, diffuse!(0, 1, 1, 0.5));

    let rect_one = aarect!(
        position!(-2.5, 0.5),
        position!(2.5, 2.5),
        2,
        axis!(Z),
        reflect!(&solid_colour!(1, 0.9, 0.9), 0.001)
    );

    primitives.push(ground);
    primitives.push(rect_one);
    primitives.push(sphere_two);
    primitives.push(sphere_three);
    primitives.push(sphere_four);

    let sky = sky!(texture_lerp!(colour!(0.5, 0.7, 1), colour!(1)));

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

pub fn scene_three(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground = sphere!(0, -1000, 0, 1000, diffuse!(0.5, 0.5, 0.5, 0.5));

    let box_left = aacuboid!(-1.6, 1, -0.5, -0.6, 2, 0.5, diffuse!(1, 0, 0, 0.5));

    let box_middle = aacuboid!(
        -0.5,
        1,
        -0.5,
        0.5,
        2,
        0.5,
        reflect!(&solid_colour!(1, 1, 1), 0)
    );

    let sphere_middle = sphere!(0, 2.5, 0, 0.3, reflect!(&solid_colour!(1, 1, 1), 0));

    let box_right = aacuboid!(0.6, 1, -0.5, 1.6, 2, 0.5, diffuse!(0, 0, 1, 0.5));

    primitives.push(ground);
    primitives.push(box_left);
    primitives.push(box_middle);
    primitives.push(sphere_middle);
    primitives.push(box_right);

    let sky = sky!(texture_lerp!(colour!(0.5, 0.7, 1), colour!(1)));

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

pub fn scene_four(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground = sphere!(0, -1000, 0, 1000, diffuse!(0.5, 0.5, 0.5, 0.5));

    let glowy = sphere!(0, 0.5, 0, 0.5, emit!(&solid_colour!(colour!(1)), 1.5));

    let cube = aacuboid!(
        -0.5,
        0.1,
        -0.5,
        -0.4,
        0.2,
        -0.4,
        diffuse!(0.5, 0.5, 0.5, 0.5)
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

pub fn scene_five(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground = sphere!(
        0,
        -1000,
        0,
        1000,
        diffuse!(&checkered!(colour!(0), colour!(0.5)), 0.5)
    );

    let cube = aacuboid!(-0.5, 0.1, -0.5, 1, 0.6, 1, diffuse!(0.5, 0.5, 0.5, 0.5));

    let earth = sphere!(0, 1.2, 0, 0.5, diffuse!(&image!("res/earth.png"), 0.5));

    primitives.push(ground);
    primitives.push(cube);
    primitives.push(earth);

    let sky = sky!(texture_lerp!(colour!(0.5, 0.7, 1), colour!(1)));

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

pub fn scene_six(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground = sphere!(0, -1000, 0, 1000, diffuse!(0.5, 0.5, 0.5, 0.5));

    primitives.push(ground);
    primitives.extend(model!("res/dragon.obj", diffuse!(0.5, 0.5, 0.5, 0.5)));

    let sky = sky!(texture_lerp!(colour!(0.5, 0.7, 1.0), colour!(1)));

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
