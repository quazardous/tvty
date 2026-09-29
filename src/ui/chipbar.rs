//! A chip of the agent bar and the line its click opens: the choices, said
//! before done, right above the chip — as wide as what they say, its edge
//! on the chip's, so the pointer barely moves.

use gpui_kit::*;

use crate::theme::p;

/// Which of the chip's edges the line lines up with: the left for a chip
/// at the bar's start, the right for one at its end.
#[derive(Clone, Copy)]
pub enum Edge {
    Left,
    Right,
}

/// The line's look: a row on the surface, framed in the warning colour (a
/// gesture on a loop), sized to its content.
pub fn line(id: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .occlude()
        .flex()
        .flex_none()
        .items_center()
        .gap_2()
        .whitespace_nowrap()
        .px_2()
        .py_1p5()
        .rounded_md()
        .bg(p().surface)
        .border_1()
        .border_color(p().warning)
        .shadow_md()
        .text_color(p().text)
}

/// `chip`, and above it `line` when open, painted over what is there.
pub fn anchored(chip: impl IntoElement, line: Option<Stateful<Div>>, edge: Edge) -> Div {
    div().relative().flex().flex_none().child(chip).children(line.map(|line| {
        let place = div().absolute().bottom_full().mb(px(6.));
        let place = match edge {
            Edge::Left => place.left_0(),
            Edge::Right => place.right_0(),
        };
        deferred(place.child(line)).with_priority(1)
    }))
}
