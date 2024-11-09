extern crate pixelmatch;

use image::*;
use pixelmatch::*;

fn main() {
    let img1 = image::open("./examples/4a.png").unwrap();
    let img2 = image::open("./examples/4b.png").unwrap();

    println!("{}, {}", img1.raw_pixels().len(), img2.raw_pixels().len());

    let result = pixelmatch(
        &img1.to_rgba(),
        &img2.to_rgba(),
        img1.dimensions(),
        Some(PixelmatchOption {
            threshold: 0.1,
            include_anti_alias: true,
            ..PixelmatchOption::default()
        }),
    )
    .unwrap();

    image::save_buffer(
        "./examples/4c.png",
        &result.diff_image,
        img1.dimensions().0,
        img1.dimensions().1,
        image::RGBA(8),
    )
    .unwrap()
}
