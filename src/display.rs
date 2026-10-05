use crate::{MonochoraError, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    terminal::{
        disable_raw_mode, enable_raw_mode, BeginSynchronizedUpdate, Clear, ClearType,
        EndSynchronizedUpdate, size,
    },
    event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};
use futures_core::Stream;
use std::future::{pending, poll_fn};
use std::io::{self, BufWriter, Write};
use std::pin::Pin;
use std::time::Duration;
use tokio::time::{sleep_until, timeout, Instant};
use tracing::{debug, warn};
use crate::terminal_watcher::{ResponsiveFrameManager, TerminalDimensions};
use tokio::sync::watch;

const RESIZE_SETTLE_TIME: Duration = Duration::from_millis(150);
const STATUS_DISPLAY_TIME: Duration = Duration::from_millis(1500);
const LOOP_GAP: Duration = Duration::from_millis(50);
const DEFAULT_FRAME_DELAY_MS: u16 = 100;
const SPEEDS: [f64; 9] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0];
const NORMAL_SPEED_INDEX: usize = 3;

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

fn render_frame<W: Write>(
    out: &mut W,
    buffer: &mut Vec<u8>,
    frame: &[String],
    status: Option<&str>,
) -> io::Result<()> {
    buffer.clear();
    queue!(buffer, BeginSynchronizedUpdate, MoveTo(0, 0))?;
    
    for line in frame {
        queue!(buffer, Clear(ClearType::CurrentLine))?;
        buffer.extend_from_slice(line.as_bytes());
        buffer.extend_from_slice(b"\r\n");
    }
    
    if let Some(status) = status {
        queue!(buffer, Clear(ClearType::CurrentLine))?;
        buffer.extend_from_slice(status.as_bytes());
    }
    
    queue!(buffer, Clear(ClearType::FromCursorDown), EndSynchronizedUpdate)?;
    out.write_all(buffer)?;
    out.flush()
}

async fn wait_for_resize_to_settle(resize_rx: &mut watch::Receiver<TerminalDimensions>) -> TerminalDimensions {
    while let Ok(Ok(())) = timeout(RESIZE_SETTLE_TIME, resize_rx.changed()).await {}
    *resize_rx.borrow()
}

async fn next_event(events: &mut Option<EventStream>) -> Option<io::Result<Event>> {
    match events {
        Some(stream) => poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)).await,
        None => pending().await,
    }
}

async fn resize_changed(resize_rx: &mut Option<watch::Receiver<TerminalDimensions>>) -> bool {
    match resize_rx {
        Some(rx) => rx.changed().await.is_ok(),
        None => pending().await,
    }
}

struct PlaybackTerminal {
    raw_mode: bool,
}

impl PlaybackTerminal {
    fn start() -> Result<Self> {
        let raw_mode = match enable_raw_mode() {
            Ok(()) => true,
            Err(e) => {
                debug!("Keyboard controls unavailable: {}", e);
                false
            }
        };
        
        if let Err(e) = execute!(io::stdout(), Hide, Clear(ClearType::All)) {
            if raw_mode {
                let _ = disable_raw_mode();
            }
            return Err(MonochoraError::Terminal(format!("Failed to hide cursor: {}", e)));
        }
        
        Ok(Self { raw_mode })
    }
}

impl Drop for PlaybackTerminal {
    fn drop(&mut self) {
        if self.raw_mode {
            let _ = disable_raw_mode();
        }
        let _ = execute!(io::stdout(), Show);
    }
}

enum FrameSource<'a> {
    Fixed {
        frames: &'a [Vec<String>],
        delays: &'a [u16],
    },
    Responsive {
        manager: &'a mut ResponsiveFrameManager,
    },
}

impl FrameSource<'_> {
    fn frame_count(&mut self) -> Result<usize> {
        match self {
            FrameSource::Fixed { frames, .. } => Ok(frames.len()),
            FrameSource::Responsive { manager } => Ok(manager.get_frames()?.len()),
        }
    }

    fn frame(&mut self, idx: usize) -> Result<&[String]> {
        match self {
            FrameSource::Fixed { frames, .. } => Ok(&frames[idx]),
            FrameSource::Responsive { manager } => Ok(&manager.get_frames()?[idx]),
        }
    }

    fn delay_ms(&self, idx: usize) -> u16 {
        match self {
            FrameSource::Fixed { delays, .. } => {
                let delay_ms = delays.get(idx).or(delays.first()).copied().unwrap_or(0);
                if delay_ms == 0 { DEFAULT_FRAME_DELAY_MS } else { delay_ms }
            }
            FrameSource::Responsive { manager } => manager.get_frame_delays()[idx],
        }
    }

    fn loop_gap(&self) -> Duration {
        match self {
            FrameSource::Fixed { .. } => LOOP_GAP,
            FrameSource::Responsive { .. } => Duration::ZERO,
        }
    }
}

enum PlaybackAction {
    Quit,
    TogglePause,
    StepBack,
    StepForward,
    Faster,
    Slower,
    NormalSpeed,
}

fn playback_action(key: KeyEvent) -> Option<PlaybackAction> {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(PlaybackAction::Quit),
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => Some(PlaybackAction::Quit),
        KeyCode::Char(' ') | KeyCode::Char('p') | KeyCode::Char('P') => Some(PlaybackAction::TogglePause),
        KeyCode::Left => Some(PlaybackAction::StepBack),
        KeyCode::Right => Some(PlaybackAction::StepForward),
        KeyCode::Char('+') | KeyCode::Char('=') => Some(PlaybackAction::Faster),
        KeyCode::Char('-') | KeyCode::Char('_') => Some(PlaybackAction::Slower),
        KeyCode::Char('0') => Some(PlaybackAction::NormalSpeed),
        _ => None,
    }
}

struct PlaybackState {
    frame_idx: usize,
    frame_count: usize,
    iteration: usize,
    iterations: usize,
    paused: bool,
    speed_index: usize,
    status_until: Option<Instant>,
}

impl PlaybackState {
    fn speed(&self) -> f64 {
        SPEEDS[self.speed_index]
    }

    fn is_last_frame(&self) -> bool {
        self.frame_idx + 1 == self.frame_count
    }

    fn frame_duration(&self, source: &FrameSource) -> Duration {
        let delay = Duration::from_secs_f64(source.delay_ms(self.frame_idx) as f64 / 1000.0 / self.speed());
        if self.is_last_frame() && self.iteration + 1 < self.iterations {
            delay + source.loop_gap()
        } else {
            delay
        }
    }

    fn status_line(&self) -> Option<String> {
        let visible = self.paused || self.status_until.is_some_and(|until| Instant::now() < until);
        if !visible {
            return None;
        }
        
        let status = format!(
            "{} · frame {}/{} · {}× · space {} · ←/→ step · +/- speed · q quit",
            if self.paused { "Paused" } else { "Playing" },
            self.frame_idx + 1,
            self.frame_count,
            self.speed(),
            if self.paused { "resume" } else { "pause" },
        );
        let max_width = size().map(|(cols, _)| cols as usize).unwrap_or(80).saturating_sub(1);
        Some(status.chars().take(max_width).collect())
    }
}

fn draw(
    out: &mut io::Stdout,
    buffer: &mut Vec<u8>,
    source: &mut FrameSource,
    state: &PlaybackState,
) -> Result<()> {
    let status = state.status_line();
    let frame = source.frame(state.frame_idx)?;
    render_frame(out, buffer, frame, status.as_deref())
        .map_err(|e| MonochoraError::Terminal(format!("Failed to write frame {}: {}", state.frame_idx, e)))
}

async fn play(
    mut source: FrameSource<'_>,
    loop_count: u16,
    mut resize_rx: Option<watch::Receiver<TerminalDimensions>>,
    clear_on_exit: bool,
) -> Result<()> {
    let terminal = PlaybackTerminal::start()?;
    let mut events = if terminal.raw_mode { Some(EventStream::new()) } else { None };
    let mut stdout = io::stdout();
    let mut buffer = Vec::new();
    
    let mut state = PlaybackState {
        frame_idx: 0,
        frame_count: source.frame_count()?,
        iteration: 0,
        iterations: if loop_count == 0 { usize::MAX } else { loop_count as usize },
        paused: false,
        speed_index: NORMAL_SPEED_INDEX,
        status_until: None,
    };
    
    draw(&mut stdout, &mut buffer, &mut source, &state)?;
    let mut next_frame_at = Instant::now() + state.frame_duration(&source);
    
    loop {
        tokio::select! {
            event = next_event(&mut events) => {
                let key = match event {
                    Some(Ok(Event::Key(key))) if key.kind != KeyEventKind::Release => key,
                    Some(Ok(_)) => continue,
                    _ => {
                        events = None;
                        continue;
                    }
                };
                
                match playback_action(key) {
                    Some(PlaybackAction::Quit) => {
                        debug!("User requested exit");
                        break;
                    }
                    Some(PlaybackAction::TogglePause) => {
                        state.paused = !state.paused;
                        if !state.paused {
                            next_frame_at = Instant::now() + state.frame_duration(&source);
                        }
                    }
                    Some(PlaybackAction::StepBack) => {
                        state.paused = true;
                        state.frame_idx = (state.frame_idx + state.frame_count - 1) % state.frame_count;
                    }
                    Some(PlaybackAction::StepForward) => {
                        state.paused = true;
                        state.frame_idx = (state.frame_idx + 1) % state.frame_count;
                    }
                    Some(PlaybackAction::Faster) => {
                        state.speed_index = (state.speed_index + 1).min(SPEEDS.len() - 1);
                        state.status_until = Some(Instant::now() + STATUS_DISPLAY_TIME);
                    }
                    Some(PlaybackAction::Slower) => {
                        state.speed_index = state.speed_index.saturating_sub(1);
                        state.status_until = Some(Instant::now() + STATUS_DISPLAY_TIME);
                    }
                    Some(PlaybackAction::NormalSpeed) => {
                        state.speed_index = NORMAL_SPEED_INDEX;
                        state.status_until = Some(Instant::now() + STATUS_DISPLAY_TIME);
                    }
                    None => continue,
                }
                
                draw(&mut stdout, &mut buffer, &mut source, &state)?;
            }
            changed = resize_changed(&mut resize_rx) => {
                let Some(rx) = resize_rx.as_mut().filter(|_| changed) else {
                    resize_rx = None;
                    continue;
                };
                
                let new_dims = wait_for_resize_to_settle(rx).await;
                if let FrameSource::Responsive { manager } = &mut source
                    && manager.update_dimensions(new_dims)
                {
                    draw(&mut stdout, &mut buffer, &mut source, &state)?;
                }
            }
            _ = sleep_until(next_frame_at), if !state.paused => {
                if state.is_last_frame() {
                    state.iteration += 1;
                    if state.iteration >= state.iterations {
                        break;
                    }
                    state.frame_idx = 0;
                } else {
                    state.frame_idx += 1;
                }
                
                draw(&mut stdout, &mut buffer, &mut source, &state)?;
                next_frame_at = (next_frame_at + state.frame_duration(&source)).max(Instant::now());
            }
        }
    }
    
    drop(events);
    
    if clear_on_exit {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))
            .map_err(|e| MonochoraError::Terminal(format!("Failed to clear screen on exit: {}", e)))?;
    }
    
    drop(terminal);
    Ok(())
}

pub async fn display_responsive_ascii_animation(
    frame_manager: &mut ResponsiveFrameManager,
    resize_rx: watch::Receiver<TerminalDimensions>,
    loop_count: u16,
) -> Result<()> {
    play(FrameSource::Responsive { manager: frame_manager }, loop_count, Some(resize_rx), true).await
}

pub async fn display_ascii_animation(
    frames: &[Vec<String>],
    frame_delays: &[u16],
    loop_count: u16,
    clear_on_exit: bool,
) -> Result<()> {
    validate_animation_input(frames, frame_delays, loop_count)?;
    play(FrameSource::Fixed { frames, delays: frame_delays }, loop_count, None, clear_on_exit).await
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
    
    if let Some(parent) = path_ref.parent().filter(|parent| !parent.as_os_str().is_empty()) {
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
