# Monochora

Monochora is a GIF to ASCII art converter written in Rust. It can transform GIF animations into playable ASCII animations in your terminal or save them as ASCII art files or even convert them back to colored GIF animations with ASCII characters.

## Features

- Convert animated GIFs to ASCII art animations
- Play the animations directly in your terminal
- Save animations as text files or GIF files
- Support for colored ASCII art (with ANSI color codes)
- Customizable character sets (simple or detailed)
- Multiple output options (terminal, text file, or GIF output)
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
# Basic usage (plays in terminal)
monochora -i input.gif

# Save as ASCII text file
monochora -i input.gif -s

# Generate colored ASCII in terminal
monochora -i input.gif -c

# Invert brightness
monochora -i input.gif -v

# Use simple character set
monochora -i input.gif -p

# Save as ASCII GIF animation
monochora -i input.gif -g

# Custom width
monochora -i input.gif -w 100

# White text on black background for GIF output
monochora -i input.gif -g --white-on-black

# Black text on white background for GIF output
monochora -i input.gif -g --black-on-white

# Custom font size for GIF output
monochora -i input.gif -g --font-size 20

# Save output to a specific file
monochora -i input.gif -o output.txt
monochora -i input.gif -g -o output.gif
```

## Options

```
Options:
  -i, --input <INPUT>                    Input GIF file path
  -o, --output <OUTPUT>                  Output file path
  -w, --width <WIDTH>                    Target width in characters
  -c, --colored                          Use colored ASCII (ANSI colors)
  -v, --invert                           Invert brightness
  -p, --simple                           Use simple character set
  -s, --save                             Save to file instead of playing
  -g, --gif-output                       Output as ASCII GIF
      --font-size <FONT_SIZE>            Font size for GIF output [default: 14.0]
      --white-on-black                   White text on black background for GIF
      --black-on-white                   Black text on white background for GIF
  -h, --help                             Print help
  -V, --version                          Print version
```

## Examples

### Terminal Animation

To play an animation in your terminal:

```bash
monochora -i animation.gif
```

Press `q` or `Esc` to exit the animation.

### Colored Terminal Animation

```bash
monochora -i animation.gif -c
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

## Library Usage

Monochora can also be used as a library in your Rust projects:

```rust
use monochora::{
    converter::{image_to_ascii, AsciiConverterConfig},
    handler::decode_gif,
    display::display_ascii_animation,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Decode the GIF
    let gif_data = decode_gif("input.gif")?;
    
    // Configure the converter
    let config = AsciiConverterConfig {
        width: Some(80),
        invert: false,
        detailed: true,
        ..Default::default()
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
    tokio::runtime::Runtime::new()?.block_on(async {
        display_ascii_animation(&ascii_frames, &frame_delays, gif_data.loop_count, true).await?;
        Ok::<(), anyhow::Error>(())
    })?;
    
    Ok(())
}
```

## How It Works

Monochora works by:

1. Decoding GIF frames using the `gif` crate
2. Converting each frame to ASCII art based on pixel brightness
3. Either displaying in terminal, saving to text file, or rendering to a new GIF

The ASCII conversion process maps pixel brightness to appropriate ASCII characters, with either a simple or detailed character set. For colored output, it includes ANSI color codes for terminal display or renders characters with their original colors into a new GIF.

## Dependencies

- `gif` - For GIF decoding and encoding
- `image` - For image manipulation
- `imageproc` - For drawing text on images
- `rusttype` - For font rendering
- `clap` - For command-line argument parsing
- `crossterm` - For terminal manipulation
- `anyhow` - For error handling
- `tokio` - For asynchronous operation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.


