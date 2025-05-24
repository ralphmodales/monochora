use anyhow::{Context, Result};
use clap::Parser;
use monochora::{
    converter::{image_to_ascii, image_to_colored_ascii, AsciiConverterConfig},
    display::{display_ascii_animation, get_terminal_size, save_ascii_to_file},
    handler::decode_gif,
    output::{ascii_frames_to_gif_with_dimensions, AsciiGifOutputOptions},
    web::get_input_path,
};
use rayon::prelude::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Convert GIF images to ASCII art animations")]
struct Args {
    #[clap(short, long)]
    input: String,

    #[clap(short, long)]
    output: Option<PathBuf>,

    #[clap(short, long)]
    width: Option<u32>,
    
    #[clap(short = 'H', long)]
    height: Option<u32>,

    #[clap(short = 'c', long, default_value_t = false)]
    colored: bool,

    #[clap(short = 'v', long, default_value_t = false)] 
    invert: bool,

    #[clap(short = 'p', long, default_value_t = false)] 
    simple: bool,

    #[clap(short = 's', long, default_value_t = false)]
    save: bool,
    
    #[clap(long)]
    gif_output: Option<PathBuf>,

    #[clap(long, default_value_t = 14.0)]
    font_size: f32,

    #[clap(long, default_value_t = false)]
    white_on_black: bool,
    
    #[clap(long, default_value_t = false)]
    black_on_white: bool,
    
    #[clap(long, default_value_t = false)]
    fit_terminal: bool,
    
    #[clap(long)]
    scale: Option<f32>,
    
    #[clap(long, default_value_t = true)]
    preserve_aspect: bool,

    #[clap(long)]
    threads: Option<usize>,

    #[clap(short = 'q', long, default_value_t = false)]
    quiet: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(thread_count) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global()
            .context("Failed to set thread pool size")?;
        if !args.quiet {
            println!("Using {} threads for parallel processing", thread_count);
        }
    }

    if !args.quiet {
        println!("Loading GIF: {}", args.input);
    }
    
    let input_path = get_input_path(&args.input).await
        .context("Failed to get input path")?;
    
    let gif_data = decode_gif(&input_path).context("Failed to decode GIF")?;
    
    if !args.quiet {
        println!(
            "Loaded GIF: {} frames, {}x{}{}",
            gif_data.frames.len(),
            gif_data.width,
            gif_data.height,
            if gif_data.loop_count == 0 { " (infinite loop)" } else { "" }
        );
    }

    let terminal_width = if args.fit_terminal && args.gif_output.is_none() && !args.save {
        get_terminal_size().ok().map(|(w, _)| w)
    } else {
        None
    };

    let (ascii_width, ascii_height) = if args.gif_output.is_some() {
        let target_gif_width = args.width.unwrap_or(gif_data.width);
        let target_gif_height = args.height.unwrap_or(gif_data.height);
        
        let char_width_pixels = args.font_size * 0.5; 
        let char_height_pixels = args.font_size;
        
        let chars_width = (target_gif_width as f32 / char_width_pixels) as u32;
        let chars_height = (target_gif_height as f32 / char_height_pixels) as u32;
        
        (Some(chars_width), Some(chars_height))
    } else {
        (args.width.or(terminal_width), args.height)
    };

    let config = AsciiConverterConfig {
        width: ascii_width,
        height: ascii_height,
        char_aspect: 0.5, 
        invert: args.invert,
        detailed: !args.simple,
        preserve_aspect_ratio: args.preserve_aspect,
        scale_factor: args.scale,
    };

    if !args.quiet {
        println!("Converting {} frames to ASCII...", gif_data.frames.len());
    }
    
    let start_time = std::time::Instant::now();
    
    let results: Vec<(Vec<String>, u16)> = gif_data.frames
        .par_iter()
        .map(|frame| {
            let ascii_frame = if args.colored {
                image_to_colored_ascii(&frame.image, &config)
            } else {
                image_to_ascii(&frame.image, &config)
            };
            (ascii_frame, frame.delay_time_ms)
        })
        .collect();
    
    let (ascii_frames, frame_delays): (Vec<_>, Vec<_>) = results.into_iter().unzip();
    
    let conversion_time = start_time.elapsed();
    if !args.quiet {
        println!("ASCII conversion completed in {:.2}s", conversion_time.as_secs_f64());
    }

    if let Some(gif_output_path) = args.gif_output {
        let output_path = if gif_output_path.extension().is_none() {
            gif_output_path.with_extension("gif")
        } else {
            gif_output_path
        };
        
        if !args.quiet {
            println!("Generating ASCII GIF animation: {}", output_path.display());
        }
        
        let gif_start = std::time::Instant::now();
        
        let mut options = AsciiGifOutputOptions::default();
        options.font_size = args.font_size;
        
        if args.black_on_white {
            options.bg_color = image::Rgb([255, 255, 255]); 
            options.text_color = image::Rgb([0, 0, 0]);     
        } else if args.white_on_black {
            options.bg_color = image::Rgb([0, 0, 0]);       
            options.text_color = image::Rgb([255, 255, 255]); 
        }
        
        let target_dimensions = Some((
            args.width.unwrap_or(gif_data.width),
            args.height.unwrap_or(gif_data.height)
        ));
        
        ascii_frames_to_gif_with_dimensions(
            &ascii_frames, 
            &frame_delays, 
            gif_data.loop_count, 
            &output_path, 
            &options,
            target_dimensions
        )?;
        
        let gif_time = gif_start.elapsed();
        if !args.quiet {
            println!("GIF generation completed in {:.2}s", gif_time.as_secs_f64());
            println!("Total time: {:.2}s", (conversion_time + gif_time).as_secs_f64());
        }
        println!("Done! Output saved to: {}", output_path.display());
    } 
    else if args.save || args.output.is_some() {
        let output_path = args.output.unwrap_or_else(|| {
            if args.input.starts_with("http") {
                PathBuf::from("downloaded_gif_ascii.txt")
            } else {
                let mut path = PathBuf::from(&args.input).file_stem().unwrap().to_os_string();
                path.push("_ascii.txt");
                PathBuf::from(path)
            }
        });
        
        if !args.quiet {
            println!("Saving ASCII animation: {}", output_path.display());
        }
        let save_start = std::time::Instant::now();
        
        save_ascii_to_file(&ascii_frames, &output_path)?;
        
        let save_time = save_start.elapsed();
        if !args.quiet {
            println!("File save completed in {:.2}s", save_time.as_secs_f64());
            println!("Total time: {:.2}s", (conversion_time + save_time).as_secs_f64());
        }
        println!("Done! Output saved to: {}", output_path.display());
    } 
    else {
        if !args.quiet {
            println!("Press 'q' or 'Esc' to exit the animation...");
        }
        display_ascii_animation(&ascii_frames, &frame_delays, gif_data.loop_count, true).await?;
    }

    Ok(())
}
