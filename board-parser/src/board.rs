use crate::num_ext::*;
use std::collections::HashMap;

const BLACK_THRESHOLD: u8 = 30;
const GRAYNESS_LIMIT: u8 = 8;
const MIN_STONE_SIZE: u32 = 5;
const MIN_BLACK_STONES_ON_BOARD: usize = 6;

pub type BoardMap = HashMap<(i8, i8), Point>;
type BlackPixels = Vec<Vec<bool>>;

#[derive(Debug)]
struct BlackStone {
    /// The left most point with the lowest y and x value.
    top_left: Point,
    /// The right most point with the highest y and x value.
    bottom_right: Point,
}

#[derive(Copy, Clone, Debug)]
pub struct Point {
    x: u32,
    y: u32,
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
    let (stone_size, stones) = find_black_stones(image)?;
    // From now on we're only concerned about the center points.
    let stones: Vec<_> =
        stones.into_iter().map(|stone| stone.center()).collect();

    // There must be at least a few black stones on the board.
    if stones.len() < MIN_BLACK_STONES_ON_BOARD {
        return None;
    }

    // We will sample distances between stones.
    // We cannot really estimate exactly how many distances are there going to
    // be because many stones can be on the same row/column and we avoid adding
    // distances which are lower than stone size. However since we add 2
    // distances for each stone and then another 2 distances for each stone in
    // the first half of the array, we can try to approximate the capacity.
    // Then we also put distances between the first and last, second and second
    // to last, etc.
    let mut sampled_distances = Vec::with_capacity(6 * stones.len());

    // Adds distances between 2 stones into the array.
    let mut add_distances = |stone_a: Point, stone_b: Point| {
        let x_diff = stone_a.x.diff(stone_b.x) as f32;
        let y_diff = stone_a.y.diff(stone_b.y) as f32;

        // We don't want distances between stones if they are on the same row,
        // as then x would be around 0.
        if x_diff > stone_size {
            sampled_distances.push(x_diff);
        }

        // We don't want distances between stones if they are on the same column
        // as then y would be around 0.
        if y_diff > stone_size {
            sampled_distances.push(y_diff);
        }
    };

    let stones_half_count = stones.len() / 2;
    for (i, stone) in stones[0..stones.len() - 2].iter().enumerate() {
        let stone = *stone;

        // These are going to be most likely stones which are adjacent on rows.
        let next_stone = stones[i + 1];
        add_distances(stone, next_stone);

        // These stones are going to have the greatest distances. This is going
        // to be important as a heuristics later on. If there are enough
        // distances which are larger than 13 intersections apart, we say that
        // the board is 19x19.
        let far_away_stone = stones[stones.len() - i - 1];
        add_distances(stone, far_away_stone);

        // The first stone gets pair with a stone in the middle of the array,
        // second stone with the stone in the middle of the array + 1, and so on
        // until we reach the middle.
        if i < stones_half_count {
            let stone_in_other_half = stones[i + stones_half_count];
            add_distances(stone, stone_in_other_half);
        }
    }

    // How far away are two stones which are places next to each other.
    // Starts as a stone size and finds a more appropriate value.
    let adjacent_intersection_distance =
        average_distance_between_adjacent_intersections(
            &sampled_distances,
            stone_size,
        );

    println!(
        "Two stones are neighbors approx by {} pixels.",
        adjacent_intersection_distance
    );

    let mut total_x = 0;
    let mut total_y = 0;
    for stone in &stones {
        total_x += stone.x;
        total_y += stone.y;
    }
    let board_center_point = Point::new(
        total_x / stones.len() as u32,
        total_y / stones.len() as u32,
    );

    let mut stones_with_distance_to_center: Vec<_> = stones
        .iter()
        .map(|stone| {
            let diff_x = stone.x.diff(board_center_point.x);
            let diff_y = stone.y.diff(board_center_point.y);
            (diff_x * diff_x + diff_y * diff_y, stone)
        })
        .collect();
    stones_with_distance_to_center.sort_by(|(a, _), (b, _)| a.cmp(b));
    let (_, board_center_point) = stones_with_distance_to_center[0];
    let board_center_point = *board_center_point;
    let mut board: HashMap<(i8, i8), Point> = HashMap::with_capacity(19);
    board.insert((0, 0), board_center_point);
    for (_, stone) in stones_with_distance_to_center {
        let stone = *stone;
        let diff_x = stone.x.diff(board_center_point.x) as f32;
        let diff_y = stone.y.diff(board_center_point.y) as f32;
        let div_x = diff_x / adjacent_intersection_distance;
        let div_y = diff_y / adjacent_intersection_distance;

        // If the distance between the center point is more than the possible
        // board size, we can rule it out immediately.
        if div_x > 20.0 || div_y > 20.0 {
            continue;
        }

        // For example an error for numbers 7.6 and 5.2 is .4 + .2 = 0.6
        // We get abs value after sub from .5. The lower the error the higher
        // this value.
        // |.5 - .6| = .1
        // |.5 - .2| = .3
        // 1 - .1 - .3 = .6
        let e = 1.0 - (0.5 - div_y.fract()).abs() - (0.5 - div_x.fract()).abs();
        if e > 0.5 {
            // This stone is positioned weirdly. Needs more heuristics based on
            // other points in the hash map. Easier is to skip it and rely on
            // sieves later on which are aware of the fact that some black
            // stones were not found during this phase.
            continue;
        }

        // If the stone is higher than center point, the row is negative.
        let row = if div_y == 0.0 {
            0
        } else if stone.y > board_center_point.y {
            div_y.round() as i8
        } else {
            -div_y.round() as i8
        };

        // If the stone is more to left than center point, the col is negative.
        let column = if div_x == 0.0 {
            0
        } else if stone.x > board_center_point.x {
            div_x.round() as i8
        } else {
            -div_x.round() as i8
        };

        board.insert((column, row), stone);
    }

    #[cfg(test)]
    debug_board(
        image.width(),
        image.height(),
        adjacent_intersection_distance,
        board_center_point,
        &board,
    );

    Some(board)
}

// We're going to try to find a number which divide distances into units.
// TODO: Add a cap on how far away stones can be, which is by 19 *
// `neighbor_stone_distance`.
// TODO: Make sure that the `neighbor_stone_distance` never goes below
// `stone_size`.
// TODO: Have a hard limit on number of iterations.
fn average_distance_between_adjacent_intersections(
    sampled_distances: &[f32],
    mut neighbor_stone_distance: f32,
) -> f32 {
    loop {
        let mut total_change = 0.0;
        for d in sampled_distances {
            let div = d / neighbor_stone_distance;
            let closest_int = div.round().max(1.0);
            total_change += d / closest_int - neighbor_stone_distance;
        }
        let average_change = total_change / sampled_distances.len() as f32;
        neighbor_stone_distance += average_change;
        if average_change.abs() < 1.0 {
            break;
        }
    }

    neighbor_stone_distance
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

    // #[cfg(test)]
    // debug_pixels(&black_pixels);

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
    debug_stones(width, height, &stones);

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
            object.width() > MIN_STONE_SIZE || object.height() > MIN_STONE_SIZE
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
mod tests {
    use super::*;

    const ASSETS_DIR: &str = "assets/test";

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

    // TODO: Test the methods based on the input text files.
}

#[cfg(test)]
#[allow(dead_code)]
fn debug_pixels(black_pixels: &BlackPixels) {
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
        .save(format!("pixels-v{}.jpeg", env!("CARGO_PKG_VERSION")))
        .expect("Cannot save image");
}

#[cfg(test)]
#[allow(dead_code)]
fn debug_stones(w: u32, h: u32, stones: &[BlackStone]) {
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
        .save(format!("stones-v{}.jpeg", env!("CARGO_PKG_VERSION")))
        .expect("Cannot save image");
}

#[cfg(test)]
#[allow(dead_code)]
fn debug_board(
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
        .save(format!("board-v{}.jpeg", env!("CARGO_PKG_VERSION")))
        .expect("Cannot save image");
}
