use image::{GenericImageView, Rgba};
use rayon::prelude::*;
use crate::{MonochoraError, Result};

static SIMPLE_CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
static DETAILED_CHARS: &[char] = &[
    ' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~', '+', '_', '-',
    '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v',
    'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k',
    'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@'
];

#[repr(C)]
pub struct AsciiConverterConfig {
    pub width: Option<u32>,        
    pub height: Option<u32>,       
    pub char_aspect: f32,         
    pub invert: bool,            
    pub detailed: bool,
    pub preserve_aspect_ratio: bool, 
    pub scale_factor: Option<f32>,
    pub custom_charset: Option<Vec<char>>,
}

impl Default for AsciiConverterConfig {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            char_aspect: 0.5,
            invert: false,
            detailed: true,
            preserve_aspect_ratio: true, 
            scale_factor: None,
            custom_charset: None,
        }
    }
}

impl AsciiConverterConfig {
    pub fn validate(&self) -> Result<()> {
        if let Some(width) = self.width {
            if width == 0 {
                return Err(MonochoraError::InvalidDimensions { width, height: self.height.unwrap_or(0) });
            }
        }
        
        if let Some(height) = self.height {
            if height == 0 {
                return Err(MonochoraError::InvalidDimensions { width: self.width.unwrap_or(0), height });
            }
        }
        
        if self.char_aspect <= 0.0 {
            return Err(MonochoraError::Config("Character aspect ratio must be positive".to_string()));
        }
        
        if let Some(scale) = self.scale_factor {
            if scale <= 0.0 {
                return Err(MonochoraError::Config("Scale factor must be positive".to_string()));
            }
        }
        
        if let Some(charset) = &self.custom_charset {
            if charset.len() < 2 {
                return Err(MonochoraError::Config("Custom character set must contain at least 2 characters".to_string()));
            }
            if charset.len() > 256 {
                return Err(MonochoraError::Config("Custom character set cannot exceed 256 characters".to_string()));
            }
        }
        
        Ok(())
    }

    fn get_charset(&self) -> &[char] {
        if let Some(custom) = &self.custom_charset {
            custom.as_slice()
        } else if self.detailed {
            DETAILED_CHARS
        } else {
            SIMPLE_CHARS
        }
    }
}

pub fn image_to_ascii<I>(image: &I, config: &AsciiConverterConfig) -> Result<Vec<String>>
where
    I: GenericImageView<Pixel = Rgba<u8>> + Sync,
{
    config.validate()?;
    
    let chars = config.get_charset();
    
    let (img_width, img_height) = image.dimensions();
    if img_width == 0 || img_height == 0 {
        return Err(MonochoraError::InvalidDimensions { width: img_width, height: img_height });
    }
    
    let (target_width, target_height) = calculate_target_dimensions(
        img_width, 
        img_height, 
        config
    )?;
    
    if target_width == 0 || target_height == 0 {
        return Err(MonochoraError::InvalidDimensions { width: target_width, height: target_height });
    }
    
    let result: Result<Vec<String>> = (0..target_height)
        .into_par_iter()
        .map(|y| {
            let mut line = String::with_capacity(target_width as usize);
            
            for x in 0..target_width {
                let img_x = ((x as f64 / target_width as f64) * img_width as f64) as u32;
                let img_y = ((y as f64 / target_height as f64) * img_height as f64) as u32;
                
                let img_x = img_x.min(img_width.saturating_sub(1));
                let img_y = img_y.min(img_height.saturating_sub(1));
                
                let pixel = image.get_pixel(img_x, img_y);
                let [r, g, b, a] = pixel.0;
                
                if a == 0 {
                    line.push(' ');
                    continue;
                }
                
                let brightness = calculate_brightness(r, g, b);
                let brightness = if config.invert { 1.0 - brightness } else { brightness };
                
                let char_index = calculate_char_index(brightness, chars.len());
                let ascii_char = chars.get(char_index)
                    .copied()
                    .unwrap_or(' '); 
                
                line.push(ascii_char);
            }
            
            Ok(line)
        })
        .collect();
    
    result
}

pub fn image_to_colored_ascii<I>(image: &I, config: &AsciiConverterConfig) -> Result<Vec<String>>
where
    I: GenericImageView<Pixel = Rgba<u8>> + Sync,
{
    config.validate()?;
    
    let chars = config.get_charset();
    
    let (img_width, img_height) = image.dimensions();
    if img_width == 0 || img_height == 0 {
        return Err(MonochoraError::InvalidDimensions { width: img_width, height: img_height });
    }
    
    let (target_width, target_height) = calculate_target_dimensions(
        img_width, 
        img_height, 
        config
    )?;
    
    if target_width == 0 || target_height == 0 {
        return Err(MonochoraError::InvalidDimensions { width: target_width, height: target_height });
    }
    
    let result: Result<Vec<String>> = (0..target_height)
        .into_par_iter()
        .map(|y| {
            let mut line = String::new();
            
            for x in 0..target_width {
                let img_x = ((x as f64 / target_width as f64) * img_width as f64) as u32;
                let img_y = ((y as f64 / target_height as f64) * img_height as f64) as u32;
                
                let img_x = img_x.min(img_width.saturating_sub(1));
                let img_y = img_y.min(img_height.saturating_sub(1));
                
                let pixel = image.get_pixel(img_x, img_y);
                let [r, g, b, a] = pixel.0;
                
                if a == 0 {
                    line.push(' ');
                    continue;
                }
                
                let brightness = calculate_brightness(r, g, b);
                let brightness = if config.invert { 1.0 - brightness } else { brightness };
                
                let char_index = calculate_char_index(brightness, chars.len());
                let ascii_char = chars.get(char_index)
                    .copied()
                    .unwrap_or(' '); 
                
                line.push_str(&format!("\x1b[38;2;{};{};{}m{}", r, g, b, ascii_char));
            }
            
            line.push_str("\x1b[0m");
            Ok(line)
        })
        .collect();
    
    result
}

fn calculate_brightness(r: u8, g: u8, b: u8) -> f32 {
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0
}

fn calculate_char_index(brightness: f32, chars_len: usize) -> usize {
    if chars_len == 0 {
        return 0;
    }
    
    let index = (brightness * (chars_len - 1) as f32).round() as usize;
    index.min(chars_len - 1) 
}

fn calculate_target_dimensions(
    img_width: u32, 
    img_height: u32, 
    config: &AsciiConverterConfig
) -> Result<(u32, u32)> {
    if img_width == 0 || img_height == 0 {
        return Err(MonochoraError::InvalidDimensions { width: img_width, height: img_height });
    }
    
    if let Some(scale) = config.scale_factor {
        if scale <= 0.0 {
            return Err(MonochoraError::Config("Scale factor must be positive".to_string()));
        }
        
        let scaled_width = (img_width as f32 * scale).max(1.0) as u32;
        let scaled_height = (img_height as f32 * scale / config.char_aspect).max(1.0) as u32;
        return Ok((scaled_width, scaled_height));
    }
    
    if let (Some(width), Some(height)) = (config.width, config.height) {
        if width == 0 || height == 0 {
            return Err(MonochoraError::InvalidDimensions { width, height });
        }
        return Ok((width, height));
    }
    
    if let Some(width) = config.width {
        if width == 0 {
            return Err(MonochoraError::InvalidDimensions { width, height: 0 });
        }
        
        let height = if config.preserve_aspect_ratio {
            let calculated_height = (width as f32 * img_height as f32 / img_width as f32 / config.char_aspect).max(1.0) as u32;
            calculated_height
        } else {
            (img_height as f32 / config.char_aspect).max(1.0) as u32
        };
        return Ok((width, height));
    }
    
    if let Some(height) = config.height {
        if height == 0 {
            return Err(MonochoraError::InvalidDimensions { width: 0, height });
        }
        
        let width = if config.preserve_aspect_ratio {
            let calculated_width = (height as f32 * img_width as f32 / img_height as f32 * config.char_aspect).max(1.0) as u32;
            calculated_width
        } else {
            img_width
        };
        return Ok((width, height));
    }
    
    let target_width = img_width;
    let target_height = if config.preserve_aspect_ratio {
        (img_height as f32 / config.char_aspect).max(1.0) as u32
    } else {
        img_height
    };
    
    Ok((target_width, target_height))
}
