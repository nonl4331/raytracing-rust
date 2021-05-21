use crate::image::hittables::MovingSphere;
use crate::image::material::Material;
use crate::image::math::random_f64;
use crate::image::scene::Scene;
use crate::image::sky::Sky;
use ultraviolet::DVec2;

use crate::image::tracing::Hittable;

use crate::image::ray::Color;

use ultraviolet::DVec3;

use crate::image::hittables::{AABox, AARect, Axis, Sphere};
use crate::image::material::*;

use crate::image::math;

pub fn check_percent(percent: u32, width: u32, x: u32, y: u32) {
    let pixel_num = (x + 1) + y * width;
    if pixel_num % percent == 0 {
        println!("generating image: {}%", pixel_num / percent);
    }
}

pub fn scene_one(aspect_ratio: f64, motion_blur: bool) -> Scene {
    let mut hittables: Vec<Hittable> = Vec::new();

    let ground_color = Color::new(0.5, 0.5, 0.5);

    let ground: Sphere = Sphere::new(
        DVec3 {
            x: 0.0,
            y: -1000.0,
            z: 1.0,
        },
        1000.0,
        Material::Diffuse(Diffuse::new(ground_color, 0.5)),
    );

    let sphere_one = Sphere::new(
        DVec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        1.0,
        Material::Refract(Refract::new(Color::one(), 1.5)),
    );

    let two_color = Color::new(0.4, 0.2, 0.1);
    let sphere_two = Sphere::new(
        DVec3 {
            x: -4.0,
            y: 1.0,
            z: 0.0,
        },
        1.0,
        Material::Diffuse(Diffuse::new(two_color, 0.5)),
    );

    let three_color = Color::new(0.7, 0.6, 0.5);
    let sphere_three = Sphere::new(
        DVec3 {
            x: 4.0,
            y: 1.0,
            z: 0.0,
        },
        1.0,
        Material::Reflect(Reflect::new(three_color, 0.0)),
    );

    hittables.push(Hittable::Sphere(ground));
    hittables.push(Hittable::Sphere(sphere_one));
    hittables.push(Hittable::Sphere(sphere_two));
    hittables.push(Hittable::Sphere(sphere_three));

    for a in -11..11 {
        for b in -11..11 {
            let center = DVec3::new(
                a as f64 + 0.9 * math::random_f64(),
                0.2,
                b as f64 + 0.9 * math::random_f64(),
            );

            if (center - DVec3::new(4.0, 0.2, 0.0)).mag() > 0.9 {
                let choose_material = math::random_f64();
                let color = Color::new(math::random_f64(), math::random_f64(), math::random_f64());

                if choose_material < 0.8 {
                    // diffuse sphere
                    if motion_blur {
                        let sphere = MovingSphere::new(
                            center,
                            center - DVec3::new(0.0, random_f64() * 0.5, 0.0),
                            0.2,
                            Material::Diffuse(Diffuse::new(color, 0.5)),
                        );
                        hittables.push(Hittable::MovingSphere(sphere));
                    } else {
                        let sphere =
                            Sphere::new(center, 0.2, Material::Diffuse(Diffuse::new(color, 0.5)));
                        hittables.push(Hittable::Sphere(sphere));
                    }
                } else if choose_material < 0.95 {
                    // metal sphere
                    let sphere = Sphere::new(
                        center,
                        0.2,
                        Material::Reflect(Reflect::new(color, math::random_f64() / 2.0)),
                    );
                    hittables.push(Hittable::Sphere(sphere));
                } else {
                    // glass sphere
                    let sphere = Sphere::new(
                        center,
                        0.2,
                        Material::Refract(Refract::new(Color::one(), 1.5)),
                    );
                    hittables.push(Hittable::Sphere(sphere));
                }
            }
        }
    }

    let sky = Sky::new(Some(Color::new(0.5, 0.7, 1.0)));

    Scene::new(
        DVec3::new(13.0, 2.0, -3.0),
        DVec3::new(0.0, 1.0, 0.1),
        DVec3::new(0.0, 1.0, 0.0),
        34.0,
        aspect_ratio,
        0.1,
        10.0,
        sky,
        Some(hittables),
    )
}

pub fn scene_two(aspect_ratio: f64) -> Scene {
    let mut hittables: Vec<Hittable> = Vec::new();

    let ground_color = Color::new(0.5, 0.5, 0.5);

    let ground: Sphere = Sphere::new(
        DVec3 {
            x: 0.0,
            y: -1000.0,
            z: 1.0,
        },
        1000.0,
        Material::Diffuse(Diffuse::new(ground_color, 0.5)),
    );

    let sphere_three = Sphere::new(
        DVec3 {
            x: 0.0,
            y: 1.5,
            z: 0.0,
        },
        0.5,
        Material::Diffuse(Diffuse::new(Color::new(1.0, 1.0, 0.0), 0.5)),
    );

    let sphere_four = Sphere::new(
        DVec3 {
            x: -1.0,
            y: 1.5,
            z: 0.0,
        },
        0.5,
        Material::Diffuse(Diffuse::new(Color::new(0.0, 1.0, 1.0), 0.5)),
    );

    let sphere_two = Sphere::new(
        DVec3 {
            x: -1.5,
            y: 0.5,
            z: 1.5,
        },
        0.5,
        Material::Diffuse(Diffuse::new(Color::new(0.0, 1.0, 0.0), 0.5)),
    );

    let rect_one = AARect::new(
        DVec2::new(-2.5, 0.5),
        DVec2::new(2.5, 2.5),
        2.0,
        Axis::Z,
        Material::Reflect(Reflect::new(Color::new(1.0, 0.9, 0.9), 0.001)),
    );

    hittables.push(Hittable::Sphere(ground));
    hittables.push(Hittable::AARect(rect_one));
    hittables.push(Hittable::Sphere(sphere_two));
    hittables.push(Hittable::Sphere(sphere_three));
    hittables.push(Hittable::Sphere(sphere_four));

    let sky = Sky::new(Some(Color::new(0.5, 0.7, 1.0)));

    Scene::new(
        DVec3::new(3.0, 1.0, -15.0),
        DVec3::new(0.0, 1.0, 0.0),
        DVec3::new(0.0, 1.0, 0.0),
        34.0,
        aspect_ratio,
        0.0,
        10.0,
        sky,
        Some(hittables),
    )
}

pub fn scene_three(aspect_ratio: f64) -> Scene {
    let mut hittables: Vec<Hittable> = Vec::new();

    let ground_color = Color::new(0.5, 0.5, 0.5);

    let ground: Sphere = Sphere::new(
        DVec3 {
            x: 0.0,
            y: -1000.0,
            z: 0.0,
        },
        1000.0,
        Material::Diffuse(Diffuse::new(ground_color, 0.5)),
    );

    let box_left = AABox::new(
        DVec3 {
            x: -1.6,
            y: 1.0,
            z: -0.5,
        },
        DVec3 {
            x: -0.6,
            y: 2.0,
            z: 0.5,
        },
        Material::Diffuse(Diffuse::new(Color::new(1.0, 0.0, 0.0), 0.5)),
    );

    let box_middle = AABox::new(
        DVec3 {
            x: -0.5,
            y: 1.0,
            z: -0.5,
        },
        DVec3 {
            x: 0.5,
            y: 2.0,
            z: 0.5,
        },
        Material::Reflect(Reflect::new(Color::one(), 0.0)),
    );

    let sphere_middle = Sphere::new(
        DVec3::new(0.0, 2.5, 0.0),
        0.3,
        Material::Reflect(Reflect::new(Color::new(1.0, 1.0, 1.0), 0.0)),
    );

    let box_right = AABox::new(
        DVec3 {
            x: 0.6,
            y: 1.0,
            z: -0.5,
        },
        DVec3 {
            x: 1.6,
            y: 2.0,
            z: 0.5,
        },
        Material::Diffuse(Diffuse::new(Color::new(0.0, 0.0, 1.0), 0.5)),
    );

    hittables.push(Hittable::Sphere(ground));
    hittables.push(Hittable::AABox(box_left));
    hittables.push(Hittable::AABox(box_middle));
    hittables.push(Hittable::Sphere(sphere_middle));
    hittables.push(Hittable::AABox(box_right));

    let sky = Sky::new(Some(Color::new(0.5, 0.7, 1.0)));

    Scene::new(
        DVec3::new(-5.0, 3.0, -3.0),
        DVec3::new(0.0, 1.5, 0.0),
        DVec3::new(0.0, 1.0, 0.0),
        34.0,
        aspect_ratio,
        0.0,
        10.0,
        sky,
        Some(hittables),
    )
}

pub fn scene_four(aspect_ratio: f64) -> Scene {
    let mut hittables: Vec<Hittable> = Vec::new();

    let ground_color = Color::new(0.5, 0.5, 0.5);

    let ground: Sphere = Sphere::new(
        DVec3 {
            x: 0.0,
            y: -1000.0,
            z: 0.0,
        },
        1000.0,
        Material::Diffuse(Diffuse::new(ground_color, 0.5)),
    );

    let glowy = Sphere::new(
        DVec3::new(0.0, 0.5, 0.0),
        0.5,
        Material::Emit(Emit::new(Color::one(), 1.5)),
    );
    let cube = AABox::new(
        DVec3::new(-0.5, 0.1, -0.5),
        DVec3::new(-0.4, 0.2, -0.4),
        Material::Diffuse(Diffuse::new(ground_color, 0.5)),
    );

    hittables.push(Hittable::Sphere(ground));
    hittables.push(Hittable::Sphere(glowy));
    hittables.push(Hittable::AABox(cube));

    let sky = Sky::new(None);

    Scene::new(
        DVec3::new(-5.0, 3.0, -3.0),
        DVec3::new(0.0, 0.5, 0.0),
        DVec3::new(0.0, 1.0, 0.0),
        34.0,
        aspect_ratio,
        0.0,
        10.0,
        sky,
        Some(hittables),
    )
}
