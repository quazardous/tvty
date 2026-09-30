//! What is on screen, by id: every element named (`ui::Named`), with
//! where each was painted — a page's DOM, for a test to read and act on
//! (`crate::control`, `scripts/tvty-ctl`). Kept only while the debug
//! control is on (`TVTY_DEBUG_CONTROL`): off, nothing is recorded.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use gpui_kit::*;

static ON: AtomicBool = AtomicBool::new(false);
static FRAME: AtomicU64 = AtomicU64::new(0);
/// Each element seen: its id, where it was painted, in which frame.
static MARKS: Mutex<Vec<(String, Bounds<Pixels>, u64)>> = Mutex::new(Vec::new());

/// Turns the recording on, once, at start.
pub fn enable() {
    ON.store(true, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ON.load(Ordering::Relaxed)
}

/// A new frame is being drawn: what is not drawn again goes.
pub fn next_frame() {
    if enabled() {
        FRAME.fetch_add(1, Ordering::Relaxed);
    }
}

/// An element, marked `id`: a zero-size layer over it (its parent's box)
/// that notes where it was painted. Put it as the element's child.
pub fn mark(id: impl Into<String>) -> impl IntoElement {
    let id = id.into();
    canvas(
        move |bounds, _, _| {
            let frame = FRAME.load(Ordering::Relaxed);
            if let Ok(mut marks) = MARKS.lock() {
                // One place per id: the last painted wins (a list's rows
                // carry distinct ids).
                marks.retain(|(m, _, at)| *m != id && *at == frame);
                marks.push((id.clone(), bounds, frame));
            }
        },
        |_, _, _, _| {},
    )
    // Pinned to its parent's box (see tips::root_mark).
    .absolute()
    .inset_0()
}

/// [`mark`], only while the control is on: `.children(mark_if(…))`.
pub fn mark_if(id: impl Into<String>) -> Option<impl IntoElement> {
    enabled().then(|| mark(id))
}

/// What the last frame painted, in paint order: read between frames (on
/// the UI thread), the last one is whole — what it did not paint is gone.
pub fn marks() -> Vec<(String, Bounds<Pixels>)> {
    let frame = FRAME.load(Ordering::Relaxed);
    MARKS
        .lock()
        .map(|marks| marks.iter().filter(|(_, _, at)| *at == frame).map(|(id, b, _)| (id.clone(), *b)).collect())
        .unwrap_or_default()
}

/// Where `id` is: the last painted of that id.
pub fn bounds_of(id: &str) -> Option<Bounds<Pixels>> {
    marks().into_iter().rev().find(|(m, _)| m == id).map(|(_, b)| b)
}
