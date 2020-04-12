use super::board::tests::*;
use crate::num_ext::*;
use std::collections::HashMap;

#[allow(dead_code)]
pub(crate) fn pixels(black_pixels: &BlackPixels) {
    let mut image = image::DynamicImage::new_luma8(
        black_pixels[0].len() as u32,
        black_pixels.len() as u32,
    );
    let gray_image = image.as_mut_luma8().unwrap();

    for (x, y, pixel) in gray_image.enumerate_pixels_mut() {
        let is_pixel_black = black_pixels
            .get(y as usize)
            .unwrap()
            .get(x as usize)
            .unwrap();

        if *is_pixel_black {
            pixel.0 = [0];
        } else {
            pixel.0 = [255];
        }
    }

    image
        .save(format!("pixels-v{}-debug.jpeg", env!("CARGO_PKG_VERSION")))
        .expect("Cannot save image");
}

#[allow(dead_code)]
pub(crate) fn stones(w: u32, h: u32, stones: &[BlackStone]) {
    let mut image = image::DynamicImage::new_luma8(w, h);
    let gray_image = image.as_mut_luma8().unwrap();

    for stone in stones {
        for y in stone.top_left.y..stone.bottom_right.y {
            for x in stone.top_left.x..stone.bottom_right.x {
                let pixel = gray_image.get_pixel_mut(x, y);
                pixel.0 = [255];
            }
        }
    }

    image
        .save(format!("stones-v{}-debug.jpeg", env!("CARGO_PKG_VERSION")))
        .expect("Cannot save image");
}

#[allow(dead_code)]
pub(crate) fn board(
    w: u32,
    h: u32,
    field_size: f32,
    center: Point,
    stones: &HashMap<(i8, i8), Point>,
) {
    let field_radius = field_size as u32 / 2;
    let mut image = image::DynamicImage::new_luma8(w, h);
    let gray_image = image.as_mut_luma8().unwrap();

    // Draws the fields.
    for (_, stone) in stones {
        for y in (stone.y - field_radius)..(stone.y + field_radius) {
            for x in (stone.x - field_radius)..(stone.x + field_radius) {
                let pixel = gray_image.get_pixel_mut(x, y);
                pixel.0 = [255];
            }
        }
    }

    // Draws the lines.
    for (x, y, pixel) in gray_image.enumerate_pixels_mut() {
        let diff_x = x.diff(center.x) as f32;
        let diff_y = y.diff(center.y) as f32;
        let div_x = diff_x / field_size;
        let div_y = diff_y / field_size;
        if div_x.fract() < 0.05 || div_y.fract() < 0.05 {
            pixel.0 = [128];
        }
    }

    image
        .save(format!("board-v{}-debug.jpeg", env!("CARGO_PKG_VERSION")))
        .expect("Cannot save image");
}

#[allow(dead_code)]
pub(crate) fn board_on_image(
    image: &image::RgbImage,
    field_size: f32,
    stones: &[Point],
    should_highlight: impl Fn(u32, u32) -> bool,
) {
    let field_radius = field_size as u32 / 2;
    let mut image = image.clone();

    // // Draws the fields.
    // for stone in stones {
    //     for y in (stone.y - field_radius)..(stone.y + field_radius) {
    //         for x in (stone.x - field_radius)..(stone.x + field_radius) {
    //             let pixel = image.get_pixel_mut(x, y);
    //             pixel.0 = [255, 255, 255];
    //         }
    //     }
    // }

    // Draws the lines.
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        if should_highlight(x, y) {
            pixel.0 = [255, 0, 0];
        }
    }

    image
        .save(format!(
            "board_on_image-v{}-debug.jpeg",
            env!("CARGO_PKG_VERSION")
        ))
        .expect("Cannot save image");
}
