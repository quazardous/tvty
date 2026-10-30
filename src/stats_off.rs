//! A build without the render debug (a release): `stats.rs`'s calls, doing
//! nothing, so that the code that makes them stays as it is. No file, no
//! `TVTY_STATS`, nothing measured.

use std::time::Instant;

pub fn init() {}
pub fn key_sent() {}
pub fn output() {}
pub fn size_seen() {}
pub fn frame() {}
pub fn prepaint(_: Instant) {}
pub fn resized(_: u16, _: u16) {}
pub fn card(_: Instant) {}
pub fn window_started() {}

pub fn probe() -> Option<gpui_kit::Div> {
    None
}

/// A view's render, not timed.
pub struct Timing;

impl Timing {
    pub fn new(_: &'static str) -> Self {
        Self
    }
}
