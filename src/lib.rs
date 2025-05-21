pub mod converter;
pub mod display;
pub mod handler;
pub mod output;

pub use converter::{image_to_ascii, image_to_colored_ascii, AsciiConverterConfig};
pub use display::{display_ascii_animation, get_terminal_size, save_ascii_to_file};
pub use handler::{decode_gif, GifData, GifFrame};
pub use output::{ascii_frames_to_gif, AsciiGifOutputOptions};
