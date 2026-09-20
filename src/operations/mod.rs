use image::DynamicImage;

pub mod convolution;
mod geometry;
pub mod histogram;
mod point;

pub use convolution::Kernel;
pub use histogram::compute_histogram;

pub enum Operation {
    MirrorHorizontal,
    MirrorVertical,
    RotateClockwise,
    RotateCounterclockwise,
    Luminance,
    Quantize(u16),
    Negative,
    Brightness(i16),
    Contrast(f64),
    EqualizeHistogram,
    MatchHistogram(DynamicImage),
    Convolve(Kernel),
}

impl Operation {
    pub fn apply(&self, image: &mut DynamicImage) {
        match self {
            Operation::MirrorHorizontal => geometry::mirror_horizontal(image),
            Operation::MirrorVertical => geometry::mirror_vertical(image),
            Operation::RotateClockwise => geometry::rotate_90_clockwise(image),
            Operation::RotateCounterclockwise => geometry::rotate_90_counterclockwise(image),
            Operation::Luminance => point::luminance(image),
            Operation::Quantize(levels) => {
                // quantize only works on grayscale images
                point::luminance(image);
                point::quantize(image, *levels);
            }
            Operation::Negative => point::negative(image),
            Operation::Brightness(delta) => point::adjust_brightness(image, *delta),
            Operation::Contrast(factor) => point::adjust_contrast(image, *factor),
            Operation::EqualizeHistogram => histogram::equalize_histogram(image),
            Operation::MatchHistogram(reference) => histogram::match_histogram(image, reference),
            Operation::Convolve(kernel) => convolution::convolve(image, kernel),
        }
    }
}
