use image::DynamicImage;

pub mod convolution;
mod geometry;
mod point;

pub use convolution::Kernel;

pub enum Operation {
    MirrorHorizontal,
    MirrorVertical,
    Luminance,
    Quantize(u16),
    Convolve(Kernel),
}

impl Operation {
    pub fn apply(&self, image: &mut DynamicImage) {
        match self {
            Operation::MirrorHorizontal => geometry::mirror_horizontal(image),
            Operation::MirrorVertical => geometry::mirror_vertical(image),
            Operation::Luminance => point::luminance(image),
            Operation::Quantize(levels) => {
                // quantize only works on grayscale images
                point::luminance(image);
                point::quantize(image, *levels);
            }
            Operation::Convolve(kernel) => convolution::convolve(image, kernel),
        }
    }
}
