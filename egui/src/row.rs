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

pub fn font(size: f32, weight: Weight) -> FontId {
    fonts::font_id(Role::Ui, weight, size)
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
