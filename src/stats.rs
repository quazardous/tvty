//! Rendering measurements, on when `$TVTY_STATS` names a file to append to:
//! - `echo <ms>`: from a keystroke sent to the PTY to the first frame painted
//!   after the PTY answered;
//! - `stream <frames> fps <wakeups> wakeups, prepaint max <ms>`: every second the PTY talked,
//!   with the slowest grid preparation of that second;
//! - `resize <columns>x<lines>`: each new size sent to the PTY;
//! - `reflow <ms> (sent after <ms>)`: from the view's new size to the first
//!   frame painted after the program answered it, and how long of that went
//!   before the size was sent (the settle while a side is dragged);
//! - `cards <n> max <ms>`: every second cards were drawn (the slider, the
//!   gallery), the most any one frame spent preparing them all;
//! - `window <n> frames, max <ms>, total <ms>`: every second the window was
//!   drawn, from the shell's render to its last paint (views, layout and
//!   paint of everything but the overlays drawn after).

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

struct State {
    file: File,
    /// A keystroke waiting for its echo, and whether output came since.
    key: Option<(Instant, bool)>,
    /// A new size: when the view saw it, when it was sent, whether output
    /// came since.
    reflow: Option<(Instant, Option<Instant>, bool)>,
    second: Instant,
    frames: u32,
    wakeups: u32,
    prepaint_max: f64,
    /// Cards: time spent this frame, the worst frame this second, how many.
    cards_frame: f64,
    cards_max: f64,
    cards_count: u32,
    /// The window: when this frame's render began, and this second's tally.
    window_started: Option<Instant>,
    window_frames: u32,
    window_max: f64,
    window_total: f64,
    window_second: Instant,
    /// The views' renders this second: name, how many, the slowest.
    parts: Vec<(&'static str, u32, f64)>,
    /// This frame: when the probe was laid out and prepainted.
    probe_layout: Option<Instant>,
    probe_prepaint: Option<Instant>,
}

static STATE: LazyLock<Option<Mutex<State>>> = LazyLock::new(|| {
    let path = std::env::var("TVTY_STATS").ok().filter(|p| !p.is_empty())?;
    let file = OpenOptions::new().create(true).append(true).open(path).ok()?;
    Some(Mutex::new(State {
        file,
        key: None,
        reflow: None,
        second: Instant::now(),
        frames: 0,
        wakeups: 0,
        prepaint_max: 0.,
        cards_frame: 0.,
        cards_max: 0.,
        cards_count: 0,
        window_started: None,
        window_frames: 0,
        window_max: 0.,
        window_total: 0.,
        window_second: Instant::now(),
        parts: Vec::new(),
        probe_layout: None,
        probe_prepaint: None,
    }))
});

fn with(f: impl FnOnce(&mut State)) {
    if let Some(state) = STATE.as_ref() {
        if let Ok(mut state) = state.lock() {
            f(&mut state);
        }
    }
}

pub fn key_sent() {
    with(|s| {
        if s.key.is_none() {
            s.key = Some((Instant::now(), false));
        }
    });
}

pub fn output() {
    with(|s| {
        s.wakeups += 1;
        if let Some((_, answered)) = s.key.as_mut() {
            *answered = true;
        }
        if let Some((_, Some(_), answered)) = s.reflow.as_mut() {
            *answered = true;
        }
    });
}

/// The view saw a new size (the first one of a drag).
pub fn size_seen() {
    with(|s| {
        if s.reflow.is_none() {
            s.reflow = Some((Instant::now(), None, false));
        }
    });
}

pub fn frame() {
    with(|s| {
        s.frames += 1;
        s.cards_max = s.cards_max.max(s.cards_frame);
        s.cards_frame = 0.;
        if let Some((at, true)) = s.key {
            let ms = at.elapsed().as_secs_f64() * 1000.;
            let _ = writeln!(s.file, "echo {ms:.1}");
            s.key = None;
        }
        if let Some((seen, Some(sent), true)) = s.reflow {
            let ms = |t: Instant| (t - seen).as_secs_f64() * 1000.;
            let _ = writeln!(s.file, "reflow {:.1} (sent after {:.1})", ms(Instant::now()), ms(sent));
            s.reflow = None;
        }
        let elapsed = s.second.elapsed().as_secs_f64();
        if elapsed >= 1. {
            if s.wakeups > 0 {
                let fps = s.frames as f64 / elapsed;
                let _ = writeln!(
                    s.file,
                    "stream {fps:.0} fps {} wakeups, prepaint max {:.1} ms",
                    s.wakeups, s.prepaint_max
                );
            }
            if s.cards_max > 0. {
                let _ = writeln!(s.file, "cards {} max {:.1} ms", s.cards_count, s.cards_max);
            }
            s.cards_max = 0.;
            s.cards_count = 0;
            s.second = Instant::now();
            s.frames = 0;
            s.wakeups = 0;
            s.prepaint_max = 0.;
        }
    });
}

/// How long the grid took to prepare, for `prepaint max`.
pub fn prepaint(started: Instant) {
    with(|s| s.prepaint_max = s.prepaint_max.max(started.elapsed().as_secs_f64() * 1000.));
}

pub fn resized(columns: u16, lines: u16) {
    with(|s| {
        let _ = writeln!(s.file, "resize {columns}x{lines}");
        if let Some((_, sent, _)) = s.reflow.as_mut() {
            *sent = Some(Instant::now());
        }
    });
}

/// A card prepared: its time adds to the frame's.
pub fn card(started: Instant) {
    with(|s| {
        s.cards_frame += started.elapsed().as_secs_f64() * 1000.;
        s.cards_count += 1;
    });
}

/// The shell starts rendering the window.
pub fn window_started() {
    with(|s| s.window_started = Some(Instant::now()));
}

/// The window's last element painted: the frame is done.
pub fn window_painted() {
    with(|s| {
        let Some(started) = s.window_started.take() else { return };
        let ms = started.elapsed().as_secs_f64() * 1000.;
        let since = |t: Option<Instant>| t.map_or(0., |t| (t - started).as_secs_f64() * 1000.);
        let (layout, prepaint) = (since(s.probe_layout.take()), since(s.probe_prepaint.take()));
        rendered_ms(s, "tree", layout);
        rendered_ms(s, "layout+prepaint", prepaint - layout);
        rendered_ms(s, "paint", ms - prepaint);
        s.window_frames += 1;
        s.window_max = s.window_max.max(ms);
        s.window_total += ms;
        if s.window_second.elapsed().as_secs_f64() >= 1. {
            let parts: String = s.parts.iter().map(|(n, c, m)| format!(", {n} {c}× max {m:.1} ms")).collect();
            let _ = writeln!(
                s.file,
                "window {} frames, max {:.1} ms, total {:.1} ms{parts}",
                s.window_frames, s.window_max, s.window_total
            );
            s.parts.clear();
            s.window_frames = 0;
            s.window_max = 0.;
            s.window_total = 0.;
            s.window_second = Instant::now();
        }
    });
}

/// An element that paints nothing and tells [`window_painted`]: laid last,
/// it is painted after everything before it.
pub struct Probe;

impl gpui_kit::IntoElement for Probe {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl gpui_kit::Element for Probe {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui_kit::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui_kit::GlobalElementId>,
        _: Option<&gpui_kit::InspectorElementId>,
        window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::App,
    ) -> (gpui_kit::LayoutId, ()) {
        with(|s| s.probe_layout = Some(Instant::now()));
        (window.request_layout(gpui_kit::Style::default(), [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui_kit::GlobalElementId>,
        _: Option<&gpui_kit::InspectorElementId>,
        _: gpui_kit::Bounds<gpui_kit::Pixels>,
        _: &mut (),
        _: &mut gpui_kit::Window,
        _: &mut gpui_kit::App,
    ) {
        with(|s| s.probe_prepaint = Some(Instant::now()));
    }

    fn paint(
        &mut self,
        _: Option<&gpui_kit::GlobalElementId>,
        _: Option<&gpui_kit::InspectorElementId>,
        _: gpui_kit::Bounds<gpui_kit::Pixels>,
        _: &mut (),
        _: &mut (),
        _: &mut gpui_kit::Window,
        _: &mut gpui_kit::App,
    ) {
        window_painted();
    }
}

/// A view rendered (its `render` alone, not its layout or paint).
pub fn rendered(name: &'static str, started: Instant) {
    with(|s| rendered_ms(s, name, started.elapsed().as_secs_f64() * 1000.));
}

fn rendered_ms(s: &mut State, name: &'static str, ms: f64) {
    match s.parts.iter_mut().find(|(n, _, _)| *n == name) {
        Some(part) => {
            part.1 += 1;
            part.2 = part.2.max(ms);
        }
        None => s.parts.push((name, 1, ms)),
    }
}

/// Times a view's render until it goes out of scope.
pub struct Timing(&'static str, Instant);

impl Timing {
    pub fn new(name: &'static str) -> Self {
        Self(name, Instant::now())
    }
}

impl Drop for Timing {
    fn drop(&mut self) {
        rendered(self.0, self.1);
    }
}
