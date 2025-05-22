use anyhow::{Context, Result};
use gif::DecodeOptions;
use image::{ImageBuffer, Rgba};
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

pub fn decode_gif<P: AsRef<Path>>(path: P) -> Result<GifData> {
    let file = File::open(path).context("Failed to open GIF file")?;
    let mut options = DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    
    let mut decoder = options.read_info(file).context("Failed to read GIF info")?;
    let mut frames = Vec::new();
    
    let width = decoder.width() as u32;
    let height = decoder.height() as u32;
    
     let _global_palette = decoder.global_palette().map(|p| p.to_vec());
    
    let mut loop_count = 0;
    
     while let Some(frame) = decoder.read_next_frame().context("Failed to read GIF frame")? {
         let mut buffer = vec![0; (width * height * 4) as usize];
        
         let frame_width = frame.width as u32;
        let frame_height = frame.height as u32;
        let frame_left = frame.left as u32;
        let frame_top = frame.top as u32;
        
         for y in 0..frame_height {
            for x in 0..frame_width {
                 if frame_left + x >= width || frame_top + y >= height {
                    continue;
                }
                
                let src_idx = (y * frame_width + x) as usize * 4;
                let dst_idx = ((frame_top + y) * width + (frame_left + x)) as usize * 4;
                
                if src_idx + 3 < frame.buffer.len() && dst_idx + 3 < buffer.len() {
                    buffer[dst_idx] = frame.buffer[src_idx];       // red
                    buffer[dst_idx + 1] = frame.buffer[src_idx + 1]; // green
                    buffer[dst_idx + 2] = frame.buffer[src_idx + 2]; // blue
                    buffer[dst_idx + 3] = frame.buffer[src_idx + 3]; // alpha
                }
            }
        }
        
        let image = ImageBuffer::from_raw(width, height, buffer)
            .context("Failed to convert GIF frame to image buffer")?;
        
        frames.push(GifFrame {
            image,
            delay_time_ms: frame.delay * 10, 
        });
    }
    
    if frames.len() > 1 {
        loop_count = 0; 
    }
    
    Ok(GifData {
        frames,
        width,
        height,
        loop_count,
    })
}
