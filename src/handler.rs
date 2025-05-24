use anyhow::{Context, Result};
use gif::DecodeOptions;
use image::{ImageBuffer, Rgba};
use rayon::prelude::*;
use std::fs::File;
use std::path::Path;

pub struct GifFrame {
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub delay_time_ms: u16,
}

pub struct GifData {
    pub frames: Vec<GifFrame>,
    pub width: u32,
    pub height: u32,
    pub loop_count: u16, 
}

struct RawFrameData {
    buffer: Vec<u8>,
    delay_time_ms: u16,
    width: u32,
    height: u32,
    left: u32,
    top: u32,
}

pub fn decode_gif<P: AsRef<Path>>(path: P) -> Result<GifData> {
    let file = File::open(path).context("Failed to open GIF file")?;
    let mut options = DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    
    let mut decoder = options.read_info(file).context("Failed to read GIF info")?;
    let mut raw_frames = Vec::new();
    
    let width = decoder.width() as u32;
    let height = decoder.height() as u32;
    
    let mut loop_count = 0;
    
    while let Some(frame) = decoder.read_next_frame().context("Failed to read GIF frame")? {
        raw_frames.push(RawFrameData {
            buffer: frame.buffer.to_vec(),
            delay_time_ms: frame.delay * 10,
            width: frame.width as u32,
            height: frame.height as u32,
            left: frame.left as u32,
            top: frame.top as u32,
        });
    }
    
    if raw_frames.len() > 1 {
        loop_count = 0; 
    }
    
    println!("Processing {} frames in parallel...", raw_frames.len());
    
    let canvas_width = width;
    let canvas_height = height;
    
    let frames: Vec<GifFrame> = raw_frames
        .into_par_iter()
        .map(|raw_frame| {
            let mut buffer = vec![0u8; (canvas_width * canvas_height * 4) as usize];
            
            for y in 0..raw_frame.height {
                for x in 0..raw_frame.width {
                    if raw_frame.left + x >= canvas_width || raw_frame.top + y >= canvas_height {
                        continue;
                    }
                    
                    let src_idx = (y * raw_frame.width + x) as usize * 4;
                    let dst_idx = ((raw_frame.top + y) * canvas_width + (raw_frame.left + x)) as usize * 4;
                    
                    if src_idx + 3 < raw_frame.buffer.len() && dst_idx + 3 < buffer.len() {
                        buffer[dst_idx] = raw_frame.buffer[src_idx];
                        buffer[dst_idx + 1] = raw_frame.buffer[src_idx + 1];
                        buffer[dst_idx + 2] = raw_frame.buffer[src_idx + 2];
                        buffer[dst_idx + 3] = raw_frame.buffer[src_idx + 3];
                    }
                }
            }
            
            let image = ImageBuffer::from_raw(canvas_width, canvas_height, buffer)
                .expect("Failed to convert GIF frame to image buffer");
            
            GifFrame {
                image,
                delay_time_ms: raw_frame.delay_time_ms,
            }
        })
        .collect();
    
    Ok(GifData {
        frames,
        width,
        height,
        loop_count,
    })
}
