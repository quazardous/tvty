//! "Did you know?": the tip card the GPUI build shows bottom left at
//! start, laid out as it is (380 wide, 12 inside, 8 between its rows), in
//! the box of egui-pop's `tip-card` kit component.

use egui_pop::core::fonts::Weight;
use egui_pop::core::kit::{Kit, State};
use egui_pop::egui::{self, Align2, Color32, pos2, vec2};
use egui_pop::prelude::Tokens;

use crate::row::{From, Row, font};

/// The card's width, border included.
const WIDTH: f32 = 380.0;

/// Text lines as GPUI spaces them: 14 points at its 1.618 line height.
const LINE: f32 = 23.0;

pub fn show(ctx: &egui::Context) {
    let t = Tokens::of(ctx);
    let kit = Kit::of(ctx);
    let violet = t.color("tvty.tip.fill").unwrap_or(t.accent.default);
    let on_violet = t.color("tvty.tip.text").unwrap_or(t.text.on_accent);
    let screen = ctx.content_rect();
    // 12 from the window's start edge, 44 from its bottom, inside its outline.
    let corner = pos2(screen.left() + 13.0, screen.bottom() - 45.0);
    egui::Area::new(egui::Id::new("tvty.tip"))
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(corner)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let inner = WIDTH - 2.0 * (kit.tip_card.data.padding_x + 1.0);
            kit.tip_card
                .show(ui, State::REST, egui::Shadow::NONE, |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 8.0);
                    ui.set_width(inner);
                    head(ui, inner, violet, t.text.muted);
                    text(ui, inner, t.text.default, t.text.muted);
                    actions(ui, inner, violet, on_violet, t.accent.default, t.text.muted);
                });
        });
}

/// "Did you know?" in the tips' violet, the close cross at the end.
fn head(ui: &mut egui::Ui, width: f32, violet: Color32, muted: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, LINE), egui::Sense::hover());
    let mut row = Row::new(ui.painter(), rect, 0.0, rect.center().y);
    row.text(From::Start, "Did you know?", font(12.0, Weight::BOLD), violet, 0.0);
    row.glyph(From::End, "✕", font(14.0, Weight::REGULAR), muted, 0.0);
}

/// The tip: its key in bold, in the text's colour; the rest muted.
fn text(ui: &mut egui::Ui, width: f32, strong: Color32, muted: Color32) {
    let format = |weight, color| egui::TextFormat {
        font_id: font(14.0, weight),
        color,
        line_height: Some(LINE),
        valign: egui::Align::Center,
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::default();
    job.append("Ctrl+Tab", 0.0, format(Weight::BOLD, strong));
    job.append(
        " shows your projects as stacks, the most recent first. Release Ctrl to open the one chosen.",
        0.0,
        format(Weight::REGULAR, muted),
    );
    job.wrap.max_width = width;
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let (rect, _) = ui.allocate_exact_size(vec2(width, galley.size().y), egui::Sense::hover());
    // egui keeps a line's extra height under its text; GPUI centres the
    // text in it.
    let natural = ui.fonts_mut(|f| f.row_height(&font(14.0, Weight::REGULAR)));
    let lift = ((LINE - natural) / 2.0).round();
    ui.painter().galley(rect.min + vec2(0.0, lift), galley, muted);
}

/// "Got it" on the violet, "Next tip", and at the end "Turn tips off".
fn actions(
    ui: &mut egui::Ui,
    width: f32,
    violet: Color32,
    on_violet: Color32,
    link: Color32,
    muted: Color32,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 24.0), egui::Sense::hover());
    let painter = ui.painter();
    let label = painter.layout_no_wrap("Got it".into(), font(14.0, Weight::MEDIUM), on_violet);
    let pill = egui::Rect::from_min_size(rect.min, vec2(label.size().x.ceil() + 24.0, 24.0));
    painter.rect_filled(pill, 6.0, violet);
    painter.text(pill.center(), Align2::CENTER_CENTER, "Got it", font(14.0, Weight::MEDIUM), on_violet);
    let mut row = Row::new(painter, rect, 0.0, rect.center().y);
    row.room(From::Start, pill.width(), 12.0);
    // Links: 4 on each side of their text.
    row.room(From::Start, 4.0, 0.0);
    row.text(From::Start, "Next tip", font(14.0, Weight::REGULAR), link, 0.0);
    row.room(From::End, 4.0, 0.0);
    row.text(From::End, "Turn tips off", font(12.0, Weight::REGULAR), muted, 0.0);
}
