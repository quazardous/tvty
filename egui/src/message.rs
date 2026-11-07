//! "Message to every agent": the popover the title bar's megaphone opens,
//! as the GPUI build lays it out (482 wide, under the title bar, at the
//! window's end): what it does, the loops it reaches, the text, then
//! Release holds, Send and Send & hold. In the kit's `popover`, `field`
//! and `button`, set to gpui-component's by the theme.

use egui_pop::core::fonts::Weight;
use egui_pop::core::kit::{Kit, KitCx, Part, State, Tone};
use egui_pop::egui::{self, Align2, Rect, pos2, vec2};
use egui_pop::prelude::Tokens;

use crate::fulllist::{Palette, Pen, SM, XS, line};

const OPEN: &str = "tvty.message.open";
/// The popover's width, its border included.
const WIDTH: f32 = 482.0;
/// The text sent when the field is left empty.
const DEFAULT: &str = "I'll be back. Until then, stabilise: finish or park your current step, commit what is done, and wait.";

pub fn is_open(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::new(OPEN))).unwrap_or(false)
}

pub fn toggle(ctx: &egui::Context) {
    let open = is_open(ctx);
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(OPEN), !open));
}

fn close(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(OPEN), false));
}

/// Shows the popover while it is open; Escape or a click outside closes it.
pub fn show(ctx: &egui::Context) {
    if !is_open(ctx) {
        return;
    }
    let t = Tokens::of(ctx);
    let kit = Kit::of(ctx);
    let p = Palette::of(&t);
    let screen = ctx.content_rect();
    // 9 from the window's end, its top line 61 down: under the title bar.
    let at = pos2(screen.right() - 9.0, screen.top() + 61.0);
    let area = egui::Area::new(egui::Id::new("tvty.message"))
        .pivot(Align2::RIGHT_TOP)
        .fixed_pos(at)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let pad = kit.popover.data.padding_x;
            let inner = WIDTH - 2.0 * (pad + 1.0);
            kit.popover.show(ui, State::REST, egui::Shadow::NONE, |ui| {
                ui.set_width(inner);
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                let painter = ui.painter().clone();
                let pen = Pen { painter: &painter };
                let cx = KitCx { painter: &painter, ppp: ui.ctx().pixels_per_point() };
                let left = ui.cursor().left();
                let mut y = ui.cursor().top();
                pen.text(left, y, "Message to every agent", SM, Weight::REGULAR, p.text);
                y += line(SM) + 4.0;
                // The explanation, wrapped, 19 apart as GPUI's small text.
                let said = "Typed into each running agent session now, whatever it is doing. Send & hold also holds every loop (not AFK ∞): no wake starts new work until you release them. Left empty, the text shown is sent.";
                let galley = painter.layout(said.into(), crate::row::font(XS, Weight::REGULAR), p.muted, inner);
                for (i, row) in galley.rows.iter().enumerate() {
                    let text: String = row.row.glyphs.iter().map(|g| g.chr).collect();
                    pen.text(left, y + i as f32 * line(XS), text.trim_end(), XS, Weight::REGULAR, p.muted);
                }
                y += galley.rows.len() as f32 * line(XS) + 6.0;
                pen.text(left, y, "No agent loop is running.", XS, Weight::REGULAR, p.muted);
                y += line(XS) + 8.0;
                // The text: a field 78 high, focused, its first line shown.
                let field = Rect::from_min_size(pos2(left - 1.0, y), vec2(inner + 2.0, 78.0));
                let id = ui.id().with("text");
                let mut text = ui.data(|d| d.get_temp::<String>(id)).unwrap_or_default();
                kit.field.paint(&cx, &Part { rect: field, room: field, state: State::FOCUS, label: None });
                let edit = egui::TextEdit::multiline(&mut text)
                    .frame(egui::Frame::NONE)
                    .hint_text(egui::RichText::new(DEFAULT).color(p.muted))
                    .font(crate::row::font(SM, Weight::REGULAR))
                    .text_color(p.text)
                    .margin(vec2(10.0, 8.0));
                let response = ui.put(field, edit);
                if !response.has_focus() && !ui.ctx().memory(|m| m.focused().is_some()) {
                    response.request_focus();
                }
                ui.data_mut(|d| d.insert_temp(id, text));
                y = field.bottom() + 6.0;
                // The buttons, from the end: Send & hold in the danger
                // tone, Send, Release holds; 8 apart, 24 high.
                let mut end = left + inner;
                let mut clicked = false;
                for (label, tone) in [("Send & hold", Some(Tone::Danger)), ("Send", None), ("Release holds", None)] {
                    let w = pen.width(label, SM, Weight::REGULAR) + 16.0;
                    let r = Rect::from_min_max(pos2(end - w, y), pos2(end, y + 24.0));
                    let hit = ui.interact(r, ui.id().with(label), egui::Sense::click());
                    let mut state = State::from_response(&hit);
                    if let Some(tone) = tone {
                        state = state.toned(tone);
                    }
                    kit.button.paint(&cx, &Part { rect: r, room: r, state, label: None });
                    let ink = kit.button.style(state).text;
                    pen.text(r.left() + 8.0, r.center().y - line(SM) / 2.0, label, SM, Weight::REGULAR, ink);
                    clicked |= hit.clicked();
                    end = r.left() - 8.0;
                }
                ui.allocate_rect(Rect::from_min_max(pos2(left, ui.cursor().top()), pos2(left + inner, y + 24.0)), egui::Sense::hover());
                if clicked {
                    close(ui.ctx());
                }
            });
        });
    // Escape, or a click outside, closes it; not the megaphone's own click,
    // which toggles it.
    let toggled = ctx.data(|d| d.get_temp::<bool>(egui::Id::new("tvty.message.toggled"))).unwrap_or(false);
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) || (area.response.clicked_elsewhere() && !toggled) {
        close(ctx);
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("tvty.message.toggled"), false));
}

/// The megaphone was clicked this pass: its own press must not close what
/// it opens.
pub fn toggled(ctx: &egui::Context) {
    toggle(ctx);
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("tvty.message.toggled"), true));
}
