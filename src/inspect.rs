//! What is on screen, by id: every element named (`ui::Named`), with
//! where each was painted — a page's DOM, for a test to read and act on
//! (`crate::control`, `scripts/tvty-ctl`). Kept only while the debug
//! control is on (`TVTY_DEBUG_CONTROL`): off, nothing is recorded.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use gpui_kit::*;

static ON: AtomicBool = AtomicBool::new(false);
static FRAME: AtomicU64 = AtomicU64::new(0);
/// Each element seen: its id, where it was painted, in which frame, and
/// what it says when that is known.
static MARKS: Mutex<Vec<(String, Bounds<Pixels>, u64, Option<String>)>> = Mutex::new(Vec::new());

/// The animations drawn: their id, how long each lasts, when it was first
/// and last drawn. One still within its time is running.
static ANIMATIONS: Mutex<Vec<(String, std::time::Duration, std::time::Instant, std::time::Instant)>> = Mutex::new(Vec::new());

/// An animation is drawn, under the id it is given to GPUI (a new id starts
/// it again), lasting `lasts`: said where it is made, so that a test can
/// wait for the screen to hold still (`still_animating`).
pub fn animation(id: &str, lasts: std::time::Duration) {
    if !enabled() {
        return;
    }
    let now = std::time::Instant::now();
    if let Ok(mut animations) = ANIMATIONS.lock() {
        // Those not drawn for a while are gone from the screen.
        animations.retain(|(_, _, _, seen)| now.duration_since(*seen) < std::time::Duration::from_secs(2));
        match animations.iter_mut().find(|(known, ..)| known == id) {
            Some(entry) => entry.3 = now,
            None => animations.push((id.to_string(), lasts, now, now)),
        }
    }
}

/// The animations still running, by id.
pub fn still_animating() -> Vec<String> {
    let now = std::time::Instant::now();
    ANIMATIONS
        .lock()
        .map(|animations| animations.iter().filter(|(_, lasts, began, _)| now.duration_since(*began) < *lasts).map(|(id, ..)| id.clone()).collect())
        .unwrap_or_default()
}

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
    mark_saying(id, None)
}

/// [`mark`], with what the element says (a button's label, a row's title).
pub fn mark_saying(id: impl Into<String>, text: Option<String>) -> impl IntoElement {
    let id = id.into();
    canvas(
        move |bounds, _, _| {
            let frame = FRAME.load(Ordering::Relaxed);
            if let Ok(mut marks) = MARKS.lock() {
                // One place per id: the last painted wins (a list's rows
                // carry distinct ids).
                marks.retain(|(m, _, at, _)| *m != id && *at == frame);
                marks.push((id.clone(), bounds, frame, text.clone()));
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
pub fn marks() -> Vec<(String, Bounds<Pixels>, Option<String>)> {
    let frame = FRAME.load(Ordering::Relaxed);
    MARKS
        .lock()
        .map(|marks| marks.iter().filter(|(_, _, at, _)| *at == frame).map(|(id, b, _, text)| (id.clone(), *b, text.clone())).collect())
        .unwrap_or_default()
}

/// Whether `id` is on screen, and what it says: `Some(None)` for an
/// element that is there and whose text is not known.
pub fn text_of(id: &str) -> Option<Option<String>> {
    marks().into_iter().rev().find(|(m, _, _)| m == id).map(|(_, _, text)| text)
}

/// Where `id` is: the last painted of that id.
pub fn bounds_of(id: &str) -> Option<Bounds<Pixels>> {
    marks().into_iter().rev().find(|(m, _, _)| m == id).map(|(_, b, _)| b)
}
