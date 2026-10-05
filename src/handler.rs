use gif::DecodeOptions;
use image::{ImageBuffer, Rgba};
use std::fs::File;
use std::path::Path;
use tracing::{info, warn};
use crate::{MonochoraError, Result};

const MAX_DIMENSION: u32 = 65535;
const MAX_PIXELS: u64 = 100_000_000;
const MAX_FRAMES: usize = 10000;

#[repr(C)]
#[derive(Clone)]
pub struct GifFrame {
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub delay_time_ms: u16,
}

#[repr(C)]
#[derive(Clone)]
pub struct GifData {
    pub frames: Vec<GifFrame>,
    pub width: u32,
    pub height: u32,
    pub loop_count: u16, 
}

pub struct GifFrameReader {
    decoder: gif::Decoder<File>,
    width: u32,
    height: u32,
    frames_read: usize,
    finished: bool,
}

impl GifFrameReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
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
        
        let decoder = options.read_info(file)
            .map_err(|e| MonochoraError::GifDecode(format!("Failed to read GIF info: {}", e)))?;
        
        let width = decoder.width() as u32;
        let height = decoder.height() as u32;
        
        if width == 0 || height == 0 {
            return Err(MonochoraError::InvalidDimensions { width, height });
        }
        
        if width > MAX_DIMENSION || height > MAX_DIMENSION {
            return Err(MonochoraError::InvalidDimensions { width, height });
        }
        
        let total_pixels = width as u64 * height as u64;
        if total_pixels > MAX_PIXELS {
            return Err(MonochoraError::InsufficientMemory);
        }
        
        info!("Decoding GIF: {}x{}", width, height);
        
        Ok(Self {
            decoder,
            width,
            height,
            frames_read: 0,
            finished: false,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn loop_count(&self) -> Result<u16> {
        match self.frames_read {
            0 => Err(MonochoraError::GifDecode("No valid frames found in GIF".to_string())),
            1 => Ok(1),
            _ => Ok(0),
        }
    }

    fn read_frame(&mut self) -> Result<Option<GifFrame>> {
        if self.finished {
            return Ok(None);
        }

        let frame = match self.decoder.read_next_frame() {
            Ok(Some(frame)) => frame,
            _ => {
                self.finished = true;
                return Ok(None);
            }
        };

        if self.frames_read >= MAX_FRAMES {
            warn!("Reached maximum frame limit of {}, stopping decode", MAX_FRAMES);
            self.finished = true;
            return Ok(None);
        }

        let delay_time_ms = if frame.delay == 0 { 100 } else { frame.delay * 10 };
        let image = compose_frame(frame, self.width, self.height)?;
        self.frames_read += 1;

        Ok(Some(GifFrame { image, delay_time_ms }))
    }
}

impl Iterator for GifFrameReader {
    type Item = Result<GifFrame>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.read_frame().transpose();
        if matches!(result, Some(Err(_))) {
            self.finished = true;
        }
        result
    }
}

pub fn decode_gif<P: AsRef<Path>>(path: P) -> Result<GifData> {
    let mut reader = GifFrameReader::open(path)?;
    let frames = reader.by_ref().collect::<Result<Vec<GifFrame>>>()?;
    let loop_count = reader.loop_count()?;
    
    Ok(GifData {
        frames,
        width: reader.width(),
        height: reader.height(),
        loop_count,
    })
}

fn validate_frame(frame: &gif::Frame, canvas_width: u32, canvas_height: u32) -> Result<()> {
    let width = frame.width as u32;
    let height = frame.height as u32;
    let left = frame.left as u32;
    let top = frame.top as u32;

    if width == 0 || height == 0 {
        return Err(MonochoraError::InvalidDimensions { width, height });
    }
    
    if left >= canvas_width || top >= canvas_height {
        return Err(MonochoraError::GifDecode(
            format!("Frame position ({}, {}) is outside canvas bounds ({}x{})", 
                left, top, canvas_width, canvas_height)
        ));
    }
    
    let expected_size = width as usize * height as usize * 4;
    if frame.buffer.len() != expected_size {
        return Err(MonochoraError::GifDecode(
            format!("Frame buffer size mismatch: expected {}, got {}", 
                expected_size, frame.buffer.len())
        ));
    }
    
    Ok(())
}

fn compose_frame(
    frame: &gif::Frame,
    canvas_width: u32,
    canvas_height: u32,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    validate_frame(frame, canvas_width, canvas_height)?;

    let canvas_w = canvas_width as usize;
    let canvas_h = canvas_height as usize;
    let left = frame.left as usize;
    let top = frame.top as usize;
    let frame_stride = frame.width as usize * 4;
    let copy_len = (frame.width as usize).min(canvas_w - left) * 4;

    let mut buffer = vec![0u8; canvas_w * canvas_h * 4];

    for (row, src) in frame.buffer.chunks_exact(frame_stride).take(canvas_h - top).enumerate() {
        let dst = ((top + row) * canvas_w + left) * 4;
        buffer[dst..dst + copy_len].copy_from_slice(&src[..copy_len]);
    }
    
    ImageBuffer::from_raw(canvas_width, canvas_height, buffer)
        .ok_or_else(|| MonochoraError::GifDecode(
            "Failed to create image buffer from frame data".to_string()
        ))
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


