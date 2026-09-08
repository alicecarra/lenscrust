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
            &[[1, 1, 1], [2, 2, 2], [3, 3, 3], [4, 4, 4], [5, 5, 5], [6, 6, 6]],
        );
        mirror_horizontal(&mut img);
        let expected = make_rgb_image(
            3,
            2,
            &[[3, 3, 3], [2, 2, 2], [1, 1, 1], [6, 6, 6], [5, 5, 5], [4, 4, 4]],
        );
        assert_eq!(img.to_rgba8(), expected.to_rgba8());
    }

    #[test]
    fn mirror_horizontal_twice_is_identity() {
        let original = make_rgb_image(
            3,
            2,
            &[[1, 1, 1], [2, 2, 2], [3, 3, 3], [4, 4, 4], [5, 5, 5], [6, 6, 6]],
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
            &[[1, 1, 1], [2, 2, 2], [3, 3, 3], [4, 4, 4], [5, 5, 5], [6, 6, 6]],
        );
        mirror_vertical(&mut img);
        let expected = make_rgb_image(
            2,
            3,
            &[[5, 5, 5], [6, 6, 6], [3, 3, 3], [4, 4, 4], [1, 1, 1], [2, 2, 2]],
        );
        assert_eq!(img.to_rgba8(), expected.to_rgba8());
    }

    #[test]
    fn mirror_vertical_twice_is_identity() {
        let original = make_rgb_image(
            2,
            3,
            &[[1, 1, 1], [2, 2, 2], [3, 3, 3], [4, 4, 4], [5, 5, 5], [6, 6, 6]],
        );
        let mut img = original.clone();
        mirror_vertical(&mut img);
        mirror_vertical(&mut img);
        assert_eq!(img.to_rgba8(), original.to_rgba8());
    }
}
