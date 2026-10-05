use crate::{MonochoraError, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    terminal::{BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate, size},
    event::{poll, read, Event, KeyCode},
};
use std::io::{self, BufWriter, Write};
use std::time::Duration;
use tokio::time::{sleep_until, timeout, Instant};
use tracing::{debug, warn};
use crate::terminal_watcher::{ResponsiveFrameManager, TerminalDimensions};
use tokio::sync::watch;

const RESIZE_SETTLE_TIME: Duration = Duration::from_millis(150);

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

fn render_frame<W: Write>(out: &mut W, buffer: &mut Vec<u8>, frame: &[String]) -> io::Result<()> {
    buffer.clear();
    queue!(buffer, BeginSynchronizedUpdate, MoveTo(0, 0))?;
    
    for line in frame {
        queue!(buffer, Clear(ClearType::CurrentLine))?;
        buffer.extend_from_slice(line.as_bytes());
        buffer.push(b'\n');
    }
    
    queue!(buffer, Clear(ClearType::FromCursorDown), EndSynchronizedUpdate)?;
    out.write_all(buffer)?;
    out.flush()
}

async fn wait_for_resize_to_settle(resize_rx: &mut watch::Receiver<TerminalDimensions>) -> TerminalDimensions {
    while let Ok(Ok(())) = timeout(RESIZE_SETTLE_TIME, resize_rx.changed()).await {}
    *resize_rx.borrow()
}

pub async fn display_responsive_ascii_animation(
    frame_manager: &mut ResponsiveFrameManager,
    mut resize_rx: watch::Receiver<TerminalDimensions>,
    loop_count: u16,
) -> Result<()> {
    let mut stdout = io::stdout();
    let mut buffer = Vec::new();
    execute!(stdout, Hide, Clear(ClearType::All))?;

    let iterations = if loop_count == 0 { usize::MAX } else { loop_count as usize };
    let mut current_iteration = 0;
    let mut next_frame_at = Instant::now();

    'outer: while current_iteration < iterations {
        let frame_count = frame_manager.get_frames()?.len();

        for frame_idx in 0..frame_count {
            let delay = Duration::from_millis(frame_manager.get_frame_delays()[frame_idx] as u64);
            next_frame_at = (next_frame_at + delay).max(Instant::now());

            tokio::select! {
                Ok(()) = resize_rx.changed() => {
                    let new_dims = wait_for_resize_to_settle(&mut resize_rx).await;
                    if frame_manager.update_dimensions(new_dims) {
                        next_frame_at = Instant::now();
                        continue 'outer;
                    }
                }
                _ = sleep_until(next_frame_at) => {
                    render_frame(&mut stdout, &mut buffer, &frame_manager.get_frames()?[frame_idx])?;

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
    let mut buffer = Vec::new();
    
    execute!(stdout, Hide, Clear(ClearType::All))
        .map_err(|e| MonochoraError::Terminal(format!("Failed to hide cursor: {}", e)))?;
    
    let iterations = if loop_count == 0 {
        usize::MAX // Infinite loop
    } else {
        loop_count as usize
    };
    
    let mut current_iteration = 0;
    let mut next_frame_at = Instant::now();
    
    'outer: while current_iteration < iterations {
        for (frame_idx, frame) in frames.iter().enumerate() {
            render_frame(&mut stdout, &mut buffer, frame)
                .map_err(|e| MonochoraError::Terminal(format!("Failed to write frame {}: {}", frame_idx, e)))?;
            
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
            
            next_frame_at = (next_frame_at + Duration::from_millis(delay as u64)).max(Instant::now());
            sleep_until(next_frame_at).await;
            
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
                                    next_frame_at = Instant::now();
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
            next_frame_at += Duration::from_millis(50);
            sleep_until(next_frame_at).await;
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
    
    let separator = "=".repeat(80);
    
    debug!("Processing {} frames for file save", frames.len());
    
    for (idx, frame) in frames.iter().enumerate() {
        write_text_frame(&mut writer, idx, frame, &separator)
            .map_err(|e| MonochoraError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    format!("Failed to write frame {} to file: {}", idx, e)
                )
            ))?;
    }
    
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

fn write_text_frame<W: Write>(writer: &mut W, idx: usize, frame: &[String], separator: &str) -> io::Result<()> {
    writeln!(writer, "{}", separator)?;
    writeln!(writer, "Frame {}", idx + 1)?;
    writeln!(writer, "{}", separator)?;
    
    for line in frame {
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    
    writer.write_all(b"\n")
}
