# Board parser
Given an image, this library finds a go board and identifies a position of each
stone in the lattice.

![19 by 19 go board](assets/docs/19x19_board.jpeg)

There are several sizes of boards. Most commons are **9x9**, **13x13** and
**19x19**. In the example above, the board is 19x19. What matters on a go board
are the lines and their intersections. We can annotate these intersections from
top left corner to the bottom right by incrementing numbers. For example, a
fourth intersection on the second row on a 19x19 board is annotated as
19 + 19 + 4 = **42**. An alternative annotation would be `row:column`, e. g.
`2:4`.

The stones are places on the intersections. There are two players in go. A move
is placement of a player's coloured stone on the go board, i. e. one player puts
down only black stones and the other only white stones. Most commonly people
have their stoned coloured in black and white.

![Black and white stones](assets/docs/stones.jpeg)

## Assumptions
- There are only 9x9, 13x13 and 19x19 boards.
- The stones are black and white.
- Each intersection is equally spaced from the others.
- Good light conditions.
- The whole board is shown on the camera.
- The board is not empty.

## Requirements
We need to analyse pictures and find boards in them. If a board is found, we
need to be able to say which pixels roughly corresponds to top left and bottom
right corners of the board. For each intersection on the board, we need to tell
whether there's a white stone, a black stone or neither.

The algorithm needs to be able to output this information for boards which are
arbitrarily rotated.

![Rotated board](assets/docs/rotated_board.jpeg)

The algorithm needs to be able to ignore other accessories that might be on the
photo but is not related to the board.

![Board with accessories](assets/docs/board_accessories.jpeg)

There needs to be some give in the angle under which the photo is taken. Ideally
the board would be viewed from angle 90deg.

![Board from low angle](assets/docs/board_low_angle.jpeg)

## Testing
During development we use pictures we took of the go board and evaluate the
algorithm against them. To bench mark an algorithm, we can take a picture of an
empty board from several angles and generate many different board
constellations. We can then use this data set to validate that the algorithm
yields satisfying results.

## Approaches
We focus on the fact that the stones are going to be black and white. Therefore
we can rule out pixels which are coloured. We now have a picture which contains
black stones, white stones, board lines and noise. To rule out noise, we can
leverage the shape of the stones. Depending on the point of view, they're going
to be either elliptical or circular. We attempt to find objects which are
approximated by the equation of ellipsis.

```
(x^2)   (y^2)
----- + ----- = 1
  a       b
```

Once we have found some objects which are identified by this equation and their
`a`, `b` and size match, we can calculate the distances between them to identify
more stones. Eventually, we will have identified enough stones to be able to say
what's the spacing between the intersections.

First attempt was to find both black and white stones by iterating over all
pixels and picking those whose RGB values are below and above some threshold.
For black stones, this works well. Black seems much easier to capture on the
photos. To capture white, the threshold needed to around 128 to at least capture
the outline of a white stone and even that was not enough for some lighting
conditions.

![Black stones are easy to find, white are not really white](assets/docs/contrast_black_white_stones.png)

However, being able to spot a black stone so easily is a win. To map the board,
we don't need white stones. Since the purpose is to count score, we can be sure
that there're going to be many black stones. Next step is therefore to be able
to identify all black stones. Then we can pick two arbitrary black stones to
calculate the distance between them. The distance determines the spacing of the
intersections. We assume that the board is not rotated for now to make progress
on the algorithm. We iterate row by row, getting views into the original image.
We already know the black stones. Each intersection we visit, we need to assert
whether it's an empty territory or a white stone.

The problem has boiled down to categorizing a slice of an image into one of two
categories: a white stone or an intersection. An intuition guides here and says
that using machine learning is unnecessary. We can look for a `+` shape or lack
thereof, or we can use the equation of ellipsis.

After filtering out noise and items which were not fit as stones (were too big
or small in comparison with the mean object on the photo, which we assume is
a stone), we have a highlight of black stones on the photo. The algorithm at
this point is quite sure that most of the highlighted items are stones. However
there might be stones outside of the board which are still visible on the photo.
We take care of this later on when we calculate a size of the board.

**Input**

![Illustrative input image](assets/docs/contrast_input.jpeg)

**Black pixels**

![All the black pixels](assets/docs/contrast_black_pixels.jpeg)

**Black stones**

![All the black pixels](assets/docs/contrast_black_stones.jpeg)

As you can see, we missed some black stones on the board. Since false positives
are worse than missing a few stones, this is acceptable drawback.

To find an average distance between two adjacent intersections in the board, we
walk an array of center points of each stone and pair it with in an arbitrary
way with another stone. The first implementation pairs adjacent stones in
the array that's returned by the function mentioned above plus first stone with
the stone in the half of the array, second stone with the stone in the half of
the array + 1 and so on. We filter out distances which are smaller than mean
stone size, because those represent stones on the same row/column (the
difference in `x` or `y` between those stones is about 0). All other distances
are collected into an array. Next we find a number which divides all those
distances with the least error. An error is the fractal part of a number after
division `distance / current_guess`. We minimize the error until the change in
the current guess is small enough (less than 1px).

```
stone_size = 42.5
sampled_distances = [
  60, 164, 160, 47, 113, 56, 175, 55, 57, 93, 171, 218, 142, 330, 155, 43, 386
]
divisor_with_least_err ~= 53.4
```

We find the center of the board by calculating an average over `x` and `y` of
all stones. We sort stones based on their distance to the center point. One by
one we start plotting the stones on a map from which later on we build the board
state. We filter out stones about which we are unsure to which intersection they
belong.

![Finding lines](assets/docs/contrast_find_lines.png)

The stones which are depicted here are stone about which the parser is
confident on which intersection to put them. The center stone is highlighted
with stronger lines.

While this solution worked well when the board was aligned with the x and y
axes, it broke quite if the board was slightly rotated. We pursued this
algorithm to have some basic benchmarks to improve on. However it doesn't yield
good enough results to make it worth it to continue developing it.

![Average distance is not performing well](assets/docs/average_distance_error.png)

We will keep the contrast finding black stones, however we discard the average
distance algorithm. Instead, we develop an algorithm which tries to find a
transformation that has the least error against measured centers of the black
stones when applied to a lattice. We start with a linear transformation for
simplicity. Although linear transformation won't address the inclination, it
will help with rotation around the center of the board. Presently, that also
seems like bigger issue because the size of the board won't change too much with
different inclination unless the user is taking a close up picture. But a close
up picture most likely won't capture the whole board.

We have four parameters which we want to fiddle around with to minimize an
error. Two are in the matrix for linear transformation of our lattice. Third and
forth are the `x` and `y` positions of the (0, 0) coordinate for the lattice
within the picture.

![Linear transformation a & b, -b & a](assets/docs/linear_transformation.png)

We only need two parameters to the LT because the go board is made of squares
which are rotated around a center point (remember that we ignore inclination for
now).

There are multiple ways for minimizing the error between intersections on the
lattice and the centers of the black stones.

We could in each iteration calculate change in error for `da`, `db`, `dx` and
`dy` and choose to change the variable which minimizes the error the most. This
means there's a lot of wasted effort.

We could have cycles where first we minimize error by applying `da`, in next
cycle `db` and so on.

Generally it's also difficult to say in which direction should the
change happen. Also the size of the change is not clear.

<!-- Invisible List of References -->
[linear-transformation]: http://www.sciweavers.org/free-online-latex-equation-editor
[latex-editor]: http://www.sciweavers.org/free-online-latex-equation-editor
[geogebra-linear-transformation-visualization]: https://www.geogebra.org/m/YCZa8TAH
