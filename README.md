# Monochora

Monochora is a GIF to ASCII art converter written in Rust. It can transform GIF animations into playable ASCII animations in your terminal or save them as ASCII art files or even convert them back to colored GIF animations with ASCII characters.

## Features

- **High-performance parallel processing** - Multi-threaded conversion for faster processing
- Convert animated GIFs to ASCII art animations
- **Support for both local files and URLs** - Download GIFs directly from the web
- Play the animations directly in your terminal
- Save animations as text files or high-quality ASCII GIF files
- Support for colored ASCII art (with ANSI color codes)
- **Customizable character sets** - Built-in sets, inline strings, or custom files
- Multiple output options (terminal, text file, or GIF output)
- **Advanced GIF output features**:
  - Adaptive color palettes for better quality
  - Font size optimization for different scales
  - Precision quantization for small fonts
  - Custom background and text colors
- Preserve original GIF dimensions and aspect ratios
- Flexible scaling and dimension control
- **Intelligent dimension calculation** - Proper aspect ratio preservation and character scaling
- **Quiet mode** for batch processing
- **Comprehensive error handling and validation**
- **Configurable logging levels** for debugging and monitoring

## Installation

### Using Cargo (Recommended)

The easiest way to install Monochora is using Cargo:

```bash
cargo install monochora
```

### From Source

1. Make sure you have Rust and Cargo installed. If not, install it from [rust-lang.org](https://www.rust-lang.org/tools/install).

2. Clone the repository:
   ```bash
   git clone https://github.com/ralphmodales/monochora.git
   cd monochora
   ```

3. Build and install:
   ```bash
   cargo install --path .
   ```

   Or just build without installing:
   ```bash
   cargo build --release
   ```

4. If you chose to just build, the binary will be available at `target/release/monochora`

## Usage

```bash
# Basic usage with local file (plays in terminal)
monochora -i input.gif

# Basic usage with URL - downloads and converts automatically
monochora -i https://example.com/animation.gif

# Save as ASCII text file
monochora -i input.gif -s

# Save to specific output file
monochora -i input.gif -o my_ascii.txt

# Generate colored ASCII in terminal
monochora -i input.gif -c

# Download from URL and generate colored ASCII
monochora -i "https://giffiles.alphacoders.com/220/220890.gif" -c -w 200 -H 100

# Invert brightness
monochora -i input.gif -v

# Use simple character set
monochora -i input.gif -p

# Use custom character set (inline)
monochora -i input.gif --charset " ·∘○●◉"

# Use custom character set from file
monochora -i input.gif --charset-file ./my-chars.txt

# List available character sets
monochora --list-charsets

# Save as high-quality ASCII GIF animation
monochora -i input.gif --gif-output output.gif

# Generate GIF with default name
monochora -i input.gif --gif-output

# Custom width (height calculated automatically to preserve aspect ratio)
monochora -i input.gif -w 100

# Custom width and height
monochora -i input.gif -w 100 -H 50

# Scale the original dimensions (0.5 = half size, 2.0 = double size)
monochora -i input.gif --scale 0.5

# Fit to terminal width 
monochora -i input.gif --fit-terminal

# Disable aspect ratio preservation
monochora -i input.gif -w 100 --preserve-aspect false

# White text on black background for GIF output
monochora -i input.gif --gif-output output.gif --white-on-black

# Black text on white background for GIF output
monochora -i input.gif --gif-output output.gif --black-on-white

# Custom font size for GIF output (optimized palettes)
monochora -i input.gif --gif-output output.gif --font-size 20

# High-performance processing with custom thread count
monochora -i input.gif --threads 8

# Quiet mode for batch processing
monochora -i input.gif -q --gif-output output.gif

# Debug with detailed logging
monochora -i input.gif --log-level debug

# Download from URL and save as high-quality ASCII GIF
monochora -i "https://example.com/cool.gif" --gif-output result.gif --black-on-white --font-size 16
```

## Options

```
Options:
  -i, --input <INPUT>                    Input GIF file path or URL (supports HTTP/HTTPS)
  -o, --output <OUTPUT>                  Output file path for text files
  -w, --width <WIDTH>                    Target width in characters
  -H, --height <HEIGHT>                  Target height in characters
  -c, --colored                          Use colored ASCII (ANSI colors)
  -v, --invert                           Invert brightness
  -p, --simple                           Use simple character set
  -s, --save                             Save to text file instead of playing
      --gif-output [<GIF_OUTPUT>]        Output as ASCII GIF file (optional path)
      --font-size <FONT_SIZE>            Font size for GIF output [default: 14.0]
      --white-on-black                   White text on black background for GIF
      --black-on-white                   Black text on white background for GIF
      --fit-terminal                     Fit ASCII art to terminal width
      --scale <SCALE>                    Scale factor for original dimensions
      --preserve-aspect <PRESERVE_ASPECT> Preserve original aspect ratio [default: true]
      --threads <THREADS>                Number of threads for parallel processing
      --charset <CHARSET>                Custom character set string (ordered darkest to lightest)
      --charset-file <CHARSET_FILE>      Path to custom character set file
      --list-charsets                    List available character sets and exit
  -q, --quiet                            Suppress progress output
      --log-level <LOG_LEVEL>            Log level (error, warn, info, debug, trace) [default: info]
  -h, --help                             Print help
  -V, --version                          Print version
```

## Character Sets

Monochora offers flexible character set options for different artistic styles and use cases:

### Built-in Character Sets

- **Simple** (`--simple`): ` .:-=+*#%@` - Basic 10-character set for clean, fast conversion
- **Detailed** (default): Full 70-character set with fine gradations for high-quality output

### Custom Character Sets

Create your own character palettes for specialized needs:

```bash
# Artistic styles
monochora -i art.gif --charset " ·∘○●◉"          # Geometric circles
monochora -i photo.jpg --charset " ░▒▓█"         # Block characters
monochora -i sketch.gif --charset " .-+*#"       # Technical style

# Cultural/linguistic
monochora -i anime.gif --charset "・〆ヲァィヵヶ"  # Japanese characters
monochora -i mandala.gif --charset " ༄༅༆༇༈"    # Tibetan symbols

# Specialized applications  
monochora -i xray.png --charset " .-+*#%@@"      # Medical imaging
monochora -i diagram.gif --charset "⠀⠁⠂⠃⠄⠅⠆⠇"  # Braille patterns
```

### Character Set Files

Store reusable character sets in text files:

```bash
# Create a character set file
echo " .-+*#%@@" > density.txt
monochora -i image.gif --charset-file density.txt

# Organize sets by category
mkdir palettes
echo " ·∘○●◉" > palettes/geometric.txt
echo " ༄༅༆༇༈" > palettes/tibetan.txt
monochora -i input.gif --charset-file palettes/geometric.txt
```

### Character Set Rules

- **Order**: Characters must be ordered from darkest to lightest
- **Length**: Minimum 2 characters, maximum 256 characters
- **Uniqueness**: All characters must be unique
- **Content**: No control characters (except tab/newline in files)
- **Unicode**: Full UTF-8 support for international characters

### Listing Available Sets

Use `--list-charsets` to see examples and usage:

```bash
monochora --list-charsets
```

## Important Notes

### Output Mode Restrictions

Monochora enforces exclusive output modes to avoid conflicts:

- **Terminal display**: Default mode when no output options are specified
- **Text file output**: Use `--save` or `--output <file>`
- **GIF output**: Use `--gif-output [path]`

**You cannot combine multiple output modes in a single command.**

### Character Set Restrictions

Character set options are mutually exclusive:

- **Built-in sets**: `--simple` (default uses detailed set)
- **Inline custom**: `--charset "characters"`
- **File-based custom**: `--charset-file path.txt`

**You cannot combine multiple character set options in a single command.**

### Background Color Options

- `--white-on-black` and `--black-on-white` can only be used with `--gif-output`
- These options are mutually exclusive
- Without these flags, GIF output uses default colors (white text on black background)

### Terminal Fitting

- `--fit-terminal` only works with terminal display mode
- Cannot be used with file output options (`--save`, `--output`, `--gif-output`)

### Font Size

- `--font-size` only applies to GIF output mode
- Valid range: 0.1 to 100.0
- Default: 14.0 for optimal quality/performance balance

## Performance Features

Monochora is optimized for high-performance processing:

- **Parallel frame processing** using Rayon for multi-core utilization
- **Configurable thread pools** - Set custom thread counts for your hardware
- **Optimized memory usage** - Efficient handling of large GIF files
- **Adaptive algorithms** - Different processing strategies based on font size and output type
- **Progress tracking** - Real-time feedback on conversion progress (unless in quiet mode)
- **Comprehensive validation** - Input validation to prevent runtime errors

### Performance Examples

```bash
# Use all CPU cores for maximum speed
monochora -i large_animation.gif --threads 16 --gif-output result.gif

# Quiet batch processing
monochora -i input.gif -q --gif-output output.gif --font-size 12

# High-performance colored terminal output
monochora -i animation.gif -c --threads 8

# Debug mode with detailed logging
monochora -i animation.gif --log-level debug --threads 4
```

## Advanced GIF Output

The ASCII GIF output feature includes several advanced optimizations:

### Adaptive Color Palettes
- **Dynamic palette generation** based on font size and colors
- **Precision-optimized quantization** for better text rendering
- **Font-size aware color steps** - More colors for smaller fonts, fewer for larger
- **Smart color variations** for enhanced text clarity

### Quality Settings by Font Size
- **Small fonts (< 2.0)**: 32 color steps with precision quantization
- **Medium fonts (2.0-6.0)**: 16 color steps with balanced quality
- **Large fonts (> 6.0)**: 8 color steps for optimal performance

### Examples

```bash
# High-quality small text (more colors, precision quantization)
monochora -i input.gif --gif-output high_quality.gif --font-size 8

# Optimized medium text (balanced quality/performance)  
monochora -i input.gif --gif-output medium.gif --font-size 14

# Fast large text (fewer colors, optimized for speed)
monochora -i input.gif --gif-output large.gif --font-size 24

# Custom styling with adaptive palettes
monochora -i input.gif --gif-output custom.gif --black-on-white --font-size 16

# Auto-generated output filename
monochora -i my_animation.gif --gif-output  # Creates ascii_my_animation.gif
```

## Examples

### Terminal Animation

To play an animation in your terminal:

```bash
monochora -i animation.gif
```

Press `q` or `Esc` to exit the animation.

### Custom Character Set Examples

Explore different artistic styles with custom character sets:

```bash
# Minimalist density gradient
monochora -i portrait.gif --charset " .oO@"

# Geometric progression
monochora -i abstract.gif --charset " ·∘○●◉" -c

# Technical documentation style
monochora -i diagram.png --charset " .-=+*#" -w 120

# High-contrast for readability
monochora -i screenshot.gif --charset " ░▓█" --fit-terminal

# Artistic braille output
monochora -i pattern.gif --charset "⠀⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋" --save
```

### URL Input Examples

Download and play a GIF from the internet:

```bash
# Simple terminal playback
monochora -i "https://media.giphy.com/media/3o7abKhOpu0NwenH3O/giphy.gif"

# Colored ASCII with custom dimensions and character set
monochora -i "https://example.com/animation.gif" -c -w 150 -H 75 --charset " ·∘○●"

# Download and save as high-quality ASCII GIF
monochora -i "https://example.com/source.gif" --gif-output converted_ascii.gif
```

### Colored Terminal Animation

```bash
monochora -i animation.gif -c
```

### Scale Animation

Scale the animation to half size while preserving aspect ratio:

```bash
monochora -i animation.gif --scale 0.5
```

### Fit to Terminal

Make the animation fit your terminal width:

```bash
monochora -i animation.gif --fit-terminal
```

### Custom Dimensions with Proper Character Scaling

The dimension system now properly accounts for character aspect ratios:

```bash
# Width-based scaling (height calculated for proper proportions)
monochora -i animation.gif -w 120

# Height-based scaling (width calculated for proper proportions)
monochora -i animation.gif -H 40

# Exact dimensions (may distort aspect ratio)
monochora -i animation.gif -w 120 -H 40 --preserve-aspect false
```

### Save as ASCII Text

```bash
# Save with default filename
monochora -i animation.gif -s

# Save to specific file
monochora -i animation.gif -o my_ascii_art.txt

# Save with custom character set
monochora -i animation.gif -s --charset " ░▒▓█"
```

### Save as High-Quality ASCII GIF

```bash
# Default output filename (ascii_animation.gif)
monochora -i animation.gif --gif-output

# Custom output filename with character set
monochora -i animation.gif --gif-output my_ascii.gif --charset " ·∘○●"
```

### Custom Style GIF with Optimization

```bash
# High-quality output with custom styling and character set
monochora -i animation.gif --gif-output result.gif --black-on-white --font-size 18 --charset " .-+*#"

# Performance-optimized batch processing
monochora -i animation.gif --gif-output result.gif --quiet --threads 12
```

### Large High-Quality ASCII GIF

Create a large, detailed ASCII GIF with optimized processing:

```bash
monochora -i small_animation.gif --gif-output large_result.gif --scale 2.0 --font-size 8 --threads 8
```

## Error Handling and Validation

Monochora includes comprehensive input validation:

### Dimension Validation
- Width and height must be between 1 and 10,000 characters
- Font size must be between 0.1 and 100.0
- Scale factor must be between 0.1 and 10.0
- Thread count must be between 1 and 1,000

### Character Set Validation
- Minimum 2 characters, maximum 256 characters
- All characters must be unique
- No control characters (except tab/newline in files)
- Proper UTF-8 encoding for Unicode characters

### Conflict Detection
- Prevents combining incompatible output modes
- Validates color scheme combinations
- Ensures options are used with appropriate output types
- Prevents conflicting character set options

### Common Error Messages
- **Invalid font size**: Font size out of valid range
- **Invalid dimensions**: Width or height out of bounds
- **Invalid character set**: Character set validation failed
- **Config error**: Conflicting or invalid option combinations
- **Thread pool error**: Issues with parallel processing setup

## URL Support

Monochora supports downloading GIFs directly from URLs:

- **Supported protocols**: HTTP and HTTPS
- **Automatic download**: Files are downloaded to temporary storage and cleaned up automatically
- **Content validation**: Warns if the URL doesn't serve image content
- **Timeout handling**: 30-second timeout for downloads
- **Progress indication**: Shows download progress and file size
- **Custom User-Agent**: Identifies as monochora-gif-converter

### URL Examples

```bash
# Download and convert a GIF from Giphy
monochora -i "https://media.giphy.com/media/example/giphy.gif" -c

# Download from any image hosting service
monochora -i "https://i.imgur.com/example.gif" -w 100

# Works with direct links to GIF files
monochora -i "https://example.com/path/to/animation.gif" --gif-output result.gif
```

## Dimension Control & Character Scaling

Monochora offers intelligent dimension control with proper character aspect ratio handling:

- **Default**: Uses original GIF dimensions with character aspect correction
- **Character aspect ratio**: Automatically accounts for the ~2:1 width-to-height ratio of monospace characters
- **--width only**: Sets width, calculates height to preserve image aspect ratio
- **--height only**: Sets height, calculates width to preserve image aspect ratio  
- **--width and --height**: Uses exact dimensions (may distort unless aspect preservation is disabled)
- **--scale**: Multiplies original dimensions by scale factor with character correction
- **--fit-terminal**: Fits to terminal width (when not saving to file)
- **--preserve-aspect false**: Disables automatic aspect ratio preservation

### Dimension Examples

```bash
# Preserve image proportions with character scaling
monochora -i wide_image.gif -w 160  # Height auto-calculated

# Scale with proper character proportions  
monochora -i image.gif --scale 1.5  # 150% size with character correction

# For GIF output, dimensions are calculated for target pixel size
monochora -i input.gif --gif-output result.gif -w 800 --font-size 12
```

## Library Usage

Monochora can also be used as a library in your Rust projects:

```rust
use monochora::{
    converter::{image_to_ascii, AsciiConverterConfig},
    handler::decode_gif,
    display::display_ascii_animation,
    output::{ascii_frames_to_gif_with_dimensions, AsciiGifOutputOptions},
    web::get_input_path,
};
use rayon::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Handle both local files and URLs
    let input_path = get_input_path("https://example.com/animation.gif").await?;
    
    // Decode the GIF
    let gif_data = decode_gif(&input_path)?;
    
    // Configure the converter with custom character set
    let custom_chars: Vec<char> = " ·∘○●◉".chars().collect();
    let config = AsciiConverterConfig {
        width: Some(80),
        height: None,
        char_aspect: 0.5,
        invert: false,
        detailed: true,
        preserve_aspect_ratio: true,
        scale_factor: Some(1.5), // 150% of original size
        custom_charset: Some(custom_chars),
    };
    
    // Convert frames to ASCII in parallel
    let results: Vec<(Vec<String>, u16)> = gif_data.frames
        .par_iter()
        .map(|frame| {
            let ascii_frame = image_to_ascii(&frame.image, &config);
            (ascii_frame, frame.delay_time_ms)
        })
        .collect();
    
    let (ascii_frames, frame_delays): (Vec<_>, Vec<_>) = results.into_iter().unzip();
    
    // Display the animation
    display_ascii_animation(&ascii_frames, &frame_delays, gif_data.loop_count, true).await?;
    
    // Or save as ASCII GIF
    let options = AsciiGifOutputOptions {
        font_size: 14.0,
        bg_color: image::Rgb([0, 0, 0]),
        text_color: image::Rgb([255, 255, 255]),
        ..Default::default()
    };
    
    ascii_frames_to_gif_with_dimensions(
        &ascii_frames,
        &frame_delays,
        gif_data.loop_count,
        "output.gif",
        &options,
        Some((800, 600))
    )?;
    
    Ok(())
}
```

## Debugging and Logging

Monochora includes comprehensive logging for debugging and monitoring:

### Log Levels
- **error**: Only critical errors
- **warn**: Warnings and errors
- **info**: General information, warnings, and errors (default)
- **debug**: Detailed processing information
- **trace**: Extremely verbose output for debugging

### Debugging Examples

```bash
# Debug mode for troubleshooting
monochora -i problematic.gif --log-level debug

# Trace mode for detailed analysis
monochora -i animation.gif --log-level trace --threads 4

# Error-only mode for production
monochora -i input.gif --log-level error --quiet
```

## How It Works

Monochora works by:

1. **Input validation**: Comprehensive validation of all command-line arguments
2. **Input handling**: Accepts both local file paths and URLs (HTTP/HTTPS)
3. **URL processing**: Downloads GIFs from URLs to temporary files when needed
4. **GIF decoding**: Decodes GIF frames using the `gif` crate with parallel processing
5. **Character set selection**: Chooses appropriate character set (built-in, custom inline, or file-based)
6. **ASCII conversion**: Converts each frame to ASCII art based on pixel brightness using parallel processing
7. **Dimension calculation**: Intelligently calculates dimensions with proper character aspect ratio handling
8. **Advanced output generation**: 
   - Terminal display and GIF output with color support
   - Text file output with frame separators
   - High-quality ASCII GIF generation with adaptive palettes

The ASCII conversion process maps pixel brightness to appropriate ASCII characters using either built-in character sets (simple or detailed) or custom user-defined sets. For colored output, it includes ANSI color codes for terminal display or renders characters with their original colors into a new GIF using advanced quantization techniques.

### Advanced GIF Generation Process

The ASCII GIF output uses several optimization techniques:

1. **Adaptive palette creation** based on font size and color scheme
2. **Precision quantization** with different algorithms for different font sizes
3. **Smart color distance calculation** optimized for text rendering
4. **Frame-by-frame parallel rendering** for improved performance
5. **Embedded font rendering** using DejaVu Sans Mono for consistent output

## Performance Characteristics

- **Multi-threaded processing**: Utilizes all available CPU cores by default
- **Memory efficient**: Streams processing to handle large GIF files
- **Scalable**: Performance improves with more CPU cores
- **Optimized algorithms**: Different processing strategies based on output type and quality settings

### Typical Performance

- **Small GIFs** (< 1MB): Near-instantaneous processing
- **Medium GIFs** (1-10MB): 1-5 seconds on modern hardware
- **Large GIFs** (10MB+): Scales linearly with thread count
- **Batch processing**: Quiet mode minimizes I/O overhead

## Dependencies

- `gif` - For GIF decoding and encoding
- `image` - For image manipulation  
- `imageproc` - For drawing text on images
- `rusttype` - For font rendering in GIF output
- `clap` - For command-line argument parsing
- `crossterm` - For terminal manipulation and event handling
- `anyhow` - For comprehensive error handling
- `tokio` - For asynchronous operations and URL handling
- `reqwest` - For HTTP downloads with timeout support
- `url` - For URL parsing and validation
- `tempfile` - For secure temporary file management
- `rayon` - For high-performance parallel processing
- `tracing` - For structured logging and debugging
- `tracing-subscriber` - For log formatting and filtering

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
