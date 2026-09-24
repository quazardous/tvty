//! A terminal: `alacritty_terminal` does the emulation and owns the PTY, a
//! GPUI element paints its grid, and keystrokes go back to the PTY as bytes.

use std::borrow::Cow;
use std::sync::Arc;

use alacritty_terminal::event::{Event, EventListener, Notify, OnResize, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, Notifier};
use alacritty_terminal::grid::Scroll;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::tty;
use alacritty_terminal::vte::ansi::{Color, CursorShape, NamedColor, Rgb};
use futures::StreamExt;
use futures::channel::mpsc::{UnboundedSender, unbounded};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::stats;

const FONT_FAMILY: &str = "Source Code Pro";
/// Tried in order for glyphs the main font lacks (emoji, CJK, symbols).
const FONT_FALLBACKS: &[&str] = &["Noto Color Emoji", "Noto Sans CJK JP", "Adwaita Mono"];
const FONT_SIZE: f32 = 14.;
const LINE_HEIGHT: f32 = 1.3;

/// Forwards the emulator's events (from its I/O thread) to the view.
#[derive(Clone)]
struct Listener(UnboundedSender<Event>);

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        let _ = self.0.unbounded_send(event);
    }
}

pub struct TerminalView {
    term: Arc<FairMutex<Term<Listener>>>,
    notifier: Notifier,
    focus: FocusHandle,
    /// Grid size last sent to the PTY: (columns, lines).
    grid_size: (u16, u16),
    exited: bool,
    /// The tmux session shown, to scroll its history in copy mode.
    tmux_session: Option<String>,
    /// Where the grid was last painted, and its cell size: for the mouse.
    layout: (Point<Pixels>, Size<Pixels>),
    /// Wheel movement not yet worth a line.
    scroll_remainder: f32,
}

impl TerminalView {
    /// Attaches to a tmux session.
    pub fn tmux(session: &str, cx: &mut Context<Self>) -> anyhow::Result<Self> {
        let mut view = Self::new("tmux", &["attach", "-t", &format!("={session}")], cx)?;
        view.tmux_session = Some(session.to_string());
        Ok(view)
    }

    /// Spawns `program args` in a new PTY.
    pub fn new(program: &str, args: &[&str], cx: &mut Context<Self>) -> anyhow::Result<Self> {
        let (tx, mut rx) = unbounded();
        let listener = Listener(tx);
        let (columns, lines) = (80u16, 24u16);
        let term = Term::new(
            Config::default(),
            &TermSize::new(columns as usize, lines as usize),
            listener.clone(),
        );
        let term = Arc::new(FairMutex::new(term));

        let options = tty::Options {
            shell: Some(tty::Shell::new(
                program.into(),
                args.iter().map(|a| a.to_string()).collect(),
            )),
            env: [
                ("TERM".into(), "xterm-256color".into()),
                ("COLORTERM".into(), "truecolor".into()),
            ]
            .into(),
            ..Default::default()
        };
        let window_size = WindowSize {
            num_lines: lines,
            num_cols: columns,
            cell_width: 8,
            cell_height: 16,
        };
        let pty = tty::new(&options, window_size, 0)?;
        let event_loop = EventLoop::new(term.clone(), listener, pty, false, false)?;
        let notifier = Notifier(event_loop.channel());
        event_loop.spawn();

        // Drain the emulator's events on the UI thread. A burst of output sends
        // many Wakeups; they collapse into one repaint per frame.
        cx.spawn(async move |this, cx| {
            while let Some(event) = rx.next().await {
                let alive = this.update(cx, |view, cx| view.on_event(event, cx)).is_ok();
                if !alive {
                    break;
                }
            }
        })
        .detach();

        Ok(Self {
            term,
            notifier,
            focus: cx.focus_handle(),
            grid_size: (columns, lines),
            exited: false,
            tmux_session: None,
            layout: (Point::default(), size(px(8.), px(16.))),
            scroll_remainder: 0.,
        })
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    fn on_event(&mut self, event: Event, cx: &mut Context<Self>) {
        match event {
            Event::Wakeup => {
                stats::output();
                cx.notify();
            }
            Event::PtyWrite(text) => self.write(text.into_bytes()),
            Event::TextAreaSizeRequest(format) => {
                let size = self.window_size();
                self.write(format(size).into_bytes());
            }
            Event::ColorRequest(index, format) => {
                let rgb = self.term.lock().colors()[index].unwrap_or_else(|| default_rgb(index));
                self.write(format(rgb).into_bytes());
            }
            Event::ChildExit(_) | Event::Exit => {
                self.exited = true;
                cx.notify();
            }
            _ => {}
        }
    }

    fn write(&self, bytes: impl Into<Cow<'static, [u8]>>) {
        self.notifier.notify(bytes);
    }

    fn window_size(&self) -> WindowSize {
        WindowSize {
            num_cols: self.grid_size.0,
            num_lines: self.grid_size.1,
            cell_width: 8,
            cell_height: 16,
        }
    }

    /// Resizes the grid and the PTY when the element's size in cells changed.
    fn resize(&mut self, columns: u16, lines: u16, cell: Size<Pixels>) {
        if (columns, lines) == self.grid_size || columns == 0 || lines == 0 {
            return;
        }
        self.grid_size = (columns, lines);
        self.term
            .lock()
            .resize(TermSize::new(columns as usize, lines as usize));
        self.notifier.on_resize(WindowSize {
            num_cols: columns,
            num_lines: lines,
            cell_width: f32::from(cell.width) as u16,
            cell_height: f32::from(cell.height) as u16,
        });
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let app_cursor = self.term.lock().mode().contains(TermMode::APP_CURSOR);
        if let Some(bytes) = keystroke_bytes(&event.keystroke, app_cursor) {
            self.write(bytes);
            stats::key_sent();
            cx.stop_propagation();
        }
    }
}

impl TerminalView {
    /// The wheel: to the program when it asked for the mouse, else through
    /// tmux's history (copy mode), else through the terminal's own.
    fn on_scroll(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let (origin, cell) = self.layout;
        let lines = match event.delta {
            ScrollDelta::Lines(delta) => delta.y,
            ScrollDelta::Pixels(delta) => f32::from(delta.y) / f32::from(cell.height),
        } + self.scroll_remainder;
        let count = lines.trunc() as i32;
        log::debug!("wheel {:?} -> {count} line(s)", event.delta);
        self.scroll_remainder = lines - count as f32;
        if count == 0 {
            return;
        }
        let up = count > 0;
        let mode = *self.term.lock().mode();
        if mode.intersects(TermMode::MOUSE_MODE) {
            let column = ((f32::from(event.position.x - origin.x) / f32::from(cell.width)).max(0.)) as u32 + 1;
            let line = ((f32::from(event.position.y - origin.y) / f32::from(cell.height)).max(0.)) as u32 + 1;
            let button = if up { 64 } else { 65 };
            let one: Vec<u8> = if mode.contains(TermMode::SGR_MOUSE) {
                format!("\x1b[<{button};{column};{line}M").into_bytes()
            } else {
                let clamp = |v: u32| (32 + v).min(255) as u8;
                vec![0x1b, b'[', b'M', 32 + button as u8, clamp(column), clamp(line)]
            };
            self.write(one.repeat(count.unsigned_abs() as usize));
        } else if let Some(session) = self.tmux_session.clone() {
            // `=name:` — exactly this session, its current pane.
            let target = format!("={session}:");
            let steps = count.unsigned_abs().to_string();
            // Off the UI thread: each tmux call is a process.
            std::thread::spawn(move || {
                let tmux = |args: &[&str]| {
                    let _ = std::process::Command::new("tmux")
                        .args(args)
                        .env_remove("TMUX")
                        .stderr(std::process::Stdio::null())
                        .status();
                };
                if up {
                    // -e: leaves copy mode when scrolled back to the bottom.
                    tmux(&["copy-mode", "-e", "-t", &target]);
                    tmux(&["send-keys", "-t", &target, "-X", "-N", &steps, "scroll-up"]);
                } else {
                    tmux(&["send-keys", "-t", &target, "-X", "-N", &steps, "scroll-down"]);
                }
            });
        } else {
            self.term.lock().scroll_display(Scroll::Delta(count));
            cx.notify();
        }
        cx.stop_propagation();
    }
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for TerminalView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("terminal")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .relative()
            .size_full()
            .bg(to_hsla(default_rgb(NamedColor::Background as usize)))
            .child(TerminalElement {
                term: self.term.clone(),
                view: Some(cx.entity()),
            })
            .when(self.exited, |d| {
                d.child(
                    div()
                        .absolute()
                        .bottom_0()
                        .w_full()
                        .p_2()
                        .bg(rgb(0x5a1d1d))
                        .child("The session ended."),
                )
            })
    }
}

/// Paints the grid of a [`TerminalView`], and sizes the grid to its bounds.
pub struct TerminalElement {
    term: Arc<FairMutex<Term<Listener>>>,
    /// The live terminal it belongs to: the grid then follows the element's
    /// size. Without one (a [`Snapshot`]), the font shrinks to fit the grid.
    view: Option<Entity<TerminalView>>,
}

impl IntoElement for TerminalElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// A piece of a row sharing one style, painted from its first column.
pub struct Segment {
    column: usize,
    line: ShapedLine,
}

pub struct Frame {
    cell: Size<Pixels>,
    backgrounds: Vec<PaintQuad>,
    rows: Vec<(usize, Vec<Segment>)>,
    cursor: Option<PaintQuad>,
}

impl Element for TerminalElement {
    type RequestLayoutState = ();
    type PrepaintState = Frame;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Frame {
        if self.view.is_some() {
            stats::frame();
        }
        let started = std::time::Instant::now();
        let regular = Font {
            fallbacks: Some(FontFallbacks::from_fonts(
                FONT_FALLBACKS.iter().map(|f| f.to_string()).collect(),
            )),
            ..font(FONT_FAMILY)
        };
        let styled = |flags: Flags| Font {
            weight: if flags.contains(Flags::BOLD) {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            },
            style: if flags.contains(Flags::ITALIC) {
                FontStyle::Italic
            } else {
                FontStyle::Normal
            },
            ..regular.clone()
        };
        let text_system = window.text_system().clone();
        let font_id = text_system.resolve_font(&regular);
        let advance = |size: Pixels| {
            text_system
                .advance(font_id, size, 'm')
                .map(|s| s.width)
                .unwrap_or(size * 0.6)
        };
        let font_size = match &self.view {
            Some(_) => px(FONT_SIZE),
            None => {
                use alacritty_terminal::grid::Dimensions as _;
                let columns = self.term.lock().columns().max(1) as f32;
                let fit = f32::from(bounds.size.width) / (columns * f32::from(advance(px(FONT_SIZE))));
                px((FONT_SIZE * fit).max(2.))
            }
        };
        let cell = size(advance(font_size), (font_size * LINE_HEIGHT).round());

        let columns = (f32::from(bounds.size.width) / f32::from(cell.width)).floor() as u16;
        let lines = (f32::from(bounds.size.height) / f32::from(cell.height)).floor() as u16;
        let term = self.term.clone();
        let focused = match &self.view {
            Some(view) => view.update(cx, |view, _| {
                view.resize(columns, lines, cell);
                view.layout = (bounds.origin, cell);
                view.focus.is_focused(window)
            }),
            None => false,
        };

        let term = term.lock();
        let content = term.renderable_content();
        let colors = content.colors;
        let resolve = |color: Color| -> Rgb {
            match color {
                Color::Spec(rgb) => rgb,
                Color::Named(named) => colors[named].unwrap_or_else(|| default_rgb(named as usize)),
                Color::Indexed(index) => {
                    colors[index as usize].unwrap_or_else(|| default_rgb(index as usize))
                }
            }
        };
        let origin = bounds.origin;
        let at = |line: usize, column: usize| {
            point(
                origin.x + cell.width * column as f32,
                origin.y + cell.height * line as f32,
            )
        };

        let mut backgrounds = Vec::new();
        let mut rules = Vec::new();
        let mut rows: Vec<(usize, Vec<Segment>)> = Vec::new();
        // The segment being built: first column, text, style.
        let mut pending: Option<(usize, String, TextRun)> = None;
        let mut current_line = usize::MAX;
        let mut segments = Vec::new();
        let offset = content.display_offset as i32;

        let flush = |pending: &mut Option<(usize, String, TextRun)>, segments: &mut Vec<Segment>| {
            if let Some((column, text, run)) = pending.take() {
                if !text.trim().is_empty() {
                    let line = text_system.shape_line(text.into(), font_size, &[run], None);
                    segments.push(Segment { column, line });
                }
            }
        };

        for indexed in content.display_iter {
            let line = (indexed.point.line.0 + offset) as usize;
            let column = indexed.point.column.0;
            if line != current_line {
                flush(&mut pending, &mut segments);
                if current_line != usize::MAX {
                    rows.push((current_line, std::mem::take(&mut segments)));
                }
                current_line = line;
            }
            let cell_data = indexed.cell;
            let flags = cell_data.flags;
            if flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
                continue;
            }

            let (mut fg, mut bg) = (resolve(cell_data.fg), resolve(cell_data.bg));
            if flags.contains(Flags::INVERSE) {
                std::mem::swap(&mut fg, &mut bg);
            }
            let mut fg = to_hsla(fg);
            if flags.contains(Flags::DIM) {
                fg.a *= 0.66;
            }
            if flags.contains(Flags::HIDDEN) {
                fg.a = 0.;
            }
            let width = if flags.contains(Flags::WIDE_CHAR) { 2 } else { 1 };
            if let Some(arms) = box_arms(cell_data.c) {
                // Drawn, not typed: a glyph's strokes stop short of the next
                // row as soon as the line height exceeds the font's.
                rules.extend(box_quads(arms, Bounds::new(at(line, column), cell), fg));
                flush(&mut pending, &mut segments);
                if bg != default_rgb(NamedColor::Background as usize) {
                    backgrounds.push(fill(Bounds::new(at(line, column), cell), to_hsla(bg)));
                }
                continue;
            }
            if bg != default_rgb(NamedColor::Background as usize) {
                backgrounds.push(fill(
                    Bounds::new(at(line, column), size(cell.width * width as f32, cell.height)),
                    to_hsla(bg),
                ));
            }

            let run = TextRun {
                len: 0,
                font: styled(flags),
                color: fg,
                background_color: None,
                underline: flags.intersects(Flags::ALL_UNDERLINES).then(|| UnderlineStyle {
                    color: Some(fg),
                    thickness: px(1.),
                    wavy: flags.contains(Flags::UNDERCURL),
                }),
                strikethrough: flags.contains(Flags::STRIKEOUT).then(|| StrikethroughStyle {
                    color: Some(fg),
                    thickness: px(1.),
                }),
            };

            let continues = matches!(&pending, Some((start, text, prev))
                if *start + text.chars().count() == column
                    && prev.font == run.font && prev.color == run.color
                    && prev.underline == run.underline && prev.strikethrough == run.strikethrough);
            if !continues {
                flush(&mut pending, &mut segments);
                pending = Some((column, String::new(), run));
            }
            let (_, text, run) = pending.as_mut().unwrap();
            let before = text.len();
            text.push(cell_data.c);
            if let Some(zerowidth) = cell_data.zerowidth() {
                text.extend(zerowidth);
            }
            run.len += text.len() - before;
            // A wide glyph may not be exactly two cells wide in its font: start
            // the next segment on the grid again.
            if width == 2 {
                flush(&mut pending, &mut segments);
            }
        }
        flush(&mut pending, &mut segments);
        if current_line != usize::MAX {
            rows.push((current_line, segments));
        }

        let live = self.view.is_some();
        let cursor = (live
            && content.mode.contains(TermMode::SHOW_CURSOR)
            && content.cursor.shape != CursorShape::Hidden)
            .then(|| {
                let cursor = content.cursor.point;
                let line = (cursor.line.0 + offset) as usize;
                let wide = term.grid()[cursor].flags.contains(Flags::WIDE_CHAR);
                let width = cell.width * if wide { 2. } else { 1. };
                let origin = at(line, cursor.column.0);
                let color = to_hsla(resolve(Color::Named(NamedColor::Foreground)));
                match (content.cursor.shape, focused) {
                    (CursorShape::Beam, true) => fill(Bounds::new(origin, size(px(2.), cell.height)), color),
                    (CursorShape::Underline, true) => fill(
                        Bounds::new(
                            point(origin.x, origin.y + cell.height - px(2.)),
                            size(width, px(2.)),
                        ),
                        color,
                    ),
                    (_, true) => fill(Bounds::new(origin, size(width, cell.height)), color.opacity(0.6)),
                    (_, false) => outline(Bounds::new(origin, size(width, cell.height)), color, BorderStyle::Solid),
                }
            });

        stats::prepaint(started);
        backgrounds.extend(rules);
        Frame {
            cell,
            backgrounds,
            rows,
            cursor,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        frame: &mut Frame,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for quad in frame.backgrounds.drain(..) {
                window.paint_quad(quad);
            }
            for (line, segments) in &frame.rows {
                for segment in segments {
                    let origin = point(
                        bounds.origin.x + frame.cell.width * segment.column as f32,
                        bounds.origin.y + frame.cell.height * *line as f32,
                    );
                    let _ = segment
                        .line
                        .paint(origin, frame.cell.height, TextAlign::Left, None, window, cx);
                }
            }
            if let Some(cursor) = frame.cursor.take() {
                window.paint_quad(cursor);
            }
        });
    }
}

/// The bytes a terminal expects for a keystroke, or `None` to let it through.
fn keystroke_bytes(keystroke: &Keystroke, app_cursor: bool) -> Option<Vec<u8>> {
    let m = &keystroke.modifiers;
    let arrow = |c: char| -> Vec<u8> {
        let modifier = 1 + m.shift as u8 + 2 * m.alt as u8 + 4 * m.control as u8;
        if modifier > 1 {
            format!("\x1b[1;{modifier}{c}").into_bytes()
        } else if app_cursor {
            format!("\x1bO{c}").into_bytes()
        } else {
            format!("\x1b[{c}").into_bytes()
        }
    };
    let named: Option<Vec<u8>> = match keystroke.key.as_str() {
        "enter" => Some(b"\r".to_vec()),
        "backspace" => Some(if m.control { b"\x08".to_vec() } else { b"\x7f".to_vec() }),
        "tab" if m.shift => Some(b"\x1b[Z".to_vec()),
        "tab" => Some(b"\t".to_vec()),
        "escape" => Some(b"\x1b".to_vec()),
        "up" => Some(arrow('A')),
        "down" => Some(arrow('B')),
        "right" => Some(arrow('C')),
        "left" => Some(arrow('D')),
        "home" => Some(arrow('H')),
        "end" => Some(arrow('F')),
        "insert" => Some(b"\x1b[2~".to_vec()),
        "delete" => Some(b"\x1b[3~".to_vec()),
        "pageup" => Some(b"\x1b[5~".to_vec()),
        "pagedown" => Some(b"\x1b[6~".to_vec()),
        "f1" => Some(b"\x1bOP".to_vec()),
        "f2" => Some(b"\x1bOQ".to_vec()),
        "f3" => Some(b"\x1bOR".to_vec()),
        "f4" => Some(b"\x1bOS".to_vec()),
        "f5" => Some(b"\x1b[15~".to_vec()),
        "f6" => Some(b"\x1b[17~".to_vec()),
        "f7" => Some(b"\x1b[18~".to_vec()),
        "f8" => Some(b"\x1b[19~".to_vec()),
        "f9" => Some(b"\x1b[20~".to_vec()),
        "f10" => Some(b"\x1b[21~".to_vec()),
        "f11" => Some(b"\x1b[23~".to_vec()),
        "f12" => Some(b"\x1b[24~".to_vec()),
        "space" if m.control => Some(vec![0]),
        _ => None,
    };
    if let Some(mut bytes) = named {
        if m.alt && bytes.len() == 1 {
            bytes.insert(0, 0x1b);
        }
        return Some(bytes);
    }
    if m.platform || m.function {
        return None;
    }
    if m.control {
        let key = keystroke.key.as_bytes();
        if key.len() == 1 {
            let byte = match key[0] {
                c @ b'a'..=b'z' => c - b'a' + 1,
                b'@' | b'2' => 0,
                b'[' | b'3' => 0x1b,
                b'\\' | b'4' => 0x1c,
                b']' | b'5' => 0x1d,
                b'^' | b'6' => 0x1e,
                b'_' | b'7' | b'-' => 0x1f,
                b'?' | b'8' => 0x7f,
                _ => return None,
            };
            return Some(if m.alt { vec![0x1b, byte] } else { vec![byte] });
        }
        return None;
    }
    let text = match (&keystroke.key_char, keystroke.key.as_str()) {
        (Some(text), _) => text.clone(),
        (None, "space") => " ".into(),
        (None, key) if key.chars().count() == 1 => key.into(),
        _ => return None,
    };
    let mut bytes = text.into_bytes();
    if m.alt {
        bytes.insert(0, 0x1b);
    }
    Some(bytes)
}

fn to_hsla(rgb: Rgb) -> Hsla {
    Rgba {
        r: rgb.r as f32 / 255.,
        g: rgb.g as f32 / 255.,
        b: rgb.b as f32 / 255.,
        a: 1.,
    }
    .into()
}

/// The palette when the program did not set a color: a dark theme, the
/// xterm 6x6x6 cube and gray ramp for indexes 16..=255.
fn default_rgb(index: usize) -> Rgb {
    const ANSI: [u32; 16] = [
        0x1e1e1e, 0xf14c4c, 0x23d18b, 0xf5f543, 0x3b8eea, 0xd670d6, 0x29b8db, 0xcccccc, //
        0x666666, 0xf14c4c, 0x23d18b, 0xf5f543, 0x3b8eea, 0xd670d6, 0x29b8db, 0xffffff,
    ];
    let hex = |v: u32| Rgb {
        r: (v >> 16) as u8,
        g: (v >> 8) as u8,
        b: v as u8,
    };
    match index {
        0..=15 => hex(ANSI[index]),
        16..=231 => {
            let i = index - 16;
            let level = |v: usize| if v == 0 { 0 } else { (55 + v * 40) as u8 };
            Rgb {
                r: level(i / 36),
                g: level(i / 6 % 6),
                b: level(i % 6),
            }
        }
        232..=255 => {
            let v = (8 + (index - 232) * 10) as u8;
            Rgb { r: v, g: v, b: v }
        }
        i if i == NamedColor::Background as usize => hex(0x1e1e1e),
        i if i >= NamedColor::DimBlack as usize && i <= NamedColor::DimWhite as usize => {
            let base = default_rgb(i - NamedColor::DimBlack as usize);
            Rgb {
                r: (base.r as f32 * 0.66) as u8,
                g: (base.g as f32 * 0.66) as u8,
                b: (base.b as f32 * 0.66) as u8,
            }
        }
        i if i == NamedColor::DimForeground as usize => hex(0x8a8a8a),
        _ => hex(0xd4d4d4),
    }
}

/// The four arms of a box-drawing character — left, right, up, down —
/// 1 for a light stroke, 2 for a heavy one. Rounded corners come out square.
fn box_arms(c: char) -> Option<[u8; 4]> {
    Some(match c {
        '─' => [1, 1, 0, 0],
        '━' => [2, 2, 0, 0],
        '│' => [0, 0, 1, 1],
        '┃' => [0, 0, 2, 2],
        '┌' | '╭' => [0, 1, 0, 1],
        '┐' | '╮' => [1, 0, 0, 1],
        '└' | '╰' => [0, 1, 1, 0],
        '┘' | '╯' => [1, 0, 1, 0],
        '├' => [0, 1, 1, 1],
        '┤' => [1, 0, 1, 1],
        '┬' => [1, 1, 0, 1],
        '┴' => [1, 1, 1, 0],
        '┼' => [1, 1, 1, 1],
        '┏' => [0, 2, 0, 2],
        '┓' => [2, 0, 0, 2],
        '┗' => [0, 2, 2, 0],
        '┛' => [2, 0, 2, 0],
        '┣' => [0, 2, 2, 2],
        '┫' => [2, 0, 2, 2],
        '┳' => [2, 2, 0, 2],
        '┻' => [2, 2, 2, 0],
        '╋' => [2, 2, 2, 2],
        '╴' => [1, 0, 0, 0],
        '╶' => [0, 1, 0, 0],
        '╵' => [0, 0, 1, 0],
        '╷' => [0, 0, 0, 1],
        _ => return None,
    })
}

/// The rectangles of a box-drawing character in its cell, on whole pixels
/// so that neighbours meet.
fn box_quads(arms: [u8; 4], cell: Bounds<Pixels>, color: Hsla) -> Vec<PaintQuad> {
    let x0 = f32::from(cell.origin.x).round();
    let y0 = f32::from(cell.origin.y).round();
    let x1 = f32::from(cell.origin.x + cell.size.width).round();
    let y1 = f32::from(cell.origin.y + cell.size.height).round();
    let light = (f32::from(cell.size.width) / 8.).round().max(1.);
    let (cx, cy) = (((x0 + x1) / 2.).floor(), ((y0 + y1) / 2.).floor());
    let rect = |left: f32, top: f32, right: f32, bottom: f32| {
        fill(
            Bounds::from_corners(point(px(left), px(top)), point(px(right), px(bottom))),
            color,
        )
    };
    let [left, right, up, down] = arms;
    let thick = |weight: u8| light * weight as f32;
    let mut quads = Vec::new();
    // Each arm runs from the cell's edge to past the centre, so arms of a
    // corner overlap instead of leaving a notch.
    if left > 0 {
        let t = thick(left);
        quads.push(rect(x0, cy - (t / 2.).floor(), cx + (t / 2.).ceil(), cy + (t / 2.).ceil()));
    }
    if right > 0 {
        let t = thick(right);
        quads.push(rect(cx - (t / 2.).floor(), cy - (t / 2.).floor(), x1, cy + (t / 2.).ceil()));
    }
    if up > 0 {
        let t = thick(up);
        quads.push(rect(cx - (t / 2.).floor(), y0, cx + (t / 2.).ceil(), cy + (t / 2.).ceil()));
    }
    if down > 0 {
        let t = thick(down);
        quads.push(rect(cx - (t / 2.).floor(), cy - (t / 2.).floor(), cx + (t / 2.).ceil(), y1));
    }
    quads
}

/// A still copy of a screen, for thumbnails: fed with what `tmux
/// capture-pane -e` prints, painted by the same element at a size that fits.
pub struct Snapshot {
    term: Arc<FairMutex<Term<Listener>>>,
}

impl Snapshot {
    pub fn new() -> Self {
        let (tx, _) = unbounded();
        let term = Term::new(Config::default(), &TermSize::new(80, 24), Listener(tx));
        Self {
            term: Arc::new(FairMutex::new(term)),
        }
    }

    /// Replaces the screen with `text`, a pane of `columns` × `lines`.
    pub fn load(&self, columns: usize, lines: usize, text: &[u8]) {
        let mut term = self.term.lock();
        term.resize(TermSize::new(columns.max(1), lines.max(1)));
        let mut bytes = b"\x1b[0m\x1b[H\x1b[2J".to_vec();
        for (i, line) in text.split(|&b| b == b'\n').enumerate() {
            if i > 0 {
                bytes.extend_from_slice(b"\r\n");
            }
            bytes.extend_from_slice(line);
        }
        let mut parser: alacritty_terminal::vte::ansi::Processor = Default::default();
        parser.advance(&mut *term, &bytes);
    }

    pub fn element(&self) -> TerminalElement {
        TerminalElement {
            term: self.term.clone(),
            view: None,
        }
    }
}
