use crate::{MonochoraError, Result};
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const PAGE_TEMPLATE: &str = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>__TITLE__</title>
<style>
html,body{margin:0;height:100%;background:__BG__;color:__FG__}
body{display:flex;flex-direction:column;align-items:center;justify-content:center;overflow:hidden}
pre,.probe{font-family:"DejaVu Sans Mono",Menlo,Consolas,"Liberation Mono","Courier New",monospace}
pre{margin:0;white-space:pre;cursor:pointer;user-select:none}
i{font-style:normal}
u{text-decoration:none;letter-spacing:var(--braille-spacing,0)}
b{display:inline-block;height:var(--line-height);vertical-align:top;font-weight:normal}
#status{font:13px/1.6 system-ui,-apple-system,"Segoe UI",sans-serif;min-height:1.6em;margin-top:6px;opacity:.75;visibility:hidden}
.probe{position:absolute;left:-9999px;top:0;visibility:hidden;white-space:pre;font-size:100px}
</style>
</head>
<body>
<pre id="art"></pre>
<div id="status"></div>
<span id="cell-probe" class="probe">MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM</span>
<span id="braille-probe" class="probe">⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿</span>
<script id="animation-data" type="application/json">__DATA__</script>
<script>
const data = JSON.parse(document.getElementById("animation-data").textContent);
const art = document.getElementById("art");
const statusLine = document.getElementById("status");
const speeds = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 2, 3, 4];
const normalSpeed = 3;
const loopGap = 50;
const statusTime = 1500;
const frameCount = data.frames.length;
const iterations = data.loop === 0 ? Infinity : data.loop;
let frame = 0;
let iteration = 0;
let paused = false;
let speedIndex = normalSpeed;
let statusUntil = 0;
let nextFrameAt = 0;
let timer = null;

function frameDelay(index) {
  const delays = data.delays;
  const delay = index < delays.length ? delays[index] : (delays.length ? delays[0] : 0);
  return delay === 0 ? 100 : delay;
}

function frameDuration() {
  const duration = frameDelay(frame) / speeds[speedIndex];
  const lastFrame = frame === frameCount - 1;
  return lastFrame && iteration + 1 < iterations ? duration + loopGap : duration;
}

function showStatus() {
  const visible = paused || performance.now() < statusUntil;
  statusLine.style.visibility = visible ? "visible" : "hidden";
  statusLine.textContent = (paused ? "Paused" : "Playing") + " · frame " + (frame + 1) + "/" + frameCount +
    " · " + speeds[speedIndex] + "× · space " + (paused ? "resume" : "pause") + " · ←/→ step · +/- speed";
}

function draw() {
  art.innerHTML = data.frames[frame];
  showStatus();
}

function schedule() {
  clearTimeout(timer);
  if (paused || frameCount < 2) {
    return;
  }
  timer = setTimeout(advance, Math.max(0, nextFrameAt - performance.now()));
}

function advance() {
  if (frame === frameCount - 1) {
    iteration += 1;
    if (iteration >= iterations) {
      return;
    }
    frame = 0;
  } else {
    frame += 1;
  }
  draw();
  nextFrameAt = Math.max(nextFrameAt + frameDuration(), performance.now());
  schedule();
}

function togglePause() {
  paused = !paused;
  if (!paused) {
    nextFrameAt = performance.now() + frameDuration();
  }
  showStatus();
  schedule();
}

function step(offset) {
  paused = true;
  frame = (frame + offset + frameCount) % frameCount;
  draw();
  schedule();
}

function changeSpeed(index) {
  speedIndex = Math.min(Math.max(index, 0), speeds.length - 1);
  statusUntil = performance.now() + statusTime;
  showStatus();
  setTimeout(showStatus, statusTime);
}

function fit() {
  art.style.fontSize = "1px";
  art.style.lineHeight = "2px";
  void art.offsetWidth;
  const cellRatio = document.getElementById("cell-probe").getBoundingClientRect().width / 10000;
  const brailleRatio = document.getElementById("braille-probe").getBoundingClientRect().width / 10000;
  const availableWidth = document.documentElement.clientWidth * 0.96;
  const availableHeight = document.documentElement.clientHeight * 0.96 - statusLine.offsetHeight - 6;
  const size = Math.max(1, Math.min(availableWidth / (data.cols * cellRatio), availableHeight / (data.rows * 2 * cellRatio)));
  const lineHeight = 2 * cellRatio * size;
  art.style.fontSize = size + "px";
  art.style.lineHeight = lineHeight + "px";
  art.style.setProperty("--line-height", lineHeight + "px");
  art.style.setProperty("--braille-spacing", (cellRatio - brailleRatio) * size + "px");
}

document.addEventListener("keydown", (event) => {
  switch (event.key) {
    case " ":
    case "p":
    case "P":
      togglePause();
      break;
    case "ArrowLeft":
      step(-1);
      break;
    case "ArrowRight":
      step(1);
      break;
    case "+":
    case "=":
      changeSpeed(speedIndex + 1);
      break;
    case "-":
    case "_":
      changeSpeed(speedIndex - 1);
      break;
    case "0":
      changeSpeed(normalSpeed);
      break;
    default:
      return;
  }
  event.preventDefault();
});

art.addEventListener("click", togglePause);
window.addEventListener("resize", fit);
if (window.visualViewport) {
  window.visualViewport.addEventListener("resize", fit);
}
fit();
draw();
nextFrameAt = performance.now() + frameDuration();
schedule();
</script>
</body>
</html>
"#;

pub struct HtmlOutputOptions {
    pub title: String,
    pub bg_color: [u8; 3],
    pub text_color: [u8; 3],
}

impl Default for HtmlOutputOptions {
    fn default() -> Self {
        Self {
            title: "monochora".to_string(),
            bg_color: [0, 0, 0],
            text_color: [255, 255, 255],
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Cell {
    Text { ch: char, fg: Option<[u8; 3]>, bg: Option<[u8; 3]> },
    Braille { ch: char, fg: Option<[u8; 3]> },
    Block { top: Option<[u8; 3]>, bottom: Option<[u8; 3]> },
}

#[derive(PartialEq)]
enum RunStyle {
    Text(Option<[u8; 3]>, Option<[u8; 3]>),
    Braille(Option<[u8; 3]>),
    Block(Option<[u8; 3]>, Option<[u8; 3]>),
}

impl Cell {
    fn style(&self) -> RunStyle {
        match *self {
            Cell::Text { fg, bg, .. } => RunStyle::Text(fg, bg),
            Cell::Braille { fg, .. } => RunStyle::Braille(fg),
            Cell::Block { top, bottom } => RunStyle::Block(top, bottom),
        }
    }
}

pub fn ascii_frames_to_html<P: AsRef<Path>>(
    ascii_frames: &[Vec<String>],
    frame_delays: &[u16],
    loop_count: u16,
    output_path: P,
    options: &HtmlOutputOptions,
) -> Result<()> {
    if ascii_frames.is_empty() {
        return Err(MonochoraError::Config("No ASCII frames to convert".to_string()));
    }

    let page = render_html_page(ascii_frames, frame_delays, loop_count, options);
    let file = File::create(output_path.as_ref()).map_err(MonochoraError::Io)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(page.as_bytes())?;
    writer.flush()?;
    Ok(())
}

fn render_html_page(
    ascii_frames: &[Vec<String>],
    frame_delays: &[u16],
    loop_count: u16,
    options: &HtmlOutputOptions,
) -> String {
    let mut cols = 0;
    let mut rows = 0;
    let mut frames_json = String::from("[");

    for (index, frame) in ascii_frames.iter().enumerate() {
        let mut frame_html = String::new();
        rows = rows.max(frame.len());

        for (line_index, line) in frame.iter().enumerate() {
            let cells = parse_line(line, options.text_color);
            cols = cols.max(cells.len());
            if line_index > 0 {
                frame_html.push('\n');
            }
            push_line_html(&mut frame_html, &cells);
        }

        if index > 0 {
            frames_json.push(',');
        }
        push_json_string(&mut frames_json, &frame_html);
    }
    frames_json.push(']');

    let delays_json = frame_delays.iter().map(|delay| delay.to_string()).collect::<Vec<_>>().join(",");
    let data = format!(
        "{{\"frames\":{},\"delays\":[{}],\"loop\":{},\"cols\":{},\"rows\":{}}}",
        frames_json,
        delays_json,
        loop_count,
        cols.max(1),
        rows.max(1)
    );

    let mut title = String::new();
    push_escaped_text(&mut title, &options.title);

    PAGE_TEMPLATE
        .replace("__TITLE__", &title)
        .replace("__BG__", &hex_color(options.bg_color))
        .replace("__FG__", &hex_color(options.text_color))
        .replace("__DATA__", &data)
}

pub(crate) fn parse_line(line: &str, text_color: [u8; 3]) -> Vec<Cell> {
    let mut cells = Vec::new();
    let mut fg: Option<[u8; 3]> = None;
    let mut bg: Option<[u8; 3]> = None;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                let mut params = String::new();
                for next in chars.by_ref() {
                    if next == 'm' {
                        break;
                    }
                    params.push(next);
                }
                apply_sgr(&params, &mut fg, &mut bg);
            }
            continue;
        }

        let foreground = fg.unwrap_or(text_color);
        let cell = match ch {
            '▀' => Cell::Block { top: Some(foreground), bottom: bg },
            '▄' => Cell::Block { top: bg, bottom: Some(foreground) },
            '█' => Cell::Block { top: Some(foreground), bottom: Some(foreground) },
            ' ' if bg.is_some() => Cell::Block { top: bg, bottom: bg },
            '\u{2800}'..='\u{28ff}' => Cell::Braille { ch, fg },
            _ => Cell::Text { ch, fg, bg },
        };
        cells.push(cell);
    }

    cells
}

fn apply_sgr(params: &str, fg: &mut Option<[u8; 3]>, bg: &mut Option<[u8; 3]>) {
    let values: Vec<u16> = params.split(';').filter_map(|value| value.parse().ok()).collect();
    let mut index = 0;

    while index < values.len() {
        match values[index] {
            0 => {
                *fg = None;
                *bg = None;
            }
            39 => *fg = None,
            49 => *bg = None,
            38 | 48 if values.get(index + 1) == Some(&2) && index + 4 < values.len() => {
                let color = Some([values[index + 2] as u8, values[index + 3] as u8, values[index + 4] as u8]);
                if values[index] == 38 {
                    *fg = color;
                } else {
                    *bg = color;
                }
                index += 4;
            }
            _ => {}
        }
        index += 1;
    }
}

fn push_line_html(html: &mut String, cells: &[Cell]) {
    let mut start = 0;

    while start < cells.len() {
        let style = cells[start].style();
        let end = cells[start..]
            .iter()
            .position(|cell| cell.style() != style)
            .map_or(cells.len(), |offset| start + offset);
        let run = &cells[start..end];

        match style {
            RunStyle::Text(fg, bg) => {
                let open = fg.is_some() || bg.is_some();
                if open {
                    html.push_str("<i style=");
                    if let Some(fg) = fg {
                        let _ = write!(html, "color:{};", hex_color(fg));
                    }
                    if let Some(bg) = bg {
                        let _ = write!(html, "background:{};", hex_color(bg));
                    }
                    html.push('>');
                }
                for cell in run {
                    if let Cell::Text { ch, .. } = cell {
                        push_escaped_char(html, *ch);
                    }
                }
                if open {
                    html.push_str("</i>");
                }
            }
            RunStyle::Braille(fg) => {
                match fg {
                    Some(fg) => {
                        let _ = write!(html, "<u style=color:{}>", hex_color(fg));
                    }
                    None => html.push_str("<u>"),
                }
                for cell in run {
                    if let Cell::Braille { ch, .. } = cell {
                        html.push(*ch);
                    }
                }
                html.push_str("</u>");
            }
            RunStyle::Block(top, bottom) => {
                let spaces = " ".repeat(run.len());
                match (top, bottom) {
                    (None, None) => html.push_str(&spaces),
                    (Some(top), Some(bottom)) if top == bottom => {
                        let _ = write!(html, "<b style=background:{}>{}</b>", hex_color(top), spaces);
                    }
                    _ => {
                        let _ = write!(
                            html,
                            "<b style=\"background:linear-gradient({} 50%,{} 50%)\">{}</b>",
                            top.map_or("transparent".to_string(), hex_color),
                            bottom.map_or("transparent".to_string(), hex_color),
                            spaces
                        );
                    }
                }
            }
        }

        start = end;
    }
}

fn hex_color([r, g, b]: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn push_escaped_char(html: &mut String, ch: char) {
    match ch {
        '&' => html.push_str("&amp;"),
        '<' => html.push_str("&lt;"),
        '>' => html.push_str("&gt;"),
        _ => html.push(ch),
    }
}

fn push_escaped_text(html: &mut String, text: &str) {
    for ch in text.chars() {
        push_escaped_char(html, ch);
    }
}

fn push_json_string(json: &mut String, value: &str) {
    json.push('"');
    let mut previous = '\0';
    for ch in value.chars() {
        match ch {
            '"' => json.push_str("\\\""),
            '\\' => json.push_str("\\\\"),
            '\n' => json.push_str("\\n"),
            '\r' => json.push_str("\\r"),
            '\t' => json.push_str("\\t"),
            '/' if previous == '<' => json.push_str("\\/"),
            ch if (ch as u32) < 0x20 => {
                let _ = write!(json, "\\u{:04x}", ch as u32);
            }
            ch => json.push(ch),
        }
        previous = ch;
    }
    json.push('"');
}
