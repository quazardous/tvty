//! The side column of a full-screen page — a ticket's fields, the full
//! list's filters, a new ticket's fields: a third of the window and no
//! wider than its fields need by default, or as the user drags its border.
//! One width for all of them, kept with the layout (the shell observes
//! [`SideWidth`]); every view that shows the column observes it too.

use crate::ui::Named as _;
use gpui_kit::*;

use crate::theme::p;

/// The narrowest the column goes, and the widest it is by default.
pub const MIN: f32 = 240.;
const DEFAULT_MAX: f32 = 460.;

/// The width the user dragged; none: the default.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SideWidth(pub Option<f32>);

impl Global for SideWidth {}

pub fn width(cx: &App) -> Option<f32> {
    cx.try_global::<SideWidth>().and_then(|w| w.0)
}

/// What the border carries while dragged.
pub struct SideDrag;

/// A drag that shows nothing under the pointer.
struct NoPreview;

impl Render for NoPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// The column, at its width.
pub fn column(cx: &App) -> Div {
    let column = div().flex_none().h_full();
    match width(cx) {
        Some(width) => column.w(px(width.max(MIN))),
        None => column.w_1_3().max_w(px(DEFAULT_MAX)),
    }
}

/// The border between the column and the rest: the resize cursor, lit
/// under the pointer; dragged, it resizes the column; a double click gives
/// the default back.
pub fn edge(id: &'static str) -> Stateful<Div> {
    div()
        .named(id)
        .w(px(5.))
        .flex_none()
        .h_full()
        .bg(p().border)
        .cursor(CursorStyle::ResizeColumn)
        .hover(|d| d.bg(p().accent))
        .on_drag(SideDrag, |_, _, _, cx| cx.new(|_| NoPreview))
        .on_click(|event: &ClickEvent, _, cx| {
            if event.click_count() == 2 {
                cx.set_global(SideWidth(None));
            }
        })
}

/// The row that holds the column, its border and the rest: the column's
/// width follows the drag, within bounds.
pub fn row() -> Div {
    div().flex().flex_1().min_h_0().on_drag_move(|event: &DragMoveEvent<SideDrag>, _, cx| {
        let bounds = event.bounds;
        let max = (f32::from(bounds.size.width) * 0.6).max(MIN);
        let width = f32::from(event.event.position.x - bounds.origin.x).clamp(MIN, max);
        cx.set_global(SideWidth(Some(width)));
    })
}
