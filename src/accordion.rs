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
    /// The rows drawn: all of them, or those of [`window`].
    pub body: Vec<AnyElement>,
    /// The height of the rows left out above and below the drawn ones, so
    /// that the scroll keeps the whole list's height.
    pub before: f32,
    pub after: f32,
    /// For a windowed section: what [`window`] drew, and whose view to draw
    /// again when the section's size or scroll turned out different.
    pub windowed: Option<Windowed>,
}

/// What a windowed section drew: the rows, their height, and its view.
pub struct Windowed {
    pub drawn: std::ops::Range<usize>,
    pub row: f32,
    pub owner: EntityId,
}

/// The rows of a section worth drawing when each is `row` pixels high:
/// those its scroll showed at the last frame, and a few around. The others
/// are not laid out at all — with hundreds of rows, that is what keeps a
/// frame cheap.
pub fn window(count: usize, row: f32, scroll: &ScrollHandle) -> std::ops::Range<usize> {
    let top = (-f32::from(scroll.offset().y)).max(0.);
    let height = match f32::from(scroll.bounds().size.height) {
        // Not laid out yet: a screenful.
        h if h <= 0. => 1200.,
        h => h,
    };
    let first = ((top / row) as usize).saturating_sub(2).min(count);
    let last = (((top + height) / row).ceil() as usize + 2).min(count);
    first..last
}

impl Section {
    /// The section, its title folding it through `on_toggle`.
    pub fn render(self, on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Div {
        let Self { id, title, count, folded, keep, scroll, body, before, after, windowed } = self;
        // The rows were chosen from the last frame's size and scroll. Once
        // this one is laid out, the choice is checked: a list that grew or
        // moved (rows came, a section above folded) would leave blank rows
        // until something else drew it again, so its view draws again now.
        let recheck = windowed.filter(|_| !folded).map(|w| {
            let scroll = scroll.clone();
            canvas(
                move |_, win, _| {
                    if window(count, w.row, &scroll) != w.drawn {
                        let owner = w.owner;
                        win.on_next_frame(move |_, cx| cx.notify(owner));
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_0()
        });
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
                    .when(before > 0., |d| d.child(div().flex_none().h(px(before))))
                    .children(body)
                    .when(after > 0., |d| d.child(div().flex_none().h(px(after))));
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
                // After the rows: by then this frame's size and scroll are known.
                .children(recheck)
            })
    }
}
