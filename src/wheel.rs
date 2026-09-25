//! The mouse wheel, faster than GPUI's. GPUI turns a notch into three lines
//! of text, about 60 px: in the ticket lists, barely one row. Every scroll
//! area is the kit's, so rather than each one, the window enlarges the notch
//! itself: it catches the wheel first (capture phase), stops it, and sends it
//! again, bigger. A touchpad's deltas (in pixels) go through untouched.

use std::cell::Cell;
use std::sync::atomic::{AtomicU32, Ordering};

use gpui_kit::*;

/// A notch in a list or a thread: GPUI's lines, times this — about three
/// ticket rows.
const LISTS: f32 = 2.5;
/// A notch in a terminal, in lines of its history.
const TERMINAL_LINES: f32 = 5.;
/// GPUI's lines for a notch, on Linux.
const NOTCH_LINES: f32 = 3.;

/// The user's multiplier (`scroll_speed`), as `f32` bits.
static SPEED: AtomicU32 = AtomicU32::new(0x3f80_0000);

thread_local! {
    /// The event being sent again: let it through.
    static AGAIN: Cell<bool> = const { Cell::new(false) };
}

pub fn set_speed(speed: f32) {
    SPEED.store(speed.clamp(0.2, 5.).to_bits(), Ordering::Relaxed);
}

fn speed() -> f32 {
    f32::from_bits(SPEED.load(Ordering::Relaxed))
}

/// Registers, for this frame, the listener that enlarges each notch.
pub fn speed_up(window: &mut Window) {
    window.on_mouse_event(|event: &ScrollWheelEvent, phase, window, cx| {
        if phase != DispatchPhase::Capture || AGAIN.get() {
            return;
        }
        let ScrollDelta::Lines(lines) = event.delta else {
            return;
        };
        cx.stop_propagation();
        let mut faster = event.clone();
        faster.delta = ScrollDelta::Lines(lines * LISTS * speed());
        window.defer(cx, move |window, cx| {
            AGAIN.set(true);
            window.dispatch_event(PlatformInput::ScrollWheel(faster), cx);
            AGAIN.set(false);
        });
    });
}

/// The lines a terminal scrolls for a wheel delta in lines, as enlarged by
/// [`speed_up`]: [`TERMINAL_LINES`] a notch, times the user's speed.
pub fn terminal_lines(lines: f32) -> f32 {
    lines / LISTS * TERMINAL_LINES / NOTCH_LINES
}

#[cfg(test)]
mod tests {
    use super::{LISTS, NOTCH_LINES, TERMINAL_LINES, speed, terminal_lines};

    #[test]
    fn a_notch_is_five_terminal_lines() {
        let notch = NOTCH_LINES * LISTS * speed();
        assert!((terminal_lines(notch) - TERMINAL_LINES).abs() < 1e-4);
    }
}
