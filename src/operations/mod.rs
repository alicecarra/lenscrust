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
    ZoomOut { factor_x: f64, factor_y: f64 },
    ZoomIn,
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
            Operation::ZoomOut { factor_x, factor_y } => {
                geometry::zoom_out(image, *factor_x, *factor_y)
            }
            Operation::ZoomIn => geometry::zoom_in(image),
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

    // for now used in history
    pub fn label(&self) -> String {
        match self {
            Operation::MirrorHorizontal => "Mirror Horizontal".to_string(),
            Operation::MirrorVertical => "Mirror Vertical".to_string(),
            Operation::RotateClockwise => "Rotate 90° Clockwise".to_string(),
            Operation::RotateCounterclockwise => "Rotate 90° Counterclockwise".to_string(),
            Operation::ZoomOut { factor_x, factor_y } => {
                format!("Zoom Out ({factor_x:.2}x, {factor_y:.2}y)")
            }
            Operation::ZoomIn => "Zoom In (2x2)".to_string(),
            Operation::Luminance => "Luminance".to_string(),
            Operation::Quantize(levels) => format!("Quantize ({levels} levels)"),
            Operation::Negative => "Negative".to_string(),
            Operation::Brightness(delta) => format!("Brightness ({delta:+})"),
            Operation::Contrast(factor) => format!("Contrast ({factor:.2}x)"),
            Operation::EqualizeHistogram => "Equalize Histogram".to_string(),
            Operation::MatchHistogram(_) => "Match Histogram".to_string(),
            Operation::Convolve(_) => "Convolve".to_string(),
        }
    }
}
