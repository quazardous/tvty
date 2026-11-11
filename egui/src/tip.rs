//! "Did you know?": the tip card the GPUI build shows bottom left at
//! start, laid out as it is (380 wide, 12 inside, 8 between its rows), in
//! the box of egui-pop's `tip-card` kit component. ✕, "Got it" and "Turn
//! tips off" close it; "Next tip" shows the next of tvty's tips.

use egui_pop::core::fonts::Weight;
use egui_pop::core::kit::{Kit, State};
use egui_pop::egui::{self, Align2, Color32, pos2, vec2};
use egui_pop::prelude::Tokens;

use crate::row::{From, Row, font};

/// The card's width, border included.
const WIDTH: f32 = 380.0;

/// Text lines as GPUI spaces them: 14 points at its 1.618 line height.
const LINE: f32 = 23.0;

/// Some of tvty's tips (its `tips.ftl`): the key, in bold, and the rest.
const TIPS: [(&str, &str); 5] = [
    ("Ctrl+Tab", " shows your projects as stacks, the most recent first. Release Ctrl to open the one chosen."),
    ("Ctrl+Shift+Space", " shows every terminal live, side by side. Type to filter, arrows to move, Enter to open."),
    ("Ctrl+Shift+J", " jumps to the newest notification: the agent's terminal, its ticket beside it."),
    ("Ctrl+G", " goes to a ticket: type its number, or paste a comment's link."),
    ("Ctrl+Shift+L", " shows every ticket full screen, the critical one at the top."),
];

/// The tip shown, or `None` once the card is closed.
fn current(ctx: &egui::Context) -> Option<usize> {
    ctx.data(|d| d.get_temp::<Option<usize>>(egui::Id::unique("tvty.tip.shown"))).unwrap_or(Some(0))
}

fn set_current(ctx: &egui::Context, tip: Option<usize>) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("tvty.tip.shown"), tip));
}

pub fn show(ctx: &egui::Context) {
    let Some(tip) = current(ctx) else { return };
    let t = Tokens::of(ctx);
    let kit = Kit::of(ctx);
    let violet = t.color("tvty.tip.fill").unwrap_or(t.accent.default);
    let on_violet = t.color("tvty.tip.text").unwrap_or(t.text.on_accent);
    let screen = ctx.content_rect();
    // 12 from the window's start edge, 44 from its bottom, inside its outline.
    let corner = pos2(screen.left() + 13.0, screen.bottom() - 45.0);
    let area = egui::Id::unique("tvty.tip");
    // The sessions lie over the centre in an egui area of the same order
    // (egui_stack's), drawn above the card once shown or clicked: the
    // card goes back on top, as GPUI's lies over the whole window, except
    // under a menu or one of tvty's own overlays.
    let overlay = egui::Popup::is_any_open(ctx)
        || crate::fulllist::is_open(ctx)
        || crate::message::is_open(ctx)
        || crate::settings::is_open(ctx);
    if !overlay {
        ctx.move_to_top(egui::LayerId::new(egui::Order::Foreground, area));
    }
    egui::Area::new(area)
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(corner)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let inner = WIDTH - 2.0 * (kit.tip_card.data.padding_x + 1.0);
            kit.tip_card
                .show(ui, State::REST, egui::Shadow::NONE, |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 8.0);
                    ui.set_width(inner);
                    if head(ui, inner, violet, t.text.muted) {
                        set_current(ui.ctx(), None);
                    }
                    text(ui, inner, TIPS[tip % TIPS.len()], t.text.default, t.text.muted);
                    match actions(ui, inner, violet, on_violet, t.accent.default, t.text.muted) {
                        Some(Action::Close) => set_current(ui.ctx(), None),
                        Some(Action::Next) => set_current(ui.ctx(), Some(tip + 1)),
                        None => {}
                    }
                });
        });
}

/// "Did you know?" in the tips' violet, the close cross at the end:
/// whether the cross was clicked.
fn head(ui: &mut egui::Ui, width: f32, violet: Color32, muted: Color32) -> bool {
    let (rect, _) = ui.allocate_exact_size(vec2(width, LINE), egui::Sense::hover());
    let mut row = Row::new(ui.painter(), rect, 0.0, rect.center().y);
    row.text(From::Start, "Did you know?", font(12.0, Weight::BOLD), violet, 0.0);
    let cross = row.glyph(From::End, "✕", font(14.0, Weight::REGULAR), muted, 0.0);
    clicked(ui, cross.expand(4.0), "close")
}

/// A click on `rect`, the pointer a hand over it.
fn clicked(ui: &mut egui::Ui, rect: egui::Rect, name: &str) -> bool {
    ui.interact(rect, ui.scope_id().with(("tvty.tip", name)), egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

enum Action {
    Close,
    Next,
}

/// The tip: its key in bold, in the text's colour; the rest muted.
fn text(ui: &mut egui::Ui, width: f32, (key, rest): (&str, &str), strong: Color32, muted: Color32) {
    let format = |weight, color| egui::TextFormat {
        font_id: font(14.0, weight),
        color,
        line_height: Some(LINE),
        valign: egui::Align::Center,
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::default();
    job.append(key, 0.0, format(Weight::BOLD, strong));
    job.append(rest, 0.0, format(Weight::REGULAR, muted));
    job.wrap.max_width = width;
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let (rect, _) = ui.allocate_exact_size(vec2(width, galley.size().y), egui::Sense::hover());
    // egui keeps a line's extra height under its text; GPUI centres the
    // text in it.
    let natural = ui.fonts_mut(|f| f.row_height(&font(14.0, Weight::REGULAR)));
    let lift = ((LINE - natural) / 2.0).round();
    ui.painter().galley(rect.min + vec2(0.0, lift), galley, muted);
}

/// "Got it" on the violet, "Next tip", and at the end "Turn tips off":
/// what was clicked.
fn actions(
    ui: &mut egui::Ui,
    width: f32,
    violet: Color32,
    on_violet: Color32,
    link: Color32,
    muted: Color32,
) -> Option<Action> {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 24.0), egui::Sense::hover());
    let painter = ui.painter().clone();
    let painter = &painter;
    let label = painter.layout_no_wrap("Got it".into(), font(14.0, Weight::MEDIUM), on_violet);
    let pill = egui::Rect::from_min_size(rect.min, vec2(label.size().x.ceil() + 24.0, 24.0));
    painter.rect_filled(pill, 6.0, violet);
    painter.text(pill.center(), Align2::CENTER_CENTER, "Got it", font(14.0, Weight::MEDIUM), on_violet);
    let mut row = Row::new(painter, rect, 0.0, rect.center().y);
    row.room(From::Start, pill.width(), 12.0);
    // Links: 4 on each side of their text.
    row.room(From::Start, 4.0, 0.0);
    let next = row.text(From::Start, "Next tip", font(14.0, Weight::REGULAR), link, 0.0);
    row.room(From::End, 4.0, 0.0);
    let off = row.text(From::End, "Turn tips off", font(12.0, Weight::REGULAR), muted, 0.0);
    if clicked(ui, pill, "got-it") || clicked(ui, off.expand(4.0), "off") {
        Some(Action::Close)
    } else if clicked(ui, next.expand(4.0), "next") {
        Some(Action::Next)
    } else {
        None
    }
}
