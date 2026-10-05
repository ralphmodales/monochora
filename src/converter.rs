use image::{GenericImageView, Rgba};
use rayon::prelude::*;
use crate::{MonochoraError, Result};

const COLORED_CHAR_CAPACITY: usize = 20;

static SIMPLE_CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
static DETAILED_CHARS: &[char] = &[
    ' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~', '+', '_', '-',
    '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v',
    'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k',
    'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@'
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DitheringAlgorithm {
    None,
    FloydSteinberg,
    Atkinson,
    Jarvis,
    Stucki,
    Burkes,
    Sierra,
    TwoRowSierra,
    SierraLite,
}

impl Default for DitheringAlgorithm {
    fn default() -> Self {
        Self::None
    }
}

impl std::str::FromStr for DitheringAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "floyd-steinberg" | "floyd" => Ok(Self::FloydSteinberg),
            "atkinson" => Ok(Self::Atkinson),
            "jarvis" => Ok(Self::Jarvis),
            "stucki" => Ok(Self::Stucki),
            "burkes" => Ok(Self::Burkes),
            "sierra" => Ok(Self::Sierra),
            "two-row-sierra" | "sierra2" => Ok(Self::TwoRowSierra),
            "sierra-lite" | "sierralite" => Ok(Self::SierraLite),
            _ => Err(format!("Unknown dithering algorithm: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DitheringKernel {
    dx: i32,
    dy: i32,
    weight: f32,
}

fn get_dithering_kernel(algorithm: DitheringAlgorithm) -> Vec<DitheringKernel> {
    match algorithm {
        DitheringAlgorithm::None => vec![],
        
         DitheringAlgorithm::FloydSteinberg => vec![
            DitheringKernel { dx: 1, dy: 0, weight: 7.0/16.0 },
            DitheringKernel { dx: -1, dy: 1, weight: 3.0/16.0 },
            DitheringKernel { dx: 0, dy: 1, weight: 5.0/16.0 },
            DitheringKernel { dx: 1, dy: 1, weight: 1.0/16.0 },
        ],
        
        DitheringAlgorithm::Atkinson => vec![
            DitheringKernel { dx: 1, dy: 0, weight: 1.0/8.0 },
            DitheringKernel { dx: 2, dy: 0, weight: 1.0/8.0 },
            DitheringKernel { dx: -1, dy: 1, weight: 1.0/8.0 },
            DitheringKernel { dx: 0, dy: 1, weight: 1.0/8.0 },
            DitheringKernel { dx: 1, dy: 1, weight: 1.0/8.0 },
            DitheringKernel { dx: 0, dy: 2, weight: 1.0/8.0 },
        ],
        
        DitheringAlgorithm::Jarvis => vec![
            DitheringKernel { dx: 1, dy: 0, weight: 7.0/48.0 },
            DitheringKernel { dx: 2, dy: 0, weight: 5.0/48.0 },
            DitheringKernel { dx: -2, dy: 1, weight: 3.0/48.0 },
            DitheringKernel { dx: -1, dy: 1, weight: 5.0/48.0 },
            DitheringKernel { dx: 0, dy: 1, weight: 7.0/48.0 },
            DitheringKernel { dx: 1, dy: 1, weight: 5.0/48.0 },
            DitheringKernel { dx: 2, dy: 1, weight: 3.0/48.0 },
            DitheringKernel { dx: -2, dy: 2, weight: 1.0/48.0 },
            DitheringKernel { dx: -1, dy: 2, weight: 3.0/48.0 },
            DitheringKernel { dx: 0, dy: 2, weight: 5.0/48.0 },
            DitheringKernel { dx: 1, dy: 2, weight: 3.0/48.0 },
            DitheringKernel { dx: 2, dy: 2, weight: 1.0/48.0 },
        ],
        
        DitheringAlgorithm::Stucki => vec![
            DitheringKernel { dx: 1, dy: 0, weight: 8.0/42.0 },
            DitheringKernel { dx: 2, dy: 0, weight: 4.0/42.0 },
            DitheringKernel { dx: -2, dy: 1, weight: 2.0/42.0 },
            DitheringKernel { dx: -1, dy: 1, weight: 4.0/42.0 },
            DitheringKernel { dx: 0, dy: 1, weight: 8.0/42.0 },
            DitheringKernel { dx: 1, dy: 1, weight: 4.0/42.0 },
            DitheringKernel { dx: 2, dy: 1, weight: 2.0/42.0 },
            DitheringKernel { dx: -2, dy: 2, weight: 1.0/42.0 },
            DitheringKernel { dx: -1, dy: 2, weight: 2.0/42.0 },
            DitheringKernel { dx: 0, dy: 2, weight: 4.0/42.0 },
            DitheringKernel { dx: 1, dy: 2, weight: 2.0/42.0 },
            DitheringKernel { dx: 2, dy: 2, weight: 1.0/42.0 },
        ],
        
        DitheringAlgorithm::Burkes => vec![
            DitheringKernel { dx: 1, dy: 0, weight: 8.0/32.0 },
            DitheringKernel { dx: 2, dy: 0, weight: 4.0/32.0 },
            DitheringKernel { dx: -2, dy: 1, weight: 2.0/32.0 },
            DitheringKernel { dx: -1, dy: 1, weight: 4.0/32.0 },
            DitheringKernel { dx: 0, dy: 1, weight: 8.0/32.0 },
            DitheringKernel { dx: 1, dy: 1, weight: 4.0/32.0 },
            DitheringKernel { dx: 2, dy: 1, weight: 2.0/32.0 },
        ],
        
        DitheringAlgorithm::Sierra => vec![
            DitheringKernel { dx: 1, dy: 0, weight: 5.0/32.0 },
            DitheringKernel { dx: 2, dy: 0, weight: 3.0/32.0 },
            DitheringKernel { dx: -2, dy: 1, weight: 2.0/32.0 },
            DitheringKernel { dx: -1, dy: 1, weight: 4.0/32.0 },
            DitheringKernel { dx: 0, dy: 1, weight: 5.0/32.0 },
            DitheringKernel { dx: 1, dy: 1, weight: 4.0/32.0 },
            DitheringKernel { dx: 2, dy: 1, weight: 2.0/32.0 },
            DitheringKernel { dx: -1, dy: 2, weight: 2.0/32.0 },
            DitheringKernel { dx: 0, dy: 2, weight: 3.0/32.0 },
            DitheringKernel { dx: 1, dy: 2, weight: 2.0/32.0 },
        ],
        
        DitheringAlgorithm::TwoRowSierra => vec![
            DitheringKernel { dx: 1, dy: 0, weight: 4.0/16.0 },
            DitheringKernel { dx: 2, dy: 0, weight: 3.0/16.0 },
            DitheringKernel { dx: -2, dy: 1, weight: 1.0/16.0 },
            DitheringKernel { dx: -1, dy: 1, weight: 2.0/16.0 },
            DitheringKernel { dx: 0, dy: 1, weight: 3.0/16.0 },
            DitheringKernel { dx: 1, dy: 1, weight: 2.0/16.0 },
            DitheringKernel { dx: 2, dy: 1, weight: 1.0/16.0 },
        ],
        
        DitheringAlgorithm::SierraLite => vec![
            DitheringKernel { dx: 1, dy: 0, weight: 2.0/4.0 },
            DitheringKernel { dx: -1, dy: 1, weight: 1.0/4.0 },
            DitheringKernel { dx: 0, dy: 1, weight: 1.0/4.0 },
        ],
    }
}

struct ErrorBuffer {
    width: u32,
    height: u32,
    current_y: u32,
    rows: [Vec<f32>; 3],
}

impl ErrorBuffer {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            current_y: 0,
            rows: std::array::from_fn(|_| vec![0.0; width as usize]),
        }
    }
    
    fn add_error(&mut self, x: u32, y: u32, error: f32) {
        if x < self.width && y < self.height && y >= self.current_y {
            if let Some(row) = self.rows.get_mut((y - self.current_y) as usize) {
                row[x as usize] += error;
            }
        }
    }
    
    fn get_error(&self, x: u32, y: u32) -> f32 {
        if y < self.current_y {
            return 0.0;
        }
        self.rows
            .get((y - self.current_y) as usize)
            .and_then(|row| row.get(x as usize))
            .copied()
            .unwrap_or(0.0)
    }
    
    fn next_row(&mut self) {
        self.rows.rotate_left(1);
        self.rows[2].fill(0.0);
        self.current_y += 1;
    }
}

pub fn image_to_ascii_with_dithering<I>(
    image: &I, 
    config: &AsciiConverterConfig
) -> Result<Vec<String>>
where
    I: GenericImageView<Pixel = Rgba<u8>> + Sync,
{
    config.validate()?;
    
    let chars = config.get_charset();
    let dithering = config.dithering_algorithm.unwrap_or(DitheringAlgorithm::None);
    
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

    if dithering == DitheringAlgorithm::None {
        return image_to_ascii(image, config);
    }

    let kernel = get_dithering_kernel(dithering);
    let columns = source_columns(target_width, img_width);
    let mut error_buffer = ErrorBuffer::new(target_width, target_height);
    let mut result = Vec::with_capacity(target_height as usize);
    
    for y in 0..target_height {
        let mut line = String::with_capacity(target_width as usize);
        let img_y = source_row(y, target_height, img_height);
        
        for (x, &img_x) in (0..target_width).zip(columns.iter()) {
            let pixel = image.get_pixel(img_x, img_y);
            let [r, g, b, a] = pixel.0;
            
            if a == 0 {
                line.push(' ');
                continue;
            }
            
            let mut brightness = calculate_brightness(r, g, b);
            if config.invert {
                brightness = 1.0 - brightness;
            }
            
            let accumulated_error = error_buffer.get_error(x, y);
            brightness = (brightness + accumulated_error).clamp(0.0, 1.0);
            
            let char_index = calculate_char_index(brightness, chars.len());
            let ascii_char = chars.get(char_index).copied().unwrap_or(' ');
            
            let target_brightness = char_index as f32 / (chars.len() - 1).max(1) as f32;
            
            let error = brightness - target_brightness;
            
            for kernel_entry in &kernel {
                let nx = x as i32 + kernel_entry.dx;
                let ny = y as i32 + kernel_entry.dy;
                
                if nx >= 0 && ny >= 0 {
                    let distributed_error = error * kernel_entry.weight;
                    error_buffer.add_error(nx as u32, ny as u32, distributed_error);
                }
            }
            
            line.push(ascii_char);
        }
        
        error_buffer.next_row();
        result.push(line);
    }
    
    Ok(result)
}

pub fn image_to_colored_ascii_with_dithering<I>(
    image: &I, 
    config: &AsciiConverterConfig
) -> Result<Vec<String>>
where
    I: GenericImageView<Pixel = Rgba<u8>> + Sync,
{
    config.validate()?;
    
    let chars = config.get_charset();
    let dithering = config.dithering_algorithm.unwrap_or(DitheringAlgorithm::None);
    
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

    if dithering == DitheringAlgorithm::None {
        return image_to_colored_ascii(image, config);
    }

    let kernel = get_dithering_kernel(dithering);
    let columns = source_columns(target_width, img_width);
    let mut error_buffer = ErrorBuffer::new(target_width, target_height);
    let mut result = Vec::with_capacity(target_height as usize);
    
    for y in 0..target_height {
        let mut line = String::with_capacity(target_width as usize * COLORED_CHAR_CAPACITY);
        let img_y = source_row(y, target_height, img_height);
        let mut last_color = None;
        
        for (x, &img_x) in (0..target_width).zip(columns.iter()) {
            let pixel = image.get_pixel(img_x, img_y);
            let [r, g, b, a] = pixel.0;
            
            if a == 0 {
                line.push(' ');
                continue;
            }
            
            let mut brightness = calculate_brightness(r, g, b);
            if config.invert {
                brightness = 1.0 - brightness;
            }
            
            let accumulated_error = error_buffer.get_error(x, y);
            brightness = (brightness + accumulated_error).clamp(0.0, 1.0);
            
            let char_index = calculate_char_index(brightness, chars.len());
            let ascii_char = chars.get(char_index).copied().unwrap_or(' ');
            
            let target_brightness = char_index as f32 / (chars.len() - 1).max(1) as f32;
            let error = brightness - target_brightness;
            
            for kernel_entry in &kernel {
                let nx = x as i32 + kernel_entry.dx;
                let ny = y as i32 + kernel_entry.dy;
                
                if nx >= 0 && ny >= 0 {
                    let distributed_error = error * kernel_entry.weight;
                    error_buffer.add_error(nx as u32, ny as u32, distributed_error);
                }
            }
            
            push_colored_char(&mut line, &mut last_color, [r, g, b], ascii_char);
        }
        
        line.push_str("\x1b[0m");
        line.shrink_to_fit();
        error_buffer.next_row();
        result.push(line);
    }
    
    Ok(result)
}

pub fn list_dithering_algorithms() {
    println!("Available Dithering Algorithms:\n");
    
    println!("Basic:");
    println!("  none           No dithering (fastest, default)");
    println!("  floyd-steinberg Classic Floyd-Steinberg error diffusion");
    println!("  sierra-lite    Minimal Sierra dithering (fast)");
    
    println!("\nAdvanced:");
    println!("  atkinson       Atkinson dithering (good for high contrast)");
    println!("  sierra         Full Sierra dithering");
    println!("  two-row-sierra Simplified Sierra (2 rows)");
    println!("  burkes         Burkes error diffusion");
    println!("  stucki         Stucki error diffusion");
    println!("  jarvis         Jarvis-Judice-Ninke (smoothest gradients)");
    
    println!("\nCharacteristics:");
    println!("  • floyd-steinberg: Best general-purpose dithering");
    println!("  • atkinson: Reduces 'bleeding', good for sharp images");
    println!("  • jarvis: Smoothest gradients, more computation");
    println!("  • sierra-lite: Good balance of quality and speed");
    println!("  • none: No dithering, fastest processing");
    
    println!("\nUsage:");
    println!("  --dither floyd-steinberg");
    println!("  --dither atkinson");
    println!("  --list-dithering");
}

#[repr(C)]
#[derive(Clone)]
pub struct AsciiConverterConfig {
    pub width: Option<u32>,        
    pub height: Option<u32>,       
    pub char_aspect: f32,         
    pub invert: bool,            
    pub detailed: bool,
    pub preserve_aspect_ratio: bool, 
    pub scale_factor: Option<f32>,
    pub custom_charset: Option<Vec<char>>,
    pub dithering_algorithm: Option<DitheringAlgorithm>
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
            dithering_algorithm: None,
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

        if let Some(_dithering) = self.dithering_algorithm {
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
    
    let columns = source_columns(target_width, img_width);
    
    let result: Result<Vec<String>> = (0..target_height)
        .into_par_iter()
        .map(|y| {
            let mut line = String::with_capacity(target_width as usize);
            let img_y = source_row(y, target_height, img_height);
            
            for &img_x in &columns {
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
    
    let columns = source_columns(target_width, img_width);
    
    let result: Result<Vec<String>> = (0..target_height)
        .into_par_iter()
        .map(|y| {
            let mut line = String::with_capacity(target_width as usize * COLORED_CHAR_CAPACITY);
            let img_y = source_row(y, target_height, img_height);
            let mut last_color = None;
            
            for &img_x in &columns {
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
                
                push_colored_char(&mut line, &mut last_color, [r, g, b], ascii_char);
            }
            
            line.push_str("\x1b[0m");
            line.shrink_to_fit();
            Ok(line)
        })
        .collect();
    
    result
}

fn source_columns(target_width: u32, img_width: u32) -> Vec<u32> {
    (0..target_width)
        .map(|x| {
            let img_x = ((x as f64 / target_width as f64) * img_width as f64) as u32;
            img_x.min(img_width.saturating_sub(1))
        })
        .collect()
}

fn source_row(y: u32, target_height: u32, img_height: u32) -> u32 {
    let img_y = ((y as f64 / target_height as f64) * img_height as f64) as u32;
    img_y.min(img_height.saturating_sub(1))
}

fn push_u8(line: &mut String, value: u8) {
    if value >= 100 {
        line.push((b'0' + value / 100) as char);
    }
    if value >= 10 {
        line.push((b'0' + value / 10 % 10) as char);
    }
    line.push((b'0' + value % 10) as char);
}

fn push_colored_char(line: &mut String, last_color: &mut Option<[u8; 3]>, color: [u8; 3], ch: char) {
    if *last_color != Some(color) {
        let [r, g, b] = color;
        line.push_str("\x1b[38;2;");
        push_u8(line, r);
        line.push(';');
        push_u8(line, g);
        line.push(';');
        push_u8(line, b);
        line.push('m');
        *last_color = Some(color);
    }
    line.push(ch);
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
        let scaled_height = (img_height as f32 * scale * config.char_aspect).max(1.0) as u32;
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
            let calculated_height = (width as f32 * img_height as f32 / img_width as f32 * config.char_aspect).max(1.0) as u32;
            calculated_height
        } else {
            (img_height as f32 * config.char_aspect).max(1.0) as u32
        };
        return Ok((width, height));
    }
    
    if let Some(height) = config.height {
        if height == 0 {
            return Err(MonochoraError::InvalidDimensions { width: 0, height });
        }
        
        let width = if config.preserve_aspect_ratio {
            let calculated_width = (height as f32 * img_width as f32 / img_height as f32 / config.char_aspect).max(1.0) as u32;
            calculated_width
        } else {
            img_width
        };
        return Ok((width, height));
    }
    
    let target_width = img_width;
    let target_height = if config.preserve_aspect_ratio {
        (img_height as f32 * config.char_aspect).max(1.0) as u32
    } else {
        img_height
    };
    
    Ok((target_width, target_height))
}

pub fn fit_dimensions(
    img_width: u32,
    img_height: u32,
    max_width: u32,
    max_height: u32,
    config: &AsciiConverterConfig,
) -> (u32, u32) {
    let max_width = max_width.max(1);
    let max_height = max_height.max(1);
    
    if !config.preserve_aspect_ratio || img_width == 0 || img_height == 0 || config.char_aspect <= 0.0 {
        return (max_width, max_height);
    }
    
    let height_at_max_width = max_width as f32 * img_height as f32 / img_width as f32 * config.char_aspect;
    if height_at_max_width <= max_height as f32 {
        return (max_width, (height_at_max_width as u32).max(1));
    }
    
    let width_at_max_height = (max_height as f32 * img_width as f32 / img_height as f32 / config.char_aspect) as u32;
    (width_at_max_height.clamp(1, max_width), max_height)
}
