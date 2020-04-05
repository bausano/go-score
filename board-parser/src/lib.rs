use std::fmt::Display;
use std::ops::Sub;

pub fn parse_image() {
    unimplemented!()
}

struct BlackPixels {
    inner: Vec<Vec<bool>>,
    dimensions: (u32, u32),
}

impl BlackPixels {
    fn new(width: u32, height: u32, img: Vec<Vec<bool>>) -> Self {
        Self {
            inner: img,
            dimensions: (width, height),
        }
    }

    fn debug(&self, note: impl Display) {
        let dyn_img = image::DynamicImage::from(self);
        dyn_img
            .save(format!("image-{}.png", note))
            .expect("Cannot save image highlight for debug");
    }
}

impl From<&BlackPixels> for image::DynamicImage {
    fn from(image_highlight: &BlackPixels) -> Self {
        let (width, height) = image_highlight.dimensions;
        let mut image = Self::new_luma8(width, height);
        let gray_image = image.as_mut_luma8().unwrap();

        for (x, y, pixel) in gray_image.enumerate_pixels_mut() {
            let is_pixel_black = image_highlight
                .inner
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
    }
}

const BLACK_THRESHOLD: u8 = 25;
const GRAYNESS_LIMIT: u8 = 5;

pub fn xd(image: image::RgbImage) {
    let (width, height) = image.dimensions();
    let width_usize = width as usize;
    let mut image_highlight: Vec<Vec<bool>> = (0..height)
        .map(|_| Vec::with_capacity(width_usize))
        .collect();

    for (y, pixels) in image.enumerate_rows() {
        let row = image_highlight
            .get_mut(y as usize)
            .expect("There aren't enough rows in image_highlight");

        for (_, _, pixel) in pixels {
            let [r, g, b] = pixel.0;
            let is_gray = || {
                r.diff(g) <= GRAYNESS_LIMIT
                    && r.diff(b) <= GRAYNESS_LIMIT
                    && g.diff(b) <= GRAYNESS_LIMIT
            };
            row.push(r < BLACK_THRESHOLD && is_gray());
        }
    }

    let image_highlight = BlackPixels::new(width, height, image_highlight);
    image_highlight.debug("xd");
}

trait NumExt
where
    Self: Sub<Output = Self> + PartialOrd<Self> + Copy + Sized,
{
    fn diff(self, other: Self) -> Self {
        if self > other {
            self - other
        } else {
            other - self
        }
    }
}

impl NumExt for u8 {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const ASSETS_DIR: &str = "assets/test";

    #[test]
    fn development_test() {
        let assets = &Path::new(ASSETS_DIR);
        let image =
            image::open(assets.join("test2.jpeg")).expect("Cannot open image");
        xd(image.to_rgb());
    }
}
