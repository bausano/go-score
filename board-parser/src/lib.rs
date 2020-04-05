mod num_ext;
mod xd;

pub fn parse_image(image: image::RgbImage) {
    let black_stones = xd::find_black_stones(&image);
    println!("Found {} black stones", black_stones.len());
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::path::Path;

//     const ASSETS_DIR: &str = "assets/test";

//     #[test]
//     fn development_test() {
//         let assets = &Path::new(ASSETS_DIR);
//         let image =
//             image::open(assets.join("test2.jpeg")).expect("Cannot open image");
//         parse_image(image.to_rgb());
//     }
// }
