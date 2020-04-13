mod board;
#[cfg(test)]
mod debug;
mod num_ext;

pub fn parse_image(image: image::RgbImage) {
    let board_map = board::board_map(&image);
    println!("Here's the board map: {:#?}", board_map);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const ASSETS_DIR: &str = "assets/test";

    #[test]
    fn development_test() {
        let assets = &Path::new(ASSETS_DIR);
        let image =
            image::open(assets.join("test11.jpeg")).expect("Cannot open image");
        board::board_map(&image.to_rgb());
    }
}
