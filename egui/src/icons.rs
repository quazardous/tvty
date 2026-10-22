//! tvty's icons: its own (the app's) and the Material Symbols the GPUI
//! build draws (`assets/icons/`), registered with egui-pop's icon set,
//! once, before the theme is installed (their textures are made then).

use std::sync::OnceLock;

use egui_pop::core::icon::Icon;
use egui_pop::core::icon_set::{self, Paint};

macro_rules! icons {
    ($($name:ident => $file:literal, $paint:ident;)*) => {
        $(
            pub fn $name() -> Option<Icon> {
                static ICON: OnceLock<Option<Icon>> = OnceLock::new();
                *ICON.get_or_init(|| {
                    icon_set::register(
                        concat!("tvty.", stringify!($name)),
                        &include_bytes!(concat!("../../assets/", $file))[..],
                        Paint::$paint,
                    )
                    .ok()
                })
            }
        )*

        /// Registers every icon.
        pub fn register() {
            $(let _ = $name();)*
        }
    };
}

icons! {
    app => "tvty.svg", Colour;
    pinned => "icons/keep_fill.svg", Mono;
    megaphone => "icons/megaphone.svg", Mono;
    message_all => "icons/bot-message-square.svg", Mono;
    window_minimize => "icons/window-minimize.svg", Mono;
    window_maximize => "icons/window-maximize.svg", Mono;
    window_restore => "icons/window-restore.svg", Mono;
    window_close => "icons/window-close.svg", Mono;
    plan => "icons/edit_note.svg", Mono;
    resolution => "icons/check_circle.svg", Mono;
    wontfix => "icons/block.svg", Mono;
    escalation => "icons/priority_high.svg", Mono;
    step => "icons/play_circle.svg", Mono;
    stalled_step => "icons/pause_circle.svg", Mono;
    rejected => "icons/cancel.svg", Mono;
    closed_resolved => "icons/task_alt.svg", Mono;
    closed => "icons/lock.svg", Mono;
    hot => "icons/local_fire_department.svg", Mono;
    critical => "icons/warning.svg", Mono;
    priority_high => "icons/arrow_shape_up.svg", Mono;
    priority_urgent => "icons/arrow_shape_up_stack.svg", Mono;
    priority_low => "icons/low_priority.svg", Mono;
    comments => "icons/chat_bubble.svg", Mono;
    comments_mine => "icons/chat_bubble_mine.svg", Mono;
    pending_comments => "icons/schedule.svg", Mono;
}

/// A row's state glyph, drawn.
pub fn of_glyph(glyph: crate::board::Glyph) -> Option<Icon> {
    use crate::board::Glyph;
    match glyph {
        Glyph::Plan => plan(),
        Glyph::Resolution => resolution(),
        Glyph::Wontfix => wontfix(),
        Glyph::Escalation => escalation(),
        Glyph::Step => step(),
        Glyph::StalledStep => stalled_step(),
        Glyph::Rejected => rejected(),
        Glyph::ClosedResolved => closed_resolved(),
        Glyph::Closed => closed(),
    }
}

/// A priority's icon, when it is not normal.
pub fn of_priority(priority: &str) -> Option<Icon> {
    match priority {
        "urgent" => priority_urgent(),
        "high" => priority_high(),
        "low" => priority_low(),
        _ => None,
    }
}
