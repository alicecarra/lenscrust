use image::{DynamicImage, GenericImageView};

pub(super) fn luminance(image: &mut DynamicImage) {
    match image {
        // already on grayscale, do nothing
        DynamicImage::ImageLuma8(..)
        | DynamicImage::ImageLumaA8(..)
        | DynamicImage::ImageLuma16(..)
        | DynamicImage::ImageLumaA16(..) => return,
        _ => {
            let mut gray = image::GrayImage::new(image.width(), image.height());
            for x in 0..image.width() {
                for y in 0..image.height() {
                    let pixel = image.get_pixel(x, y);
                    let gray_pixel = (pixel[0] as f64 * 0.299
                        + pixel[1] as f64 * 0.587
                        + pixel[2] as f64 * 0.114)
                        .round() as u8;
                    gray.put_pixel(x, y, image::Luma([gray_pixel]));
                }
            }
            *image = DynamicImage::ImageLuma8(gray);
        }
    }
}

pub(super) fn quantize(image: &mut DynamicImage, levels: u16) {
    let luma_image = image.as_mut_luma8().expect("Image not in luma!!!");

    // maybe use a more idiomatic way?
    let mut min_tone = 255;
    let mut max_tone = 0;
    for pixel in luma_image.pixels() {
        let pixel = pixel[0];
        if pixel > max_tone {
            max_tone = pixel;
        }
        if pixel < min_tone {
            min_tone = pixel;
        }
    }
    // sanity check
    assert!(max_tone >= min_tone);

    let tone_range_size = (max_tone - min_tone) as u16 + 1;

    if levels >= tone_range_size {
        // maybe give feedback to user?
        return;
    }

    let bin_width = tone_range_size as f64 / levels as f64;

    for pixel in luma_image.pixels_mut() {
        let t_orig = pixel[0] as f64;
        let bin_index = (((t_orig - (min_tone as f64 - 0.5)) / bin_width).floor() as i64)
            .clamp(0, levels as i64 - 1);
        let quantized_tone = (min_tone as f64 - 0.5) + (bin_index as f64 + 0.5) * bin_width;
        pixel[0] = quantized_tone.round() as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma, Rgb, RgbImage};
    use std::collections::HashSet;

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
    fn luminance_applies_bt601_formula() {
        let mut img = make_rgb_image(
            2,
            2,
            &[[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]],
        );
        luminance(&mut img);
        let gray = img.as_luma8().expect("image should be grayscale");
        assert_eq!(gray.get_pixel(0, 0)[0], 76);
        assert_eq!(gray.get_pixel(1, 0)[0], 150);
        assert_eq!(gray.get_pixel(0, 1)[0], 29);
        assert_eq!(gray.get_pixel(1, 1)[0], 255);
    }

    #[test]
    fn luminance_is_noop_on_already_grayscale() {
        let mut img = make_gray_image(2, 1, &[10, 20]);
        luminance(&mut img);
        let gray = img.as_luma8().unwrap();
        assert_eq!(gray.get_pixel(0, 0)[0], 10);
        assert_eq!(gray.get_pixel(1, 0)[0], 20);
    }

    #[test]
    fn quantize_noop_when_levels_cover_full_range() {
        let mut img = make_gray_image(2, 1, &[20, 10]);
        quantize(&mut img, 11);
        let gray = img.as_luma8().unwrap();
        assert_eq!(gray.get_pixel(0, 0)[0], 20);
        assert_eq!(gray.get_pixel(1, 0)[0], 10);
    }

    #[test]
    fn quantize_produces_at_most_requested_levels() {
        let pixels: Vec<u8> = (0..16).map(|x| (x * 17) as u8).collect();
        let mut img = make_gray_image(16, 1, &pixels);
        quantize(&mut img, 4);
        let gray = img.as_luma8().unwrap();
        let distinct: HashSet<u8> = gray.pixels().map(|p| p[0]).collect();
        assert!(distinct.len() <= 4);
    }

    #[test]
    fn quantize_tracks_min_even_when_first_pixel_is_the_minimum() {
        let mut img = make_gray_image(2, 1, &[10, 200]);
        quantize(&mut img, 200); // tone_range_size = 191, levels(200) >= it -> no-op
        let gray = img.as_luma8().unwrap();
        assert_eq!(gray.get_pixel(0, 0)[0], 10);
        assert_eq!(gray.get_pixel(1, 0)[0], 200);
    }
}
