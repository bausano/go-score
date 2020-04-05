use crate::num_ext::*;

const BLACK_THRESHOLD: u8 = 30;
const GRAYNESS_LIMIT: u8 = 8;

pub(crate) type BlackPixels = Vec<Vec<bool>>;

pub struct BlackStone {
    /// The left most point with the lowest y and x value.
    pub top_left: Point,
    /// The right most point with the highest y and x value.
    pub bottom_right: Point,
}

#[derive(Copy, Clone)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

/// Helper function for accessing values at given address in vector. If the
/// address is out of bounds, it delivers the default value instead.
fn pixel_value<T: Copy>(
    vec: &Vec<Vec<T>>,
    x: isize,
    y: isize,
    default: T,
) -> T {
    if x < 0 || y < 0 {
        return default;
    }

    match vec.get(y as usize) {
        None => default,
        Some(row) => match row.get(x as usize) {
            None => default,
            Some(value) => *value,
        },
    }
}

impl BlackStone {
    /// Factory function initializing new object from given dimensions.
    fn new(at: Point) -> Self {
        Self {
            top_left: at,
            bottom_right: at,
        }
    }

    /// Pushes new point to the object and refreshes size cache.
    fn push(&mut self, point: Point) {
        // We're trying to minimize the top left point's x and y, because
        // the more top the less y and the more left the less x.
        self.top_left.x = self.top_left.x.min(point.x);
        self.top_left.y = self.top_left.y.min(point.y);

        // We're trying to maximize the bottom right point's x and y,
        // because the more bottom the more y and the more right the more x.
        self.bottom_right.x = self.bottom_right.x.max(point.x);
        self.bottom_right.y = self.bottom_right.y.max(point.y);
    }
}

pub fn find_black_stones(image: &image::RgbImage) -> Vec<BlackStone> {
    let (width, height) = image.dimensions();
    let width_usize = width as usize;
    let mut black_pixels: BlackPixels = (0..height)
        .map(|_| Vec::with_capacity(width_usize))
        .collect();

    for (y, pixels) in image.enumerate_rows() {
        let row = black_pixels
            .get_mut(y as usize)
            .expect("There aren't enough rows in black_pixels");

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

    find_black_objects(black_pixels)

    // TODO: Remove mistakes.
}

/// Finds objects within given 2D array which has black pixels only. Uses flood
/// fill algorithm which, after finding any highlighted unvisited point within
/// the image, selects all highlighted other points in the neighborhood. This
/// happens recursively for each highlighted unvisited point.
fn find_black_objects(mut image: BlackPixels) -> Vec<BlackStone> {
    // Currently iterated point in the image.
    let mut current_point: Point = Point::new(0, 0);
    // Instantiates the return vector.
    let mut objects: Vec<BlackStone> = Vec::new();

    // A checkpoint as image dimensions. When the cycle reaches this point, we
    // can abort.
    let last_point: Point =
        Point::new(image[0].len() as u32 - 1, image.len() as u32 - 1);

    // As long as the currently iterated point is not the last one, run the cycle.
    while current_point != last_point {
        // If the value at currently iterated point is positive, flood fill the
        // object and remove it from the original map.
        if pixel_value(
            &image,
            current_point.x as isize,
            current_point.y as isize,
            false,
        ) {
            let mut object: BlackStone = BlackStone::new(current_point);
            flood_fill(current_point, &mut object, &mut image);
            objects.push(object);
        }

        // Increments the row starting from 0 if current_point reached the end of
        // the line otherwise moves to the pixel to the right.
        if current_point.x == last_point.x {
            current_point.x = 0;
            current_point.y += 1;
        } else {
            current_point.x += 1;
        }
    }

    objects
}

/// Recursively finds a single object within given image. It calls this function
/// for every new highlighted point.
fn flood_fill(point: Point, object: &mut BlackStone, image: &mut BlackPixels) {
    // Adds currently iterated point to the object and set that point to no
    // highlighted.
    object.push(point);
    image[point.y as usize][point.x as usize] = false;

    // Iterates over the Moore neighborhood of currently iterated point.
    for y in (point.y as isize - 1)..(point.y as isize + 2) {
        if y < 0 {
            continue;
        }

        for x in (point.x as isize - 1)..(point.x as isize + 2) {
            // If the Moore's point is not highlighted, skips.
            if x < 0 || !pixel_value(image, x, y, false) {
                continue;
            }

            // Visits the Moore's point.
            flood_fill(Point::new(x as u32, y as u32), object, image);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASSETS_DIR: &str = "assets/test";
    const TEST_BLACK_STONE_COUNTS: &[(&str, usize)] = &[
        ("test1", 4),
        ("test2", 9),
        ("test3", 9),
        ("test4", 13),
        ("test5", 18),
        ("test6", 18),
        ("test7", 2),
        ("test8", 2),
    ];

    #[test]
    fn test_count_black_stones() {
        for (test, count) in TEST_BLACK_STONE_COUNTS {
            let image = image::open(format!("{}/{}.jpeg", ASSETS_DIR, test))
                .expect("Cannot open image");
            let stones_count = find_black_stones(&image.to_rgb()).len();
            assert_eq!(
                stones_count, *count,
                "test file {} has a mismatched count of black stones",
                test
            )
        }
    }
}
