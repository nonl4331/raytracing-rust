use crate::image::generate;

mod image;

fn main() {
    let scene = generate::scene_three();

    scene.generate_image_sample_threaded("out.png", 1920, 30);
}
