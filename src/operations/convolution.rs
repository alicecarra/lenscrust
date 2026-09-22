use image::{DynamicImage, GenericImage, GenericImageView};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kernel {
    pub weights: [[f64; 3]; 3],
    pub bias_before_clamping: bool,
}

impl Kernel {
    pub const GAUSSIAN_LOW_PASS: Kernel = Kernel {
        weights: [
            [0.0625, 0.125, 0.0625],
            [0.125, 0.25, 0.125],
            [0.0625, 0.125, 0.0625],
        ],
        bias_before_clamping: false,
    };

    pub const POSITIVE_LAPLACIAN_HIGH_PASS: Kernel = Kernel {
        weights: [[0.0, -1.0, 0.0], [-1.0, 4.0, -1.0], [0.0, -1.0, 0.0]],
        bias_before_clamping: false,
    };

    pub const NEGATIVE_LAPLACIAN_HIGH_PASS: Kernel = Kernel {
        weights: [[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]],
        bias_before_clamping: false,
    };

    pub const PREWITT_HORIZONTAL_GRADIENT: Kernel = Kernel {
        weights: [[-1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [-1.0, 0.0, 1.0]],
        bias_before_clamping: true,
    };

    pub const PREWITT_VERTICAL_GRADIENT: Kernel = Kernel {
        weights: [[-1.0, -1.0, -1.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
        bias_before_clamping: true,
    };

    pub const SOBEL_HORIZONTAL_GRADIENT: Kernel = Kernel {
        weights: [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]],
        bias_before_clamping: true,
    };

    pub const SOBEL_VERTICAL_GRADIENT: Kernel = Kernel {
        weights: [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]],
        bias_before_clamping: true,
    };
}

pub fn convolve(image: &mut DynamicImage, kernel: &Kernel) {
    // only gaussian low pass with color images
    if *kernel != Kernel::GAUSSIAN_LOW_PASS {
        super::point::luminance(image);
    }

    let (image_width, image_height) = image.dimensions();
    if image_width < 3 || image_height < 3 {
        // TODO: give feedback to user?
        return;
    }

    let source_image = image.clone();

    for center_y in 1..image_height - 1 {
        for center_x in 1..image_width - 1 {
            let mut channel_sums = [0.0f64; 3];

            for kernel_row in 0..3u32 {
                for kernel_column in 0..3u32 {
                    // rotated kernel
                    let weight =
                        kernel.weights[(2 - kernel_row) as usize][(2 - kernel_column) as usize];

                    let neighbor_x = center_x + kernel_column - 1;
                    let neighbor_y = center_y + kernel_row - 1;
                    let neighbor_pixel = source_image.get_pixel(neighbor_x, neighbor_y);

                    for channel_index in 0..3 {
                        channel_sums[channel_index] +=
                            neighbor_pixel[channel_index] as f64 * weight;
                    }
                }
            }

            let mut output_pixel = source_image.get_pixel(center_x, center_y);
            for channel_index in 0..3 {
                let mut channel_value = channel_sums[channel_index];
                if kernel.bias_before_clamping {
                    channel_value += 127.0;
                }
                output_pixel[channel_index] = channel_value.round().clamp(0.0, 255.0) as u8;
            }

            image.put_pixel(center_x, center_y, output_pixel);
        }
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
    fn convolve_applies_the_kernel_rotated_180_degrees() {
        let kernel = Kernel {
            weights: [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            bias_before_clamping: false,
        };
        let mut image = make_gray_image(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);

        convolve(&mut image, &kernel);

        let gray_image = image.as_luma8().unwrap();
        // 9*1 + 8*2 + 7*3 + 6*4 + 5*5 + 4*6 + 3*7 + 2*8 + 1*9 = 165
        assert_eq!(gray_image.get_pixel(1, 1)[0], 165);
    }

    #[test]
    fn convolve_leaves_the_border_pixels_untouched() {
        let kernel = Kernel {
            weights: [[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]],
            bias_before_clamping: false,
        };
        let original_pixels = [10, 20, 30, 40, 50, 60, 70, 80, 90];
        let mut image = make_gray_image(3, 3, &original_pixels);

        convolve(&mut image, &kernel);

        let gray_image = image.as_luma8().unwrap();
        let border_positions = [
            (0, 0),
            (1, 0),
            (2, 0),
            (0, 1),
            (2, 1),
            (0, 2),
            (1, 2),
            (2, 2),
        ];
        for (x, y) in border_positions {
            let original_value = original_pixels[(y * 3 + x) as usize];
            assert_eq!(gray_image.get_pixel(x, y)[0], original_value);
        }
    }

    #[test]
    fn convolve_adds_bias_before_clamping_when_requested() {
        let pixels = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let kernel_without_bias = Kernel {
            weights: [[-2.0; 3]; 3],
            bias_before_clamping: false,
        };
        let mut image_without_bias = make_gray_image(3, 3, &pixels);
        convolve(&mut image_without_bias, &kernel_without_bias);
        assert_eq!(image_without_bias.as_luma8().unwrap().get_pixel(1, 1)[0], 0);

        let kernel_with_bias = Kernel {
            bias_before_clamping: true,
            ..kernel_without_bias
        };
        let mut image_with_bias = make_gray_image(3, 3, &pixels);
        convolve(&mut image_with_bias, &kernel_with_bias);
        // -90.0 + 127.0 = 37.0, within [0, 255], so no clamping.
        assert_eq!(image_with_bias.as_luma8().unwrap().get_pixel(1, 1)[0], 37);
    }

    #[test]
    fn convolve_forces_grayscale_unless_the_kernel_supports_color() {
        let mut color_image = make_rgb_image(
            3,
            3,
            &[
                [10, 20, 30],
                [40, 50, 60],
                [70, 80, 90],
                [15, 25, 35],
                [45, 55, 65],
                [75, 85, 95],
                [11, 21, 31],
                [41, 51, 61],
                [71, 81, 91],
            ],
        );

        convolve(&mut color_image, &Kernel::POSITIVE_LAPLACIAN_HIGH_PASS);

        assert!(color_image.as_luma8().is_some());
    }

    #[test]
    fn convolve_preserves_color_when_the_kernel_supports_it() {
        // the Gaussian weights sum to 1.0, so a uniformly-colored image
        // should come out of the blur unchanged, on every channel
        let mut color_image = make_rgb_image(3, 3, &[[10, 20, 30]; 9]);

        convolve(&mut color_image, &Kernel::GAUSSIAN_LOW_PASS);

        assert!(color_image.as_luma8().is_none());
        let rgb_image = color_image.to_rgb8();
        assert_eq!(rgb_image.get_pixel(1, 1).0, [10, 20, 30]);
    }
}
