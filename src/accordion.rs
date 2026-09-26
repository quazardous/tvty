//! Foldable sections, the one way tvty lists things by group: the tickets
//! by band, the sessions by state. Each section has its title — ▾ or ▸, the
//! name in small capitals, its count; a click folds it — and its own
//! scroll: the sections share the height, a long one never pushes the
//! others out of sight, and none shrinks below its title and a couple of
//! rows. When even those minimums do not fit, the whole list scrolls
//! rather than letting them overlap: put the sections in [`list`].

use gpui_kit::component::scroll::{Scrollbar, ScrollbarAxis};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::p;

/// A section's title line, in pixels.
pub const TITLE_HEIGHT: f32 = 28.;

/// The column that holds the sections.
pub fn list(id: impl Into<ElementId>) -> Stateful<Div> {
    div().id(id).flex().flex_col().flex_1().min_h_0().overflow_y_scroll()
}

/// One section.
pub struct Section {
    pub id: SharedString,
    pub title: String,
    pub count: usize,
    pub folded: bool,
    /// The least height its body keeps when unfolded (a couple of rows, or
    /// all of them when fewer).
    pub keep: f32,
    /// Its scroll, kept by the owner across renders.
    pub scroll: ScrollHandle,
    pub body: Vec<AnyElement>,
}

impl Section {
    /// The section, its title folding it through `on_toggle`.
    pub fn render(self, on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Div {
        let Self { id, title, count, folded, keep, scroll, body } = self;
        let header = div()
            .id(SharedString::from(format!("{id}-title")))
            .flex()
            .flex_none()
            .items_center()
            .gap_1()
            .h(px(TITLE_HEIGHT))
            .px_3()
            .pt_1()
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .text_color(p().muted)
            .cursor_pointer()
            .hover(|d| d.text_color(p().text))
            .child(div().w(px(10.)).child(if folded { "▸" } else { "▾" }))
            .child(title.to_uppercase())
            .child(div().font_weight(FontWeight::NORMAL).child(count.to_string()))
            .on_click(on_toggle);
        let keep = if folded { 0. } else { keep };
        div()
            .flex()
            .flex_col()
            .flex_initial()
            .min_h(px(TITLE_HEIGHT + keep))
            .child(header)
            .when(!folded, |d| {
                let rows = div()
                    .id(SharedString::from(format!("{id}-rows")))
                    .flex()
                    .flex_col()
                    .flex_initial()
                    .min_h_0()
                    .track_scroll(&scroll)
                    .overflow_y_scroll()
                    .children(body);
                d.child(
                    div()
                        .relative()
                        .flex()
                        .flex_col()
                        .flex_initial()
                        .min_h(px(keep))
                        .child(rows)
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                .child(Scrollbar::new(&scroll).axis(ScrollbarAxis::Vertical).viewport_from_layout()),
                        ),
                )
            })
    }
}
