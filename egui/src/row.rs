//! A row of items laid out as the GPUI build's flex rows: each item placed
//! from the row's start or from its end, its margins given, centred on the
//! row's middle. Painted only: the experiment compares looks.

use egui_pop::core::fonts::{self, Role, Weight};
use egui_pop::core::icon::Icon;
use egui_pop::egui::{Align2, Color32, FontId, Painter, Rect, pos2, vec2};

pub struct Row<'a> {
    painter: &'a Painter,
    mid: f32,
    /// The next item's start edge, from the start.
    start: f32,
    /// The next item's end edge, from the end.
    end: f32,
}

/// Where an item goes.
#[derive(Clone, Copy)]
pub enum From {
    Start,
    End,
}

/// The font GPUI draws `weight` with. On Linux GPUI draws a variable
/// face (Adwaita Sans, GNOME's) at an instance of its own whatever the
/// weight asked: its bold comes out near 500 (it came out regular with
/// the GPUI of October 2026). egui-pop sets the face's `wght` axis, so the
/// weight is capped here to draw what GPUI draws: `TVTY_EGUI_WEIGHTS`
/// sets the cap (400 for that older GPUI), `all` for a static face
/// (DejaVu Sans) where GPUI's bold is bold.
pub fn font(size: f32, weight: Weight) -> FontId {
    static CAP: std::sync::OnceLock<u16> = std::sync::OnceLock::new();
    let cap = *CAP.get_or_init(|| match std::env::var("TVTY_EGUI_WEIGHTS") {
        Ok(v) => v.parse().unwrap_or(900),
        Err(_) => 500,
    });
    fonts::font_id(Role::Ui, Weight(weight.0.min(cap)), size)
}

impl<'a> Row<'a> {
    /// The row inside `rect`, `pad` from its sides, items centred on `mid`.
    pub fn new(painter: &'a Painter, rect: Rect, pad: f32, mid: f32) -> Self {
        Row {
            painter,
            mid,
            start: rect.left() + pad,
            end: rect.right() - pad,
        }
    }

    /// Where the next item from the start goes.
    pub fn start(&self) -> f32 {
        self.start
    }

    pub fn mid(&self) -> f32 {
        self.mid
    }

    /// Room `width` wide with `margin` on its end side (GPUI's `mr_*`):
    /// the item's rect, 20 high.
    pub fn room(&mut self, from: From, width: f32, margin: f32) -> Rect {
        let (left, right) = match from {
            From::Start => {
                let l = self.start;
                self.start += width + margin;
                (l, l + width)
            }
            From::End => {
                let r = self.end - margin;
                self.end = r - width;
                (r - width, r)
            }
        };
        Rect::from_min_max(pos2(left, self.mid - 10.0), pos2(right, self.mid + 10.0))
    }

    /// Text as it is.
    pub fn text(&mut self, from: From, s: &str, font: FontId, color: Color32, margin: f32) -> Rect {
        let galley = self.painter.layout_no_wrap(s.into(), font, color);
        let r = self.room(from, galley.size().x.ceil(), margin);
        self.painter.galley(pos2(r.left(), self.mid - galley.size().y / 2.0), galley, color);
        r
    }

    /// A glyph button: at least 20 wide, 4 on each side of its glyph.
    pub fn glyph(&mut self, from: From, s: &str, font: FontId, color: Color32, margin: f32) -> Rect {
        // GPUI lays text out in whole pixels.
        let width = self.painter.layout_no_wrap(s.into(), font.clone(), color).size().x.ceil();
        let r = self.room(from, (width + 8.0).max(20.0), margin);
        self.painter.text(pos2(r.center().x, self.mid), Align2::CENTER_CENTER, s, font, color);
        r
    }

    /// An icon button: a 15 point icon, 4 on each side.
    pub fn icon(&mut self, from: From, icon: Option<Icon>, color: Color32, margin: f32) -> Rect {
        let r = self.room(from, 23.0, margin);
        if let Some(i) = icon {
            let at = Rect::from_center_size(pos2(r.center().x, self.mid), vec2(15.0, 15.0));
            i.paint(self.painter, at, color);
        }
        r
    }
}

/// A button drawn by hand: its hover fill goes in the place kept for it
/// under its glyph (`slot`, an `egui::Shape::Noop` added before), and a
/// hand over it. Its response.
pub fn button(ui: &egui_pop::egui::Ui, painter: &Painter, slot: egui_pop::egui::layers::ShapeIdx, rect: Rect, name: &str) -> egui_pop::egui::Response {
    use egui_pop::egui;
    let response = ui
        .interact(rect, ui.scope_id().with(("tvty.button", name)), egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if response.hovered() {
        let fill = egui_pop::prelude::Tokens::of(ui.ctx()).surface.hover;
        painter.set(slot, egui::Shape::rect_filled(rect, 4.0, fill));
    }
    response
}
