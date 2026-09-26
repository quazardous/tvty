//! The window's resize band: a few pixels along each edge the window is free
//! to move, above everything else, where a press resizes the window and does
//! nothing else. The kit's own band is narrower (4 px) and lies under the
//! window's content, whose cursors and clicks win over it: on the left, the
//! folded list's strip took the press to open the list.

use gpui_kit::*;

/// How far the band reaches into the window.
const BAND: f32 = 6.;
/// A corner's side: both edges at once.
const CORNER: f32 = 12.;

/// The band's zones, for a window drawing its own frame; none when the
/// compositor draws it, or on the edges tiled against the screen's.
pub(super) fn resize_band(window: &Window) -> Vec<AnyElement> {
    let Decorations::Client { tiling } = window.window_decorations() else {
        return Vec::new();
    };
    let (top, bottom, left, right) = (!tiling.top, !tiling.bottom, !tiling.left, !tiling.right);
    let mut zones = Vec::new();
    // Edges first, corners after: a corner lies over both its edges.
    if top {
        zones.push(zone(ResizeEdge::Top, div().top_0().left_0().right_0().h(px(BAND))));
    }
    if bottom {
        zones.push(zone(ResizeEdge::Bottom, div().bottom_0().left_0().right_0().h(px(BAND))));
    }
    if left {
        zones.push(zone(ResizeEdge::Left, div().top_0().bottom_0().left_0().w(px(BAND))));
    }
    if right {
        zones.push(zone(ResizeEdge::Right, div().top_0().bottom_0().right_0().w(px(BAND))));
    }
    let corner = || div().size(px(CORNER));
    if top && left {
        zones.push(zone(ResizeEdge::TopLeft, corner().top_0().left_0()));
    }
    if top && right {
        zones.push(zone(ResizeEdge::TopRight, corner().top_0().right_0()));
    }
    if bottom && left {
        zones.push(zone(ResizeEdge::BottomLeft, corner().bottom_0().left_0()));
    }
    if bottom && right {
        zones.push(zone(ResizeEdge::BottomRight, corner().bottom_0().right_0()));
    }
    zones
}

fn zone(edge: ResizeEdge, place: Div) -> AnyElement {
    let cursor = match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    };
    place
        .absolute()
        .occlude()
        .cursor(cursor)
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            window.start_window_resize(edge);
        })
        .into_any_element()
}
