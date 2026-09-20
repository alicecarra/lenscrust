use image::DynamicImage;

// TODO: colored histogram
pub fn compute_histogram(image: &DynamicImage) -> [u32; 256] {
    let mut grayscale_image = image.clone();
    super::point::luminance(&mut grayscale_image);
    let luma_image = grayscale_image.as_luma8().expect("Image not in luma!!!");

    let mut histogram = [0u32; 256];
    for pixel in luma_image.pixels() {
        histogram[pixel[0] as usize] += 1;
    }
    histogram
}

pub(super) fn equalize_histogram(image: &mut DynamicImage) {
    super::point::luminance(image);

    let histogram = compute_histogram(image);
    let total_pixel_count: u32 = histogram.iter().sum();
    if total_pixel_count == 0 {
        return;
    }

    let mut cumulative_count = 0u32;
    let mut tone_mapping = [0u8; 256];
    for tone in 0..256 {
        cumulative_count += histogram[tone];
        let equalized_tone = (cumulative_count as f64 * 255.0 / total_pixel_count as f64).round();
        tone_mapping[tone] = equalized_tone as u8;
    }

    let luma_image = image.as_mut_luma8().expect("Image not in luma!!!");
    for pixel in luma_image.pixels_mut() {
        pixel[0] = tone_mapping[pixel[0] as usize];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma, Rgb, RgbImage};

    fn make_gray_image(width: u32, height: u32, pixels: &[u8]) -> DynamicImage {
        let mut image = GrayImage::new(width, height);
        for (index, &pixel_value) in pixels.iter().enumerate() {
            let x = (index as u32) % width;
            let y = (index as u32) / width;
            image.put_pixel(x, y, Luma([pixel_value]));
        }
        DynamicImage::ImageLuma8(image)
    }

    fn make_rgb_image(width: u32, height: u32, pixels: &[[u8; 3]]) -> DynamicImage {
        let mut image = RgbImage::new(width, height);
        for (index, pixel_value) in pixels.iter().enumerate() {
            let x = (index as u32) % width;
            let y = (index as u32) / width;
            image.put_pixel(x, y, Rgb(*pixel_value));
        }
        DynamicImage::ImageRgb8(image)
    }

    #[test]
    fn compute_histogram_counts_each_tone() {
        let image = make_gray_image(4, 1, &[10, 10, 200, 10]);
        let histogram = compute_histogram(&image);
        assert_eq!(histogram[10], 3);
        assert_eq!(histogram[200], 1);
        assert_eq!(histogram.iter().sum::<u32>(), 4);
    }

    #[test]
    fn compute_histogram_converts_color_images_to_grayscale_first() {
        // luminance(255, 0, 0) = round(0.299 * 255) = 76
        let image = make_rgb_image(1, 1, &[[255, 0, 0]]);
        let histogram = compute_histogram(&image);
        assert_eq!(histogram[76], 1);
        assert_eq!(histogram.iter().sum::<u32>(), 1);
    }

    #[test]
    fn equalize_histogram_remaps_tones_by_cumulative_distribution() {
        // histogram: 0 -> 2, 128 -> 1, 255 -> 1, total = 4
        // tone 0:   cumulative 2, round(2 * 255 / 4) = round(127.5) = 128
        // tone 128: cumulative 3, round(3 * 255 / 4) = round(191.25) = 191
        // tone 255: cumulative 4, round(4 * 255 / 4) = round(255.0) = 255
        let mut image = make_gray_image(4, 1, &[0, 0, 128, 255]);
        equalize_histogram(&mut image);
        let gray_image = image.as_luma8().unwrap();
        assert_eq!(gray_image.get_pixel(0, 0)[0], 128);
        assert_eq!(gray_image.get_pixel(1, 0)[0], 128);
        assert_eq!(gray_image.get_pixel(2, 0)[0], 191);
        assert_eq!(gray_image.get_pixel(3, 0)[0], 255);
    }
}
