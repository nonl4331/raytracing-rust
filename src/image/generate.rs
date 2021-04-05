pub fn check_percent(percent: u32, width: u32, x: u32, y: u32) {
    let pixel_num = (x + 1) + y * width;
    if pixel_num % percent == 0 {
        println!("generating image: {}%", pixel_num / percent);
    }
}
