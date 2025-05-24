use crate::{MonochoraError, Result};
use gif::{Encoder, Frame, Repeat};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use rayon::prelude::*;
use tracing::debug;

pub struct AsciiGifOutputOptions {
    pub font_size: f32,
    pub bg_color: Rgb<u8>,
    pub text_color: Rgb<u8>,
    pub line_height_multiplier: f32,
    pub preserve_input_dimensions: bool,
}

impl Default for AsciiGifOutputOptions {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            bg_color: Rgb([0, 0, 0]),  
            text_color: Rgb([255, 255, 255]),  
            line_height_multiplier: 1.0,
            preserve_input_dimensions: true,
        }
    }
}

impl AsciiGifOutputOptions {
    pub fn validate(&self) -> Result<()> {
        if self.font_size <= 0.0 || self.font_size > 200.0 {
            return Err(MonochoraError::InvalidFontSize { size: self.font_size });
        }
        
        if self.line_height_multiplier <= 0.0 || self.line_height_multiplier > 10.0 {
            return Err(MonochoraError::Config(
                format!("Invalid line height multiplier: {}", self.line_height_multiplier)
            ));
        }
        
        Ok(())
    }
}

fn create_adaptive_palette(bg_color: Rgb<u8>, text_color: Rgb<u8>, font_size: f32) -> Vec<u8> {
    let mut palette = Vec::with_capacity(256 * 3);
    
    palette.extend_from_slice(&[bg_color[0], bg_color[1], bg_color[2]]);
    palette.extend_from_slice(&[text_color[0], text_color[1], text_color[2]]);
    
    let steps = if font_size < 2.0 {
        32  
    } else if font_size < 6.0 {
        16  
    } else {
        8   
    };
    
    for i in 1..steps {
        let ratio = i as f32 / steps as f32;
        let r = (bg_color[0] as f32 * (1.0 - ratio) + text_color[0] as f32 * ratio) as u8;
        let g = (bg_color[1] as f32 * (1.0 - ratio) + text_color[1] as f32 * ratio) as u8;
        let b = (bg_color[2] as f32 * (1.0 - ratio) + text_color[2] as f32 * ratio) as u8;
        palette.extend_from_slice(&[r, g, b]);
    }
    
    if font_size < 4.0 {
        let variations = [
            [bg_color[0].saturating_add(1), bg_color[1], bg_color[2]],
            [bg_color[0], bg_color[1].saturating_add(1), bg_color[2]],
            [bg_color[0], bg_color[1], bg_color[2].saturating_add(1)],
            [bg_color[0].saturating_sub(1), bg_color[1], bg_color[2]],
            [bg_color[0], bg_color[1].saturating_sub(1), bg_color[2]],
            [bg_color[0], bg_color[1], bg_color[2].saturating_sub(1)],
            [text_color[0].saturating_add(1), text_color[1], text_color[2]],
            [text_color[0], text_color[1].saturating_add(1), text_color[2]],
            [text_color[0], text_color[1], text_color[2].saturating_add(1)],
            [text_color[0].saturating_sub(1), text_color[1], text_color[2]],
            [text_color[0], text_color[1].saturating_sub(1), text_color[2]],
            [text_color[0], text_color[1], text_color[2].saturating_sub(1)],
        ];
        
        for variation in &variations {
            if palette.len() < 240 * 3 { 
                palette.extend_from_slice(variation);
            }
        }
    }
    
    while palette.len() < 256 * 3 {
        palette.extend_from_slice(&[bg_color[0], bg_color[1], bg_color[2]]);
    }
    
    palette.truncate(256 * 3);
    palette
}

fn quantize_to_adaptive_palette(image: &RgbImage, palette: &[u8], font_size: f32) -> Result<Vec<u8>> {
    let colors_count = palette.len() / 3;
    if colors_count == 0 {
        return Err(MonochoraError::Config("Empty color palette".to_string()));
    }
    
    let pixel_count = image.width() as usize * image.height() as usize;
    let mut indexed_data = Vec::with_capacity(pixel_count);
    
    let precision_threshold = if font_size < 2.0 { 5 } else { 15 };
    
    for pixel in image.pixels() {
        let rgb = [pixel[0], pixel[1], pixel[2]];
        
        let mut min_distance = u32::MAX;
        let mut best_index = 0u8;
        
        for i in 0..colors_count {
            let palette_idx = i * 3;
            if palette_idx + 2 < palette.len() {
                let pr = palette[palette_idx];
                let pg = palette[palette_idx + 1];
                let pb = palette[palette_idx + 2];
                
                let dr = rgb[0] as i32 - pr as i32;
                let dg = rgb[1] as i32 - pg as i32;
                let db = rgb[2] as i32 - pb as i32;
                
                let distance = if font_size < 2.0 {
                    ((dr * dr * 2 + dg * dg * 4 + db * db * 1) as f32 * 0.3) as u32
                } else if font_size < 6.0 {
                    ((dr * dr * 3 + dg * dg * 4 + db * db * 2) / 2) as u32
                } else {
                    ((dr * dr * 3 + dg * dg * 4 + db * db * 2) / 3) as u32
                };
                
                if distance < min_distance {
                    min_distance = distance;
                    best_index = i as u8;
                }
                
                if font_size < 2.0 && distance <= precision_threshold {
                    break;
                }
            }
        }
        
        indexed_data.push(best_index);
    }
    
    Ok(indexed_data)
}

fn render_ascii_to_image(
    ascii_frame: &[String],
    width: u32,
    height: u32,
    scale: Scale,
    font: &Font,
    options: &AsciiGifOutputOptions,
) -> Result<RgbImage> {
    if width == 0 || height == 0 {
        return Err(MonochoraError::InvalidDimensions { width, height });
    }
    
    let mut image = RgbImage::from_pixel(width, height, options.bg_color);
    let line_height = scale.y;
    let start_y = 0;
    let start_x = 0;

    for (line_idx, line) in ascii_frame.iter().enumerate() {
        let y = (line_idx as f32 * line_height) as u32 + start_y;
        
        if y < height.saturating_sub(scale.y as u32) {
            draw_text_mut(
                &mut image,
                options.text_color,
                start_x as i32, 
                y as i32, 
                scale,
                font,
                line,
            );
        }
    }
    
    Ok(image)
}

fn calculate_dimensions_from_ascii(
    ascii_frames: &[Vec<String>],
    _options: &AsciiGifOutputOptions,
) -> Result<(u32, u32, usize, usize)> {
    if ascii_frames.is_empty() {
        return Err(MonochoraError::Config("No ASCII frames provided".to_string()));
    }
    
    let max_line_length = ascii_frames
        .iter()
        .flat_map(|frame| frame.iter().map(|line| line.chars().count()))
        .max()
        .unwrap_or(80);

    let max_lines = ascii_frames
        .iter()
        .map(|frame| frame.len())
        .max()
        .unwrap_or(24);
    
    if max_line_length == 0 || max_lines == 0 {
        return Err(MonochoraError::Config("ASCII frames contain no content".to_string()));
    }
    
    Ok((max_line_length as u32, max_lines as u32, max_line_length, max_lines))
}

pub fn ascii_frames_to_gif<P: AsRef<Path>>(
    ascii_frames: &[Vec<String>],
    frame_delays: &[u16],
    loop_count: u16,
    output_path: P,
    options: &AsciiGifOutputOptions,
) -> Result<()> {
    ascii_frames_to_gif_with_dimensions(
        ascii_frames,
        frame_delays,
        loop_count,
        output_path,
        options,
        None,
    )
}

pub fn ascii_frames_to_gif_with_dimensions<P: AsRef<Path>>(
    ascii_frames: &[Vec<String>],
    frame_delays: &[u16],
    loop_count: u16,
    output_path: P,
    options: &AsciiGifOutputOptions,
    target_dimensions: Option<(u32, u32)>,
) -> Result<()> {
    options.validate()?;
    
    if ascii_frames.is_empty() {
        return Err(MonochoraError::Config("No ASCII frames to convert".to_string()));
    }
    
    if frame_delays.is_empty() {
        return Err(MonochoraError::Config("No frame delays provided".to_string()));
    }
    
    let font_data = include_bytes!("../resources/DejaVuSansMono.ttf");
    let font = Arc::new(
        Font::try_from_bytes(font_data as &[u8])
            .ok_or_else(|| MonochoraError::FontLoad("Failed to load embedded font".to_string()))?
    );

    let (_, _, max_line_length, max_lines) = calculate_dimensions_from_ascii(ascii_frames, options)?;

    let (width, height, scale) = match target_dimensions {
        Some((target_width, target_height)) => {
            if target_width == 0 || target_height == 0 {
                return Err(MonochoraError::InvalidDimensions { 
                    width: target_width, 
                    height: target_height 
                });
            }
            
            let scale = Scale {
                x: options.font_size,
                y: options.font_size,
            };
            
            (target_width, target_height, scale)
        }
        None => {
            let scale = Scale {
                x: options.font_size,
                y: options.font_size,
            };

            let char_width = (options.font_size * 0.6) as u32;
            let width = max_line_length as u32 * char_width;
            let line_height = (options.font_size * options.line_height_multiplier) as u32;
            let height = max_lines as u32 * line_height + 20;
            
            (width, height, scale)
        }
    };

    if width == 0 || height == 0 {
        return Err(MonochoraError::InvalidDimensions { width, height });
    }

    let file = File::create(output_path.as_ref())
        .map_err(|e| MonochoraError::Io(e))?;
    
    let palette = create_adaptive_palette(options.bg_color, options.text_color, options.font_size);
    
    let mut encoder = Encoder::new(file, width as u16, height as u16, &palette)
        .map_err(|e| MonochoraError::GifDecode(format!("Failed to create GIF encoder: {}", e)))?;

    let repeat_setting = if loop_count == 0 {
        Repeat::Infinite
    } else {
        Repeat::Finite(loop_count)
    };
    
    encoder.set_repeat(repeat_setting)
        .map_err(|e| MonochoraError::GifDecode(format!("Failed to set GIF repeat: {}", e)))?;
    
    debug!("Rendering {} frames in parallel", ascii_frames.len());
    
    let frame_results: Result<Vec<(Vec<u8>, u16)>> = ascii_frames
        .par_iter()
        .enumerate()
        .map(|(frame_idx, ascii_frame)| -> Result<(Vec<u8>, u16)> {
            let image = render_ascii_to_image(
                ascii_frame, 
                width, 
                height, 
                scale, 
                &font, 
                options
            )?;

            let frame_delay = if frame_idx < frame_delays.len() {
                frame_delays[frame_idx]
            } else if !frame_delays.is_empty() {
                frame_delays[0]
            } else {
                100
            };

            let indexed_data = quantize_to_adaptive_palette(&image, &palette, options.font_size)?;
            Ok((indexed_data, frame_delay))
        })
        .collect();
    
    let rendered_frames = frame_results?;
    
    for (frame_idx, (indexed_data, frame_delay)) in rendered_frames.into_iter().enumerate() {
        if indexed_data.len() != (width * height) as usize {
            return Err(MonochoraError::GifDecode(
                format!("Frame {} has incorrect data size: expected {}, got {}", 
                    frame_idx, width * height, indexed_data.len())
            ));
        }
        
        let mut frame = Frame::from_palette_pixels(
            width as u16,
            height as u16,
            &indexed_data,
            &palette,
            None,
        );

        frame.delay = (frame_delay / 10).max(1);
        
        encoder.write_frame(&frame)
            .map_err(|e| MonochoraError::GifDecode(format!("Failed to write frame {}: {}", frame_idx, e)))?;
    }

    debug!("Successfully wrote {} frames to GIF", ascii_frames.len());
    Ok(())
}
