//! A terminal: `alacritty_terminal` does the emulation and owns the PTY, a
//! GPUI element paints its grid, and keystrokes go back to the PTY as bytes.

use std::borrow::Cow;
use std::sync::Arc;

use alacritty_terminal::event::{Event, EventListener, Notify, OnResize, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, Msg, Notifier};
use alacritty_terminal::grid::{Dimensions as _, Scroll};
use alacritty_terminal::index::{Column, Line, Point as GridPoint, Side};
use alacritty_terminal::selection::{Selection, SelectionType};
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

use crate::keymap;

use crate::stats;

const FONT_FAMILY: &str = "Source Code Pro";
/// Tried in order for glyphs the main font lacks (emoji, CJK, symbols).
const FONT_FALLBACKS: &[&str] = &["Noto Color Emoji", "Noto Sans CJK JP", "Adwaita Mono"];
/// The terminals' font size, in pixels: by default, and its bounds.
pub const FONT_SIZE_DEFAULT: f32 = 14.;
pub const FONT_SIZE_MIN: f32 = 8.;
pub const FONT_SIZE_MAX: f32 = 32.;
/// The size chosen, as `f32` bits: one for every terminal.
static FONT_SIZE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x4160_0000);

/// Sets the terminals' font size, within its bounds; answers the size kept.
/// Each terminal takes it at its next frame (its grid, then its PTY, resize).
pub fn set_font_size(size: f32) -> f32 {
    let size = size.round().clamp(FONT_SIZE_MIN, FONT_SIZE_MAX);
    FONT_SIZE.store(size.to_bits(), std::sync::atomic::Ordering::Relaxed);
    size
}

/// The terminals' background opacity, 0 to 1 (1: opaque), as bits.
static OPACITY: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3f80_0000);

/// Sets the terminals' background opacity from a percentage (none: opaque);
/// answers it, from 0 to 1. The colours a program sets stay opaque.
pub fn set_opacity(percent: Option<f32>) -> f32 {
    let opacity = (percent.unwrap_or(100.) / 100.).clamp(0.05, 1.);
    OPACITY.store(opacity.to_bits(), std::sync::atomic::Ordering::Relaxed);
    opacity
}

pub fn opacity() -> f32 {
    f32::from_bits(OPACITY.load(std::sync::atomic::Ordering::Relaxed))
}

/// A terminal's background as painted: the theme's, see-through as set.
pub fn background() -> Hsla {
    background_colour().opacity(opacity())
}

/// The theme's background for the terminals, opaque.
pub fn background_colour() -> Hsla {
    to_hsla(default_rgb(NamedColor::Background as usize))
}

pub fn font_size() -> f32 {
    f32::from_bits(FONT_SIZE.load(std::sync::atomic::Ordering::Relaxed))
}
const LINE_HEIGHT: f32 = 1.3;

/// Forwards the emulator's events (from its I/O thread) to the view.
#[derive(Clone)]
struct Listener(UnboundedSender<Event>);

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        let _ = self.0.unbounded_send(event);
    }
}

/// How long a new size must hold before the PTY hears of it.
const RESIZE_SETTLE: std::time::Duration = std::time::Duration::from_millis(120);

/// The terminal's program ended (the tmux session is gone).
pub struct Ended;

impl EventEmitter<Ended> for TerminalView {}

/// Where the terminal's bytes come from and its keys go.
enum Backend {
    /// A program in a PTY of tvty's own (today, a tmux client).
    Pty(Notifier),
    /// A session a host holds, attached to over its socket.
    Attach(Arc<crate::attach::Attach>),
    /// A session that could not be reached: the view shows its end.
    Closed,
}

pub struct TerminalView {
    term: Arc<FairMutex<Term<Listener>>>,
    backend: Backend,
    focus: FocusHandle,
    /// Grid size last sent to the PTY: (columns, lines).
    grid_size: (u16, u16),
    /// A new size waiting to be sent, and whether one was ever sent.
    pending_size: Option<(u16, u16)>,
    /// When the element last saw a new size: sizes that follow each other
    /// fast (a side dragged, the window resized) wait to hold still.
    last_new_size: Option<std::time::Instant>,
    sized: bool,
    /// A copy: its grid is the session's size, never the element's (it
    /// does not resize the session, so it draws it as it is).
    follows_session: bool,
    /// The program's pid, run in its PTY: a tmux client's is how tmux
    /// (and aiball) tell it from the session's other clients.
    pub child_pid: Option<u32>,
    exited: bool,
    /// The tmux session shown, to scroll its history in copy mode.
    tmux_session: Option<String>,
    /// Where the grid was last painted, and its cell size: for the mouse.
    layout: (Point<Pixels>, Size<Pixels>),
    /// Wheel movement not yet worth a line.
    scroll_remainder: f32,
    /// Lines to scroll tmux's history by (up: positive), to the one thread
    /// that runs tmux for this view; started on the first notch.
    tmux_scroll: Option<std::sync::mpsc::Sender<i32>>,
    /// The left button is down on a selection being made.
    selecting: bool,
    /// The selection being made, kept by tvty during the drag: the
    /// emulator drops a selection whose lines the program writes over (Claude
    /// Code draws its screen again and again), and the drag would lose it
    /// under the pointer. Put back after each output and each move.
    dragging: Option<Selection>,
    /// The selection once the button is released, kept by tvty too: the
    /// program (or tmux, redrawing its client) writes the same text again,
    /// the emulator drops the selection, and tvty puts it back — as long as
    /// the text under it is still what was selected.
    kept: Option<Selection>,
    /// The right click's menu (Copy, Paste), where it was opened.
    menu: Option<Point<Pixels>>,
    /// The text selected last, read as the drag goes: a program that draws
    /// its screen again (Claude Code) clears the selection it writes over,
    /// and what was selected must still be copied.
    selected_text: Option<String>,
    /// The link under the pointer, underlined; Ctrl+click opens it.
    hover_link: Option<Link>,
    /// The link the right click's menu was opened on.
    menu_link: Option<Link>,
}

/// A link in the terminal: where it goes, and the cells it spans — (grid
/// line, first column, column after the last), one piece per row.
#[derive(Clone, Debug, PartialEq)]
struct Link {
    uri: String,
    cells: Vec<(i32, usize, usize)>,
    /// Its text as shown differs from where it goes (a program's OSC 8 link).
    named: bool,
}

impl Drop for TerminalView {
    /// Stops the PTY's event loop, so the child (a tmux client) goes with the
    /// view — a card's watcher must not stay attached once the slider closes.
    fn drop(&mut self) {
        match &self.backend {
            Backend::Pty(notifier) => {
                let _ = notifier.0.send(Msg::Shutdown);
            }
            // The session goes on: only this client leaves.
            Backend::Attach(attach) => attach.close(),
            Backend::Closed => {}
        }
    }
}

impl TerminalView {
    /// Attaches to a tmux session.
    pub fn tmux(session: &str, cx: &mut Context<Self>) -> anyhow::Result<Self> {
        let args = crate::mux::args(&["attach", "-t", &format!("={session}")]);
        let mut view = Self::new(crate::mux::program(), &args.iter().map(String::as_str).collect::<Vec<_>>(), cx)?;
        view.tmux_session = Some(session.to_string());
        Ok(view)
    }

    /// Attaches to a tmux session as a copy: read-only (`attach -r`), it
    /// never types nor resizes the session.
    /// Drawn at the session's window size, followed: the window changes
    /// with its other clients (asked of tmux each second, off the UI thread).
    pub fn tmux_copy(session: &str, cx: &mut Context<Self>) -> anyhow::Result<Self> {
        let (columns, lines) = crate::sessions::window_size(session).unwrap_or((80, 24));
        let mut view = Self::watch(session, columns, lines, cx)?;
        view.follows_session = true;
        let session = session.to_string();
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(std::time::Duration::from_secs(1)).await;
                let asked = session.clone();
                let window = cx.background_executor().spawn(async move { crate::sessions::window_size(&asked) }).await;
                let alive = this.update(cx, |view, cx| {
                    if let Some((columns, lines)) = window.filter(|s| *s != view.grid_size) {
                        view.apply_size(columns, lines, size(px(8.), px(16.)));
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        })
        .detach();
        Ok(view)
    }

    /// Watches a tmux session live without ever resizing it: a read-only
    /// client that ignores its own size (`attach -r`), as large as the
    /// session's window so it sees all of it.
    pub fn watch(session: &str, columns: u16, lines: u16, cx: &mut Context<Self>) -> anyhow::Result<Self> {
        let args = crate::mux::args(&["attach", "-r", "-t", &format!("={session}")]);
        let mut view = Self::new(crate::mux::program(), &args.iter().map(String::as_str).collect::<Vec<_>>(), cx)?;
        view.tmux_session = Some(session.to_string());
        view.apply_size(columns.max(1), lines.max(1), size(px(8.), px(16.)));
        Ok(view)
    }

    /// Attaches to a session a host holds (aiball's session host, or a
    /// loop's proxy), over its attach socket: no tmux.
    /// A session that cannot be reached ends at once (the end screen).
    pub fn attach(socket: &std::path::Path, cx: &mut Context<Self>) -> Self {
        Self::attached(socket, true, cx)
    }

    /// Watches a session a host holds, over its attach socket, as an
    /// observer: it never types nor resizes the session (a card).
    /// Its grid follows the session's size, as the host says it.
    pub fn observe(socket: &std::path::Path, cx: &mut Context<Self>) -> Self {
        let mut view = Self::attached(socket, false, cx);
        view.follows_session = true;
        view
    }

    fn attached(socket: &std::path::Path, interactive: bool, cx: &mut Context<Self>) -> Self {
        let (tx, rx) = unbounded();
        let listener = Listener(tx);
        let (columns, lines) = (80u16, 24u16);
        let term = Term::new(Config::default(), &TermSize::new(columns as usize, lines as usize), listener.clone());
        let term = Arc::new(FairMutex::new(term));
        let backend = match crate::attach::Attach::connect(socket, (columns, lines), interactive, term.clone(), listener.clone()) {
            Ok(attach) => Backend::Attach(attach),
            Err(error) => {
                log::warn!("attach {}: {error:#}", socket.display());
                listener.send_event(Event::Exit);
                Backend::Closed
            }
        };
        Self::with(term, backend, rx, cx)
    }

    /// Its screen, live, to draw elsewhere (a card).
    pub fn screen(&self) -> Snapshot {
        Snapshot { term: self.term.clone(), status_line: self.tmux_session.is_some() }
    }

    /// Spawns `program args` in a new PTY.
    pub fn new(program: &str, args: &[&str], cx: &mut Context<Self>) -> anyhow::Result<Self> {
        let (tx, rx) = unbounded();
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
        #[cfg(unix)]
        let child_pid = Some(pty.child().id());
        #[cfg(windows)]
        let child_pid = pty.child_watcher().pid().map(|pid| pid.get());
        let event_loop = EventLoop::new(term.clone(), listener, pty, false, false)?;
        let notifier = Notifier(event_loop.channel());
        event_loop.spawn();
        let mut view = Self::with(term, Backend::Pty(notifier), rx, cx);
        view.child_pid = child_pid;
        Ok(view)
    }

    fn with(
        term: Arc<FairMutex<Term<Listener>>>,
        backend: Backend,
        mut rx: futures::channel::mpsc::UnboundedReceiver<Event>,
        cx: &mut Context<Self>,
    ) -> Self {
        // One lock: the same mutex taken twice in one statement deadlocks.
        let (columns, lines) = {
            let term = term.lock();
            (term.columns() as u16, term.screen_lines() as u16)
        };
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

        Self {
            term,
            backend,
            focus: cx.focus_handle(),
            grid_size: (columns, lines),
            pending_size: None,
            last_new_size: None,
            follows_session: false,
            child_pid: None,
            sized: false,
            exited: false,
            tmux_session: None,
            layout: (Point::default(), size(px(8.), px(16.))),
            scroll_remainder: 0.,
            tmux_scroll: None,
            selecting: false,
            menu: None,
            dragging: None,
            kept: None,
            selected_text: None,
            hover_link: None,
            menu_link: None,
        }
    }

    /// Shown: on aiball's host, this client's size becomes the session's
    /// (another client may have typed since). Nothing to do through tmux.
    pub fn take_size(&self) {
        if let Backend::Attach(attach) = &self.backend {
            attach.focus();
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    fn on_event(&mut self, event: Event, cx: &mut Context<Self>) {
        match event {
            Event::Wakeup => {
                stats::output();
                // The program wrote over the selection being dragged: it
                // stays, until the button is released.
                if let Some(selection) = self.dragging.clone() {
                    let mut term = self.term.lock();
                    if term.selection.is_none() {
                        term.selection = Some(selection);
                    }
                } else if self.kept.is_some() {
                    self.keep_selection();
                }
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
                if !self.exited {
                    self.exited = true;
                    cx.emit(Ended);
                }
                cx.notify();
            }
            _ => {}
        }
    }

    /// Bytes to the program, as typed.
    pub fn send_bytes(&self, bytes: &'static [u8]) {
        self.write(bytes);
    }

    fn write(&self, bytes: impl Into<Cow<'static, [u8]>>) {
        match &self.backend {
            Backend::Pty(notifier) => notifier.notify(bytes),
            Backend::Attach(attach) => attach.input(&bytes.into()),
            Backend::Closed => {}
        }
    }

    fn window_size(&self) -> WindowSize {
        WindowSize {
            num_cols: self.grid_size.0,
            num_lines: self.grid_size.1,
            cell_width: 8,
            cell_height: 16,
        }
    }

    /// Resizes the grid and the PTY when the element's size in cells changed:
    /// at once for a size alone (a side opened or closed, the font), once
    /// the size holds still when they follow each other — while a side is
    /// dragged, every column would make the program redraw its whole screen.
    fn resize(&mut self, columns: u16, lines: u16, cell: Size<Pixels>, cx: &mut Context<Self>) {
        if columns == 0 || lines == 0 || self.follows_session {
            return;
        }
        if (columns, lines) == self.grid_size {
            self.pending_size = None;
            return;
        }
        if !self.sized {
            self.apply_size(columns, lines, cell);
            return;
        }
        if self.pending_size == Some((columns, lines)) {
            return;
        }
        stats::size_seen();
        let alone = self.last_new_size.is_none_or(|at| at.elapsed() >= RESIZE_SETTLE);
        self.last_new_size = Some(std::time::Instant::now());
        if alone && self.pending_size.is_none() {
            self.apply_size(columns, lines, cell);
            return;
        }
        self.pending_size = Some((columns, lines));
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RESIZE_SETTLE).await;
            let _ = this.update(cx, |view, cx| {
                if view.pending_size == Some((columns, lines)) {
                    view.pending_size = None;
                    view.apply_size(columns, lines, cell);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn apply_size(&mut self, columns: u16, lines: u16, cell: Size<Pixels>) {
        self.sized = true;
        stats::resized(columns, lines);
        self.grid_size = (columns, lines);
        self.term
            .lock()
            .resize(TermSize::new(columns as usize, lines as usize));
        match &mut self.backend {
            Backend::Pty(notifier) => notifier.on_resize(WindowSize {
                num_cols: columns,
                num_lines: lines,
                cell_width: f32::from(cell.width) as u16,
                cell_height: f32::from(cell.height) as u16,
            }),
            // The session's size is its owner's: the last client that typed
            // or took focus. The one on screen takes it, so that its new
            // size applies, whoever typed last elsewhere.
            Backend::Attach(attach) => {
                attach.resize(columns, lines);
                attach.focus();
            }
            Backend::Closed => {}
        }
    }

    /// Tab and shift+tab, which the window's root would take to move the
    /// focus: in a terminal they are the program's.
    fn send_tab(&mut self, _: &keymap::SendTab, _: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection(cx);
        self.write(b"\t".to_vec());
        stats::key_sent();
    }

    fn send_back_tab(&mut self, _: &keymap::SendBackTab, _: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection(cx);
        self.write(b"\x1b[Z".to_vec());
        stats::key_sent();
    }

    /// The terminal's copy (`terminal.copy`, ctrl+shift+c): the selection
    /// to the clipboard — ctrl+c stays the program's ^C.
    fn copy(&mut self, _: &keymap::TerminalCopy, _: &mut Window, cx: &mut Context<Self>) {
        // The selection, or, when the program drew over it, what it held.
        let text = self.term.lock().selection_to_string().filter(|t| !t.is_empty()).or_else(|| self.selected_text.clone());
        if let Some(text) = text {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            crate::bus::emit(cx, crate::bus::Signal::Copied);
        }
    }

    /// The terminal's paste (`terminal.paste`, ctrl+shift+v, the menu's
    /// Paste): the clipboard's text. An image and no text: the program is
    /// handed Ctrl+V, the key on which it reads an image itself (Claude
    /// Code does) — a terminal only ever pastes text.
    fn paste_clipboard(&mut self, _: &keymap::TerminalPaste, _: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = cx.read_from_clipboard() else { return };
        if let Some(text) = item.text() {
            self.paste(&text);
        } else if item.entries().iter().any(|entry| matches!(entry, ClipboardEntry::Image(_))) {
            self.write(b"\x16".to_vec());
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        // The menu open, Esc closes it — and is not sent to the program.
        if self.menu.is_some() && keystroke.key == "escape" {
            self.menu = None;
            cx.stop_propagation();
            cx.notify();
            return;
        }
        let app_cursor = self.term.lock().mode().contains(TermMode::APP_CURSOR);
        if let Some(bytes) = keystroke_bytes(keystroke, app_cursor) {
            self.clear_selection(cx);
            self.write(bytes);
            stats::key_sent();
            cx.stop_propagation();
        }
    }

    /// Sends text as pasted: between bracketed-paste markers when the
    /// program asked for them (so it knows this is not typing), else with
    /// the line ends a terminal sends.
    fn paste(&self, text: &str) {
        let bracketed = self.term.lock().mode().contains(TermMode::BRACKETED_PASTE);
        let bytes = if bracketed {
            // A pasted end marker would end the paste early: drop escapes.
            format!("\x1b[200~{}\x1b[201~", text.replace('\x1b', ""))
        } else {
            text.replace("\r\n", "\r").replace('\n', "\r")
        };
        self.write(bytes.into_bytes());
    }

    /// After output: the selection the emulator still has is the one kept
    /// (it follows the lines as they scroll); one it dropped is put back if
    /// the same text is still under it, else let go.
    fn keep_selection(&mut self) {
        let mut term = self.term.lock();
        if let Some(selection) = term.selection.clone() {
            self.kept = Some(selection);
            return;
        }
        term.selection = self.kept.clone();
        let same = term.selection_to_string().filter(|t| !t.is_empty()) == self.selected_text;
        if !same {
            term.selection = None;
            self.kept = None;
        }
    }

    fn clear_selection(&mut self, cx: &mut Context<Self>) {
        self.selected_text = None;
        self.kept = None;
        let mut term = self.term.lock();
        if term.selection.take().is_some() {
            cx.notify();
        }
    }

    /// The grid cell under a window position, and which half of it.
    fn grid_point(&self, position: Point<Pixels>) -> (GridPoint, Side) {
        let (origin, cell) = self.layout;
        let term = self.term.lock();
        let x = f32::from(position.x - origin.x) / f32::from(cell.width);
        let y = f32::from(position.y - origin.y) / f32::from(cell.height);
        let column = (x.max(0.) as usize).min(term.columns().saturating_sub(1));
        let row = (y.max(0.) as usize).min(term.screen_lines().saturating_sub(1));
        let line = row as i32 - term.grid().display_offset() as i32;
        let side = if x.fract() < 0.5 { Side::Left } else { Side::Right };
        (GridPoint::new(Line(line), Column(column)), side)
    }

    /// The link under `position`: one the program declared (OSC 8), else an
    /// address in the text — the line the program wrapped joined up.
    fn link_at(&self, position: Point<Pixels>) -> Option<Link> {
        let (point, _) = self.grid_point(position);
        let term = self.term.lock();
        let grid = term.grid();
        let columns = term.columns();
        let (top, bottom) = (-(grid.history_size() as i32), term.screen_lines() as i32 - 1);
        let wraps = |line: i32| grid[Line(line)][Column(columns - 1)].flags.contains(Flags::WRAPLINE);
        // The logical line: the rows the program wrapped into one.
        let mut first = point.line.0;
        while first > top && wraps(first - 1) {
            first -= 1;
        }
        let mut last = point.line.0;
        while last < bottom && wraps(last) {
            last += 1;
        }
        // Its text, each char's cell.
        let mut text = String::new();
        let mut at: Vec<(i32, usize)> = Vec::new();
        let mut uris: Vec<Option<String>> = Vec::new();
        for line in first..=last {
            for column in 0..columns {
                let cell = &grid[Line(line)][Column(column)];
                if cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
                    continue;
                }
                text.push(if cell.c == '\0' { ' ' } else { cell.c });
                at.push((line, column));
                uris.push(cell.hyperlink().map(|h| h.uri().to_string()));
            }
        }
        let here = at.iter().position(|&(l, c)| l == point.line.0 && c == point.column.0)?;
        // The chars from `start` to `end`, as cell pieces row by row.
        let cells = |start: usize, end: usize| {
            let mut pieces: Vec<(i32, usize, usize)> = Vec::new();
            for &(line, column) in &at[start..end] {
                match pieces.last_mut() {
                    Some(last) if last.0 == line && last.2 == column => last.2 = column + 1,
                    _ => pieces.push((line, column, column + 1)),
                }
            }
            pieces
        };
        if let Some(uri) = uris[here].clone() {
            let mut start = here;
            while start > 0 && uris[start - 1].as_deref() == Some(uri.as_str()) {
                start -= 1;
            }
            let mut end = here + 1;
            while end < uris.len() && uris[end].as_deref() == Some(uri.as_str()) {
                end += 1;
            }
            let shown: String = text.chars().skip(start).take(end - start).collect();
            return Some(Link { named: shown.trim() != uri, uri, cells: cells(start, end) });
        }
        let (start, end) = crate::links::find(&text).into_iter().find(|&(a, b)| a <= here && here < b)?;
        let uri: String = text.chars().skip(start).take(end - start).collect();
        Some(Link { uri, cells: cells(start, end), named: false })
    }

    fn open_link(&self, link: &Link, cx: &mut Context<Self>) {
        log::info!("terminal: opens {}", link.uri);
        cx.open_url(&link.uri);
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        self.menu = None;
        // Ctrl+click on a link opens it (a plain click selects, as ever).
        if event.modifiers.control
            && let Some(link) = self.link_at(event.position)
        {
            self.open_link(&link, cx);
            cx.stop_propagation();
            return;
        }
        // A drag selects, whatever the program: one that takes the mouse
        // (tmux with its mouse on) got the clicks without the drag, and
        // nothing could be selected at all.
        let (point, side) = self.grid_point(event.position);
        let kind = match event.click_count {
            2 => SelectionType::Semantic,
            3.. => SelectionType::Lines,
            _ => SelectionType::Simple,
        };
        let selection = Selection::new(kind, point, side);
        self.term.lock().selection = Some(selection.clone());
        self.dragging = Some(selection);
        self.kept = None;
        self.selected_text = None;
        self.selecting = true;
        cx.notify();
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, window: &mut Window, cx: &mut Context<Self>) {
        // The pointer gives the keyboard (Options > Mouse > Focus): no button
        // held, not in a selection or a drag.
        if event.pressed_button.is_none() && !self.focus.is_focused(window) && crate::focusmode::hover(cx) {
            window.focus(&self.focus, cx);
        }
        // The link under the pointer, underlined.
        if event.pressed_button.is_none() {
            let link = self.link_at(event.position);
            if link != self.hover_link {
                self.hover_link = link;
                cx.notify();
            }
        }
        if !self.selecting || event.pressed_button != Some(MouseButton::Left) {
            return;
        }
        let (point, side) = self.grid_point(event.position);
        let mut term = self.term.lock();
        // From tvty's own copy: the emulator's may have been dropped.
        if let Some(selection) = self.dragging.as_mut() {
            selection.update(point, side);
            term.selection = Some(selection.clone());
        }
        if let Some(text) = term.selection_to_string().filter(|t| !t.is_empty()) {
            self.selected_text = Some(text);
        }
        drop(term);
        cx.notify();
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selecting {
            return;
        }
        self.selecting = false;
        // A click without a drag selects nothing.
        let mut term = self.term.lock();
        if let Some(selection) = self.dragging.take() {
            term.selection = Some(selection);
        }
        if term.selection.as_ref().is_some_and(|s| s.is_empty()) {
            term.selection = None;
        }
        // What is selected is copied at once to the primary selection, as
        // on Linux: a middle click pastes it.
        if let Some(text) = term.selection_to_string().filter(|t| !t.is_empty()) {
            self.selected_text = Some(text);
        }
        self.kept = term.selection.clone();
        drop(term);
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        if let Some(text) = self.selected_text.clone() {
            cx.write_to_primary(ClipboardItem::new_string(text));
        }
        cx.notify();
    }

    /// The middle click pastes the primary selection (Linux), else the
    /// clipboard.
    fn on_middle_click(&mut self, _: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        let item = cx.read_from_primary();
        #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
        let item = cx.read_from_clipboard();
        if let Some(text) = item.and_then(|item| item.text()) {
            self.paste(&text);
        }
        cx.stop_propagation();
    }

    /// The right click opens the menu (Copy, Paste) under the pointer.
    fn on_right_click(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        self.menu_link = self.link_at(event.position);
        self.menu = Some(event.position);
        cx.stop_propagation();
        cx.notify();
    }

    /// The right click's menu: Copy (when something is selected), Paste.
    fn menu_view(&self, at: Point<Pixels>, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.term.lock().selection.as_ref().is_some_and(|s| !s.is_empty()) || self.selected_text.is_some();
        let item = |id: &'static str, label: &'static str, keys: &'static str, enabled: bool| {
            div()
                .id(id)
                .flex()
                .gap_6()
                .justify_between()
                .px_3()
                .py_1()
                .rounded_sm()
                .text_sm()
                .when(enabled, |d| d.cursor_pointer().hover(|d| d.bg(crate::theme::p().hover)))
                .when(!enabled, |d| d.text_color(crate::theme::p().muted))
                .child(label)
                .child(div().text_xs().text_color(crate::theme::p().muted).child(keys))
        };
        deferred(
            anchored().position(at).child(
                div()
                    .id("terminal-menu")
                    .occlude()
                    .min_w(px(180.))
                    .p_1()
                    .rounded_md()
                    .border_1()
                    .border_color(crate::theme::p().border)
                    .bg(crate::theme::p().surface)
                    .text_color(crate::theme::p().text)
                    .shadow_lg()
                    .on_mouse_down_out(cx.listener(|view, _, _, cx| {
                        view.menu = None;
                        cx.notify();
                    }))
                    // On a link: open it, copy where it goes.
                    .when_some(self.menu_link.clone(), |d, link| {
                        let copied = link.uri.clone();
                        d.child(item("terminal-menu-open-link", "Open link", "Ctrl+click", true).on_click(cx.listener(move |view, _, _, cx| {
                            view.menu = None;
                            view.open_link(&link, cx);
                            cx.notify();
                        })))
                        .child(item("terminal-menu-copy-link", "Copy link", "", true).on_click(cx.listener(move |view, _, _, cx| {
                            view.menu = None;
                            cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                            cx.notify();
                        })))
                        .child(div().my_1().h(px(1.)).bg(crate::theme::p().border))
                    })
                    .child(item("terminal-menu-copy", "Copy", "Ctrl+Shift+C", selected).when(selected, |d| {
                        d.on_click(cx.listener(|view, _, window, cx| {
                            view.menu = None;
                            view.copy(&keymap::TerminalCopy, window, cx);
                            cx.notify();
                        }))
                    }))
                    .child(item("terminal-menu-paste", "Paste", "Ctrl+Shift+V", true).on_click(cx.listener(|view, _, window, cx| {
                        view.menu = None;
                        view.paste_clipboard(&keymap::TerminalPaste, window, cx);
                        cx.notify();
                    }))),
            ),
        )
        .with_priority(2)
    }
}

impl TerminalView {
    /// The wheel: to the program when it asked for the mouse, else through
    /// tmux's history (copy mode), else through the terminal's own.
    fn on_scroll(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let (origin, cell) = self.layout;
        let lines = match event.delta {
            ScrollDelta::Lines(delta) => crate::wheel::terminal_lines(delta.y),
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
            let queue = self.tmux_scroll.get_or_insert_with(|| tmux_scroller(session));
            let _ = queue.send(count);
        } else {
            self.term.lock().scroll_display(Scroll::Delta(count));
            cx.notify();
        }
        cx.stop_propagation();
    }
}

/// The thread that scrolls a tmux session's history: one tmux call at a
/// time, in order, off the UI thread; the notches that come meanwhile add up
/// into the next call. Ends with its view (the sender dropped).
fn tmux_scroller(session: String) -> std::sync::mpsc::Sender<i32> {
    let (send, receive) = std::sync::mpsc::channel::<i32>();
    std::thread::spawn(move || {
        // `=name:` — exactly this session, its current pane.
        let target = format!("={session}:");
        let tmux = |args: &[&str]| {
            let _ = crate::mux::command(args)
                .stderr(std::process::Stdio::null())
                .status();
        };
        while let Ok(first) = receive.recv() {
            let lines: i32 = first + receive.try_iter().sum::<i32>();
            let steps = lines.unsigned_abs().to_string();
            if lines > 0 {
                // -e: leaves copy mode when scrolled back to the bottom.
                tmux(&["copy-mode", "-e", "-t", &target]);
                tmux(&["send-keys", "-t", &target, "-X", "-N", &steps, "scroll-up"]);
            } else if lines < 0 {
                tmux(&["send-keys", "-t", &target, "-X", "-N", &steps, "scroll-down"]);
            }
        }
    });
    send
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for TerminalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _timing = crate::stats::Timing::new("terminal");
        div()
            .id("terminal")
            .key_context(keymap::TERMINAL)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::send_tab))
            .on_action(cx.listener(Self::send_back_tab))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::paste_clipboard))
            .on_key_down(cx.listener(Self::on_key_down))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_down(MouseButton::Middle, cx.listener(Self::on_middle_click))
            .on_mouse_down(MouseButton::Right, cx.listener(Self::on_right_click))
            // Ctrl held over a link: a hand, it opens.
            .on_modifiers_changed(cx.listener(|view, _: &ModifiersChangedEvent, _, cx| {
                if view.hover_link.is_some() {
                    cx.notify();
                }
            }))
            .cursor(if self.hover_link.is_some() && window.modifiers().control { CursorStyle::PointingHand } else { CursorStyle::IBeam })
            .relative()
            .size_full()
            .bg(background())
            .child(TerminalElement {
                term: self.term.clone(),
                view: Some(cx.entity()),
                viewport: None,
                status_line: false,
            })
            .children(self.menu.map(|at| self.menu_view(at, cx)))
            // A link whose text is not its address (a program's OSC 8 link):
            // the address, just above it.
            .children(self.hover_link.as_ref().filter(|l| l.named).and_then(|link| {
                let (_, cell) = self.layout;
                let offset = self.term.lock().grid().display_offset() as i32;
                let &(line, column, _) = link.cells.first()?;
                let row = (line + offset).max(1);
                Some(
                    div()
                        .absolute()
                        .left(cell.width * column as f32)
                        .top(cell.height * (row - 1) as f32 - px(4.))
                        .px_1p5()
                        .rounded_sm()
                        .bg(crate::theme::p().surface)
                        .border_1()
                        .border_color(crate::theme::p().border)
                        .text_xs()
                        .text_color(crate::theme::p().text)
                        .child(format!("→ {}", link.uri)),
                )
            }))
            .when(self.exited, |d| {
                d.child(
                    div()
                        .absolute()
                        .bottom_0()
                        .w_full()
                        .p_2()
                        .bg(crate::theme::p().danger)
                        .text_color(crate::theme::on(crate::theme::p().danger))
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
    /// Without a view: only the bottom `lines` × the first `columns` of the
    /// grid — a card's viewport, where a program such as Claude Code writes.
    viewport: Option<(usize, usize)>,
    /// The grid's last row is tmux's status line: it does not count as
    /// written, so that a shell's first rows stay in the viewport.
    status_line: bool,
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
    /// Colour emoji, drawn as images over their cells.
    emoji: Vec<(Bounds<Pixels>, Arc<RenderImage>)>,
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
        // The rows above the viewport, and the columns it keeps: the viewport
        // ends at the last row in use — the bottom for a program that fills
        // the screen (Claude Code), the prompt for a shell that has written
        // only its first rows.
        let (skip_lines, keep_columns) = match self.viewport {
            Some((lines, columns)) => {
                let last = last_used_line(&self.term.lock(), self.status_line);
                (last.saturating_add(1).saturating_sub(lines), columns)
            }
            None => (0, usize::MAX),
        };
        let chosen = font_size();
        let font_size = match &self.view {
            Some(_) => px(chosen),
            None => {
                use alacritty_terminal::grid::Dimensions as _;
                let columns = self.term.lock().columns().min(keep_columns).max(1) as f32;
                let fit = f32::from(bounds.size.width) / (columns * f32::from(advance(px(chosen))));
                px((chosen * fit).max(2.))
            }
        };
        let cell = size(advance(font_size), (font_size * LINE_HEIGHT).round());

        let columns = (f32::from(bounds.size.width) / f32::from(cell.width)).floor() as u16;
        let lines = (f32::from(bounds.size.height) / f32::from(cell.height)).floor() as u16;
        let term = self.term.clone();
        let focused = match &self.view {
            Some(view) => view.update(cx, |view, cx| {
                view.resize(columns, lines, cell, cx);
                view.layout = (bounds.origin, cell);
                (view.focus.is_focused(window), view.hover_link.as_ref().map(|l| l.cells.clone()))
            }),
            None => (false, None),
        };
        let (focused, hover_cells) = focused;

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
        let mut emoji = Vec::new();
        let mut rules = Vec::new();
        let mut rows: Vec<(usize, Vec<Segment>)> = Vec::new();
        // The segment being built: first column, text, style.
        let mut pending: Option<(usize, String, TextRun)> = None;
        let mut current_line = usize::MAX;
        let mut segments = Vec::new();
        let offset = content.display_offset as i32;
        let selection = content.selection;

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
            if line < skip_lines || column >= keep_columns {
                continue;
            }
            let line = line - skip_lines;
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
            if selection.is_some_and(|range| range.contains(indexed.point)) {
                let selection = crate::theme::terminal().selection;
                bg = Rgb {
                    r: (selection >> 16) as u8,
                    g: (selection >> 8) as u8,
                    b: selection as u8,
                };
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

            // A colour emoji: an image over its cells, not text.
            let wide = flags.contains(Flags::WIDE_CHAR);
            if wide || cell_data.zerowidth().is_some() {
                let mut text = String::from(cell_data.c);
                if let Some(zerowidth) = cell_data.zerowidth() {
                    text.extend(zerowidth);
                }
                if crate::emoji::is_emoji(&text, wide) {
                    if let Some(image) = crate::emoji::image(&text) {
                        flush(&mut pending, &mut segments);
                        emoji.push((
                            crate::emoji::bounds(at(line, column), cell.width * width as f32, cell.height),
                            image,
                        ));
                        continue;
                    }
                }
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
                let color = to_hsla(resolve(Color::Named(NamedColor::Cursor)));
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
        if !live {
            stats::card(started);
        }
        backgrounds.extend(rules);
        // The link under the pointer: a line under its cells.
        if let Some(cells) = hover_cells {
            let offset = term.grid().display_offset() as i32;
            let screen = term.screen_lines() as i32;
            let colour = to_hsla(default_rgb(NamedColor::Foreground as usize));
            for (line, from, to) in cells {
                let row = line + offset;
                if (0..screen).contains(&row) {
                    let at = point(
                        bounds.origin.x + cell.width * from as f32,
                        bounds.origin.y + cell.height * (row + 1) as f32 - px(2.),
                    );
                    backgrounds.push(fill(Bounds::new(at, size(cell.width * (to - from) as f32, px(1.))), colour));
                }
            }
        }
        Frame {
            cell,
            backgrounds,
            emoji,
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
            for (at, image) in frame.emoji.drain(..) {
                let _ = window.paint_image(at, at, Corners::default(), image, 0, false);
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

/// The palette when the program did not set a colour: the theme's 16 ANSI
/// colours and its text, background and cursor; the xterm 6x6x6 cube and
/// gray ramp for indexes 16..=255.
fn default_rgb(index: usize) -> Rgb {
    let colours = crate::theme::terminal();
    let hex = |v: u32| Rgb {
        r: (v >> 16) as u8,
        g: (v >> 8) as u8,
        b: v as u8,
    };
    match index {
        0..=15 => hex(colours.ansi[index]),
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
        i if i == NamedColor::Background as usize => hex(colours.background),
        i if i == NamedColor::Cursor as usize => hex(colours.cursor),
        i if i >= NamedColor::DimBlack as usize && i <= NamedColor::DimWhite as usize => {
            let base = default_rgb(i - NamedColor::DimBlack as usize);
            Rgb {
                r: (base.r as f32 * 0.66) as u8,
                g: (base.g as f32 * 0.66) as u8,
                b: (base.b as f32 * 0.66) as u8,
            }
        }
        i if i == NamedColor::DimForeground as usize => {
            let base = hex(colours.foreground);
            Rgb {
                r: (base.r as f32 * 0.66) as u8,
                g: (base.g as f32 * 0.66) as u8,
                b: (base.b as f32 * 0.66) as u8,
            }
        }
        _ => hex(colours.foreground),
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

/// A terminal's screen drawn elsewhere — a card of the slider or the
/// gallery —, read-only, at a size that fits.
pub struct Snapshot {
    term: Arc<FairMutex<Term<Listener>>>,
    /// A tmux client's: its last row is tmux's status line.
    status_line: bool,
}

/// The last row of the screen in use: the cursor's, or below it the last
/// one with something written — tmux's status line left aside.
fn last_used_line<T>(term: &Term<T>, status_line: bool) -> usize {
    use alacritty_terminal::grid::Dimensions as _;
    use alacritty_terminal::index::{Column, Line};
    let grid = term.grid();
    let rows = grid.screen_lines().saturating_sub(usize::from(status_line));
    let cursor = (grid.cursor.point.line.0.max(0) as usize).min(rows.saturating_sub(1));
    let columns = grid.columns();
    let written = (0..rows)
        .rev()
        .find(|&line| (0..columns).any(|column| grid[Line(line as i32)][Column(column)].c != ' '))
        .unwrap_or(0);
    cursor.max(written)
}

impl Snapshot {
    /// Only the bottom `lines` × the first `columns`: a card's viewport.
    pub fn viewport(&self, lines: usize, columns: usize) -> TerminalElement {
        TerminalElement {
            term: self.term.clone(),
            view: None,
            viewport: Some((lines, columns)),
            status_line: self.status_line,
        }
    }
}
