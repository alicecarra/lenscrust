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

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    fn make_rgb_image(width: u32, height: u32, pixels: &[[u8; 3]]) -> DynamicImage {
        let mut img = RgbImage::new(width, height);
        for (i, p) in pixels.iter().enumerate() {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            img.put_pixel(x, y, Rgb(*p));
        }
        DynamicImage::ImageRgb8(img)
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
}
