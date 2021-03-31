use crate::image::camera::Camera;
use crate::image::ray::Color;
use image::ImageBuffer;
use rand::Rng;
use ultraviolet::vec::Vec3;

fn check_percent(percent: u32, width: u32, x: u32, y: u32) {
    let pixel_num = (x + 1) + y * width;
    if pixel_num % percent == 0 {
        println!("generating image: {}%", pixel_num / percent);
    }
}

pub fn generate_test_image() {
    let width = 1280u32;
    let aspect_ratio: f32 = 16.0 / 9.0;
    let height = (width as f32 / aspect_ratio) as u32;

    let viewport_height = 2.0f32;
    let focal_length = 1.0f32;
    let origin = Vec3::new(0.0, 0.0, 0.0);
    let pixel_samples = 50;

    let camera = Camera::new(
        viewport_height,
        aspect_ratio,
        focal_length,
        origin,
        pixel_samples,
    );

    let mut image = ImageBuffer::new(width, height);

    let percent = ((width * height) as f32 / 100.0) as u32;

    let mut rng = rand::thread_rng();
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        check_percent(percent, width, x, y);
        let mut color = Color::new(0.0, 0.0, 0.0);
        for _ in 0..camera.pixel_samples {
            let u = (rng.gen_range(0.0..1.0) + x as f32) / width as f32;
            let v = (rng.gen_range(0.0..1.0) + y as f32) / height as f32;

            let mut ray = camera.get_ray(u, v);
            color += ray.get_color(0);
        }
        color /= camera.pixel_samples as f32;

        *pixel = image::Rgb([
            (color.x * 255.0) as u8,
            (color.y * 255.0) as u8,
            (color.z * 255.0) as u8,
        ]);
    }
    println!("Image done generating:");
    println!("Width: {}", width);
    println!("Height: {}", height);
    println!("Samples per pixel: {}", camera.pixel_samples);
    image.save("test.png").unwrap();
}
