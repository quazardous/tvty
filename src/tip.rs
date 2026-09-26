//! Tooltips: every element that means something says what, under the
//! pointer — a glyph, a colour, a count. One way everywhere: `.tip("…")` on
//! an element with an id.

use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::*;

pub trait Tip: StatefulInteractiveElement + Sized {
    /// Shows `text` while the pointer rests on it.
    fn tip(self, text: impl Into<SharedString>) -> Self {
        let text: SharedString = text.into();
        self.tooltip(move |window, cx| Tooltip::new(text.clone()).build(window, cx))
    }
}

impl<E: StatefulInteractiveElement + Sized> Tip for E {}
