use image::{DynamicImage, GenericImage, GenericImageView};

pub(super) fn mirror_horizontal(image: &mut DynamicImage) {
    let (width, height) = image.dimensions();

    for y in 0..height {
        for x in 0..width / 2 {
            let left = image.get_pixel(x, y);
            let right = image.get_pixel(width - 1 - x, y);
            image.put_pixel(x, y, right);
            image.put_pixel(width - 1 - x, y, left);
        }
    }
}

pub(super) fn mirror_vertical(image: &mut DynamicImage) {
    let (width, height) = image.dimensions();

    for y in 0..height / 2 {
        for x in 0..width {
            let top = image.get_pixel(x, y);
            let bottom = image.get_pixel(x, height - 1 - y);
            image.put_pixel(x, y, bottom);
            image.put_pixel(x, height - 1 - y, top);
        }
    }
}

// TODO: rotate operations with arrays, not pixel at pixel
pub(super) fn rotate_90_clockwise(image: &mut DynamicImage) {
    let (old_width, old_height) = image.dimensions();
    let source_image = image.clone();

    let mut rotated_image = DynamicImage::new(old_height, old_width, source_image.color());
    for new_y in 0..old_width {
        for new_x in 0..old_height {
            let old_x = new_y;
            let old_y = old_height - 1 - new_x;
            let pixel = source_image.get_pixel(old_x, old_y);
            rotated_image.put_pixel(new_x, new_y, pixel);
        }
    }

    *image = rotated_image;
}

pub(super) fn rotate_90_counterclockwise(image: &mut DynamicImage) {
    let (old_width, old_height) = image.dimensions();
    let source_image = image.clone();

    let mut rotated_image = DynamicImage::new(old_height, old_width, source_image.color());
    for new_y in 0..old_width {
        for new_x in 0..old_height {
            let old_x = old_width - 1 - new_y;
            let old_y = new_x;
            let pixel = source_image.get_pixel(old_x, old_y);
            rotated_image.put_pixel(new_x, new_y, pixel);
        }
    }

    *image = rotated_image;
}

pub(super) fn zoom_in(image: &mut DynamicImage) {
    let (old_width, old_height) = image.dimensions();
    if old_width == 0 || old_height == 0 {
        return;
    }

    let new_width = 2 * old_width - 1;
    let new_height = 2 * old_height - 1;

    let source_image = image.clone();
    let mut zoomed_image = DynamicImage::new(new_width, new_height, source_image.color());

    // place original pixels at the even coordinates leaving a blank row/column between each pair
    for old_y in 0..old_height {
        for old_x in 0..old_width {
            let pixel = source_image.get_pixel(old_x, old_y);
            zoomed_image.put_pixel(old_x * 2, old_y * 2, pixel);
        }
    }

    // interpolate every original row filling the gaps between horizontally adjacent original columns.
    for old_y in 0..old_height {
        let y = old_y * 2;
        for old_x in 0..old_width.saturating_sub(1) {
            let left = zoomed_image.get_pixel(old_x * 2, y);
            let right = zoomed_image.get_pixel(old_x * 2 + 2, y);
            zoomed_image.put_pixel(old_x * 2 + 1, y, average_pixel(left, right));
        }
    }

    // interpolate along every column filling the still-blank rows using the rows above and below
    for new_x in 0..new_width {
        for old_y in 0..old_height.saturating_sub(1) {
            let top = zoomed_image.get_pixel(new_x, old_y * 2);
            let bottom = zoomed_image.get_pixel(new_x, old_y * 2 + 2);
            zoomed_image.put_pixel(new_x, old_y * 2 + 1, average_pixel(top, bottom));
        }
    }

    *image = zoomed_image;
}

fn average_pixel(first: image::Rgba<u8>, second: image::Rgba<u8>) -> image::Rgba<u8> {
    let mut averaged = first;
    for channel_index in 0..3 {
        let value = (first[channel_index] as f64 + second[channel_index] as f64) / 2.0;
        averaged[channel_index] = value.round() as u8;
    }
    averaged
}

// shrinks the image by averaging the pixels under a x per y window slid over it without overlap.
// Windows that extend past the edge are averaged using only the pixels inside it.
pub(super) fn zoom_out(image: &mut DynamicImage, factor_x: f64, factor_y: f64) {
    let (old_width, old_height) = image.dimensions();
    let new_width = (old_width as f64 / factor_x).ceil() as u32;
    let new_height = (old_height as f64 / factor_y).ceil() as u32;

    let source_image = image.clone();
    let mut zoomed_image = DynamicImage::new(new_width, new_height, source_image.color());

    for new_y in 0..new_height {
        let source_start_y = (new_y as f64 * factor_y).floor() as u32;
        let source_end_y = (((new_y + 1) as f64 * factor_y).floor() as u32).min(old_height);

        for new_x in 0..new_width {
            let source_start_x = (new_x as f64 * factor_x).floor() as u32;
            let source_end_x = (((new_x + 1) as f64 * factor_x).floor() as u32).min(old_width);

            let mut channel_sums = [0.0f64; 3];
            let mut pixel_count = 0u32;
            for source_y in source_start_y..source_end_y {
                for source_x in source_start_x..source_end_x {
                    let pixel = source_image.get_pixel(source_x, source_y);
                    for channel_index in 0..3 {
                        channel_sums[channel_index] += pixel[channel_index] as f64;
                    }
                    pixel_count += 1;
                }
            }

            let mut averaged_pixel = source_image.get_pixel(source_start_x, source_start_y);
            for channel_index in 0..3 {
                averaged_pixel[channel_index] =
                    (channel_sums[channel_index] / pixel_count as f64).round() as u8;
            }
            zoomed_image.put_pixel(new_x, new_y, averaged_pixel);
        }
    }

    *image = zoomed_image;
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma, Rgb, RgbImage};

    fn make_rgb_image(width: u32, height: u32, pixels: &[[u8; 3]]) -> DynamicImage {
        let mut img = RgbImage::new(width, height);
        for (i, p) in pixels.iter().enumerate() {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            img.put_pixel(x, y, Rgb(*p));
        }
        DynamicImage::ImageRgb8(img)
    }

    fn make_gray_image(width: u32, height: u32, pixels: &[u8]) -> DynamicImage {
        let mut img = GrayImage::new(width, height);
        for (i, &p) in pixels.iter().enumerate() {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            img.put_pixel(x, y, Luma([p]));
        }
        DynamicImage::ImageLuma8(img)
    }

    #[test]
    fn mirror_horizontal_reverses_rows() {
        let mut img = make_rgb_image(
            3,
            2,
            &[
                [1, 1, 1],
                [2, 2, 2],
                [3, 3, 3],
                [4, 4, 4],
                [5, 5, 5],
                [6, 6, 6],
            ],
        );
        mirror_horizontal(&mut img);
        let expected = make_rgb_image(
            3,
            2,
            &[
                [3, 3, 3],
                [2, 2, 2],
                [1, 1, 1],
                [6, 6, 6],
                [5, 5, 5],
                [4, 4, 4],
            ],
        );
        assert_eq!(img.to_rgba8(), expected.to_rgba8());
    }

    #[test]
    fn mirror_horizontal_twice_is_identity() {
        let original = make_rgb_image(
            3,
            2,
            &[
                [1, 1, 1],
                [2, 2, 2],
                [3, 3, 3],
                [4, 4, 4],
                [5, 5, 5],
                [6, 6, 6],
            ],
        );
        let mut img = original.clone();
        mirror_horizontal(&mut img);
        mirror_horizontal(&mut img);
        assert_eq!(img.to_rgba8(), original.to_rgba8());
    }

    #[test]
    fn mirror_vertical_reverses_columns() {
        let mut img = make_rgb_image(
            2,
            3,
            &[
                [1, 1, 1],
                [2, 2, 2],
                [3, 3, 3],
                [4, 4, 4],
                [5, 5, 5],
                [6, 6, 6],
            ],
        );
        mirror_vertical(&mut img);
        let expected = make_rgb_image(
            2,
            3,
            &[
                [5, 5, 5],
                [6, 6, 6],
                [3, 3, 3],
                [4, 4, 4],
                [1, 1, 1],
                [2, 2, 2],
            ],
        );
        assert_eq!(img.to_rgba8(), expected.to_rgba8());
    }

    #[test]
    fn mirror_vertical_twice_is_identity() {
        let original = make_rgb_image(
            2,
            3,
            &[
                [1, 1, 1],
                [2, 2, 2],
                [3, 3, 3],
                [4, 4, 4],
                [5, 5, 5],
                [6, 6, 6],
            ],
        );
        let mut img = original.clone();
        mirror_vertical(&mut img);
        mirror_vertical(&mut img);
        assert_eq!(img.to_rgba8(), original.to_rgba8());
    }

    // 2-wide x 3-tall grid used to hand-verify rotation direction:
    //   A B
    //   C D
    //   E F
    fn make_rotation_test_grid() -> DynamicImage {
        make_rgb_image(
            2,
            3,
            &[
                [1, 1, 1],
                [2, 2, 2],
                [3, 3, 3],
                [4, 4, 4],
                [5, 5, 5],
                [6, 6, 6],
            ],
        )
    }

    #[test]
    fn rotate_90_clockwise_transforms_dimensions_and_pixels() {
        let mut img = make_rotation_test_grid();
        rotate_90_clockwise(&mut img);

        assert_eq!((img.width(), img.height()), (3, 2));
        // clockwise: leftmost column (A, C, E), read bottom-to-top, becomes
        // the top row; rightmost column (B, D, F) becomes the bottom row.
        //   E C A
        //   F D B
        let expected = make_rgb_image(
            3,
            2,
            &[
                [5, 5, 5],
                [3, 3, 3],
                [1, 1, 1],
                [6, 6, 6],
                [4, 4, 4],
                [2, 2, 2],
            ],
        );
        assert_eq!(img.to_rgba8(), expected.to_rgba8());
    }

    #[test]
    fn rotate_90_counterclockwise_transforms_dimensions_and_pixels() {
        let mut img = make_rotation_test_grid();
        rotate_90_counterclockwise(&mut img);

        assert_eq!((img.width(), img.height()), (3, 2));
        // counter-clockwise: rightmost column (B, D, F), read top-to-bottom,
        // becomes the top row; leftmost column (A, C, E) becomes the bottom row.
        //   B D F
        //   A C E
        let expected = make_rgb_image(
            3,
            2,
            &[
                [2, 2, 2],
                [4, 4, 4],
                [6, 6, 6],
                [1, 1, 1],
                [3, 3, 3],
                [5, 5, 5],
            ],
        );
        assert_eq!(img.to_rgba8(), expected.to_rgba8());
    }

    #[test]
    fn rotating_clockwise_four_times_is_identity() {
        let original = make_rotation_test_grid();
        let mut img = original.clone();
        for _ in 0..4 {
            rotate_90_clockwise(&mut img);
        }
        assert_eq!(
            (img.width(), img.height()),
            (original.width(), original.height())
        );
        assert_eq!(img.to_rgba8(), original.to_rgba8());
    }

    #[test]
    fn rotating_clockwise_then_counterclockwise_is_identity() {
        let original = make_rotation_test_grid();
        let mut img = original.clone();
        rotate_90_clockwise(&mut img);
        rotate_90_counterclockwise(&mut img);
        assert_eq!(
            (img.width(), img.height()),
            (original.width(), original.height())
        );
        assert_eq!(img.to_rgba8(), original.to_rgba8());
    }

    #[test]
    fn zoom_out_averages_windows_and_handles_a_partial_last_window() {
        // width 5, factor 2.0 -> windows [0,2), [2,4), [4,5) (partial)
        let mut img = make_gray_image(5, 1, &[0, 10, 20, 30, 40]);
        zoom_out(&mut img, 2.0, 1.0);

        assert_eq!((img.width(), img.height()), (3, 1));
        let gray_image = img.as_luma8().unwrap();
        assert_eq!(gray_image.get_pixel(0, 0)[0], 5); // avg(0, 10)
        assert_eq!(gray_image.get_pixel(1, 0)[0], 25); // avg(20, 30)
        assert_eq!(gray_image.get_pixel(2, 0)[0], 40); // avg(40) alone
    }

    #[test]
    fn zoom_out_averages_each_color_channel_independently() {
        let mut img = make_rgb_image(2, 1, &[[10, 20, 30], [30, 40, 50]]);
        zoom_out(&mut img, 2.0, 1.0);

        assert_eq!((img.width(), img.height()), (1, 1));
        let rgb_image = img.to_rgb8();
        assert_eq!(rgb_image.get_pixel(0, 0).0, [20, 30, 40]);
    }

    #[test]
    fn zoom_out_supports_different_factors_per_axis() {
        let mut img = make_gray_image(
            6,
            4,
            &[
                0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, //
            ],
        );
        zoom_out(&mut img, 3.0, 2.0);
        assert_eq!((img.width(), img.height()), (2, 2));
    }

    #[test]
    fn zoom_in_interpolates_a_linear_ramp_in_both_directions() {
        // a linear ramp interpolates back to itself, making every value
        // easy to verify by hand: value(x, y) = 5*x + 15*y
        let mut img = make_gray_image(3, 3, &[0, 10, 20, 30, 40, 50, 60, 70, 80]);
        zoom_in(&mut img);

        assert_eq!((img.width(), img.height()), (5, 5));
        let expected = make_gray_image(
            5,
            5,
            &[
                0, 5, 10, 15, 20, //
                15, 20, 25, 30, 35, //
                30, 35, 40, 45, 50, //
                45, 50, 55, 60, 65, //
                60, 65, 70, 75, 80, //
            ],
        );
        assert_eq!(img.to_rgba8(), expected.to_rgba8());
    }

    #[test]
    fn zoom_in_interpolates_each_color_channel_independently() {
        let mut img = make_rgb_image(2, 1, &[[10, 20, 30], [50, 60, 70]]);
        zoom_in(&mut img);

        assert_eq!((img.width(), img.height()), (3, 1));
        let expected = make_rgb_image(3, 1, &[[10, 20, 30], [30, 40, 50], [50, 60, 70]]);
        assert_eq!(img.to_rgba8(), expected.to_rgba8());
    }

    #[test]
    fn zoom_in_output_dimensions_match_the_two_minus_one_rule() {
        let mut img = make_gray_image(4, 2, &[0; 8]);
        zoom_in(&mut img);
        assert_eq!((img.width(), img.height()), (7, 3));
    }
}
