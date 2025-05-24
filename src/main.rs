use clap::Parser;
use monochora::{
    converter::{image_to_ascii, image_to_colored_ascii, AsciiConverterConfig},
    display::{display_ascii_animation, get_terminal_size, save_ascii_to_file},
    handler::decode_gif,
    output::{ascii_frames_to_gif_with_dimensions, AsciiGifOutputOptions},
    web::get_input_path,
    MonochoraError,
};
use rayon::prelude::*;
use std::path::PathBuf;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[clap(author, version, about = "Convert GIF images to ASCII art animations")]
struct Args {
    #[clap(short, long, help = "Input GIF file path or URL")]
    input: String,

    #[clap(short, long, help = "Output file path for text format")]
    output: Option<PathBuf>,

    #[clap(short, long, help = "Target width in characters")]
    width: Option<u32>,
    
    #[clap(short = 'H', long, help = "Target height in characters")]
    height: Option<u32>,

    #[clap(short = 'c', long, default_value_t = false, help = "Enable colored output")]
    colored: bool,

    #[clap(short = 'v', long, default_value_t = false, help = "Invert brightness")] 
    invert: bool,

    #[clap(short = 'p', long, default_value_t = false, help = "Use simple character set")] 
    simple: bool,

    #[clap(short = 's', long, default_value_t = false, help = "Save to file")]
    save: bool,
    
    #[clap(long, help = "Output path for GIF format")]
    gif_output: Option<PathBuf>,

    #[clap(long, default_value_t = 14.0, help = "Font size for GIF output")]
    font_size: f32,

    #[clap(long, default_value_t = false, help = "White text on black background")]
    white_on_black: bool,
    
    #[clap(long, default_value_t = false, help = "Black text on white background")]
    black_on_white: bool,
    
    #[clap(long, default_value_t = false, help = "Fit output to terminal size")]
    fit_terminal: bool,
    
    #[clap(long, help = "Scale factor for dimensions")]
    scale: Option<f32>,
    
    #[clap(long, default_value_t = true, help = "Preserve aspect ratio")]
    preserve_aspect: bool,

    #[clap(long, help = "Number of threads for parallel processing")]
    threads: Option<usize>,

    #[clap(short = 'q', long, default_value_t = false, help = "Quiet mode")]
    quiet: bool,

    #[clap(long, default_value = "info", help = "Log level (error, warn, info, debug, trace)")]
    log_level: String,
}

fn validate_args(args: &Args) -> Result<(), MonochoraError> {
    if args.font_size <= 0.0 || args.font_size > 100.0 {
        return Err(MonochoraError::InvalidFontSize { size: args.font_size });
    }

    if let Some(width) = args.width {
        if width == 0 || width > 10000 {
            return Err(MonochoraError::InvalidDimensions { width, height: args.height.unwrap_or(0) });
        }
    }

    if let Some(height) = args.height {
        if height == 0 || height > 10000 {
            return Err(MonochoraError::InvalidDimensions { width: args.width.unwrap_or(0), height });
        }
    }

    if let Some(scale) = args.scale {
        if scale <= 0.0 || scale > 10.0 {
            return Err(MonochoraError::Config(format!("Invalid scale factor: {}", scale)));
        }
    }

    if let Some(threads) = args.threads {
        if threads == 0 || threads > 1000 {
            return Err(MonochoraError::Config(format!("Invalid thread count: {}", threads)));
        }
    }

    Ok(())
}

fn setup_logging(level: &str) -> Result<(), MonochoraError> {
    let filter = match level.to_lowercase().as_str() {
        "error" => "error",
        "warn" => "warn", 
        "info" => "info",
        "debug" => "debug",
        "trace" => "trace",
        _ => "info",
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    Ok(())
}

fn setup_thread_pool(thread_count: Option<usize>, quiet: bool) -> Result<(), MonochoraError> {
    if let Some(threads) = thread_count {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .map_err(|e| MonochoraError::ThreadPool(e.to_string()))?;
        
        if !quiet {
            info!("Using {} threads for parallel processing", threads);
        }
    }
    Ok(())
}

fn calculate_gif_dimensions(
    args: &Args, 
    gif_width: u32, 
    gif_height: u32
) -> Result<(Option<u32>, Option<u32>), MonochoraError> {
    if args.gif_output.is_some() {
        let target_gif_width = args.width.unwrap_or(gif_width);
        let target_gif_height = args.height.unwrap_or(gif_height);
        
        let char_width_pixels = args.font_size * 0.5; 
        let char_height_pixels = args.font_size;
        
        if char_width_pixels <= 0.0 || char_height_pixels <= 0.0 {
            return Err(MonochoraError::InvalidFontSize { size: args.font_size });
        }
        
        let chars_width = (target_gif_width as f32 / char_width_pixels) as u32;
        let chars_height = (target_gif_height as f32 / char_height_pixels) as u32;
        
        if chars_width == 0 || chars_height == 0 {
            return Err(MonochoraError::InvalidDimensions { 
                width: chars_width, 
                height: chars_height 
            });
        }
        
        Ok((Some(chars_width), Some(chars_height)))
    } else {
        let terminal_width = if args.fit_terminal && args.gif_output.is_none() && !args.save {
            match get_terminal_size() {
                Ok((w, _)) => Some(w),
                Err(e) => {
                    warn!("Failed to get terminal size: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Ok((args.width.or(terminal_width), args.height))
    }
}

fn generate_default_output_path(input: &str) -> PathBuf {
    if input.starts_with("http") {
        PathBuf::from("downloaded_gif_ascii.txt")
    } else {
        let input_path = PathBuf::from(input);
        match input_path.file_stem() {
            Some(stem) => {
                let mut name = stem.to_os_string();
                name.push("_ascii.txt");
                PathBuf::from(name)
            }
            None => PathBuf::from("output_ascii.txt")
        }
    }
}

async fn process_ascii_conversion(
    args: &Args,
    gif_data: &monochora::handler::GifData,
    config: &AsciiConverterConfig,
) -> Result<(Vec<Vec<String>>, Vec<u16>), MonochoraError> {
    if !args.quiet {
        info!("Converting {} frames to ASCII...", gif_data.frames.len());
    }
    
    let start_time = std::time::Instant::now();
    
    let results: Vec<Result<(Vec<String>, u16), MonochoraError>> = gif_data.frames
        .par_iter()
        .map(|frame| {
            let ascii_frame = if args.colored {
                image_to_colored_ascii(&frame.image, config)
            } else {
                image_to_ascii(&frame.image, config)
            };
            ascii_frame.map(|ascii| (ascii, frame.delay_time_ms))
        })
        .collect();
    
    let results: Result<Vec<(Vec<String>, u16)>, MonochoraError> = results.into_iter().collect();
    let results = results?;
    
    let (ascii_frames, frame_delays): (Vec<_>, Vec<_>) = results.into_iter().unzip();
    
    let conversion_time = start_time.elapsed();
    if !args.quiet {
        info!("ASCII conversion completed in {:.2}s", conversion_time.as_secs_f64());
    }

    Ok((ascii_frames, frame_delays))
}

async fn handle_gif_output(
    args: &Args,
    ascii_frames: &[Vec<String>],
    frame_delays: &[u16],
    gif_data: &monochora::handler::GifData,
) -> Result<(), MonochoraError> {
    let gif_output_path = match &args.gif_output {
        Some(path) => path.clone(),
        None => return Ok(()),
    };

    let output_path = if gif_output_path.extension().is_none() {
        gif_output_path.with_extension("gif")
    } else {
        gif_output_path
    };
    
    if !args.quiet {
        info!("Generating ASCII GIF animation: {}", output_path.display());
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
        ascii_frames, 
        frame_delays, 
        gif_data.loop_count, 
        &output_path, 
        &options,
        target_dimensions
    ).map_err(|e| MonochoraError::Animation(e.to_string()))?;
    
    let gif_time = gif_start.elapsed();
    if !args.quiet {
        info!("GIF generation completed in {:.2}s", gif_time.as_secs_f64());
    }
    
    println!("Done! Output saved to: {}", output_path.display());
    Ok(())
}

async fn handle_text_output(
    args: &Args,
    ascii_frames: &[Vec<String>],
) -> Result<(), MonochoraError> {
    let output_path = args.output.clone().unwrap_or_else(|| {
        generate_default_output_path(&args.input)
    });
    
    if !args.quiet {
        info!("Saving ASCII animation: {}", output_path.display());
    }
    
    let save_start = std::time::Instant::now();
    
    save_ascii_to_file(ascii_frames, &output_path)?;
    
    let save_time = save_start.elapsed();
    if !args.quiet {
        info!("File save completed in {:.2}s", save_time.as_secs_f64());
    }
    
    println!("Done! Output saved to: {}", output_path.display());
    Ok(())
}

async fn handle_terminal_display(
    args: &Args,
    ascii_frames: &[Vec<String>],
    frame_delays: &[u16],
    loop_count: u16,
) -> Result<(), MonochoraError> {
    if !args.quiet {
        info!("Press 'q' or 'Esc' to exit the animation...");
    }
    
    display_ascii_animation(ascii_frames, frame_delays, loop_count, true).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if let Err(e) = setup_logging(&args.log_level) {
        eprintln!("Warning: Failed to setup logging: {}", e);
    }

    if let Err(e) = validate_args(&args) {
        error!("Invalid arguments: {}", e);
        return Err(e.into());
    }

    if let Err(e) = setup_thread_pool(args.threads, args.quiet) {
        error!("Failed to setup thread pool: {}", e);
        return Err(e.into());
    }

    if !args.quiet {
        info!("Loading GIF: {}", args.input);
    }
    
    let input_path = get_input_path(&args.input).await
        .map_err(|e| {
            error!("Failed to get input path: {}", e);
            e
        })?;
    
    let gif_data = decode_gif(&input_path)
        .map_err(|e| {
            error!("Failed to decode GIF: {}", e);
            e
        })?;
    
    if !args.quiet {
        info!(
            "Loaded GIF: {} frames, {}x{}{}",
            gif_data.frames.len(),
            gif_data.width,
            gif_data.height,
            if gif_data.loop_count == 0 { " (infinite loop)" } else { "" }
        );
    }

    let (ascii_width, ascii_height) = calculate_gif_dimensions(&args, gif_data.width, gif_data.height)?;

    let config = AsciiConverterConfig {
        width: ascii_width,
        height: ascii_height,
        char_aspect: 0.5, 
        invert: args.invert,
        detailed: !args.simple,
        preserve_aspect_ratio: args.preserve_aspect,
        scale_factor: args.scale,
    };

    let (ascii_frames, frame_delays) = process_ascii_conversion(&args, &gif_data, &config).await?;

    if args.gif_output.is_some() {
        handle_gif_output(&args, &ascii_frames, &frame_delays, &gif_data).await?;
    } else if args.save || args.output.is_some() {
        handle_text_output(&args, &ascii_frames).await?;
    } else {
        handle_terminal_display(&args, &ascii_frames, &frame_delays, gif_data.loop_count).await?;
    }

    Ok(())
}
