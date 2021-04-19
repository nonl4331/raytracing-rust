use crate::image::hittables::Material;
use crate::image::scene::Scene;

use crate::image::tracing::Hittable;

use crate::image::ray::Color;

use ultraviolet::DVec3;

use crate::image::hittables::{Diffuse, Reflect, Refract, Sphere};

use crate::image::math;

pub fn check_percent(percent: u32, width: u32, x: u32, y: u32) {
    let pixel_num = (x + 1) + y * width;
    if pixel_num % percent == 0 {
        println!("generating image: {}%", pixel_num / percent);
    }
}

pub fn scene_one() -> Scene {
    let mut hittables: Vec<Hittable> = Vec::new();

    let ground_color = Color::new(0.5, 0.5, 0.5);

    let ground: Sphere = Sphere::new(
        DVec3 {
            x: 0.0,
            y: 1000.0,
            z: 1.0,
        },
        1000.0,
        Material::Diffuse(Diffuse::new(ground_color, 0.5)),
    );

    let sphere_one = Sphere::new(
        DVec3 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        },
        1.0,
        Material::Refract(Refract::new(Color::one(), 1.5)),
    );

    let two_color = Color::new(0.4, 0.2, 0.1);
    let sphere_two = Sphere::new(
        DVec3 {
            x: -4.0,
            y: -1.0,
            z: 0.0,
        },
        1.0,
        Material::Diffuse(Diffuse::new(two_color, 0.5)),
    );

    let three_color = Color::new(0.7, 0.6, 0.5);
    let sphere_three = Sphere::new(
        DVec3 {
            x: 4.0,
            y: -1.0,
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
                -0.2,
                b as f64 + 0.9 * math::random_f64(),
            );

            if (center - DVec3::new(4.0, 0.2, 0.0)).mag() > 0.9 {
                let choose_material = math::random_f64();
                let color = Color::new(math::random_f64(), math::random_f64(), math::random_f64());
                let sphere: Sphere;

                if choose_material < 0.8 {
                    // diffuse sphere
                    sphere = Sphere::new(center, 0.2, Material::Diffuse(Diffuse::new(color, 0.5)));
                } else if choose_material < 0.95 {
                    // metal sphere
                    sphere = Sphere::new(
                        center,
                        0.2,
                        Material::Reflect(Reflect::new(color, math::random_f64() / 2.0)),
                    );
                } else {
                    // glass sphere
                    sphere = Sphere::new(
                        center,
                        0.2,
                        Material::Refract(Refract::new(Color::one(), 1.5)),
                    );
                }
                hittables.push(Hittable::Sphere(sphere));
            }
        }
    }
    Scene::new(
        DVec3::new(13.0, -2.0, 3.0),
        DVec3::new(0.0, 0.0, 0.0),
        DVec3::new(0.0, -1.0, 0.0),
        34.0,
        16.0 / 9.0,
        0.1,
        10.0,
        Some(hittables),
    )
}
