//! Rendering measurements, on when `$TVTY_STATS` names a file to append to:
//! - `echo <ms>`: from a keystroke sent to the PTY to the first frame painted
//!   after the PTY answered;
//! - `stream <frames> fps <wakeups> wakeups, prepaint max <ms>`: every second the PTY talked,
//!   with the slowest grid preparation of that second;
//! - `resize <columns>x<lines>`: each new size sent to the PTY;
//! - `cards <n> max <ms>`: every second cards were drawn (the slider, the
//!   gallery), the most any one frame spent preparing them all.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

struct State {
    file: File,
    /// A keystroke waiting for its echo, and whether output came since.
    key: Option<(Instant, bool)>,
    second: Instant,
    frames: u32,
    wakeups: u32,
    prepaint_max: f64,
    /// Cards: time spent this frame, the worst frame this second, how many.
    cards_frame: f64,
    cards_max: f64,
    cards_count: u32,
}

static STATE: LazyLock<Option<Mutex<State>>> = LazyLock::new(|| {
    let path = std::env::var("TVTY_STATS").ok().filter(|p| !p.is_empty())?;
    let file = OpenOptions::new().create(true).append(true).open(path).ok()?;
    Some(Mutex::new(State {
        file,
        key: None,
        second: Instant::now(),
        frames: 0,
        wakeups: 0,
        prepaint_max: 0.,
        cards_frame: 0.,
        cards_max: 0.,
        cards_count: 0,
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
    });
}

/// A card prepared: its time adds to the frame's.
pub fn card(started: Instant) {
    with(|s| {
        s.cards_frame += started.elapsed().as_secs_f64() * 1000.;
        s.cards_count += 1;
    });
}
