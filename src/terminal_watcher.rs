use crate::{MonochoraError, Result};
use crate::converter::{image_to_ascii, image_to_colored_ascii, AsciiConverterConfig};
use crate::handler::GifData;
use crossterm::terminal::size;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, warn};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalDimensions {
    pub width: u32,
    pub height: u32,
}

impl TerminalDimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn current() -> Result<Self> {
        let (cols, rows) = size()
            .map_err(|e| MonochoraError::Terminal(format!("Failed to get terminal size: {}", e)))?;
        Ok(Self::new(cols as u32, rows as u32))
    }
}

pub struct TerminalWatcher {
    dimensions_tx: watch::Sender<TerminalDimensions>,
    dimensions_rx: watch::Receiver<TerminalDimensions>,
    stop_tx: Option<Sender<()>>,
}

impl TerminalWatcher {
    pub fn new() -> Result<Self> {
        let initial_dims = TerminalDimensions::current()?;
        let (dimensions_tx, dimensions_rx) = watch::channel(initial_dims);
        
        Ok(Self {
            dimensions_tx,
            dimensions_rx,
            stop_tx: None,
        })
    }

    pub fn start_watching(&mut self) -> Result<()> {
        let (stop_tx, stop_rx) = mpsc::channel();
        let tx = self.dimensions_tx.clone();
        
        thread::spawn(move || {
            let mut last_dimensions = match TerminalDimensions::current() {
                Ok(dims) => dims,
                Err(_) => return,
            };

            loop {
                if stop_rx.try_recv().is_ok() {
                    debug!("Terminal watcher stopping");
                    break;
                }

                match TerminalDimensions::current() {
                    Ok(current_dims) => {
                        if current_dims != last_dimensions {
                            debug!(
                                "Terminal resize detected: {}x{} -> {}x{}",
                                last_dimensions.width,
                                last_dimensions.height,
                                current_dims.width,
                                current_dims.height
                            );
                            
                            if let Err(e) = tx.send(current_dims) {
                                warn!("Failed to send dimension update: {}", e);
                                break;
                            }
                            
                            last_dimensions = current_dims;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to get terminal dimensions: {}", e);
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        });

        self.stop_tx = Some(stop_tx);
        Ok(())
    }

    pub fn get_receiver(&self) -> watch::Receiver<TerminalDimensions> {
        self.dimensions_rx.clone()
    }

    pub fn current_dimensions(&self) -> TerminalDimensions {
        *self.dimensions_rx.borrow()
    }

    pub fn stop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
    }
}

impl Drop for TerminalWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct ResponsiveFrameManager {
    gif_data: GifData,
    config_template: AsciiConverterConfig,
    frame_delays: Vec<u16>,
    current_dimensions: TerminalDimensions,
    cached_frames: Option<Vec<Vec<String>>>,
    colored: bool,
}

impl ResponsiveFrameManager {
    pub fn new(
        gif_data: GifData,
        config_template: AsciiConverterConfig,
        frame_delays: Vec<u16>,
        initial_dimensions: TerminalDimensions,
        colored: bool,
    ) -> Self {
        Self {
            gif_data,
            config_template,
            frame_delays,
            current_dimensions: initial_dimensions,
            cached_frames: None,
            colored,
        }
    }

    pub fn update_dimensions(&mut self, new_dimensions: TerminalDimensions) -> bool {
        if new_dimensions != self.current_dimensions {
            self.current_dimensions = new_dimensions;
            self.cached_frames = None;
            true
        } else {
            false
        }
    }

    pub fn get_frames(&mut self) -> Result<&[Vec<String>]> {
        if self.cached_frames.is_none() {
            self.regenerate_frames()?;
        }
        Ok(self.cached_frames.as_ref().unwrap())
    }

    pub fn get_frame_delays(&self) -> &[u16] {
        &self.frame_delays
    }

    fn regenerate_frames(&mut self) -> Result<()> {
        let target_width = self.current_dimensions.width.saturating_sub(2);
        let target_height = self.current_dimensions.height.saturating_sub(4);

        if target_width == 0 || target_height == 0 {
            return Err(MonochoraError::Terminal("Terminal too small for display".to_string()));
        }

        let mut config = self.config_template.clone();
        config.width = Some(target_width);
        config.height = Some(target_height);

        let new_frames: Result<Vec<Vec<String>>> = self.gif_data.frames
            .iter()
            .map(|frame| {
                if self.colored {
                    image_to_colored_ascii(&frame.image, &config)
                } else {
                    image_to_ascii(&frame.image, &config)
                }
            })
            .collect();

        self.cached_frames = Some(new_frames?);
        Ok(())
    }

    fn _resize_frame(&self, frame: &[String], target_width: usize, target_height: usize) -> Vec<String> {
        if frame.is_empty() {
            return vec![];
        }

        let current_height = frame.len();
        let current_width = frame.iter().map(|line| line.chars().count()).max().unwrap_or(0);

        if current_width <= target_width && current_height <= target_height {
            return frame.to_vec();
        }

        let mut resized_frame = Vec::new();

        let height_ratio = current_height as f32 / target_height as f32;
        let width_ratio = current_width as f32 / target_width as f32;

        for y in 0..target_height {
            let source_y = ((y as f32 * height_ratio) as usize).min(current_height - 1);
            let source_line = &frame[source_y];
            let source_chars: Vec<char> = source_line.chars().collect();

            let mut new_line = String::new();
            for x in 0..target_width {
                let source_x = ((x as f32 * width_ratio) as usize).min(source_chars.len().saturating_sub(1));
                if source_x < source_chars.len() {
                    new_line.push(source_chars[source_x]);
                } else {
                    new_line.push(' ');
                }
            }
            resized_frame.push(new_line);
        }

        resized_frame
    }
}
