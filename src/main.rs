use crate::image::ray::Color;
use crate::image::scene::Diffuse;
use crate::image::scene::Reflect;
use crate::image::scene::Refract;
use crate::image::scene::Scene;
use crate::image::scene::Sphere;
use std::sync::Arc;

use ultraviolet::DVec3;

mod image;

fn main() {
    let mut scene = Scene::new(16.0 / 9.0, 1.0, 2.0);

    let main_sphere: Sphere = Sphere {
        center: DVec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        radius: 0.5,
        material: Arc::new(Box::new(Diffuse {
            color: Color::new(0.1, 0.2, 0.5),
            absorption: 0.5,
        })),
    };
    let ground: Sphere = Sphere {
        center: DVec3 {
            x: 0.0,
            y: 100.5,
            z: 1.0,
        },
        radius: 100.0,
        material: Arc::new(Box::new(Diffuse {
            color: Color::new(0.8, 0.8, 0.0),
            absorption: 0.5,
        })),
    };
    let sphere_two: Sphere = Sphere {
        center: DVec3 {
            x: 1.0,
            y: 0.0,
            z: 1.0,
        },
        radius: 0.5,
        material: Arc::new(Box::new(Reflect {
            color: Color::new(0.8, 0.6, 0.2),
            fuzz: 0.0,
        })),
    };
    let sphere_three: Sphere = Sphere {
        center: DVec3 {
            x: -1.0,
            y: 0.0,
            z: 1.0,
        },
        radius: -0.4,
        material: Arc::new(Box::new(Refract {
            color: Color::new(1.0, 1.0, 1.0),
            eta: 1.5,
        })),
    };

    let sphere_four: Sphere = Sphere {
        center: DVec3 {
            x: -1.0,
            y: 0.0,
            z: 1.0,
        },
        radius: 0.5,
        material: Arc::new(Box::new(Refract {
            color: Color::new(1.0, 1.0, 1.0),
            eta: 1.5,
        })),
    };

    scene.add(Box::new(main_sphere));
    scene.add(Box::new(ground));
    scene.add(Box::new(sphere_two));
    scene.add(Box::new(sphere_three));
    scene.add(Box::new(sphere_four));

    scene.generate_image(800, 50);
}
