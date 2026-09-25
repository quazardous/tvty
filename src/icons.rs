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
    Rejected => "undo.svg",
    ClosedResolved => "task_alt.svg",
    Closed => "lock.svg",
    Hot => "local_fire_department.svg",
    Critical => "warning.svg",
    PriorityHigh => "arrow_shape_up.svg",
    PriorityUrgent => "arrow_shape_up_stack.svg",
    PriorityLow => "low_priority.svg",
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
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
