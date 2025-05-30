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
use regex::Regex;
use std::sync::OnceLock;
use std::collections::HashMap;

const MAX_FONT_SIZE: f32 = 200.0;
const MAX_LINE_HEIGHT_MULTIPLIER: f32 = 10.0;
const DEFAULT_CHAR_WIDTH_RATIO: f32 = 0.6;
const DEFAULT_PADDING: u32 = 20;
const MAX_PALETTE_COLORS: usize = 256;
const DEFAULT_FRAME_DELAY: u16 = 100;
const MIN_FRAME_DELAY: u16 = 1;

#[repr(C)]
pub struct AsciiGifOutputOptions {
    pub font_size: f32,
    pub bg_color: Rgb<u8>,
    pub text_color: Rgb<u8>,
    pub line_height_multiplier: f32,
    pub preserve_input_dimensions: bool,
    pub colored: bool,
}

impl Default for AsciiGifOutputOptions {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            bg_color: Rgb([0, 0, 0]),  
            text_color: Rgb([255, 255, 255]),  
            line_height_multiplier: 1.0,
            preserve_input_dimensions: true,
            colored: false,
        }
    }
}

impl AsciiGifOutputOptions {
    pub fn validate(&self) -> Result<()> {
        if self.font_size <= 0.0 || self.font_size > MAX_FONT_SIZE {
            return Err(MonochoraError::InvalidFontSize { size: self.font_size });
        }
        
        if self.line_height_multiplier <= 0.0 || self.line_height_multiplier > MAX_LINE_HEIGHT_MULTIPLIER {
            return Err(MonochoraError::Config(
                format!("Invalid line height multiplier: {}", self.line_height_multiplier)
            ));
        }
        
        Ok(())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ColoredCharacter {
    character: char,
    color: Rgb<u8>,
}

#[repr(C)]
#[derive(Debug, Clone)]
struct RenderDimensions {
    width: u32,
    height: u32,
    max_line_length: usize,
    max_lines: usize,
}

static ANSI_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_ansi_regex() -> &'static Regex {
    ANSI_REGEX.get_or_init(|| {
        Regex::new(r"\x1b\[38;2;(\d+);(\d+);(\d+)m([^\x1b]*)")
            .expect("Failed to compile ANSI regex")
    })
}

fn validate_font_charset_support(
    ascii_frames: &[Vec<String>],
    font: &Font,
) -> Result<()> {
    let mut unique_chars = std::collections::HashSet::new();
    
    for frame in ascii_frames {
        for line in frame {
            if line.contains('\x1b') {
                let regex = get_ansi_regex();
                let mut last_end = 0;
                
                for mat in regex.find_iter(line) {
                    if mat.start() > last_end {
                        let uncolored_text = &line[last_end..mat.start()];
                        for ch in uncolored_text.chars() {
                            if ch != '\x1b' && !ch.is_control() {
                                unique_chars.insert(ch);
                            }
                        }
                    }
                    
                    if let Some(captures) = regex.captures(&line[mat.start()..mat.end()]) {
                        for ch in captures[4].chars() {
                            if !ch.is_control() {
                                unique_chars.insert(ch);
                            }
                        }
                    }
                    
                    last_end = mat.end();
                }
                
                if last_end < line.len() {
                    let remaining = &line[last_end..];
                    for ch in remaining.chars() {
                        if ch != '\x1b' && ch != '\0' && !ch.is_control() {
                            unique_chars.insert(ch);
                        }
                    }
                }
            } else {
                for ch in line.chars() {
                    if !ch.is_control() {
                        unique_chars.insert(ch);
                    }
                }
            }
        }
    }
    
    let mut unsupported_chars = Vec::new();
    
    for &ch in &unique_chars {
        let glyph = font.glyph(ch);
        if glyph.id().0 == 0 {
            unsupported_chars.push(ch);
        }
    }
    
    if !unsupported_chars.is_empty() {
        unsupported_chars.sort();
        let unsupported_str: String = unsupported_chars.iter().collect();
        return Err(MonochoraError::UnsupportedFontCharacters {
            characters: unsupported_str
        });    }
    
    Ok(())
}

fn parse_line_to_colored_characters(line: &str, default_color: Rgb<u8>) -> Vec<ColoredCharacter> {
    if !line.contains('\x1b') {
        return line.chars().map(|c| ColoredCharacter { 
            character: c, 
            color: default_color 
        }).collect();
    }

    let regex = get_ansi_regex();
    let mut result = Vec::new();
    let mut last_end = 0;
    let mut current_color = default_color;
    
    for mat in regex.find_iter(line) {
        if mat.start() > last_end {
            let uncolored_text = &line[last_end..mat.start()];
            for ch in uncolored_text.chars() {
                if ch != '\x1b' {
                    result.push(ColoredCharacter { 
                        character: ch, 
                        color: current_color 
                    });
                }
            }
        }
        
        if let Some(captures) = regex.captures(&line[mat.start()..mat.end()]) {
            if let (Ok(r), Ok(g), Ok(b)) = (
                captures[1].parse::<u8>(),
                captures[2].parse::<u8>(),
                captures[3].parse::<u8>(),
            ) {
                current_color = Rgb([r, g, b]);
                for ch in captures[4].chars() {
                    result.push(ColoredCharacter { 
                        character: ch, 
                        color: current_color 
                    });
                }
            }
        }
        
        last_end = mat.end();
    }
    
    if last_end < line.len() {
        let remaining = &line[last_end..];
        for ch in remaining.chars() {
            if ch != '\x1b' && ch != '\0' {
                result.push(ColoredCharacter { 
                    character: ch, 
                    color: current_color 
                });
            }
        }
    }
    
    result
}

fn render_colored_line_to_image(
    image: &mut RgbImage,
    line: &str,
    y_position: u32,
    scale: Scale,
    font: &Font,
    options: &AsciiGifOutputOptions,
) -> Result<()> {
    let colored_chars = parse_line_to_colored_characters(line, options.text_color);
    
    if colored_chars.is_empty() {
        return Ok(());
    }
    
    let mut i = 0;
    while i < colored_chars.len() {
        let current_color = colored_chars[i].color;
        let mut segment_chars = String::new();
        let start_pos = i;
        
        while i < colored_chars.len() && colored_chars[i].color.0 == current_color.0 {
            segment_chars.push(colored_chars[i].character);
            i += 1;
        }
        
        let mut positioned_line = vec![' '; colored_chars.len()];
        let segment_char_vec: Vec<char> = segment_chars.chars().collect();
        
        for (idx, &ch) in segment_char_vec.iter().enumerate() {
            if start_pos + idx < positioned_line.len() {
                positioned_line[start_pos + idx] = ch;
            }
        }
        
        let positioned_text: String = positioned_line.into_iter().collect();
        
        draw_text_mut(
            image,
            current_color,
            0,
            y_position as i32,
            scale,
            font,
            &positioned_text,
        );
    }
    
    Ok(())
}

fn render_ascii_to_image_colored(
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

    for (line_idx, line) in ascii_frame.iter().enumerate() {
        let y = (line_idx as f32 * line_height) as u32;
        
        if y >= height.saturating_sub(scale.y as u32) {
            break;
        }
        
        if line.contains('\x1b') {
            render_colored_line_to_image(&mut image, line, y, scale, font, options)?;
        } else {
            draw_text_mut(
                &mut image,
                options.text_color,
                0,
                y as i32,
                scale,
                font,
                line,
            );
        }
    }
    
    Ok(image)
}

fn create_enhanced_color_palette(bg_color: Rgb<u8>) -> Vec<u8> {
    let mut palette = Vec::with_capacity(MAX_PALETTE_COLORS * 3);
    
    palette.extend_from_slice(&[bg_color[0], bg_color[1], bg_color[2]]);
    
    let primary_colors = [
        [255, 255, 255], [255, 0, 0], [0, 255, 0], [0, 0, 255],
        [255, 255, 0], [255, 0, 255], [0, 255, 255], [0, 0, 0],
        [128, 128, 128], [192, 192, 192], [64, 64, 64], [160, 160, 160],
        [255, 128, 0], [128, 255, 0], [0, 255, 128], [128, 0, 255],
        [255, 0, 128], [0, 128, 255], [255, 192, 192], [192, 255, 192],
        [192, 192, 255], [255, 255, 192], [255, 192, 255], [192, 255, 255],
    ];
    
    for color in &primary_colors {
        palette.extend_from_slice(color);
    }
    
    for i in 0..24 {
        let hue = (i as f32 / 24.0) * 360.0;
        for (sat, val) in &[(1.0, 1.0), (0.8, 0.9), (0.6, 0.8), (0.4, 0.7), (0.2, 0.6)] {
            let (r, g, b) = hsv_to_rgb(hue, *sat, *val);
            palette.extend_from_slice(&[r, g, b]);
        }
    }
    
    for i in 0..64 {
        let gray_value = (i * 255 / 63) as u8;
        palette.extend_from_slice(&[gray_value, gray_value, gray_value]);
    }
    
    while palette.len() < MAX_PALETTE_COLORS * 3 {
        palette.extend_from_slice(&[bg_color[0], bg_color[1], bg_color[2]]);
    }
    
    palette.truncate(MAX_PALETTE_COLORS * 3);
    palette
}

fn create_optimized_palette(bg_color: Rgb<u8>, text_color: Rgb<u8>) -> Vec<u8> {
    let mut palette = Vec::with_capacity(MAX_PALETTE_COLORS * 3);
    
    palette.extend_from_slice(&[bg_color[0], bg_color[1], bg_color[2]]);
    palette.extend_from_slice(&[text_color[0], text_color[1], text_color[2]]);
    
    for i in 1..32 {
        let ratio = i as f32 / 32.0;
        let r = (bg_color[0] as f32 * (1.0 - ratio) + text_color[0] as f32 * ratio) as u8;
        let g = (bg_color[1] as f32 * (1.0 - ratio) + text_color[1] as f32 * ratio) as u8;
        let b = (bg_color[2] as f32 * (1.0 - ratio) + text_color[2] as f32 * ratio) as u8;
        palette.extend_from_slice(&[r, g, b]);
    }
    
    while palette.len() < MAX_PALETTE_COLORS * 3 {
        palette.extend_from_slice(&[bg_color[0], bg_color[1], bg_color[2]]);
    }
    
    palette.truncate(MAX_PALETTE_COLORS * 3);
    palette
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r_prime, g_prime, b_prime) = match h as i32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    
    let r = ((r_prime + m) * 255.0) as u8;
    let g = ((g_prime + m) * 255.0) as u8;
    let b = ((b_prime + m) * 255.0) as u8;
    
    (r, g, b)
}

type ColorCache = HashMap<[u8; 3], u8>;

fn create_color_cache(palette: &[u8]) -> ColorCache {
    let mut cache = HashMap::with_capacity(MAX_PALETTE_COLORS);
    let colors_count = palette.len() / 3;
    
    for i in 0..colors_count {
        let idx = i * 3;
        if idx + 2 < palette.len() {
            let key = [palette[idx], palette[idx + 1], palette[idx + 2]];
            cache.insert(key, i as u8);
        }
    }
    
    cache
}

fn find_closest_color(rgb: [u8; 3], palette: &[u8], cache: &ColorCache) -> u8 {
    if let Some(&cached_index) = cache.get(&rgb) {
        return cached_index;
    }
    
    let colors_count = palette.len() / 3;
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
            
            let distance = (dr * dr + dg * dg + db * db) as u32;
            
            if distance < min_distance {
                min_distance = distance;
                best_index = i as u8;
                if distance == 0 { break; }
            }
        }
    }
    
    best_index
}

fn quantize_image(image: &RgbImage, palette: &[u8], cache: &ColorCache) -> Result<Vec<u8>> {
    let colors_count = palette.len() / 3;
    if colors_count == 0 {
        return Err(MonochoraError::Config("Empty color palette".to_string()));
    }
    
    let pixels: Vec<&Rgb<u8>> = image.pixels().collect();
    let indexed_data: Vec<u8> = pixels
        .par_iter()
        .map(|pixel| {
            let rgb = [pixel[0], pixel[1], pixel[2]];
            find_closest_color(rgb, palette, cache)
        })
        .collect();
    
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
    if options.colored {
        render_ascii_to_image_colored(ascii_frame, width, height, scale, font, options)
    } else {
        if width == 0 || height == 0 {
            return Err(MonochoraError::InvalidDimensions { width, height });
        }
        
        let mut image = RgbImage::from_pixel(width, height, options.bg_color);
        let line_height = scale.y;

        for (line_idx, line) in ascii_frame.iter().enumerate() {
            let y = (line_idx as f32 * line_height) as u32;
            
            if y < height.saturating_sub(scale.y as u32) {
                draw_text_mut(
                    &mut image,
                    options.text_color,
                    0,
                    y as i32, 
                    scale,
                    font,
                    line,
                );
            }
        }
        
        Ok(image)
    }
}

fn calculate_line_character_count(line: &str) -> usize {
    if line.contains('\x1b') {
        let regex = get_ansi_regex();
        let mut count = 0;
        let mut last_end = 0;
        
        for mat in regex.find_iter(line) {
            if mat.start() > last_end {
                count += line[last_end..mat.start()].chars().filter(|&c| c != '\x1b').count();
            }
            
            if let Some(captures) = regex.captures(&line[mat.start()..mat.end()]) {
                count += captures[4].chars().count();
            }
            
            last_end = mat.end();
        }
        
        if last_end < line.len() {
            let remaining = &line[last_end..];
            count += remaining.replace("\x1b[0m", "").chars().filter(|&c| c != '\x1b').count();
        }
        
        count
    } else {
        line.chars().count()
    }
}

fn calculate_dimensions_from_ascii(
    ascii_frames: &[Vec<String>],
    _options: &AsciiGifOutputOptions,
) -> Result<RenderDimensions> {
    if ascii_frames.is_empty() {
        return Err(MonochoraError::Config("No ASCII frames provided".to_string()));
    }
    
    let max_line_length = ascii_frames
        .iter()
        .flat_map(|frame| frame.iter().map(|line| calculate_line_character_count(line)))
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
    
    Ok(RenderDimensions {
        width: max_line_length as u32,
        height: max_lines as u32,
        max_line_length,
        max_lines,
    })
}

fn calculate_render_scale_and_dimensions(
    dimensions: &RenderDimensions,
    options: &AsciiGifOutputOptions,
    target_dimensions: Option<(u32, u32)>,
) -> (u32, u32, Scale) {
    match target_dimensions {
        Some((target_width, target_height)) => {
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

            let char_width = (options.font_size * DEFAULT_CHAR_WIDTH_RATIO) as u32;
            let width = dimensions.max_line_length as u32 * char_width;
            let line_height = (options.font_size * options.line_height_multiplier) as u32;
            let height = dimensions.max_lines as u32 * line_height + DEFAULT_PADDING;
            
            (width, height, scale)
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

    validate_font_charset_support(ascii_frames, &font)?;

    let dimensions = calculate_dimensions_from_ascii(ascii_frames, options)?;

    if let Some((target_width, target_height)) = target_dimensions {
        if target_width == 0 || target_height == 0 {
            return Err(MonochoraError::InvalidDimensions { 
                width: target_width, 
                height: target_height 
            });
        }
    }

    let (width, height, scale) = calculate_render_scale_and_dimensions(&dimensions, options, target_dimensions);

    if width == 0 || height == 0 {
        return Err(MonochoraError::InvalidDimensions { width, height });
    }

    let file = File::create(output_path.as_ref())
        .map_err(|e| MonochoraError::Io(e))?;
    
    let palette = if options.colored {
        create_enhanced_color_palette(options.bg_color)
    } else {
        create_optimized_palette(options.bg_color, options.text_color)
    };
    
    let color_cache = create_color_cache(&palette);
    
    let mut encoder = Encoder::new(file, width as u16, height as u16, &palette)
        .map_err(|e| MonochoraError::GifDecode(format!("Failed to create GIF encoder: {}", e)))?;

    let repeat_setting = if loop_count == 0 {
        Repeat::Infinite
    } else {
        Repeat::Finite(loop_count)
    };
    
    encoder.set_repeat(repeat_setting)
        .map_err(|e| MonochoraError::GifDecode(format!("Failed to set GIF repeat: {}", e)))?;
    
    debug!("Rendering {} frames in parallel (colored: {})", ascii_frames.len(), options.colored);
    
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
                DEFAULT_FRAME_DELAY
            };

            let indexed_data = quantize_image(&image, &palette, &color_cache)?;
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

        frame.delay = (frame_delay / 10).max(MIN_FRAME_DELAY);
        
        encoder.write_frame(&frame)
            .map_err(|e| MonochoraError::GifDecode(format!("Failed to write frame {}: {}", frame_idx, e)))?;
    }

    debug!("Successfully wrote {} frames to GIF", ascii_frames.len());
    Ok(())
}
