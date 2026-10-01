//! A chip of the agent bar and the line its click opens: the choices, said
//! before done, right above the chip — as wide as what they say, its edge
//! on the chip's, so the pointer barely moves. A press anywhere else
//! closes it; on the chip, the chip's own click does (it toggles).

use crate::ui::Named as _;
use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::*;

use crate::theme::p;

/// Which of the chip's edges the line lines up with: the left for a chip
/// at the bar's start, the right for one at its end.
#[derive(Clone, Copy)]
pub enum Edge {
    Left,
    Right,
}

/// The line's look: a row on the surface, framed in `border` (the warning
/// colour for a gesture on a loop, say), sized to its content.
pub fn line(id: &'static str, border: Hsla) -> Stateful<Div> {
    div()
        .named(id)
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
        .border_color(border)
        .shadow_md()
        .text_color(p().text)
}

thread_local! {
    /// Where each anchored chip began, as last painted (by its call site):
    /// the room a line lined up with its right edge has on its left.
    static CHIP_LEFT: std::cell::RefCell<std::collections::HashMap<&'static std::panic::Location<'static>, Pixels>> = Default::default();
}

/// `chip`, and above it `line` when open, painted over what is there; a
/// press outside both calls `close` (a `cx.listener`).
/// A line on the chip's right edge never runs past the window's left: short
/// of room, its choices wrap onto more lines.
#[track_caller]
pub fn anchored(chip: impl IntoElement, line: Option<Stateful<Div>>, edge: Edge, close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static) -> Div {
    // Where the chip is, as painted: a press on it is its own click's.
    let chip_at: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::default();
    let seen = chip_at.clone();
    let site = std::panic::Location::caller();
    let room = CHIP_LEFT.with(|r| r.borrow().get(site).copied());
    div()
        .relative()
        .flex()
        .flex_none()
        .child(chip)
        .child(
            canvas(
                move |bounds, _, _| {
                    seen.set(Some(bounds));
                    CHIP_LEFT.with(|r| r.borrow_mut().insert(site, bounds.left()));
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        )
        .children(line.map(|line| {
            let line = line.on_mouse_down_out(move |event, window, cx| {
                if !chip_at.get().is_some_and(|b| b.contains(&event.position)) {
                    close(event, window, cx);
                }
            });
            let place = div().absolute().bottom_full().mb(px(6.));
            let place = match (edge, room) {
                (Edge::Left, _) => place.left_0().child(line),
                // From the window's edge to the chip's: the line keeps its
                // width there, and wraps only short of it.
                (Edge::Right, Some(room)) => place
                    .right_0()
                    .left(px(8.) - room)
                    .flex()
                    .justify_end()
                    .child(line.flex_shrink(1.).min_w_0().flex_wrap().justify_end()),
                (Edge::Right, None) => place.right_0().child(line),
            };
            deferred(place).with_priority(1)
        }))
}
