use crate::{MonochoraError, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    terminal::{Clear, ClearType, size},
    event::{poll, read, Event, KeyCode},
};
use rayon::prelude::*;
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};
use crate::terminal_watcher::{ResponsiveFrameManager, TerminalDimensions};
use tokio::sync::watch;

pub fn get_terminal_size() -> Result<(u32, u32)> {
    let (cols, rows) = size()
        .map_err(|e| MonochoraError::Terminal(format!("Failed to get terminal size: {}", e)))?;
    
    if cols == 0 || rows == 0 {
        return Err(MonochoraError::Terminal("Terminal has zero dimensions".to_string()));
    }
    
    Ok((cols as u32, rows as u32))
}

fn validate_animation_input(
    frames: &[Vec<String>],
    frame_delays: &[u16],
    _loop_count: u16,
) -> Result<()> {
    if frames.is_empty() {
        return Err(MonochoraError::Animation("No frames provided for animation".to_string()));
    }
    
    if frame_delays.is_empty() {
        return Err(MonochoraError::Animation("No frame delays provided".to_string()));
    }
    
    let first_frame_lines = frames.first()
        .ok_or_else(|| MonochoraError::Animation("First frame is missing".to_string()))?
        .len();
    
    for (idx, frame) in frames.iter().enumerate() {
        if frame.is_empty() {
            warn!("Frame {} is empty", idx);
        }
        
        if frame.len() != first_frame_lines {
            debug!("Frame {} has {} lines, expected {}", idx, frame.len(), first_frame_lines);
        }
    }
    
    for (idx, &delay) in frame_delays.iter().enumerate() {
        if delay == 0 {
            debug!("Frame {} has zero delay, will use default", idx);
        }
    }
    
    Ok(())
}

pub async fn display_responsive_ascii_animation(
    frame_manager: &mut ResponsiveFrameManager,
    mut resize_rx: watch::Receiver<TerminalDimensions>,
    loop_count: u16,
) -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Hide)?;

    let iterations = if loop_count == 0 { usize::MAX } else { loop_count as usize };
    let mut current_iteration = 0;

    'outer: while current_iteration < iterations {
        let frames = frame_manager.get_frames()?.to_vec(); 
        let delays = frame_manager.get_frame_delays().to_vec(); 

        for (frame_idx, frame) in frames.iter().enumerate() {
            tokio::select! {
                _ = resize_rx.changed() => {
                    let new_dims = *resize_rx.borrow();
                    if frame_manager.update_dimensions(new_dims) {
                        continue 'outer;
                    }
                }
                _ = sleep(Duration::from_millis(delays[frame_idx] as u64)) => {
                    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
                    
                    for line in frame {
                        writeln!(stdout, "{}", line)?;
                    }
                    stdout.flush()?;

                    if poll(Duration::from_millis(0))? {
                        if let Ok(Event::Key(key)) = read() {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break 'outer,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        current_iteration += 1;
    }

    execute!(stdout, Show, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

pub async fn display_ascii_animation(
    frames: &[Vec<String>],
    frame_delays: &[u16],
    loop_count: u16,
    clear_on_exit: bool,
) -> Result<()> {
    validate_animation_input(frames, frame_delays, loop_count)?;
    
    let mut stdout = io::stdout();
    
    execute!(stdout, Hide)
        .map_err(|e| MonochoraError::Terminal(format!("Failed to hide cursor: {}", e)))?;
    
    let iterations = if loop_count == 0 {
        usize::MAX // Infinite loop
    } else {
        loop_count as usize
    };
    
    let mut current_iteration = 0;
    
    'outer: while current_iteration < iterations {
        for (frame_idx, frame) in frames.iter().enumerate() {
            execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))
                .map_err(|e| MonochoraError::Terminal(format!("Failed to clear screen: {}", e)))?;
            
            for (line_idx, line) in frame.iter().enumerate() {
                match writeln!(stdout, "{}", line) {
                    Ok(_) => {},
                    Err(e) => {
                        warn!("Failed to write line {} of frame {}: {}", line_idx, frame_idx, e);
                    }
                }
            }
            
            stdout.flush()
                .map_err(|e| MonochoraError::Terminal(format!("Failed to flush stdout: {}", e)))?;
            
            // Calculate frame delay
            let delay = if frame_idx < frame_delays.len() {
                let delay_ms = frame_delays[frame_idx];
                if delay_ms == 0 { 100 } else { delay_ms }
            } else if !frame_delays.is_empty() {
                let delay_ms = frame_delays[0];
                if delay_ms == 0 { 100 } else { delay_ms }
            } else {
                100 
            };
            
            sleep(Duration::from_millis(delay as u64)).await;
            
            match poll(Duration::from_millis(0)) {
                Ok(true) => {
                    match read() {
                        Ok(Event::Key(key)) => {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                    debug!("User requested exit");
                                    break 'outer;
                                }
                                KeyCode::Char('p') | KeyCode::Char('P') => {
                                    debug!("Animation paused, press any key to continue");
                                    match read() {
                                        Ok(_) => debug!("Animation resumed"),
                                        Err(e) => warn!("Failed to read resume input: {}", e),
                                    }
                                }
                                _ => {
                                }
                            }
                        }
                        Ok(_) => {
                        }
                        Err(e) => {
                            warn!("Failed to read terminal event: {}", e);
                        }
                    }
                }
                Ok(false) => {
                }
                Err(e) => {
                    warn!("Failed to poll for terminal events: {}", e);
                }
            }
        }
        
        current_iteration += 1;
        
        if current_iteration < iterations {
            sleep(Duration::from_millis(50)).await;
        }
    }
    
    execute!(stdout, Show)
        .map_err(|e| MonochoraError::Terminal(format!("Failed to show cursor: {}", e)))?;
    
    if clear_on_exit {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))
            .map_err(|e| MonochoraError::Terminal(format!("Failed to clear screen on exit: {}", e)))?;
    }
    
    Ok(())
}

pub fn save_ascii_to_file<P: AsRef<std::path::Path>>(
    frames: &[Vec<String>],
    path: P,
) -> Result<()> {
    use std::fs::File;
    
    if frames.is_empty() {
        return Err(MonochoraError::Animation("No frames to save".to_string()));
    }
    
    let path_ref = path.as_ref();
    
    if let Some(parent) = path_ref.parent() {
        if !parent.exists() {
            return Err(MonochoraError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Parent directory does not exist: {}", parent.display())
                )
            ));
        }
    }
    
    let file = File::create(path_ref)
        .map_err(|e| MonochoraError::Io(e))?;
    let mut writer = BufWriter::new(file);
    
    let separator = match String::from_utf8(vec![b'='; 80]) {
        Ok(s) => s,
        Err(_) => "=".repeat(80), 
    };
    
    debug!("Processing {} frames for file save", frames.len());
    
    let frame_results: Result<Vec<String>> = frames
        .par_iter()
        .enumerate()
        .map(|(i, frame)| -> Result<String> {
            let mut frame_content = String::new();
            
            frame_content.push_str(&separator);
            frame_content.push('\n');
            frame_content.push_str(&format!("Frame {}\n", i + 1));
            frame_content.push_str(&separator);
            frame_content.push('\n');
            
            for line in frame {
                frame_content.push_str(line);
                frame_content.push('\n');
            }
            frame_content.push('\n');
            
            Ok(frame_content)
        })
        .collect();
    
    let frame_strings = frame_results?;
    
    for (idx, frame_string) in frame_strings.iter().enumerate() {
        write!(writer, "{}", frame_string)
            .map_err(|e| MonochoraError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    format!("Failed to write frame {} to file: {}", idx, e)
                )
            ))?;
    }
    
    use std::io::BufWriter;
    match writer.into_inner() {
        Ok(file) => {
            file.sync_all()
                .map_err(|e| MonochoraError::Io(e))?;
        }
        Err(into_inner_error) => {
            return Err(MonochoraError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to finalize file write: {}", into_inner_error.error())
                )
            ));
        }
    }
    
    debug!("Successfully saved {} frames to {}", frames.len(), path_ref.display());
    Ok(())
}
