use crate::image::generate;

mod image;

fn main() {
    let scene = generate::scene_one();

    scene.generate_image_sample_threaded("out.png", 800, 50);
}
