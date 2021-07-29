use crate::{colour, diffuse, lerp, position, reflect, refract, scene, sky, solid_colour, sphere};

use crate::bvh::split::SplitType;

use crate::image::scene::Scene;

use crate::math;

use crate::ray_tracing::{
    material::*,
    primitives::{AABox, AARect, Axis, Primitive, Sphere},
    ray::Colour,
    sky::Sky,
    texture::{CheckeredTexture, ImageTexture, Lerp, SolidColour, Texture},
};

use ultraviolet::{Vec2, Vec3};

const GROUND_COLOUR: Texture = Texture::SolidColour(SolidColour {
    colour: Colour {
        x: 0.5,
        y: 0.5,
        z: 0.5,
    },
});

pub fn scene_one(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground = sphere!(0, -1000, 0, 1000, diffuse!(0.5, 0.5, 0.5, 0.5));

    let sphere_one = sphere!(0, 1, 0, 1, refract!(colour!(1), 1.5));

    let sphere_two = sphere!(-4, 1, 0, 1, diffuse!(0.4, 0.2, 0.1, 0.5));

    let sphere_three = sphere!(4, 1, 0, 1, reflect!(colour!(0.7, 0.6, 0.5), 0));

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
                    sphere = sphere!(center, 0.2, diffuse!(solid_colour!(colour), 0.5));
                } else if choose_material < 0.95 {
                    sphere = sphere!(center, 0.2, reflect!(colour, random_f32() / 2.0));
                } else {
                    sphere = sphere!(center, 0.2, refract!(colour!(1), 1.5));
                }
                primitives.push(sphere);
            }
        }
    }

    let sky = sky!(lerp!(colour!(0.5, 0.7, 1.0), colour!(1)));

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

    let ground: Sphere = Sphere::new(
        Vec3 {
            x: 0.0,
            y: -1000.0,
            z: 1.0,
        },
        1000.0,
        Material::Diffuse(Diffuse::new(GROUND_COLOUR, 0.5)),
    );

    let sphere_three = Sphere::new(
        Vec3 {
            x: 0.0,
            y: 1.5,
            z: 0.0,
        },
        0.5,
        Material::Diffuse(Diffuse::new(solid_colour!(1.0, 1.0, 0.0), 0.5)),
    );

    let sphere_four = Sphere::new(
        Vec3 {
            x: -1.0,
            y: 1.5,
            z: 0.0,
        },
        0.5,
        Material::Diffuse(Diffuse::new(solid_colour!(0.0, 1.0, 1.0), 0.5)),
    );

    let sphere_two = Sphere::new(
        Vec3 {
            x: -1.5,
            y: 0.5,
            z: 1.5,
        },
        0.5,
        Material::Diffuse(Diffuse::new(solid_colour!(0.0, 1.0, 0.0), 0.5)),
    );

    let rect_one = AARect::new(
        Vec2::new(-2.5, 0.5),
        Vec2::new(2.5, 2.5),
        2.0,
        Axis::Z,
        Material::Reflect(Reflect::new(Colour::new(1.0, 0.9, 0.9), 0.001)),
    );

    primitives.push(Primitive::Sphere(ground));
    primitives.push(Primitive::AARect(rect_one));
    primitives.push(Primitive::Sphere(sphere_two));
    primitives.push(Primitive::Sphere(sphere_three));
    primitives.push(Primitive::Sphere(sphere_four));

    let sky = Sky::new(Some(Texture::Lerp(Lerp::new(
        Colour::new(0.5, 0.7, 1.0),
        Colour::one(),
    ))));

    Scene::new(
        Vec3::new(3.0, 1.0, -15.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        34.0,
        aspect_ratio,
        0.0,
        10.0,
        sky,
        bvh_type,
        primitives,
    )
}

pub fn scene_three(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground: Sphere = Sphere::new(
        Vec3 {
            x: 0.0,
            y: -1000.0,
            z: 0.0,
        },
        1000.0,
        Material::Diffuse(Diffuse::new(GROUND_COLOUR, 0.5)),
    );

    let box_left = AABox::new(
        Vec3 {
            x: -1.6,
            y: 1.0,
            z: -0.5,
        },
        Vec3 {
            x: -0.6,
            y: 2.0,
            z: 0.5,
        },
        Material::Diffuse(Diffuse::new(solid_colour!(1.0, 0.0, 0.0), 0.5)),
    );

    let box_middle = AABox::new(
        Vec3 {
            x: -0.5,
            y: 1.0,
            z: -0.5,
        },
        Vec3 {
            x: 0.5,
            y: 2.0,
            z: 0.5,
        },
        Material::Reflect(Reflect::new(Colour::one(), 0.0)),
    );

    let sphere_middle = Sphere::new(
        Vec3::new(0.0, 2.5, 0.0),
        0.3,
        Material::Reflect(Reflect::new(Colour::new(1.0, 1.0, 1.0), 0.0)),
    );

    let box_right = AABox::new(
        Vec3 {
            x: 0.6,
            y: 1.0,
            z: -0.5,
        },
        Vec3 {
            x: 1.6,
            y: 2.0,
            z: 0.5,
        },
        Material::Diffuse(Diffuse::new(solid_colour!(0.0, 0.0, 1.0), 0.5)),
    );

    primitives.push(Primitive::Sphere(ground));
    primitives.push(Primitive::AABox(box_left));
    primitives.push(Primitive::AABox(box_middle));
    primitives.push(Primitive::Sphere(sphere_middle));
    primitives.push(Primitive::AABox(box_right));

    let sky = Sky::new(Some(Texture::Lerp(Lerp::new(
        Colour::new(0.5, 0.7, 1.0),
        Colour::one(),
    ))));

    Scene::new(
        Vec3::new(-5.0, 3.0, -3.0),
        Vec3::new(0.0, 1.5, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        34.0,
        aspect_ratio,
        0.0,
        10.0,
        sky,
        bvh_type,
        primitives,
    )
}

pub fn scene_four(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground: Sphere = Sphere::new(
        Vec3 {
            x: 0.0,
            y: -1000.0,
            z: 0.0,
        },
        1000.0,
        Material::Diffuse(Diffuse::new(GROUND_COLOUR, 0.5)),
    );

    let glowy_mat = solid_colour!(1.0, 1.0, 1.0);
    let glowy = Sphere::new(
        Vec3::new(0.0, 0.5, 0.0),
        0.5,
        Material::Emit(Emit::new(glowy_mat, 1.5)),
    );
    let cube = AABox::new(
        Vec3::new(-0.5, 0.1, -0.5),
        Vec3::new(-0.4, 0.2, -0.4),
        Material::Diffuse(Diffuse::new(GROUND_COLOUR, 0.5)),
    );

    primitives.push(Primitive::Sphere(ground));
    primitives.push(Primitive::Sphere(glowy));
    primitives.push(Primitive::AABox(cube));

    let sky = Sky::new(None);

    Scene::new(
        Vec3::new(-5.0, 3.0, -3.0),
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        34.0,
        aspect_ratio,
        0.0,
        10.0,
        sky,
        bvh_type,
        primitives,
    )
}

// WIP
pub fn scene_five(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground_mat = Texture::CheckeredTexture(CheckeredTexture::new(
        Colour::new(0.0, 0.0, 0.0),
        Colour::new(0.5, 0.5, 0.5),
    ));

    let ground: Sphere = Sphere::new(
        Vec3 {
            x: 0.0,
            y: -1000.0,
            z: 0.0,
        },
        1000.0,
        Material::Diffuse(Diffuse::new(ground_mat, 0.5)),
    );

    let cube = AABox::new(
        Vec3::new(-0.5, 0.1, -0.5),
        Vec3::new(1.0, 0.6, 1.0),
        Material::Diffuse(Diffuse::new(GROUND_COLOUR, 0.5)),
    );

    let earth_mat = Material::Diffuse(Diffuse::new(
        Texture::ImageTexture(ImageTexture::new("res/earth.png")),
        0.5,
    ));
    let earth = Sphere::new(Vec3::new(0.0, 1.2, 0.0), 0.5, earth_mat);

    primitives.push(Primitive::Sphere(ground));
    primitives.push(Primitive::AABox(cube));
    primitives.push(Primitive::Sphere(earth));

    let sky = Sky::new(Some(Texture::Lerp(Lerp::new(
        Colour::new(0.5, 0.7, 1.0),
        Colour::one(),
    ))));

    Scene::new(
        Vec3::new(-5.0, 4.0, -3.0),
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        34.0,
        aspect_ratio,
        0.0,
        10.0,
        sky,
        bvh_type,
        primitives,
    )
}

pub fn scene_six(bvh_type: SplitType, aspect_ratio: f32) -> Scene {
    let mut primitives: Vec<Primitive> = Vec::new();

    let ground: Sphere = Sphere::new(
        Vec3 {
            x: 0.0,
            y: -1001.0,
            z: 0.0,
        },
        1000.0,
        Material::Diffuse(Diffuse::new(GROUND_COLOUR, 0.5)),
    );

    primitives.push(Primitive::Sphere(ground));
    primitives.extend(crate::ray_tracing::load_model::load_model(
        "res/dragon.obj",
        Material::Diffuse(Diffuse::new(solid_colour!(0.5, 0.5, 0.5), 0.5)),
    ));

    let sky = Sky::new(Some(Texture::Lerp(Lerp::new(
        Colour::new(0.5, 0.7, 1.0),
        Colour::one(),
    ))));

    Scene::new(
        Vec3::new(-20.0, 20.0, -25.0),
        Vec3::new(0.0, 3.5, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        34.0,
        aspect_ratio,
        0.0,
        10.0,
        sky,
        bvh_type,
        primitives,
    )
}
