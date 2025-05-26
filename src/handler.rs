use gif::DecodeOptions;
use image::{ImageBuffer, Rgba};
use rayon::prelude::*;
use std::fs::File;
use std::path::Path;
use tracing::{info, warn};
use crate::{MonochoraError, Result};

#[repr(C)]
pub struct GifFrame {
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub delay_time_ms: u16,
}

#[repr(C)]
pub struct GifData {
    pub frames: Vec<GifFrame>,
    pub width: u32,
    pub height: u32,
    pub loop_count: u16, 
}

#[repr(C)]
struct RawFrameData {
    buffer: Vec<u8>,
    delay_time_ms: u16,
    width: u32,
    height: u32,
    left: u32,
    top: u32,
}

impl RawFrameData {
    fn validate(&self, canvas_width: u32, canvas_height: u32) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(MonochoraError::InvalidDimensions { 
                width: self.width, 
                height: self.height 
            });
        }
        
        if self.left >= canvas_width || self.top >= canvas_height {
            return Err(MonochoraError::GifDecode(
                format!("Frame position ({}, {}) is outside canvas bounds ({}x{})", 
                    self.left, self.top, canvas_width, canvas_height)
            ));
        }
        
        let expected_size = (self.width * self.height * 4) as usize;
        if self.buffer.len() != expected_size {
            return Err(MonochoraError::GifDecode(
                format!("Frame buffer size mismatch: expected {}, got {}", 
                    expected_size, self.buffer.len())
            ));
        }
        
        Ok(())
    }
}

pub fn decode_gif<P: AsRef<Path>>(path: P) -> Result<GifData> {
    let path_ref = path.as_ref();
    
    if !path_ref.exists() {
        return Err(MonochoraError::Io(
            std::io::Error::new(std::io::ErrorKind::NotFound, "GIF file not found")
        ));
    }
    
    let file = File::open(path_ref)
        .map_err(|e| MonochoraError::Io(e))?;
    
    let mut options = DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    
    let mut decoder = options.read_info(file)
        .map_err(|e| MonochoraError::GifDecode(format!("Failed to read GIF info: {}", e)))?;
    
    let width = decoder.width() as u32;
    let height = decoder.height() as u32;
    
    if width == 0 || height == 0 {
        return Err(MonochoraError::InvalidDimensions { width, height });
    }
    
    const MAX_DIMENSION: u32 = 65535;
    const MAX_PIXELS: u64 = 100_000_000; 
    
    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(MonochoraError::InvalidDimensions { width, height });
    }
    
    let total_pixels = width as u64 * height as u64;
    if total_pixels > MAX_PIXELS {
        return Err(MonochoraError::InsufficientMemory);
    }
    
    let mut raw_frames = Vec::new();
    let mut frame_count = 0;
    const MAX_FRAMES: usize = 10000; 
    
    info!("Decoding GIF: {}x{}", width, height);
    
    while let Ok(Some(frame)) = decoder.read_next_frame() {
        if frame_count >= MAX_FRAMES {
            warn!("Reached maximum frame limit of {}, stopping decode", MAX_FRAMES);
            break;
        }
        
        let delay_ms = if frame.delay == 0 { 100 } else { frame.delay * 10 };
        
        let raw_frame = RawFrameData {
            buffer: frame.buffer.to_vec(),
            delay_time_ms: delay_ms,
            width: frame.width as u32,
            height: frame.height as u32,
            left: frame.left as u32,
            top: frame.top as u32,
        };
        
        raw_frame.validate(width, height)?;
        raw_frames.push(raw_frame);
        frame_count += 1;
    }
    
    if raw_frames.is_empty() {
        return Err(MonochoraError::GifDecode("No valid frames found in GIF".to_string()));
    }
    
    info!("Processing {} frames in parallel...", raw_frames.len());
    
    let canvas_width = width;
    let canvas_height = height;
    
    let frame_results: std::result::Result<Vec<GifFrame>, MonochoraError> = raw_frames
        .into_par_iter()
        .map(|raw_frame| -> Result<GifFrame> {
            create_frame_from_raw(raw_frame, canvas_width, canvas_height)
        })
        .collect();
    
    let frames = frame_results?;
    
    let loop_count = if frames.len() > 1 { 0 } else { 1 };
    
    Ok(GifData {
        frames,
        width,
        height,
        loop_count,
    })
}

fn create_frame_from_raw(
    raw_frame: RawFrameData, 
    canvas_width: u32, 
    canvas_height: u32
) -> Result<GifFrame> {
    let canvas_size = (canvas_width * canvas_height * 4) as usize;
    let mut buffer = vec![0u8; canvas_size];
    
    for y in 0..raw_frame.height {
        for x in 0..raw_frame.width {
            let canvas_x = raw_frame.left + x;
            let canvas_y = raw_frame.top + y;
            
            if canvas_x >= canvas_width || canvas_y >= canvas_height {
                continue;
            }
            
            let src_idx = (y * raw_frame.width + x) as usize * 4;
            let dst_idx = (canvas_y * canvas_width + canvas_x) as usize * 4;
            
            if src_idx + 3 < raw_frame.buffer.len() && dst_idx + 3 < buffer.len() {
                buffer[dst_idx] = raw_frame.buffer[src_idx];         // red
                buffer[dst_idx + 1] = raw_frame.buffer[src_idx + 1]; // green
                buffer[dst_idx + 2] = raw_frame.buffer[src_idx + 2]; // blue
                buffer[dst_idx + 3] = raw_frame.buffer[src_idx + 3]; // alpha
            }
        }
    }
    
    let image = ImageBuffer::from_raw(canvas_width, canvas_height, buffer)
        .ok_or_else(|| MonochoraError::GifDecode(
            "Failed to create image buffer from frame data".to_string()
        ))?;
    
    Ok(GifFrame {
        image,
        delay_time_ms: raw_frame.delay_time_ms,
    })
}

impl GifData {
    pub fn total_duration_ms(&self) -> u64 {
        self.frames.iter()
            .map(|frame| frame.delay_time_ms as u64)
            .sum()
    }
    
    pub fn average_frame_delay(&self) -> u16 {
        if self.frames.is_empty() {
            return 100; // Default delay
        }
        
        let total: u64 = self.frames.iter()
            .map(|frame| frame.delay_time_ms as u64)
            .sum();
        
        (total / self.frames.len() as u64) as u16
    }
    
    pub fn validate(&self) -> Result<()> {
        if self.frames.is_empty() {
            return Err(MonochoraError::GifDecode("GIF has no frames".to_string()));
        }
        
        if self.width == 0 || self.height == 0 {
            return Err(MonochoraError::InvalidDimensions { 
                width: self.width, 
                height: self.height 
            });
        }
        
        for (i, frame) in self.frames.iter().enumerate() {
            let (frame_width, frame_height) = frame.image.dimensions();
            if frame_width != self.width || frame_height != self.height {
                return Err(MonochoraError::GifDecode(
                    format!("Frame {} has incorrect dimensions: {}x{}, expected {}x{}", 
                        i, frame_width, frame_height, self.width, self.height)
                ));
            }
        }
        
        Ok(())
    }
}


