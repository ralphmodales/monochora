use anyhow::{Context, Result};
use gif::{Encoder, Frame, Repeat};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::fs::File;
use std::path::Path;

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

    let width = (max_line_length as f32 * (options.font_size * 0.6)) as u32;
    let line_height = (options.font_size * options.line_height_multiplier) as u32;
    let height = max_lines as u32 * line_height;

    let file = File::create(output_path).context("Failed to create output GIF file")?;
    let mut encoder = Encoder::new(file, width as u16, height as u16, &[])
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
            let y = line_idx as u32 * line_height;
            
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

        let frame_delay = if frame_idx < frame_delays.len() {
            frame_delays[frame_idx]
        } else if !frame_delays.is_empty() {
            frame_delays[0]
        } else {
            100
        };

        let flat_samples = image.as_flat_samples();
        let mut frame = Frame::from_rgb(
            width as u16,
            height as u16,
            flat_samples.as_slice(),
        );

        frame.delay = frame_delay / 10;

        encoder.write_frame(&frame).context("Failed to write GIF frame")?;
    }

    Ok(())
}
