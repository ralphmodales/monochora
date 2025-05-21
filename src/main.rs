use anyhow::{Context, Result};
use clap::Parser;
use monochora::{
    converter::{image_to_ascii, image_to_colored_ascii, AsciiConverterConfig},
    display::{display_ascii_animation, get_terminal_size, save_ascii_to_file},
    handler::decode_gif,
    output::{ascii_frames_to_gif, AsciiGifOutputOptions},
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Convert GIF images to ASCII art animations")]
struct Args {
    #[clap(short, long)]
    input: PathBuf,

    #[clap(short, long)]
    output: Option<PathBuf>,

    #[clap(short, long)]
    width: Option<u32>,

    #[clap(short = 'c', long, default_value_t = false)]
    colored: bool,

    #[clap(short = 'v', long, default_value_t = false)] 
    invert: bool,

    #[clap(short = 'p', long, default_value_t = false)] 
    simple: bool,

    #[clap(short = 's', long, default_value_t = false)]
    save: bool,
    
    #[clap(short = 'g', long, default_value_t = false)]
    gif_output: bool,

    #[clap(long, default_value_t = 14.0)]
    font_size: f32,

    #[clap(long, default_value_t = false)]
    white_on_black: bool,
    
    #[clap(long, default_value_t = false)]
    black_on_white: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("Loading GIF: {}", args.input.display());
    let gif_data = decode_gif(&args.input).context("Failed to decode GIF")?;
    
    println!(
        "Loaded GIF: {} frames, {}x{}, loop count: {}",
        gif_data.frames.len(),
        gif_data.width,
        gif_data.height,
        if gif_data.loop_count == 0 {
            "infinite".to_string()
        } else {
            gif_data.loop_count.to_string()
        }
    );

    let terminal_width = if args.save {
        None 
    } else {
        get_terminal_size().ok().map(|(w, _)| w)
    };

    let config = AsciiConverterConfig {
        width: args.width.or(terminal_width),
        char_aspect: 0.5, 
        invert: args.invert,
        detailed: !args.simple,
    };

    println!("Converting frames to ASCII...");
    
    let mut ascii_frames = Vec::new();
    let mut frame_delays = Vec::new();
    
    for frame in &gif_data.frames {
        let ascii_frame = if args.colored {
            image_to_colored_ascii(&frame.image, &config)
        } else {
            image_to_ascii(&frame.image, &config)
        };
        
        ascii_frames.push(ascii_frame);
        frame_delays.push(frame.delay_time_ms);
    }

    if args.gif_output {
        let output_path = args.output.unwrap_or_else(|| {
            let mut path = args.input.file_stem().unwrap().to_os_string();
            path.push("_ascii.gif");
            PathBuf::from(path)
        });
        
        println!("Generating ASCII GIF animation to: {}", output_path.display());
        
        let mut options = AsciiGifOutputOptions::default();
        options.font_size = args.font_size;
        
        if args.black_on_white {
            options.bg_color = image::Rgb([255, 255, 255]); 
            options.text_color = image::Rgb([0, 0, 0]);     
        } else if args.white_on_black {
            options.bg_color = image::Rgb([0, 0, 0]);       
            options.text_color = image::Rgb([255, 255, 255]); 
        }
        
        ascii_frames_to_gif(&ascii_frames, &frame_delays, gif_data.loop_count, &output_path, &options)?;
        println!("Done!");
    } else if args.save || args.output.is_some() {
        let output_path = args.output.unwrap_or_else(|| {
            let mut path = args.input.file_stem().unwrap().to_os_string();
            path.push("_ascii.txt");
            PathBuf::from(path)
        });
        
        println!("Saving ASCII animation to: {}", output_path.display());
        save_ascii_to_file(&ascii_frames, &output_path)?;
        println!("Done!");
    } else {
        println!("Press 'q' or 'Esc' to exit the animation...");
        display_ascii_animation(&ascii_frames, &frame_delays, gif_data.loop_count, true).await?;
    }

    Ok(())
}
