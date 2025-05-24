use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonochoraError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("GIF decoding error: {0}")]
    GifDecode(String),

    #[error("Font loading error: {0}")]
    FontLoad(String),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid dimensions: width={width}, height={height}")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("Invalid font size: {size}")]
    InvalidFontSize { size: f32 },

    #[error("Invalid URL scheme: {scheme}")]
    InvalidUrlScheme { scheme: String },

    #[error("Terminal operation error: {0}")]
    Terminal(String),

    #[error("Thread pool error: {0}")]
    ThreadPool(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("File format not supported: {format}")]
    UnsupportedFormat { format: String },

    #[error("Network timeout")]
    NetworkTimeout,

    #[error("Insufficient memory for operation")]
    InsufficientMemory,

    #[error("Animation processing error: {0}")]
    Animation(String),
}

pub type Result<T> = std::result::Result<T, MonochoraError>;
