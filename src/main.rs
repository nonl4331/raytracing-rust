use crate::image::generate;

mod image;

fn main() {
    let scene = generate::scene_one();

    scene.generate_image(1920, 500);
}
