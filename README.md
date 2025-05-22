# Monochora

Monochora is a GIF to ASCII art converter written in Rust. It can transform GIF animations into playable ASCII animations in your terminal or save them as ASCII art files or even convert them back to colored GIF animations with ASCII characters.

## Features

- Convert animated GIFs to ASCII art animations
- **Support for both local files and URLs** - Download GIFs directly from the web
- Play the animations directly in your terminal
- Save animations as text files or GIF files
- Support for colored ASCII art (with ANSI color codes)
- Customizable character sets (simple or detailed)
- Multiple output options (terminal, text file, or GIF output)
- Preserve original GIF dimensions and aspect ratios
- Flexible scaling and dimension control
- Customize font style, size, and colors for GIF output

## Installation

### From Source

1. Make sure you have Rust and Cargo installed. If not, install it from [rust-lang.org](https://www.rust-lang.org/tools/install).

2. Clone the repository:
   ```bash
   git clone https://github.com/ralphmodales/monochora.git
   cd monochora
   ```

3. Build the project:
   ```bash
   cargo build --release
   ```

4. The binary will be available at `target/release/monochora`

## Usage

```bash
# Basic usage with local file (plays in terminal with original GIF dimensions)
monochora -i input.gif

# Basic usage with URL - downloads and converts automatically
monochora -i https://example.com/animation.gif

# Save as ASCII text file
monochora -i input.gif -s

# Generate colored ASCII in terminal
monochora -i input.gif -c

# Download from URL and generate colored ASCII
monochora -i "https://giffiles.alphacoders.com/220/220890.gif" -c -w 200 -H 100

# Invert brightness
monochora -i input.gif -v

# Use simple character set
monochora -i input.gif -p

# Save as ASCII GIF animation
monochora -i input.gif -g

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
monochora -i input.gif -g --white-on-black

# Black text on white background for GIF output
monochora -i input.gif -g --black-on-white

# Custom font size for GIF output
monochora -i input.gif -g --font-size 20

# Save output to a specific file
monochora -i input.gif -o output.txt
monochora -i input.gif -g -o output.gif

# Download from URL and save as ASCII GIF with custom styling
monochora -i "https://example.com/cool.gif" -g --black-on-white --font-size 16
```

## Options

```
Options:
  -i, --input <INPUT>                    Input GIF file path or URL (supports HTTP/HTTPS)
  -o, --output <OUTPUT>                  Output file path
  -w, --width <WIDTH>                    Target width in characters
  -H, --height <HEIGHT>                  Target height in characters
  -c, --colored                          Use colored ASCII (ANSI colors)
  -v, --invert                           Invert brightness
  -p, --simple                           Use simple character set
  -s, --save                             Save to file instead of playing
  -g, --gif-output                       Output as ASCII GIF
      --font-size <FONT_SIZE>            Font size for GIF output [default: 14.0]
      --white-on-black                   White text on black background for GIF
      --black-on-white                   Black text on white background for GIF
      --fit-terminal                     Fit ASCII art to terminal width
      --scale <SCALE>                    Scale factor for original dimensions
      --preserve-aspect <PRESERVE_ASPECT> Preserve original aspect ratio [default: true]
  -h, --help                             Print help
  -V, --version                          Print version
```

## Examples

### Terminal Animation (Original Size)

To play an animation in your terminal with original GIF dimensions:

```bash
monochora -i animation.gif
```

Press `q` or `Esc` to exit the animation.

### URL Input Examples

Download and play a GIF from the internet:

```bash
# Simple terminal playback
monochora -i "https://media.giphy.com/media/3o7abKhOpu0NwenH3O/giphy.gif"

# Colored ASCII with custom dimensions
monochora -i "https://example.com/animation.gif" -c -w 150 -H 75

# Download and save as ASCII GIF
monochora -i "https://example.com/source.gif" -g -o "converted_ascii.gif"
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

Make the animation fit your terminal width (may distort aspect ratio):

```bash
monochora -i animation.gif --fit-terminal
```

### Custom Dimensions

Set specific width and height:

```bash
monochora -i animation.gif -w 120 -H 40
```

### Save as ASCII Text

```bash
monochora -i animation.gif -s -o animation_ascii.txt
```

### Save as ASCII GIF

```bash
monochora -i animation.gif -g -o animation_ascii.gif
```

### Custom Style GIF

```bash
monochora -i animation.gif -g --black-on-white --font-size 18
```

### Large High-Quality ASCII GIF

Create a large, detailed ASCII GIF:

```bash
monochora -i small_animation.gif -g --scale 2.0 --font-size 8
```

## URL Support

Monochora supports downloading GIFs directly from URLs:

- **Supported protocols**: HTTP and HTTPS
- **Automatic download**: Files are downloaded to temporary storage and cleaned up automatically
- **Content validation**: Warns if the URL doesn't serve image content
- **Timeout handling**: 30-second timeout for downloads
- **Progress indication**: Shows download progress and file size

### URL Examples

```bash
# Download and convert a GIF from Giphy
monochora -i "https://media.giphy.com/media/example/giphy.gif" -c

# Download from any image hosting service
monochora -i "https://i.imgur.com/example.gif" -w 100

# Works with direct links to GIF files
monochora -i "https://example.com/path/to/animation.gif" -g
```

## Dimension Control

Monochora offers flexible dimension control:

- **Default**: Uses original GIF dimensions (preserves full image)
- **--width only**: Sets width, calculates height to preserve aspect ratio
- **--height only**: Sets height, calculates width to preserve aspect ratio
- **--width and --height**: Uses exact dimensions (may distort)
- **--scale**: Multiplies original dimensions by scale factor
- **--fit-terminal**: Fits to terminal width (legacy behavior)
- **--preserve-aspect false**: Disables automatic aspect ratio preservation

## Library Usage

Monochora can also be used as a library in your Rust projects:

```rust
use monochora::{
    converter::{image_to_ascii, AsciiConverterConfig},
    handler::decode_gif,
    display::display_ascii_animation,
    web::get_input_path,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Handle both local files and URLs
    let input_path = get_input_path("https://example.com/animation.gif").await?;
    
    // Decode the GIF
    let gif_data = decode_gif(&input_path)?;
    
    // Configure the converter
    let config = AsciiConverterConfig {
        width: Some(80),
        height: None,
        char_aspect: 0.5,
        invert: false,
        detailed: true,
        preserve_aspect_ratio: true,
        scale_factor: Some(1.5), // 150% of original size
    };
    
    // Convert frames to ASCII
    let mut ascii_frames = Vec::new();
    let mut frame_delays = Vec::new();
    
    for frame in &gif_data.frames {
        let ascii_frame = image_to_ascii(&frame.image, &config);
        ascii_frames.push(ascii_frame);
        frame_delays.push(frame.delay_time_ms);
    }
    
    // Display the animation
    display_ascii_animation(&ascii_frames, &frame_delays, gif_data.loop_count, true).await?;
    
    Ok(())
}
```

## How It Works

Monochora works by:

1. **Input handling**: Accepts both local file paths and URLs (HTTP/HTTPS)
2. **URL processing**: Downloads GIFs from URLs to temporary files when needed
3. **GIF decoding**: Decodes GIF frames using the `gif` crate
4. **ASCII conversion**: Converts each frame to ASCII art based on pixel brightness
5. **Dimension preservation**: Preserves original dimensions and aspect ratios by default
6. **Output generation**: Either displays in terminal, saves to text file, or renders to a new GIF

The ASCII conversion process maps pixel brightness to appropriate ASCII characters, with either a simple or detailed character set. For colored output, it includes ANSI color codes for terminal display or renders characters with their original colors into a new GIF.

The dimension calculation system ensures that:
- Landscape GIFs remain landscape
- Portrait GIFs remain portrait  
- Original proportions are maintained unless explicitly overridden
- Full image content is preserved without cropping

## Dependencies

- `gif` - For GIF decoding and encoding
- `image` - For image manipulation
- `imageproc` - For drawing text on images
- `rusttype` - For font rendering
- `clap` - For command-line argument parsing
- `crossterm` - For terminal manipulation
- `anyhow` - For error handling
- `tokio` - For asynchronous operation
- `reqwest` - For HTTP downloads
- `url` - For URL parsing and validation
- `tempfile` - For temporary file management

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
