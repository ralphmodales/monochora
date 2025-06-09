# Monochora

Monochora is a GIF to ASCII art converter written in Rust. It can transform GIF animations into playable ASCII animations in your terminal or save them as ASCII art files or even convert them back to colored GIF animations with ASCII characters.

## Features

- **High-performance parallel processing** - Multi-threaded conversion for faster processing
- Convert animated GIFs to ASCII art animations
- **Support for both local files and URLs** - Download GIFs directly from the web
- Play the animations directly in your terminal
- **Speed control** - Adjust animation speed with multipliers or target FPS
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

# Control animation speed - play at half speed
monochora -i input.gif --speed 0.5

# Control animation speed - play at double speed
monochora -i input.gif --speed 2.0

# Set target frames per second
monochora -i input.gif --fps 30

# Save as ASCII text file
monochora -i input.gif -s

# Save to specific output file
monochora -i input.gif -o my_ascii.txt

# Generate colored ASCII in terminal
monochora -i input.gif -c

# Download from URL and generate colored ASCII with speed control
monochora -i "https://giffiles.alphacoders.com/220/220890.gif" -c -w 200 -H 100 --speed 1.5

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

# Save as high-quality ASCII GIF animation with speed adjustment
monochora -i input.gif --gif-output output.gif --speed 0.8

# Generate GIF with default name and target FPS
monochora -i input.gif --gif-output --fps 24

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

# White text on black background for GIF output with slow motion
monochora -i input.gif --gif-output output.gif --white-on-black --speed 0.3

# Black text on white background for GIF output
monochora -i input.gif --gif-output output.gif --black-on-white

# Custom font size for GIF output (optimized palettes)
monochora -i input.gif --gif-output output.gif --font-size 20

# High-performance processing with custom thread count and speed control
monochora -i input.gif --threads 8 --fps 60

# Quiet mode for batch processing
monochora -i input.gif -q --gif-output output.gif

# Debug with detailed logging
monochora -i input.gif --log-level debug

# Download from URL and save as high-quality ASCII GIF with speed control
monochora -i "https://example.com/cool.gif" --gif-output result.gif --black-on-white --font-size 16 --speed 1.2
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
      --speed <SPEED>                    Animation speed multiplier (0.1-10.0, where 1.0 = original speed)
      --fps <FPS>                        Target frames per second (1-120)
      --fit-terminal                     Fit ASCII art to terminal width
      --scale <SCALE>                    Scale factor for original dimensions
      --preserve-aspect <PRESERVE_ASPECT> Preserve original aspect ratio [default: true]
      --threads <THREADS>                Number of threads for parallel processing
      --charset <CHARSET>                Custom character set string (ordered darkest to lightest)
      --charset-file <CHARSET_FILE>      Path to custom character set file
      --list-charsets                    List available character sets and exit
      --responsive                       Enable responsive mode - auto-adjust when terminal is resized
      --watch-terminal                   Watch terminal for resize events (requires responsive mode)
  -q, --quiet                            Suppress progress output
      --log-level <LOG_LEVEL>            Log level (error, warn, info, debug, trace) [default: info]
  -h, --help                             Print help
  -V, --version                          Print version
```

## Speed Control

Monochora provides flexible animation speed control through two mutually exclusive options:

### Speed Multiplier (`--speed`)

Control animation playback speed with a multiplier:

- **Values**: 0.1 to 10.0
- **Default**: 1.0 (original speed)
- **Examples**:
  - `--speed 0.5` - Half speed (slower, more detailed viewing)
  - `--speed 2.0` - Double speed (faster playback)
  - `--speed 0.25` - Quarter speed (very slow for detailed analysis)
  - `--speed 5.0` - Five times faster

### Target FPS (`--fps`)

Set a specific target frame rate:

- **Values**: 1 to 120 FPS
- **Behavior**: Adjusts frame delays to achieve the target frame rate
- **Examples**:
  - `--fps 30` - Standard video frame rate
  - `--fps 60` - Smooth high frame rate
  - `--fps 12` - Cinematic slow motion effect
  - `--fps 120` - Ultra-smooth playback

### Speed Control Examples

```bash
# Slow motion for detailed viewing
monochora -i fast_animation.gif --speed 0.3 -c

# Speed up a slow GIF
monochora -i slow_animation.gif --speed 3.0

# Set consistent 24 FPS for cinematic feel
monochora -i variable_fps.gif --fps 24 --gif-output cinema.gif

# Ultra-smooth 60 FPS terminal playback
monochora -i animation.gif --fps 60 -c --fit-terminal

# Very slow analysis mode
monochora -i complex_animation.gif --speed 0.1 -w 150

# Fast preview mode
monochora -i long_animation.gif --speed 8.0 --simple

# Combine with other features for optimal viewing
monochora -i "https://example.com/animation.gif" --speed 1.5 -c -w 120 --charset " ·∘○●"
```

## Responsive Terminal Display

Monochora can adapt to terminal size changes during playback for optimal viewing:

### Responsive Mode (`--responsive`)

Enables automatic adjustment of ASCII art dimensions when terminal size changes:

- **Behavior**: Recalculates ASCII conversion when terminal is resized
- **Use case**: Manually resize terminal during playback for better viewing
- **Restriction**: Only works with terminal display mode (not file output)

### Terminal Watching (`--watch-terminal`)

Actively monitors terminal for resize events in real-time:

- **Requirement**: Must be used with `--responsive` mode
- **Behavior**: Continuously watches for terminal resize events
- **Performance**: Minimal overhead for real-time monitoring

### Responsive Examples

```bash
# Basic responsive mode - adapts when terminal is manually resized
monochora -i animation.gif --responsive -c

# Active terminal monitoring with speed control
monochora -i animation.gif --responsive --watch-terminal --speed 0.8

# Responsive colored output with custom character set
monochora -i animation.gif --responsive --watch-terminal -c --charset " ·∘○●"

# Responsive mode with FPS control
monochora -i animation.gif --responsive --fps 30 --fit-terminal
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

### Speed Control Restrictions

Speed control options are mutually exclusive:

- **Speed multiplier**: `--speed <multiplier>` (0.1-10.0)
- **Target FPS**: `--fps <frame_rate>` (1-120)

**You cannot use both `--speed` and `--fps` in the same command.**

### Responsive Mode Restrictions

- `--responsive` only works with terminal display mode
- Cannot be used with file output options (`--save`, `--output`, `--gif-output`)
- `--watch-terminal` requires `--responsive` mode to be enabled
- Responsive features are not available during file generation

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
- **Smart frame timing** - Efficient speed adjustment calculations
- **Progress tracking** - Real-time feedback on conversion progress (unless in quiet mode)
- **Comprehensive validation** - Input validation to prevent runtime errors

### Performance Examples

```bash
# Use all CPU cores for maximum speed with fast playback
monochora -i large_animation.gif --threads 16 --gif-output result.gif --speed 2.0

# Quiet batch processing with speed control
monochora -i input.gif -q --gif-output output.gif --font-size 12 --fps 30

# High-performance colored terminal output with smooth 60 FPS
monochora -i animation.gif -c --threads 8 --fps 60

# Debug mode with detailed logging and slow motion
monochora -i animation.gif --log-level debug --threads 4 --speed 0.5
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

### Speed Control in GIF Output
- **Frame timing preservation** - Speed adjustments maintain smooth playback
- **Optimized delay calculations** - Ensures proper timing across different speeds
- **Quality maintenance** - Speed changes don't affect visual quality

### Examples

```bash
# High-quality small text with slow motion (more colors, precision quantization)
monochora -i input.gif --gif-output high_quality.gif --font-size 8 --speed 0.4

# Optimized medium text with standard cinema FPS (balanced quality/performance)  
monochora -i input.gif --gif-output medium.gif --font-size 14 --fps 24

# Fast large text with accelerated playback (fewer colors, optimized for speed)
monochora -i input.gif --gif-output large.gif --font-size 24 --speed 3.0

# Custom styling with adaptive palettes and speed control
monochora -i input.gif --gif-output custom.gif --black-on-white --font-size 16 --fps 30

# Auto-generated output filename with speed adjustment
monochora -i my_animation.gif --gif-output --speed 1.5  # Creates ascii_my_animation.gif
```

## Examples

### Terminal Animation

To play an animation in your terminal:

```bash
monochora -i animation.gif
```

Press `q` or `Esc` to exit the animation.

### Speed Control Examples

Control animation playback speed for different viewing experiences:

```bash
# Slow motion for detailed analysis
monochora -i complex_animation.gif --speed 0.25 -c -w 150

# Speed up slow animations
monochora -i slow_gif.gif --speed 4.0 --fit-terminal

# Set consistent frame rate for smooth playback
monochora -i variable_speed.gif --fps 30 -c

# Ultra-smooth high frame rate display
monochora -i animation.gif --fps 60 --charset " ·∘○●"

# Cinematic 24 FPS with custom styling
monochora -i movie_clip.gif --fps 24 --charset " .-=+*#" -w 120
```

### Custom Character Set Examples

Explore different artistic styles with custom character sets:

```bash
# Minimalist density gradient with speed control
monochora -i portrait.gif --charset " .oO@" --speed 0.8

# Geometric progression with smooth FPS
monochora -i abstract.gif --charset " ·∘○●◉" -c --fps 45

# Technical documentation style with slow playback
monochora -i diagram.png --charset " .-=+*#" -w 120 --speed 0.5

# High-contrast for readability with fast preview
monochora -i screenshot.gif --charset " ░▓█" --fit-terminal --speed 3.0

# Artistic braille output with controlled timing
monochora -i pattern.gif --charset "⠀⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋" --save --fps 20
```

### URL Input Examples

Download and play a GIF from the internet:

```bash
# Simple terminal playback with speed control
monochora -i "https://media.giphy.com/media/3o7abKhOpu0NwenH3O/giphy.gif" --speed 1.5

# Colored ASCII with custom dimensions, character set, and FPS
monochora -i "https://example.com/animation.gif" -c -w 150 -H 75 --charset " ·∘○●" --fps 30

# Download and save as high-quality ASCII GIF with speed adjustment
monochora -i "https://example.com/source.gif" --gif-output converted_ascii.gif --speed 0.7
```

### Colored Terminal Animation

```bash
# Standard colored output
monochora -i animation.gif -c

# Colored output with speed control
monochora -i animation.gif -c --fps 45
```

### Scale Animation

Scale the animation to half size while preserving aspect ratio:

```bash
# Basic scaling
monochora -i animation.gif --scale 0.5

# Scaling with speed control
monochora -i animation.gif --scale 0.5 --speed 2.0
```

### Fit to Terminal

Make the animation fit your terminal width:

```bash
# Basic terminal fitting
monochora -i animation.gif --fit-terminal

# Terminal fitting with speed control
monochora -i animation.gif --fit-terminal --fps 60
```

### Custom Dimensions with Proper Character Scaling

The dimension system now properly accounts for character aspect ratios:

```bash
# Width-based scaling (height calculated for proper proportions)
monochora -i animation.gif -w 120

# Height-based scaling (width calculated for proper proportions)
monochora -i animation.gif -H 40

# Exact dimensions with speed control (may distort aspect ratio)
monochora -i animation.gif -w 120 -H 40 --preserve-aspect false --speed 1.3
```

### Save as ASCII Text

```bash
# Save with default filename
monochora -i animation.gif -s

# Save to specific file with speed adjustment (affects timing comments if included)
monochora -i animation.gif -o my_ascii_art.txt --speed 2.0

# Save with custom character set and FPS control
monochora -i animation.gif -s --charset " ░▒▓█" --fps 24
```

### Save as High-Quality ASCII GIF

```bash
# Default output filename (ascii_animation.gif) with speed control
monochora -i animation.gif --gif-output --speed 1.2

# Custom output filename with character set and FPS
monochora -i animation.gif --gif-output my_ascii.gif --charset " ·∘○●" --fps 30
```

### Custom Style GIF with Optimization

```bash
# High-quality output with custom styling, character set, and speed control
monochora -i animation.gif --gif-output result.gif --black-on-white --font-size 18 --charset " .-+*#" --speed 0.8

# Performance-optimized batch processing with FPS control
monochora -i animation.gif --gif-output result.gif --quiet --threads 12 --fps 30
```

### Large High-Quality ASCII GIF

Create a large, detailed ASCII GIF with optimized processing:

```bash
# Large scale with speed control
monochora -i small_animation.gif --gif-output large_result.gif --scale 2.0 --font-size 8 --threads 8 --speed 1.5

# High quality with smooth FPS
monochora -i input.gif --gif-output quality_result.gif --scale 1.8 --font-size 10 --fps 45
```

## Error Handling and Validation

Monochora includes comprehensive input validation:

### Dimension Validation
- Width and height must be between 1 and 10,000 characters
- Font size must be between 0.1 and 100.0
- Scale factor must be between 0.1 and 10.0
- Thread count must be between 1 and 1,000

### Speed Control Validation
- Speed multiplier must be between 0.1 and 10.0
- Target FPS must be between 1 and 120
- Speed and FPS options are mutually exclusive

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
- Prevents conflicting speed control options

### Common Error Messages
- **Invalid font size**: Font size out of valid range
- **Invalid dimensions**: Width or height out of bounds
- **Invalid speed**: Speed multiplier out of valid range (0.1-10.0)
- **Invalid FPS**: Target FPS out of valid range (1-120)
- **Speed conflict**: Cannot use both --speed and --fps options
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
# Download and convert a GIF from Giphy with speed control
monochora -i "https://media.giphy.com/media/example/giphy.gif" -c --speed 1.8

# Download from any image hosting service with FPS control
monochora -i "https://i.imgur.com/example.gif" -w 100 --fps 30

# Works with direct links to GIF files with speed adjustment
monochora -i "https://example.com/path/to/animation.gif" --gif-output result.gif --speed 0.6
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
# Preserve image proportions with character scaling and speed control
monochora -i wide_image.gif -w 160 --speed 1.3  # Height auto-calculated

# Scale with proper character proportions and FPS control
monochora -i image.gif --scale 1.5 --fps 30  # 150% size with character correction

# For GIF output, dimensions are calculated for target pixel size with speed adjustment
monochora -i input.gif --gif-output result.gif -w 800 --font-size 12 --speed 0.8
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
    timing::calculate_adjusted_frame_delays,
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
    
    let (ascii_frames, mut frame_delays): (Vec<_>, Vec<_>) = results.into_iter().unzip();
    
    // Adjust frame delays for speed control (example: 2x speed)
    frame_delays = calculate_adjusted_frame_delays(&frame_delays, Some(2.0), None)?;
    
    // Display the animation
    display_ascii_animation(&ascii_frames, &frame_delays, gif_data.loop_count, true).await?;
    
    // Or save as ASCII GIF with speed control
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
- **debug**: Detailed processing information including speed adjustments
- **trace**: Extremely verbose output for debugging

### Debugging Examples

```bash
# Debug mode for troubleshooting with speed control
monochora -i problematic.gif --log-level debug --speed 1.5

# Trace mode for detailed analysis including timing calculations
monochora -i animation.gif --log-level trace --threads 4 --fps 45

# Error-only mode for production with speed control
monochora -i input.gif --log-level error --quiet --speed 2.0
```

## How It Works

Monochora works by:

1. **Input validation**: Comprehensive validation of all command-line arguments including speed parameters
2. **Input handling**: Accepts both local file paths and URLs (HTTP/HTTPS)
3. **URL processing**: Downloads GIFs from URLs to temporary files when needed
4. **GIF decoding**: Decodes GIF frames using the `gif` crate with parallel processing
5. **Character set selection**: Chooses appropriate character set (built-in, custom inline, or file-based)
6. **ASCII conversion**: Converts each frame to ASCII art based on pixel brightness using parallel processing
7. **Dimension calculation**: Intelligently calculates dimensions with proper character aspect ratio handling
8. **Speed adjustment**: Calculates adjusted frame delays based on speed multiplier or target FPS
9. **Advanced output generation**: 
   - Terminal display and GIF output with color support and speed control
   - Text file output with frame separators
   - High-quality ASCII GIF generation with adaptive palettes and timing preservation

The ASCII conversion process maps pixel brightness to appropriate ASCII characters using either built-in character sets (simple or detailed) or custom user-defined sets. For colored output, it includes ANSI color codes for terminal display or renders characters with their original colors into a new GIF using advanced quantization techniques.

### Speed Control Processing

The speed control system handles frame timing adjustments:

1. **Speed multiplier mode**: Multiplies original frame delays by the inverse of the speed factor
2. **Target FPS mode**: Calculates uniform frame delays to achieve the specified frame rate
3. **Validation**: Ensures resulting delays are within reasonable bounds (10ms minimum)
4. **Conflict resolution**: Prevents simultaneous use of both speed control methods
5. **Timing preservation**: Maintains smooth playback across different output formats

### Advanced GIF Generation Process

The ASCII GIF output uses several optimization techniques:

1. **Adaptive palette creation** based on font size and color scheme
2. **Precision quantization** with different algorithms for different font sizes
3. **Smart color distance calculation** optimized for text rendering
4. **Frame-by-frame parallel rendering** for improved performance
5. **Embedded font rendering** using DejaVu Sans Mono for consistent output
6. **Speed-aware timing** that preserves smooth playback at different speeds

## Performance Characteristics

- **Multi-threaded processing**: Utilizes all available CPU cores by default
- **Memory efficient**: Streams processing to handle large GIF files
- **Scalable**: Performance improves with more CPU cores
- **Optimized algorithms**: Different processing strategies based on output type and quality settings
- **Efficient speed calculations**: Minimal overhead for frame timing adjustments

### Typical Performance

- **Small GIFs** (< 1MB): Near-instantaneous processing with speed control
- **Medium GIFs** (1-10MB): 1-5 seconds on modern hardware, timing adjustments add <1% overhead
- **Large GIFs** (10MB+): Scales linearly with thread count, speed control remains efficient
- **Batch processing**: Quiet mode minimizes I/O overhead, speed adjustments are computed once

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
