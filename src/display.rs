use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    terminal::{Clear, ClearType, size},
};
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;

pub fn get_terminal_size() -> Result<(u32, u32)> {
    let (cols, rows) = size().context("Failed to get terminal size")?;
    Ok((cols as u32, rows as u32))
}

pub async fn display_ascii_animation(
    frames: &[Vec<String>],
    frame_delays: &[u16],
    loop_count: u16,
    clear_on_exit: bool,
) -> Result<()> {
    let mut stdout = io::stdout();
    
    execute!(stdout, Hide)?;
    
    let iterations = if loop_count == 0 {
        usize::MAX
    } else {
        loop_count as usize
    };
    
    'outer: for _ in 0..iterations {
        for (frame_idx, frame) in frames.iter().enumerate() {
            execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
            
            for line in frame {
                writeln!(stdout, "{}", line)?;
            }
            
            stdout.flush()?;
            
            let delay = if frame_idx < frame_delays.len() {
                frame_delays[frame_idx]
            } else if !frame_delays.is_empty() {
                frame_delays[0] 
            } else {
                100 
            };
            
            sleep(Duration::from_millis(delay as u64)).await;
            
            if crossterm::event::poll(Duration::from_millis(0))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    if key.code == crossterm::event::KeyCode::Esc
                        || key.code == crossterm::event::KeyCode::Char('q')
                        || key.code == crossterm::event::KeyCode::Char('Q')
                    {
                        break 'outer;
                    }
                }
            }
        }
    }
    
    execute!(stdout, Show)?;
    
    if clear_on_exit {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    }
    
    Ok(())
}

pub fn save_ascii_to_file<P: AsRef<std::path::Path>>(
    frames: &[Vec<String>],
    path: P,
) -> Result<()> {
    use std::fs::File;
    use std::io::BufWriter;
    
    let file = File::create(path).context("Failed to create output file")?;
    let mut writer = BufWriter::new(file);
    
    let separator = String::from_utf8(vec![b'='; 80]).unwrap();
    
    for (i, frame) in frames.iter().enumerate() {
        writeln!(writer, "{}", separator)?;
        writeln!(writer, "Frame {}", i + 1)?;
        writeln!(writer, "{}", separator)?;
        
        for line in frame {
            writeln!(writer, "{}", line)?;
        }
        
        writeln!(writer)?;
    }
    
    Ok(())
}
