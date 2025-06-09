use clap::Parser;
use monochora::{
    converter::{image_to_ascii, image_to_colored_ascii, AsciiConverterConfig},
    display::{display_ascii_animation, get_terminal_size, save_ascii_to_file, display_responsive_ascii_animation},
    handler::decode_gif,
    output::{ascii_frames_to_gif_with_dimensions, AsciiGifOutputOptions},
    terminal_watcher::{TerminalWatcher, ResponsiveFrameManager, TerminalDimensions},
    web::get_input_path,
    MonochoraError,
};
use rayon::prelude::*;
use std::path::PathBuf;
use tracing::{error, info, warn};


#[derive(Parser, Debug)]
#[clap(author, version, about = "Convert GIF images to ASCII art animations")]
#[repr(C)]
struct Args {
    #[clap(short, long, help = "Input GIF file path or URL")]
    input: Option<String>,

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
    
    #[clap(long, help = "Generate GIF output. Optionally specify path (e.g., --gif-output or --gif-output path/name.gif)")]
    gif_output: Option<Option<PathBuf>>,

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

    #[clap(long, help = "Path to custom character set file")]
    charset_file: Option<PathBuf>,

    #[clap(long, help = "Inline character set string (ordered from darkest to lightest)")]
    charset: Option<String>,

    #[clap(long, default_value_t = false, help = "List available character sets and exit")]
    list_charsets: bool,

    #[clap(long, help = "Speed multiplier for animation (e.g., 0.5 for half speed, 2.0 for double speed)")]
    speed: Option<f32>,

    #[clap(long, help = "Target frames per second (overrides speed setting)")]
    fps: Option<f32>,

    #[clap(long, default_value_t = false, help = "Enable responsive mode - auto-adjust when terminal is resized")]
    responsive: bool,

    #[clap(long, default_value_t = false, help = "Watch terminal for resize events (requires responsive mode)")]
    watch_terminal: bool,
}

fn validate_args(args: &Args) -> Result<(), MonochoraError> {
    if args.input.is_none() {
        return Err(MonochoraError::Config("Input file path or URL is required".to_string()));
    }

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

    if let Some(speed) = args.speed {
        if speed <= 0.0 || speed > 100.0 {
            return Err(MonochoraError::Config(format!("Invalid speed multiplier: {}", speed)));
        }
    }

    if let Some(fps) = args.fps {
        if fps <= 0.0 || fps > 1000.0 {
            return Err(MonochoraError::Config(format!("Invalid FPS value: {}", fps)));
        }
    }

    if args.speed.is_some() && args.fps.is_some() {
        return Err(MonochoraError::Config(
            "Cannot use both --speed and --fps at the same time".to_string()
        ));
    }

    if args.watch_terminal && !args.responsive {
        return Err(MonochoraError::Config(
            "Terminal watching (--watch-terminal) requires responsive mode (--responsive)".to_string()
        ));
    }

    if args.responsive && (args.gif_output.is_some() || args.save || args.output.is_some()) {
        return Err(MonochoraError::Config(
            "Responsive mode cannot be used with file output options".to_string()
        ));
    }

    validate_conflicting_options(args)?;
    validate_charset_options(args)?;

    Ok(())
}

fn validate_conflicting_options(args: &Args) -> Result<(), MonochoraError> {

    if args.white_on_black && args.black_on_white {
        return Err(MonochoraError::Config(
            "Cannot use both --white-on-black and --black-on-white at the same time".to_string()
        ));
    }

    let output_modes = [
        args.gif_output.is_some(),
        args.save || args.output.is_some(),
    ];
    let active_modes = output_modes.iter().filter(|&&x| x).count();
    
    if active_modes > 1 {
        return Err(MonochoraError::Config(
            "Cannot use multiple output modes simultaneously. Choose one: --gif-output, --save/--output, or terminal display".to_string()
        ));
    }

    if (args.white_on_black || args.black_on_white) && args.gif_output.is_none() {
        return Err(MonochoraError::Config(
            "Background color options (--white-on-black, --black-on-white) can only be used with --gif-output".to_string()
        ));
    }

    if args.font_size != 14.0 && args.gif_output.is_none() {
        return Err(MonochoraError::Config(
            "Font size (--font-size) can only be used with --gif-output".to_string()
        ));
    }

    if args.fit_terminal && (args.gif_output.is_some() || args.save || args.output.is_some()) {
        return Err(MonochoraError::Config(
            "Terminal fitting (--fit-terminal) cannot be used with file output options".to_string()
        ));
    }

    Ok(())
}

fn validate_charset_options(args: &Args) -> Result<(), MonochoraError> {
    if args.charset.is_some() && args.charset_file.is_some() {
        return Err(MonochoraError::Config(
            "Cannot use both --charset and --charset-file at the same time".to_string()
        ));
    }

    let charset_options_count = [
        args.simple,
        args.charset.is_some(),
        args.charset_file.is_some(),
    ].iter().filter(|&&x| x).count();

    if charset_options_count > 1 {
        return Err(MonochoraError::Config(
            "Cannot use multiple character set options simultaneously. Choose one: --simple, --charset, or --charset-file".to_string()
        ));
    }

    if let Some(charset) = &args.charset {
        validate_charset_string(charset)?;
    }

    Ok(())
}

fn validate_charset_string(charset: &str) -> Result<(), MonochoraError> {
    let chars: Vec<char> = charset.chars().collect();
    
    if chars.len() < 2 {
        return Err(MonochoraError::Config(
            "Character set must contain at least 2 characters".to_string()
        ));
    }
    
    if chars.len() > 256 {
        return Err(MonochoraError::Config(
            "Character set cannot exceed 256 characters".to_string()
        ));
    }
    
    for &ch in &chars {
        if ch.is_control() && ch != '\t' && ch != '\n' {
            return Err(MonochoraError::Config(
                format!("Character set contains invalid control character: {:?}", ch)
            ));
        }
    }
    
    let unique_chars: std::collections::HashSet<_> = chars.iter().collect();
    if unique_chars.len() != chars.len() {
        return Err(MonochoraError::Config(
            "Character set contains duplicate characters".to_string()
        ));
    }
    
    Ok(())
}

fn load_charset_from_file(path: &PathBuf) -> Result<String, MonochoraError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| MonochoraError::Config(format!("Failed to read charset file: {}", e)))?;
    
    let charset = content.trim().to_string();
    validate_charset_string(&charset)?;
    
    Ok(charset)
}

fn list_available_charsets() {
    println!("Available Character Sets:\n");
    
    println!("Built-in Sets:");
    println!("  simple:   {}", " .:-=+*#%@");
    println!("  detailed: {}", " .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@");
    
    println!("\nExample Custom Sets:");
    println!("  density:    \" .-+*#%@@\"");
    println!("  minimal:    \" .oO@\"");
    println!("  technical:  \" .-=+*#\"");
    println!("  artistic:   \" ·∘○●◉\"");
    println!("  japanese:   \"・〆ヲァィヵヶ\"");
    println!("  blocks:     \" ░▒▓█\"");
    
    println!("\nUsage:");
    println!("  --charset \" .oO@\"              # Inline character set");
    println!("  --charset-file ./my-chars.txt  # Load from file");
    println!("  --simple                       # Use simple built-in set");
    println!("  (default)                      # Use detailed built-in set");
    
    println!("\nCharacter Set Rules:");
    println!("  • Order from darkest to lightest");
    println!("  • Minimum 2 characters, maximum 256");
    println!("  • Must contain unique characters");
    println!("  • No control characters (except tab/newline in files)");
}

fn get_custom_charset(args: &Args) -> Result<Option<Vec<char>>, MonochoraError> {
    if let Some(charset_string) = &args.charset {
        return Ok(Some(charset_string.chars().collect()));
    }
    
    if let Some(charset_file) = &args.charset_file {
        let charset_string = load_charset_from_file(charset_file)?;
        return Ok(Some(charset_string.chars().collect()));
    }
    
    Ok(None)
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

fn generate_gif_output_path(input: &str, gif_output: &Option<Option<PathBuf>>) -> PathBuf {
    match gif_output {
        Some(Some(path)) => {
            if path.extension().is_none() {
                path.with_extension("gif")
            } else {
                path.clone()
            }
        }
        Some(None) => {
            if input.starts_with("http") {
                PathBuf::from("ascii_downloaded.gif")
            } else {
                let input_path = PathBuf::from(input);
                match input_path.file_stem() {
                    Some(stem) => {
                        let mut name = String::from("ascii_");
                        name.push_str(&stem.to_string_lossy());
                        name.push_str(".gif");
                        PathBuf::from(name)
                    }
                    None => PathBuf::from("ascii_output.gif")
                }
            }
        }
        None => unreachable!("This function should only be called when gif_output is Some"),
    }
}

fn calculate_adjusted_frame_delays(
    original_delays: &[u16],
    speed: Option<f32>,
    fps: Option<f32>,
    quiet: bool
) -> Vec<u16> {
    let adjusted_delays = if let Some(target_fps) = fps {
        let target_delay_ms = (1000.0 / target_fps) as u16;
        if !quiet {
            info!("Setting consistent frame rate to {:.1} FPS ({} ms per frame)", target_fps, target_delay_ms);
        }
        vec![target_delay_ms; original_delays.len()]
    } else if let Some(speed_mult) = speed {
        if !quiet {
            info!("Adjusting animation speed by {:.2}x", speed_mult);
        }
        original_delays.iter()
            .map(|&delay| {
                let adjusted = (delay as f32 / speed_mult) as u16;
                adjusted.max(1)
            })
            .collect()
    } else {
        original_delays.to_vec()
    };

    adjusted_delays
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
    
    let (ascii_frames, original_delays): (Vec<_>, Vec<_>) = results.into_iter().unzip();
    
    let adjusted_delays = calculate_adjusted_frame_delays(
        &original_delays,
        args.speed,
        args.fps,
        args.quiet
    );
    
    let conversion_time = start_time.elapsed();
    if !args.quiet {
        info!("ASCII conversion completed in {:.2}s", conversion_time.as_secs_f64());
    }

    Ok((ascii_frames, adjusted_delays))
}

async fn handle_gif_output(
    args: &Args,
    ascii_frames: &[Vec<String>],
    frame_delays: &[u16],
    gif_data: &monochora::handler::GifData,
) -> Result<(), MonochoraError> {
    let input = args.input.as_ref().unwrap();
    let output_path = generate_gif_output_path(input, &args.gif_output);
    
    if !args.quiet {
        info!("Generating ASCII GIF animation: {}", output_path.display());
    }
    
    let gif_start = std::time::Instant::now();
    
    let mut options = AsciiGifOutputOptions::default();
    options.font_size = args.font_size;
    options.colored = args.colored; 
    
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
    let input = args.input.as_ref().unwrap();
    let output_path = args.output.clone().unwrap_or_else(|| {
        generate_default_output_path(input)
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

async fn handle_responsive_terminal_display(
    args: &Args,
    _initial_frames: &[Vec<String>],
    frame_delays: &[u16],
    gif_data: &monochora::handler::GifData,
    config: &AsciiConverterConfig,
) -> Result<(), MonochoraError> {
    let initial_dims = TerminalDimensions::current()?;
    let mut frame_manager = ResponsiveFrameManager::new(
        gif_data.clone(),
        config.clone(),
        frame_delays.to_vec(),
        initial_dims,
        args.colored,
    );

    if args.watch_terminal {
        let mut watcher = TerminalWatcher::new()?;
        watcher.start_watching()?;
        let resize_rx = watcher.get_receiver();
        
        display_responsive_ascii_animation(&mut frame_manager, resize_rx, gif_data.loop_count).await
    } else {
        let frames = frame_manager.get_frames()?;
        display_ascii_animation(frames, frame_delays, gif_data.loop_count, true).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.list_charsets {
        list_available_charsets();
        return Ok(());
    }

    if let Err(e) = setup_logging(&args.log_level) {
        eprintln!("Warning: Failed to setup logging: {}", e);
    }

    if let Err(e) = validate_args(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = setup_thread_pool(args.threads, args.quiet) {
        error!("Failed to setup thread pool: {}", e);
        return Err(e.into());
    }

    let input = args.input.as_ref().unwrap();

    if !args.quiet {
        info!("Loading GIF: {}", input);
    }
    
    let input_path = get_input_path(input).await
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

    let custom_charset = get_custom_charset(&args)?;

    let config = AsciiConverterConfig {
        width: ascii_width,
        height: ascii_height,
        char_aspect: 0.5, 
        invert: args.invert,
        detailed: !args.simple,
        preserve_aspect_ratio: args.preserve_aspect,
        scale_factor: args.scale,
        custom_charset,
    };

    if !args.quiet && config.custom_charset.is_some() {
        info!("Using custom character set with {} characters", 
            config.custom_charset.as_ref().unwrap().len());
    }

    let (ascii_frames, frame_delays) = process_ascii_conversion(&args, &gif_data, &config).await?;

    if args.gif_output.is_some() {
        handle_gif_output(&args, &ascii_frames, &frame_delays, &gif_data).await?;
    } else if args.save || args.output.is_some() {
        handle_text_output(&args, &ascii_frames).await?;
    } else {
        if args.responsive {
            handle_responsive_terminal_display(&args, &ascii_frames, &frame_delays, &gif_data, &config).await?;
        } else {
            handle_terminal_display(&args, &ascii_frames, &frame_delays, gif_data.loop_count).await?;
        }
    }

    Ok(())
}
