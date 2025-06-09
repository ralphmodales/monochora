pub mod converter;
pub mod display;
pub mod handler;
pub mod output;
pub mod terminal_watcher;
pub mod web;
pub mod error;

pub use converter::{image_to_ascii, image_to_colored_ascii, AsciiConverterConfig};
pub use display::{display_ascii_animation, get_terminal_size, save_ascii_to_file, display_responsive_ascii_animation};
pub use handler::{decode_gif, GifData, GifFrame};
pub use output::{ascii_frames_to_gif, ascii_frames_to_gif_with_dimensions, AsciiGifOutputOptions};
pub use terminal_watcher::{TerminalWatcher, ResponsiveFrameManager, TerminalDimensions};
pub use web::{download_gif_from_url, get_input_path, is_url};
pub use error::{MonochoraError, Result};
