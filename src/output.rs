use anyhow::{Context, Result};
use gif::{Encoder, Frame, Repeat};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

pub struct AsciiGifOutputOptions {
    pub font_size: f32,
    pub bg_color: Rgb<u8>,
    pub text_color: Rgb<u8>,
    pub line_height_multiplier: f32,
}

impl Default for AsciiGifOutputOptions {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            bg_color: Rgb([0, 0, 0]),  
            text_color: Rgb([255, 255, 255]),  
            line_height_multiplier: 1.2,  
        }
    }
}

fn quantize_colors(image: &RgbImage, max_colors: usize) -> (Vec<u8>, Vec<u8>) {
    let mut color_counts: HashMap<[u8; 3], u32> = HashMap::new();
    
    for pixel in image.pixels() {
        let rgb = [pixel[0], pixel[1], pixel[2]];
        *color_counts.entry(rgb).or_insert(0) += 1;
    }
    
    let mut colors: Vec<([u8; 3], u32)> = color_counts.into_iter().collect();
    colors.sort_by(|a, b| b.1.cmp(&a.1));
    colors.truncate(max_colors);
    
    let mut palette = Vec::new();
    let mut color_to_index = HashMap::new();
    
    for (i, (color, _)) in colors.iter().enumerate() {
        palette.extend_from_slice(color);
        color_to_index.insert(*color, i as u8);
    }
    
    while palette.len() < 12 { 
        palette.extend_from_slice(&[0, 0, 0]);
    }
    
    let mut indexed_data = Vec::new();
    for pixel in image.pixels() {
        let rgb = [pixel[0], pixel[1], pixel[2]];
        
        let index = color_to_index.get(&rgb).copied().unwrap_or_else(|| {
            let mut min_distance = u32::MAX;
            let mut closest_index = 0u8;
            
            for (i, palette_color) in colors.iter().enumerate() {
                let distance = ((rgb[0] as i32 - palette_color.0[0] as i32).pow(2) +
                               (rgb[1] as i32 - palette_color.0[1] as i32).pow(2) +
                               (rgb[2] as i32 - palette_color.0[2] as i32).pow(2)) as u32;
                
                if distance < min_distance {
                    min_distance = distance;
                    closest_index = i as u8;
                }
            }
            closest_index
        });
        
        indexed_data.push(index);
    }
    
    (palette, indexed_data)
}

pub fn ascii_frames_to_gif<P: AsRef<Path>>(
    ascii_frames: &[Vec<String>],
    frame_delays: &[u16],
    loop_count: u16,
    output_path: P,
    options: &AsciiGifOutputOptions,
) -> Result<()> {
    let font_data = include_bytes!("../resources/DejaVuSansMono.ttf");
    let font = Font::try_from_bytes(font_data as &[u8])
        .context("Failed to load default font")?;

    let scale = Scale {
        x: options.font_size,
        y: options.font_size,
    };

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

    let char_width = (options.font_size * 0.6) as u32;
    let width = max_line_length as u32 * char_width;
    let line_height = (options.font_size * options.line_height_multiplier) as u32;
    let height = max_lines as u32 * line_height + 20; 

    let file = File::create(output_path).context("Failed to create output GIF file")?;
    
    let mut sample_image = RgbImage::from_pixel(width, height, options.bg_color);
    
    if let Some(first_frame) = ascii_frames.first() {
        for (line_idx, line) in first_frame.iter().enumerate().take(5) { // Sample first few lines
            let y = line_idx as u32 * line_height + 10;
            
            draw_text_mut(
                &mut sample_image,
                options.text_color,
                10, 
                y as i32, 
                scale,
                &font,
                line,
            );
        }
    }
    
    let (palette, _) = quantize_colors(&sample_image, 128);
    
    let mut encoder = Encoder::new(file, width as u16, height as u16, &palette)
        .context("Failed to create GIF encoder")?;

    if loop_count == 0 {
        encoder.set_repeat(Repeat::Infinite)
            .context("Failed to set GIF to loop infinitely")?;
    } else {
        encoder.set_repeat(Repeat::Finite(loop_count))
            .context("Failed to set GIF loop count")?;
    }

    for (frame_idx, ascii_frame) in ascii_frames.iter().enumerate() {
        let mut image = RgbImage::from_pixel(width, height, options.bg_color);

        for (line_idx, line) in ascii_frame.iter().enumerate() {
            let y = line_idx as u32 * line_height + 10; 
            
            if y < height {
                draw_text_mut(
                    &mut image,
                    options.text_color,
                    10, 
                    y as i32, 
                    scale,
                    &font,
                    line,
                );
            }
        }

        let frame_delay = if frame_idx < frame_delays.len() {
            frame_delays[frame_idx]
        } else if !frame_delays.is_empty() {
            frame_delays[0]
        } else {
            100
        };

        let (_, indexed_data) = quantize_colors(&image, 128);
        
        let mut frame = Frame::from_palette_pixels(
            width as u16,
            height as u16,
            &indexed_data,
            &palette,
            None,
        );

        frame.delay = (frame_delay / 10).max(1); 

        encoder.write_frame(&frame).context("Failed to write GIF frame")?;
    }

    Ok(())
}
