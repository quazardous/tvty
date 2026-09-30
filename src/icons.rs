//! tvty's icons: a few Google Material Symbols (Apache 2.0, see
//! `assets/icons/README.md`), drawn as SVG in the theme's colours — as
//! aiball's web UI inlines them. They replace the glyphs and emoji of the
//! ticket list, which a font cannot colour (the emoji font is COLRv1, which
//! GPUI does not read).

use std::borrow::Cow;

use gpui_kit::*;

use crate::rowstate::Glyph;
use crate::theme::p;

/// The kit's assets, and tvty's icons under `tvty/icons/`.
pub struct Assets;

const PREFIX: &str = "tvty/icons/";

/// The app's own icon, drawn in its colours (an image, not a tinted glyph).
pub const APP: &str = "tvty/app.svg";
const APP_SVG: &[u8] = include_bytes!("../assets/tvty.svg");

macro_rules! icons {
    ($($variant:ident => $file:literal),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Icon { $($variant),* }

        impl Icon {
            fn file(self) -> &'static str {
                match self { $(Icon::$variant => $file),* }
            }
        }

        const FILES: &[(&str, &[u8])] = &[
            $(($file, include_bytes!(concat!("../assets/icons/", $file)))),*
        ];
    };
}

icons! {
    Plan => "edit_note.svg",
    Resolution => "check_circle.svg",
    Wontfix => "block.svg",
    Escalation => "priority_high.svg",
    Step => "play_circle.svg",
    StalledStep => "pause_circle.svg",
    Rejected => "cancel.svg",
    ClosedResolved => "task_alt.svg",
    Closed => "lock.svg",
    Hot => "local_fire_department.svg",
    Megaphone => "megaphone.svg",
    MessageAgents => "bot-message-square.svg",
    Critical => "warning.svg",
    PriorityHigh => "arrow_shape_up.svg",
    PriorityUrgent => "arrow_shape_up_stack.svg",
    PriorityLow => "low_priority.svg",
    Comments => "chat_bubble.svg",
    // The same bubble mirrored: its point on the right, the user's word.
    CommentsMine => "chat_bubble_mine.svg",
    PendingComments => "schedule.svg",
    Sunk => "snooze.svg",
    Unassign => "person_off.svg",
    // What runs on aiball's hub, seen from another machine.
    Hub => "cloud.svg",
    // The ticket panel held beside the terminal, or let over it.
    Pinned => "keep_fill.svg",
    Unpinned => "keep.svg",
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        if path == APP {
            return Ok(Some(Cow::Borrowed(APP_SVG)));
        }
        if let Some(file) = path.strip_prefix(PREFIX) {
            return Ok(FILES.iter().find(|(f, _)| *f == file).map(|(_, bytes)| Cow::Borrowed(*bytes)));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        gpui_kit::assets::Assets.list(path)
    }
}

/// An icon `size` pixels square, in `colour`.
pub fn icon(icon: Icon, colour: Hsla, size: f32) -> Svg {
    svg()
        .path(SharedString::from(format!("{PREFIX}{}", icon.file())))
        .size(px(size))
        .flex_none()
        .text_color(colour)
}

/// The pin of the ticket panel: full when it is held beside the terminal.
pub fn pin(pinned: bool, colour: Hsla, size: f32) -> Svg {
    icon(if pinned { Icon::Pinned } else { Icon::Unpinned }, colour, size)
}

/// The 📢 (what steers a project's agents), the kit's megaphone: an emoji
/// draws only where a font has it.
pub fn megaphone(colour: Hsla, size: f32) -> Svg {
    icon(Icon::Megaphone, colour, size)
}

/// A loop's glyph (▶ ‖ ■ …) in `colour`: the pause drawn as two bars
/// filling a square `size` pixels wide — the font's ‖ is a thin, tall pair
/// beside ▶ and ■ — any other as text.
///
/// Every glyph sits in a box `size` pixels high — square for one sign, as
/// wide as its words otherwise ("… boot") — centred: one sign replacing
/// another moves nothing around it.
pub fn loop_glyph(glyph: &str, colour: Hsla, size: f32) -> AnyElement {
    let slot = div().flex_none().h(px(size)).min_w(px(size)).flex().items_center().justify_center();
    if glyph == "‖" {
        let bar = || div().h_full().w(px((size * 0.34).round().max(2.))).rounded(px(1.)).bg(colour);
        return slot.w(px(size)).justify_between().child(bar()).child(bar()).into_any_element();
    }
    let slot = if glyph.chars().count() == 1 { slot.w(px(size)) } else { slot };
    slot.text_color(colour).text_size(px(size * 1.3)).line_height(px(size)).child(glyph.to_string()).into_any_element()
}

/// A plain dot in the same box as [`loop_glyph`]'s.
pub fn loop_dot(colour: Hsla, size: f32) -> AnyElement {
    div()
        .flex_none()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .child(div().size(px((size * 0.85).round())).rounded_full().bg(colour))
        .into_any_element()
}

/// A row's state glyph, drawn.
pub fn of_glyph(glyph: Glyph) -> Icon {
    match glyph {
        Glyph::Plan => Icon::Plan,
        Glyph::Resolution => Icon::Resolution,
        Glyph::Wontfix => Icon::Wontfix,
        Glyph::Escalation => Icon::Escalation,
        Glyph::Step => Icon::Step,
        Glyph::StalledStep => Icon::StalledStep,
        Glyph::Rejected => Icon::Rejected,
        Glyph::ClosedResolved => Icon::ClosedResolved,
        Glyph::Closed => Icon::Closed,
    }
}

/// A decision's kind, drawn.
pub fn of_kind(kind: &str) -> Icon {
    match kind {
        "resolution" => Icon::Resolution,
        "wontfix" => Icon::Wontfix,
        "escalation" => Icon::Escalation,
        _ => Icon::Plan,
    }
}

/// The colour of a priority's icon.
pub fn priority_colour(priority: &str) -> Hsla {
    match priority {
        "urgent" => p().danger,
        "high" => p().warning,
        _ => p().info,
    }
}

/// An icon and a few words, on one line.
pub fn labelled(icon: Icon, colour: Hsla, size: f32, text: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .gap_1()
        .min_w_0()
        .child(self::icon(icon, colour, size))
        // The words wrap next to the icon rather than run under what follows.
        .child(div().min_w_0().child(text.into()))
}

/// A pill with an icon: `colour` behind, readable ink on it.
pub fn pill(icon: Icon, text: impl Into<SharedString>, colour: Hsla) -> Div {
    let ink = crate::theme::on(colour);
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap_0p5()
        .px_1p5()
        .rounded_sm()
        .text_xs()
        .bg(colour)
        .text_color(ink)
        .child(self::icon(icon, ink, 12.))
        .child(text.into())
}

/// The icon of a priority, when it is not normal.
pub fn priority(priority: &str) -> Option<Icon> {
    match priority {
        "urgent" => Some(Icon::PriorityUrgent),
        "high" => Some(Icon::PriorityHigh),
        "low" => Some(Icon::PriorityLow),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Assets, FILES, PREFIX};
    use gpui_kit::AssetSource as _;

    #[test]
    fn every_icon_is_bundled() {
        for (file, bytes) in FILES {
            assert!(bytes.starts_with(b"<svg"), "{file} is not an SVG");
            let loaded = Assets.load(&format!("{PREFIX}{file}")).unwrap();
            assert!(loaded.is_some(), "{file} does not load");
        }
    }
}
