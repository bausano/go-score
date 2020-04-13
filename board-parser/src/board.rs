#[cfg(test)]
use crate::debug;
use crate::num_ext::*;
use std::collections::HashMap;

const BLACK_THRESHOLD: u8 = 30;
const GRAYNESS_LIMIT: u8 = 8;
const MIN_STONE_SIZE: f32 = 8.0;
const MIN_BLACK_STONES_ON_BOARD: usize = 6;

pub type BoardMap = HashMap<(i8, i8), Point>;
pub(crate) type BlackPixels = Vec<Vec<bool>>;

#[derive(Debug)]
pub(crate) struct BlackStone {
    /// The left most point with the lowest y and x value.
    pub top_left: Point,
    /// The right most point with the highest y and x value.
    pub bottom_right: Point,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    fn new(x: u32, y: u32) -> Point {
        Point { x, y }
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

    fn width(&self) -> u32 {
        self.bottom_right.x - self.top_left.x
    }

    fn height(&self) -> u32 {
        self.bottom_right.y - self.top_left.y
    }

    fn center(&self) -> Point {
        let x =
            self.bottom_right.x - (self.bottom_right.x - self.top_left.x) / 2;
        let y =
            self.bottom_right.y - (self.bottom_right.y - self.top_left.y) / 2;
        Point::new(x, y)
    }
}

pub fn board_map(image: &image::RgbImage) -> Option<BoardMap> {
    let (_stone_size, stones) = find_black_stones(image)?;
    // From now on we're only concerned about the center points.
    let stones: Vec<_> =
        stones.into_iter().map(|stone| stone.center()).collect();

    // There must be at least a few black stones on the board.
    if stones.len() < MIN_BLACK_STONES_ON_BOARD {
        return None;
    }

    let a = -1.0 / 10000.0;
    let b = -1.0 / 4000.0;
    let cx = 788.0;
    let cy = 585.0;
    let sx = 46.0;
    let sy = 55.0;
    let asx = -1.0 / 110.0;
    let asy = 0.0;

    let e = transformation_error(&stones, cx, cy, a, b, sx, sy, asx, asy);

    println!("This transformation has an error of {}", e);

    // 8 parameters to get right.
    // rotation????????/

    #[cfg(test)]
    debug::highlight_pixels_in_image(&image, |x, y| {
        let x = x as f32;
        let y = y as f32;
        let x = x - cx;
        let y = y - cy;
        let x = x - a * y * x;
        let div_x = x / (sx + asx * x);
        let y = y - b * y * x;
        let div_y = y / (sy + asy * y);
        div_x.fract().abs() < 0.02 || div_y.fract().abs() < 0.02
    });

    None
}

// Given transformation parameters, calculate how well it approximates the found
// black stones positions.
// TODO: Code clean up lol.
fn transformation_error(
    stones: &[Point],
    cx: f32,
    cy: f32,
    a: f32,
    b: f32,
    sx: f32,
    sy: f32,
    asx: f32,
    asy: f32,
) -> f32 {
    let mut stone_errors: HashMap<Point, [((f32, f32), f32); 4]> =
        HashMap::with_capacity(stones.len());
    let mut intersection_stones: HashMap<(isize, isize), Vec<(Point, f32)>> =
        HashMap::with_capacity(stones.len());
    for stone in stones {
        let stone = *stone;
        let (stone_x, stone_y) = (stone.x as f32, stone.y as f32);
        let (from_center_x, from_center_y) = (stone_x - cx, stone_y - cy);

        let inverse_transformation_x =
            from_center_x - a * from_center_y * from_center_x;
        let inverse_transformation_y =
            from_center_y - b * from_center_y * from_center_x;

        let intersections_from_center_x =
            inverse_transformation_x / (sx + asx * inverse_transformation_x);
        let intersections_from_center_y =
            inverse_transformation_y / (sy + asy * inverse_transformation_y);

        let top_left_x = intersections_from_center_x.floor();
        let top_left_y = intersections_from_center_y.floor();
        let bottom_left_x = top_left_x;
        let bottom_left_y = intersections_from_center_y.ceil();
        let top_right_x = intersections_from_center_x.ceil();
        let top_right_y = top_left_y;
        let bottom_right_x = top_right_x;
        let bottom_right_y = bottom_left_y;

        let top_left_e = (top_left_x - intersections_from_center_x).powi(2)
            + (top_left_y - intersections_from_center_y).powi(2);
        let top_right_e = (top_right_x - intersections_from_center_x).powi(2)
            + (top_right_y - intersections_from_center_y).powi(2);
        let bottom_left_e = (bottom_left_x - intersections_from_center_x)
            .powi(2)
            + (bottom_left_y - intersections_from_center_y).powi(2);
        let bottom_right_e = (bottom_right_x - intersections_from_center_x)
            .powi(2)
            + (bottom_right_y - intersections_from_center_y).powi(2);

        let errors = [
            ((top_left_x, top_left_y), top_left_e),
            ((top_right_x, top_right_y), top_right_e),
            ((bottom_left_x, bottom_left_y), bottom_left_e),
            ((bottom_right_x, bottom_right_y), bottom_right_e),
        ];
        let ((least_e_x, least_e_y), least_e) = errors
            .iter()
            .min_by(|(_, a_e), (_, b_e)| a_e.partial_ord(*b_e))
            .copied()
            .expect("There must be one point which has least error");
        stone_errors.insert(stone, errors);
        let least_e_pos = (least_e_x as isize, least_e_y as isize);
        let intersection_candidates =
            intersection_stones.entry(least_e_pos).or_default();
        intersection_candidates.push((stone, least_e));
    }

    // For each intersection which has more than 2 stones, discard all except
    // the one with least error. Each discarded stone must be placed on another
    // intersection. Look at the other 3 possible intersections of the stone.
    // Put it on first empty one. If all intersections are non empty, for now
    // panic with `todo!`.

    let mut stones_to_reassign: Vec<Point> = Vec::default();
    for stones in intersection_stones.values_mut() {
        if stones.len() > 1 {
            stones.sort_by(|(_, a_e), (_, b_e)| a_e.partial_ord(*b_e));
            stones_to_reassign
                .extend(stones.drain(1..).map(|(p, _)| p).collect::<Vec<_>>());
        }
    }

    if !stones_to_reassign.is_empty() {
        unimplemented!(
            "Stones to reassign is not empty. You need to place them \
            somewhere before we can calculate the error."
        );

        // here's where stone_errors comes into play.
    }

    // Each intersection should have exactly one stone.
    let total_e = intersection_stones
        .values()
        .fold(0.0, |acc, vec| acc + vec[0].1);

    // Average error.
    total_e / stones.len() as f32
}

fn find_black_stones(
    image: &image::RgbImage,
) -> Option<(f32, Vec<BlackStone>)> {
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

    #[cfg(test)]
    debug::pixels(&black_pixels);

    let black_objects = find_black_objects(black_pixels);
    if black_objects.is_empty() {
        return None;
    }

    // We collect widths and heights of all objects. Majority of the objects are
    // going to be black stones because the user is taking a picture of a go
    // board in an endgame.
    let mut widths = Vec::with_capacity(black_objects.len());
    let mut heights = Vec::with_capacity(black_objects.len());
    for object in &black_objects {
        widths.push(object.width());
        heights.push(object.height());
    }

    // Since majority of objects are stones, we're going to grab the width from
    // the middle of the array. This is going to be our standard for width for
    // the rest of the objects.
    widths.sort();
    debug_assert_ne!(0, widths.len());
    let mean_width = widths[widths.len() / 2] as f32;

    // The same applies to the height.
    heights.sort();
    debug_assert_ne!(0, heights.len());
    let mean_height = heights[heights.len() / 2] as f32;

    // Filters out objects which are too big or too small to be a stone.
    let stones: Vec<_> = black_objects
        .into_iter()
        .filter(|object| {
            let w = object.width() as f32;
            let h = object.height() as f32;

            w < mean_width * 1.5
                && w > mean_width * 0.66
                && h < mean_height * 1.5
                && h > mean_height * 0.66
        })
        .collect();

    #[cfg(test)]
    debug::stones(width, height, &stones);

    Some(((mean_height + mean_width) / 2.0, stones))
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
            flood_fill(&mut object, &mut image);
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
        .into_iter()
        // Filters out some noise by removing tiny objects.
        .filter(|object| {
            let w = object.width() as f32;
            let h = object.height() as f32;
            w > MIN_STONE_SIZE
                && h > MIN_STONE_SIZE
                && w > h * 0.66
                && w < h * 1.5
        })
        .collect()
}

/// Recursively finds a single object within given image. It calls this function
/// for every new highlighted point.
fn flood_fill(object: &mut BlackStone, image: &mut BlackPixels) {
    let mut point_queue = vec![object.top_left];
    while let Some(point) = point_queue.pop() {
        object.push(point);
        // Adds currently iterated point to the object and set that point to no
        // highlighted.
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
                point_queue.push(Point::new(x as u32, y as u32));
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    pub use super::*;
    use std::cmp::Ordering;
    use std::fs;

    const ASSETS_DIR: &str = "assets/test";

    #[derive(Eq, PartialEq)]
    enum Intersection {
        BlackStone,
        WhiteStone,
        Empty,
    }

    struct BoardFile {
        inner: Vec<Vec<Intersection>>,
    }

    impl BoardFile {
        fn new(test_name: &str) -> Self {
            let file = fs::read(format!("{}/{}.txt", ASSETS_DIR, test_name))
                .expect("Cannot read test file");
            let file = String::from_utf8_lossy(&file);
            let inner = file
                .lines()
                .map(|l| {
                    l.chars()
                        .map(|c| match c {
                            'x' => Intersection::Empty,
                            '1' => Intersection::WhiteStone,
                            '0' => Intersection::BlackStone,
                            _ => panic!("Unrecognized board char"),
                        })
                        .collect()
                })
                .collect();

            Self { inner }
        }

        // Get positions of all black stones.
        fn black_stones(&self) -> Vec<(u8, u8)> {
            self.inner
                .iter()
                .enumerate()
                .map(|(y, row)| {
                    row.iter()
                        .enumerate()
                        .filter(|(_, intersection)| {
                            **intersection == Intersection::BlackStone
                        })
                        .map(|(x, _)| (x as u8, y as u8))
                        .collect::<Vec<(u8, u8)>>()
                })
                .flatten()
                .collect()
        }
    }

    // Note that actual number of stones differs from these numbers. These are
    // the counts which is my algorithm able to tell. We don't need to be
    // precise and we rather sacrifice in precise number of stones for less
    // false positives.
    const TEST_BLACK_STONE_COUNTS: &[(&str, usize)] = &[
        ("test1", 4),
        ("test2", 10),
        ("test3", 9),
        ("test4", 14),
        ("test5", 17),
        ("test6", 17),
        ("test7", 2),
        ("test8", 2),
    ];

    #[test]
    fn test_count_black_stones() {
        for (test, count) in TEST_BLACK_STONE_COUNTS {
            let image = image::open(format!("{}/{}.jpeg", ASSETS_DIR, test))
                .expect("Cannot open image");
            let (_, stones) = find_black_stones(&image.to_rgb())
                .expect("The test was expected to find some stones");
            assert_eq!(
                stones.len(),
                *count,
                "test file {} has a mismatched count of black stones",
                test,
            )
        }
    }

    // A name of test file and whether the algorithm is supposed to find any
    // stones in them. Note that there must be at least about 6 black stones for
    // the algorithm to work.
    const TEST_IMAGES: &[(&str, bool)] = &[
        ("test1", false),
        ("test2", true),
        ("test3", true),
        ("test4", true),
        ("test5", true),
        ("test6", true),
        ("test7", false),
        ("test8", false),
    ];

    #[test]
    fn test_place_black_stones_on_intersections() {
        for (test, should_yield_board) in TEST_IMAGES {
            println!("Running {}", test);
            let image = image::open(format!("{}/{}.jpeg", ASSETS_DIR, test))
                .expect("Cannot open image");
            let board = board_map(&image.to_rgb());
            if !should_yield_board {
                assert!(board.is_none());
                continue;
            }

            // Loads the file which has a text representation of the actual
            // board.
            let mut black_stones_on_board = BoardFile::new(test).black_stones();

            // Gets the black stones.
            let black_stones_found: Vec<_> = board
                .expect("Algorithm should be able to find stones")
                .keys()
                .copied()
                .collect();
            // Finds the lowest value of either x and y and that's going to
            // become the value 0 now. All other xs and ys are going to be
            // incremented by this value.
            let min_x = {
                let (min_x, _) = black_stones_found
                    .iter()
                    .min_by(|(xa, _), (xb, _)| xa.cmp(xb))
                    .unwrap();
                min_x.abs()
            };
            let mix_y = {
                let (_, min_y) = black_stones_found
                    .iter()
                    .min_by(|(_, ya), (_, yb)| ya.cmp(yb))
                    .unwrap();
                min_y.abs()
            };
            let mut black_stones_found: Vec<_> = black_stones_found
                .into_iter()
                .map(|(x, y)| ((x + min_x) as u8, (y + mix_y) as u8))
                .collect();

            // Sorts given slice of (x, y) in a way that the left most stones
            // are in the beginning.
            let sort_stones = |stones: &mut [(u8, u8)]| {
                stones.sort_by(|(xa, ya), (xb, yb)| match xa.cmp(xb) {
                    Ordering::Equal => ya.cmp(yb),
                    ordering => ordering,
                });
            };

            sort_stones(&mut black_stones_on_board);
            sort_stones(&mut black_stones_found);

            println!(
                "here are two two things: \n{:?}\n\n{:?}",
                black_stones_found, black_stones_on_board
            );

            panic!()
        }
    }
}
