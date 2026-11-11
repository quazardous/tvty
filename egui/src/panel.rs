//! The tickets panel showing a project's tickets, as the GPUI build lays it
//! out (`panel.rs` there): its header (the project's name and alerts), then
//! the tickets in sections by band, egui-pop's `sections`, each a list of
//! rows 56 high scrolling on its own.

use egui_pop::core::fonts::Weight;
use egui_pop::core::kit::{Kit, KitCx, Part, State};
use egui_pop::core::sections::{Section, Sections};
use egui_pop::egui::{self, Color32, Rect, Sense, pos2, vec2};
use egui_pop::prelude::Tokens;

use crate::board::{self, Alerts, Band, RowState, Ticket, Turn};
use crate::fulllist::{Palette, Pen, SM, XS, line};
use crate::scene::Scene;

/// GPUI's `ROW_HEIGHT`.
const ROW: f32 = 56.0;

/// The project of the terminal shown: the first one tvty keeps open, in
/// the project whose order holds it.
pub fn project(scene: &Scene) -> Option<String> {
    let open = scene.open.first()?;
    scene.order.iter().find(|(_, order)| order.contains(open)).map(|(name, _)| name.clone())
}

/// The project's tickets, open, in their bands.
fn tickets<'a>(scene: &'a Scene, name: &str) -> Vec<&'a Ticket> {
    scene.projects.iter().filter(|p| p.name == name).flat_map(|p| p.tickets.iter()).filter(|t| !t.closed).collect()
}

/// The header's project: its name, bold, then its alerts (the badges).
pub fn header_title(ui: &egui::Ui, scene: &Scene, name: &str, x: f32, mid: f32) {
    let t = Tokens::of(ui.ctx());
    let kit = Kit::of(ui.ctx());
    let p = Palette::of(&t);
    let painter = ui.painter();
    let pen = Pen { painter };
    let cx = KitCx { painter, ppp: ui.ctx().pixels_per_point() };
    let w = pen.text(x, mid - line(SM) / 2.0, name, SM, Weight::BOLD, p.text);
    let rows = tickets(scene, name);
    let alerts = Alerts::of(rows.iter().copied());
    crate::fulllist::draw_badges(&kit, &pen, &cx, pos2(x + w + 8.0, mid), &alerts);
}

/// The sections of the project's tickets, under the header.
pub fn show(ui: &mut egui::Ui, scene: &Scene, name: &str) {
    let t = Tokens::of(ui.ctx());
    let kit = Kit::of(ui.ctx());
    let p = Palette::of(&t);
    let user = scene.user.as_str();
    let rows: Vec<(RowState, &Ticket)> = tickets(scene, name).into_iter().map(|t| (board::state_of(t, user), t)).collect();
    let (kit, p) = (&kit, &p);
    Sections::new("tvty.tickets").grab_titles(true).fill(crate::sections_fill()).show(ui, |stack| {
        for band in Band::ALL.into_iter().filter(|b| *b != Band::Closed) {
            let mine: Vec<(RowState, &Ticket)> = rows.iter().filter(|(s, _)| s.band == band).cloned().collect();
            if mine.is_empty() {
                continue;
            }
            let count = mine.len();
            let section = Section::new(band.key(), band.title()).count(count as u64).row_height(ROW);
            stack.rows(section, count, move |ui, range| {
                for i in range {
                    let (state, ticket) = &mine[i];
                    row(ui, kit, p, ticket, state.clone(), user);
                }
            });
        }
    });
}

/// One ticket: its edge, its glyph, "#N" and its title, then what is said
/// of it and how long ago, 8 from the start and 12 from the end.
fn row(ui: &mut egui::Ui, kit: &Kit, p: &Palette, ticket: &Ticket, state: RowState, user: &str) {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
    if response.clicked() {
        crate::detail::set_shown(ui.ctx(), Some(ticket.id));
    }
    let painter = ui.painter().clone();
    let pen = Pen { painter: &painter };
    let cx = KitCx { painter: &painter, ppp: ui.ctx().pixels_per_point() };
    kit.list_row.paint(&cx, &Part { rect, room: rect, state: State::from_response(&response), label: None });
    // py 6: the content between.
    let (top, bottom) = (rect.top() + 6.0, rect.bottom() - 6.0);
    let x0 = rect.left() + 8.0;
    crate::fulllist::edge(&painter, p, &state, x0, top, bottom);
    let yours = state.turn == Turn::You;
    // The glyph: a column 16 wide, 8 after the edge (3), 2 down.
    let gx = x0 + 3.0 + 8.0;
    if let Some(glyph) = state.glyph {
        if let Some(icon) = crate::icons::of_glyph(glyph) {
            icon.paint(&painter, Rect::from_min_size(pos2(gx, top + 2.0), vec2(16.0, 16.0)), p.glyph(glyph, yours));
        }
    }
    let x = gx + 16.0 + 8.0;
    let right = rect.right() - 12.0;
    // The title's line: "#N" muted, then the title, bold when unread.
    let mut x1 = x;
    x1 += pen.text(x1, top, &format!("#{}", ticket.id), SM, Weight::REGULAR, p.muted) + 8.0;
    let (weight, color) = if ticket.unread { (Weight::BOLD, p.text) } else { (Weight::REGULAR, p.text.gamma_multiply(0.72)) };
    pen.text_max(x1, top, &ticket.title, SM, weight, color, Some(right - x1));
    // The meta line: who spoke last, the comments, the holder, the
    // critical pill, the priority; how long ago at the end. 8 apart.
    let y = top + line(SM) + 2.0;
    let mid = y + line(XS) / 2.0;
    let mut x = x;
    let gap = 8.0;
    if let Some(speaker) = &ticket.last_speaker {
        let who = if speaker == user { "you" } else { speaker.as_str() };
        x += pen.text(x, y, who, XS, Weight::REGULAR, p.muted) + gap;
    }
    if let Some((icon, color, n)) = crate::fulllist::comments(p, ticket, user) {
        if let Some(icon) = icon {
            icon.paint(&painter, Rect::from_min_size(pos2(x, mid - 6.0), vec2(12.0, 12.0)), color);
        }
        x += 12.0 + 4.0;
        x += pen.text(x, y, &n, XS, Weight::REGULAR, color) + gap;
    }
    if let Some(holder) = &ticket.holder {
        if ticket.hot {
            if let Some(icon) = crate::icons::hot() {
                icon.paint(&painter, Rect::from_min_size(pos2(x, mid - 6.0), vec2(12.0, 12.0)), p.warning);
            }
            x += 12.0 + 4.0;
        }
        x += pen.text(x, y, holder, XS, Weight::REGULAR, p.muted) + gap;
    }
    if let Some(critical) = &ticket.critical {
        let n = critical.quiet().map_or(critical.holds.to_string(), |q| format!("{} · {q}", critical.holds));
        let w = 6.0 + 12.0 + 2.0 + pen.width(&n, XS, Weight::REGULAR) + 6.0;
        let pill = Rect::from_min_size(pos2(x, y), vec2(w, line(XS)));
        painter.rect_filled(pill, 2.0, p.danger);
        let ink = Color32::from_rgb(0x14, 0x14, 0x14);
        if let Some(icon) = crate::icons::critical() {
            icon.paint(&painter, Rect::from_min_size(pos2(x + 6.0, mid - 6.0), vec2(12.0, 12.0)), ink);
        }
        pen.text(x + 20.0, y, &n, XS, Weight::REGULAR, ink);
        x = pill.right() + gap;
    }
    if let Some(priority) = ticket.priority.as_deref() {
        if let Some(icon) = crate::icons::of_priority(priority) {
            icon.paint(&painter, Rect::from_min_size(pos2(x, mid - 7.0), vec2(14.0, 14.0)), p.priority(priority));
        }
    }
    if let Some(when) = ticket.last_activity.as_deref().and_then(board::ago) {
        let w = pen.width(&when, XS, Weight::REGULAR);
        pen.text(right - w, y, &when, XS, Weight::REGULAR, p.muted);
    }
}
