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
}
