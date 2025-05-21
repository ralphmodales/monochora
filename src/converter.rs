use image::{GenericImageView, Rgba};

static SIMPLE_CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
static DETAILED_CHARS: &[char] = &[
    ' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~', '+', '_', '-',
    '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v',
    'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k',
    'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@'
];

pub struct AsciiConverterConfig {
    pub width: Option<u32>,        
    pub char_aspect: f32,         
    pub invert: bool,            
    pub detailed: bool,         
}

impl Default for AsciiConverterConfig {
    fn default() -> Self {
        Self {
            width: None,           // Default: use terminal width or image width
            char_aspect: 0.5,      // Default aspect ratio for terminal chars
            invert: false,         // Default: don't invert brightness
            detailed: true,        // Default: use detailed character set
        }
    }
}

pub fn image_to_ascii<I>(image: &I, config: &AsciiConverterConfig) -> Vec<String> 
where
    I: GenericImageView<Pixel = Rgba<u8>>,
{
    let chars = if config.detailed { DETAILED_CHARS } else { SIMPLE_CHARS };
    
    let (img_width, img_height) = image.dimensions();
    let target_width = config.width.unwrap_or(img_width);
    let target_height = ((target_width as f32) * ((img_height as f32) / (img_width as f32)) / config.char_aspect) as u32;
    
    let mut result = Vec::with_capacity(target_height as usize);
    
    for y in 0..target_height {
        let mut line = String::with_capacity(target_width as usize);
        
        for x in 0..target_width {
            let img_x = (x as f32 / target_width as f32 * img_width as f32) as u32;
            let img_y = (y as f32 / target_height as f32 * img_height as f32) as u32;
            
            let pixel = image.get_pixel(img_x, img_y);
            let [r, g, b, a] = pixel.0;
            
            if a == 0 {
                line.push(' ');
                continue;
            }
            
            let brightness = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
            
            let brightness = if config.invert { 1.0 - brightness } else { brightness };
            
            let index = (brightness * (chars.len() - 1) as f32).round() as usize;
            let ascii_char = chars[index];
            
            line.push(ascii_char);
        }
        
        result.push(line);
    }
    
    result
}

pub fn image_to_colored_ascii<I>(image: &I, config: &AsciiConverterConfig) -> Vec<String> 
where
    I: GenericImageView<Pixel = Rgba<u8>>,
{
    let chars = if config.detailed { DETAILED_CHARS } else { SIMPLE_CHARS };
    
    let (img_width, img_height) = image.dimensions();
    let target_width = config.width.unwrap_or(img_width);
    let target_height = ((target_width as f32) * ((img_height as f32) / (img_width as f32)) / config.char_aspect) as u32;
    
    let mut result = Vec::with_capacity(target_height as usize);
    
    for y in 0..target_height {
        let mut line = String::new();
        
        for x in 0..target_width {
            let img_x = (x as f32 / target_width as f32 * img_width as f32) as u32;
            let img_y = (y as f32 / target_height as f32 * img_height as f32) as u32;
            
            let pixel = image.get_pixel(img_x, img_y);
            let [r, g, b, a] = pixel.0;
            
            if a == 0 {
                line.push(' ');
                continue;
            }
            
            let brightness = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
            
            let brightness = if config.invert { 1.0 - brightness } else { brightness };
            
            let index = (brightness * (chars.len() - 1) as f32).round() as usize;
            let ascii_char = chars[index];
            
            line.push_str(&format!("\x1b[38;2;{};{};{}m{}", r, g, b, ascii_char));
        }
        
        // Reset color at end of line
        line.push_str("\x1b[0m");
        result.push(line);
    }
    
    result
}
