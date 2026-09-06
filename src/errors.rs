#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("could not read current directory: {0}")]
    CurrentDir(#[source] std::io::Error),

    #[error("could not open {name}: {source}")]
    OpenImage {
        name: String,
        #[source]
        source: std::io::Error,
    },

    #[error("could not decode {name}: {source}")]
    DecodeImage {
        name: String,
        #[source]
        source: image::ImageError,
    },

    #[error("could not create file: {0}")]
    CreateFile(#[source] std::io::Error),

    #[error("could not save image: {0}")]
    SaveImage(#[source] image::ImageError),
}
