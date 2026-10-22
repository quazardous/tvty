//! The ticket list, full screen, as the GPUI build lays it out
//! (`src/fulllist.rs`): over the window under its title bar, a header
//! (the title, the badges, how many shown, "+ New ticket", "✕ Esc"),
//! the side column (projects, bands, sort, filters) and the rows.
//!
//! Its boxes are egui-pop's kit components (`indicator-badge`,
//! `selectable-row`, `toggle-chip`, `search-field`, `list-row`), their
//! looks from tvty's theme; the text, laid out as GPUI does (its line box
//! 1.618 times the size in whole pixels, the text centred in it), is
//! painted.

use egui_pop::core::fonts::Weight;
use egui_pop::core::kit::{Kit, KitCx, Part, State, Tone};
use egui_pop::egui::{self, Color32, Pos2, Rect, Sense, pos2, vec2};
use egui_pop::prelude::Tokens;

use crate::board::{self, Alerts, Band, Glyph, RowState, Stripe, Ticket, Turn};
use crate::row::font;
use crate::scene::Scene;

/// GPUI's sizes: `text_xs`, `text_sm`, the window's text, `text_lg`.
pub(crate) const XS: f32 = 12.0;
pub(crate) const SM: f32 = 14.0;
pub(crate) const BASE: f32 = 16.0;
const LG: f32 = 18.0;

/// GPUI's line box: 1.618 times the text's size, in whole pixels.
pub(crate) fn line(size: f32) -> f32 {
    (size * 1.618).round()
}

/// The side column's width: what the test layout keeps (`fields_width`).
const SIDE: f32 = 279.0;

const OPEN: &str = "tvty.full-list.open";

/// Open until closed; at start, when `TVTY_EGUI_FULL` is set (to compare).
pub fn is_open(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::new(OPEN)))
        .unwrap_or_else(|| std::env::var_os("TVTY_EGUI_FULL").is_some())
}

pub fn set_open(ctx: &egui::Context, open: bool) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(OPEN), open));
}

/// The colours of the GPUI build's palette the list draws with.
pub(crate) struct Palette {
    pub(crate) bg: Color32,
    pub(crate) border: Color32,
    pub(crate) text: Color32,
    pub(crate) muted: Color32,
    pub(crate) accent: Color32,
    pub(crate) danger: Color32,
    pub(crate) warning: Color32,
    pub(crate) success: Color32,
    pub(crate) info: Color32,
}

impl Palette {
    pub(crate) fn of(t: &Tokens) -> Self {
        let c = |name: &str, or: Color32| t.color(name).unwrap_or(or);
        Palette {
            bg: t.surface.canvas,
            border: t.border.default,
            text: t.text.default,
            muted: t.text.muted,
            accent: t.accent.default,
            danger: c("danger", Color32::from_rgb(0xf8, 0x71, 0x71)),
            warning: c("warning", Color32::from_rgb(0xfa, 0xcc, 0x15)),
            success: c("success", Color32::from_rgb(0x4a, 0xde, 0x80)),
            info: c("info", Color32::from_rgb(0x22, 0xd3, 0xee)),
        }
    }

    /// A glyph's colour: a rejection for everyone, a step the agent's own
    /// marker; the rest coloured only when it is the reader's turn.
    fn glyph(&self, glyph: Glyph, yours: bool) -> Color32 {
        match glyph {
            // A rejection is news for everyone.
            Glyph::Rejected => return self.danger,
            Glyph::Step => return self.accent,
            Glyph::StalledStep => return self.warning,
            _ => {}
        }
        if !yours {
            return self.muted;
        }
        match glyph {
            Glyph::Escalation | Glyph::Rejected => self.danger,
            Glyph::Plan | Glyph::StalledStep => self.warning,
            Glyph::Resolution => self.success,
            Glyph::Step => self.accent,
            Glyph::Wontfix | Glyph::ClosedResolved | Glyph::Closed => self.muted,
        }
    }

    fn stripe(&self, state: &RowState) -> Color32 {
        state
            .glyph
            .filter(|_| state.stripe != Stripe::Neutral)
            .map_or(self.border, |g| self.glyph(g, state.turn == Turn::You))
    }

    fn priority(&self, priority: &str) -> Color32 {
        match priority {
            "urgent" => self.danger,
            "high" => self.warning,
            _ => self.info,
        }
    }
}

/// Text in a line box: `size` high text centred in `line(size)`.
pub(crate) struct Pen<'a> {
    pub(crate) painter: &'a egui::Painter,
}

impl Pen<'_> {
    pub(crate) fn galley(&self, s: &str, size: f32, weight: Weight, color: Color32, max: Option<f32>) -> std::sync::Arc<egui::Galley> {
        let mut job = egui::text::LayoutJob::simple_singleline(s.to_string(), font(size, weight), color);
        if let Some(width) = max {
            job.wrap = egui::text::TextWrapping::truncate_at_width(width.max(0.0));
        }
        self.painter.layout_job(job)
    }

    /// The text from `x`, in the line box from `top`; its width.
    pub(crate) fn text(&self, x: f32, top: f32, s: &str, size: f32, weight: Weight, color: Color32) -> f32 {
        self.text_max(x, top, s, size, weight, color, None)
    }

    pub(crate) fn text_max(&self, x: f32, top: f32, s: &str, size: f32, weight: Weight, color: Color32, max: Option<f32>) -> f32 {
        let g = self.galley(s, size, weight, color, max);
        let w = g.size().x;
        // GPUI's line box starts on a whole pixel: a box centred on a
        // half one goes up.
        let y = top.floor() + (line(size) - g.size().y) / 2.0;
        self.painter.galley(pos2(x, y), g, color);
        w
    }

    pub(crate) fn width(&self, s: &str, size: f32, weight: Weight) -> f32 {
        self.galley(s, size, weight, Color32::WHITE, None).size().x
    }
}

/// Shows the list while it is open.
pub fn show(ctx: &egui::Context, scene: &Scene) {
    if !is_open(ctx) {
        return;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        set_open(ctx, false);
        return;
    }
    let screen = ctx.content_rect();
    // Inside the window's outline, under the title bar and its line.
    let rect = Rect::from_min_max(pos2(screen.left() + 1.0, screen.top() + 41.0), pos2(screen.right() - 1.0, screen.bottom() - 1.0));
    egui::Area::new(egui::Id::new("tvty.full-list"))
        .fixed_pos(rect.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.set_clip_rect(rect);
            // The list takes the pointer: nothing under it reacts.
            ui.allocate_exact_size(rect.size(), Sense::click());
            draw(ui, rect, scene);
        });
}

fn draw(ui: &mut egui::Ui, rect: Rect, scene: &Scene) {
    let t = Tokens::of(ui.ctx());
    let kit = Kit::of(ui.ctx());
    let p = Palette::of(&t);
    let painter = ui.painter().clone();
    painter.rect_filled(rect, 0.0, p.bg);
    let pen = Pen { painter: &painter };
    let user = scene.user.as_str();

    // The open tickets, the most recent activity first.
    let mut rows: Vec<(RowState, &Ticket)> = scene
        .projects
        .iter()
        .flat_map(|p| p.tickets.iter())
        .filter(|t| !t.closed)
        .map(|t| (board::state_of(t, user), t))
        .collect();
    rows.sort_by(|(_, a), (_, b)| b.last_activity.cmp(&a.last_activity));
    let criticals: Vec<&Ticket> = rows.iter().map(|(_, t)| *t).filter(|t| t.critical.is_some()).collect();
    let alerts = Alerts::of(rows.iter().map(|(_, t)| *t));

    // ── The header: 40 high, 16 from the sides, a line under it.
    let header = Rect::from_min_size(rect.min, vec2(rect.width(), 40.0));
    painter.hline(header.x_range(), header.bottom() + 0.5, egui::Stroke::new(1.0, p.border));
    let mid = header.center().y;
    let mut x = header.left() + 16.0;
    let title = "Tickets — all projects";
    x += pen.text(x, mid - line(LG) / 2.0, title, LG, Weight::BOLD, p.text);
    let cx = KitCx { painter: &painter, ppp: ui.ctx().pixels_per_point() };
    // GPUI's badges there sit a pixel under the header's middle.
    draw_badges(&kit, &pen, &cx, pos2(x + 12.0, mid + 1.0), &alerts);
    // From the end: "✕  Esc", a separator, "+ New ticket", how many shown.
    let mut end = header.right() - 16.0;
    let link = |end: &mut f32, s: &str, size: f32, color: Color32| {
        let w = pen.width(s, size, Weight::REGULAR);
        *end -= w + 4.0;
        pen.text(*end, mid - line(size) / 2.0, s, size, Weight::REGULAR, color);
        *end -= 4.0;
    };
    link(&mut end, "✕  Esc", SM, p.accent);
    end -= 4.0;
    painter.vline(end - 0.5, egui::Rangef::new(mid - 6.0, mid + 6.0), egui::Stroke::new(1.0, p.border));
    end -= 1.0 + 4.0;
    link(&mut end, "+ New ticket", SM, p.accent);
    let shown = format!("{} shown", rows.len());
    let w = pen.width(&shown, XS, Weight::REGULAR);
    end -= 16.0 + w;
    pen.text(end, mid - line(XS) / 2.0, &shown, XS, Weight::REGULAR, p.muted);

    // ── The side column, its 5 wide edge, the list.
    let body = Rect::from_min_max(pos2(rect.left(), header.bottom() + 1.0), rect.max);
    let side = Rect::from_min_size(body.min, vec2(SIDE, body.height()));
    painter.rect_filled(Rect::from_min_size(pos2(side.right(), body.top()), vec2(5.0, body.height())), 0.0, p.border);
    side_column(ui, &kit, &pen, &p, side, &rows, &alerts);
    let list = Rect::from_min_max(pos2(side.right() + 5.0, body.top()), body.max);
    // The rows scroll; the scroll bar is egui-pop's (egui's own style:
    // the kit has no scroll bar component yet).
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(list));
    egui::ScrollArea::vertical().id_salt("tvty.full-list.rows").auto_shrink([false, false]).show(&mut child, |ui| {
        let painter = ui.painter().clone();
        let pen = Pen { painter: &painter };
        let area = Rect::from_min_size(ui.cursor().min, vec2(list.width(), 0.0));
        let mut y = area.top();
        if !criticals.is_empty() {
            y = group(&pen, &p, area.left() + 16.0, y, "Critical — what holds the most back");
            for ticket in &criticals {
                y = row(ui, &kit, &pen, &p, area, y, ticket, board::state_of(ticket, user), user);
            }
            y = group(&pen, &p, area.left() + 16.0, y, "All");
        }
        for (state, ticket) in &rows {
            y = row(ui, &kit, &pen, &p, area, y, ticket, *state, user);
        }
        // GPUI's 16 under the last row.
        ui.allocate_rect(Rect::from_min_max(area.min, pos2(area.right(), y + 16.0)), Sense::hover());
    });
}

/// A group's title: 12 above, 4 under, small, bold, muted, in capitals.
fn group(pen: &Pen<'_>, p: &Palette, x: f32, top: f32, title: &str) -> f32 {
    let top = top + 12.0;
    pen.text(x, top, &title.to_uppercase(), XS, Weight::BOLD, p.muted);
    top + line(XS) + 4.0
}

fn side_column(ui: &mut egui::Ui, kit: &Kit, pen: &Pen<'_>, p: &Palette, side: Rect, rows: &[(RowState, &Ticket)], alerts: &Alerts) {
    let inner = side.shrink(12.0);
    let mut y = side.top() + 12.0;
    let cx = KitCx { painter: pen.painter, ppp: ui.ctx().pixels_per_point() };
    // A choice: 8 from its sides, 4 above and under its text, 2 apart.
    let choice = |ui: &mut egui::Ui, y: &mut f32, label: &str, chosen: bool, trailing: &dyn Fn(f32, f32) -> f32| {
        let h = line(SM) + 8.0;
        let rect = Rect::from_min_size(pos2(inner.left(), *y), vec2(inner.width(), h));
        let response = ui.interact(rect, ui.id().with(("choice", label)), Sense::click());
        let state = State::from_response(&response).with(State::SELECTED, chosen);
        kit.selectable_row.paint(&cx, &Part { rect, room: rect, state, label: None });
        let text = kit.selectable_row.style(state).text;
        pen.text(rect.left() + 8.0, rect.top() + 4.0, label, SM, Weight::REGULAR, text);
        trailing(rect.right() - 8.0, rect.center().y);
        *y += h + 2.0;
    };
    y = group(pen, p, inner.left(), y, "Projects") + 2.0;
    let projects: Vec<&str> = {
        let mut v: Vec<&str> = rows.iter().map(|(_, t)| t.project.as_str()).collect();
        v.sort();
        v.dedup();
        v
    };
    choice(ui, &mut y, "All projects", true, &|end, mid| {
        let w = badges_width(kit, pen, alerts);
        draw_badges(kit, pen, &cx, pos2(end - w, mid), alerts)
    });
    for project in &projects {
        let a = Alerts::of(rows.iter().map(|(_, t)| *t).filter(|t| t.project == *project));
        choice(ui, &mut y, project, false, &|end, mid| {
            let w = badges_width(kit, pen, &a);
            draw_badges(kit, pen, &cx, pos2(end - w, mid), &a)
        });
    }
    y = group(pen, p, inner.left(), y, "Bands") + 2.0;
    for band in Band::ALL {
        if band == Band::Closed {
            continue;
        }
        let n = rows.iter().filter(|(s, _)| s.band == band).count().to_string();
        choice(ui, &mut y, band.title(), false, &|end, mid| {
            let w = pen.width(&n, XS, Weight::REGULAR);
            pen.text(end - w, mid - line(XS) / 2.0, &n, XS, Weight::REGULAR, p.muted)
        });
    }
    y = group(pen, p, inner.left(), y, "Sort") + 2.0;
    for (i, sort) in ["Last activity", "Whose turn (bands)", "Priority", "Created", "Number"].iter().enumerate() {
        let arrow = if i == 0 { "↓" } else { "" };
        choice(ui, &mut y, sort, i == 0, &|end, mid| {
            let w = pen.width(arrow, XS, Weight::REGULAR);
            pen.text(end - w, mid - line(XS) / 2.0, arrow, XS, Weight::REGULAR, p.muted)
        });
    }
    y = group(pen, p, inner.left(), y, "Filters") + 2.0;
    // The filters: chips 4 apart, small text, 6 from their sides.
    let mut x = inner.left();
    let h = line(XS) + 4.0 + 2.0;
    for (label, on) in [("Open", true), ("All", false), ("Unread", false)] {
        let w = pen.width(label, XS, Weight::REGULAR) + 12.0 + 2.0;
        let rect = Rect::from_min_size(pos2(x, y), vec2(w, h));
        let response = ui.interact(rect, ui.id().with(("filter", label)), Sense::click());
        let state = State::from_response(&response).with(State::SELECTED, on);
        kit.toggle_chip.paint(&cx, &Part { rect, room: rect, state, label: None });
        let color = if on { kit.toggle_chip.style(state).text } else { p.muted };
        pen.text(rect.left() + 7.0, rect.top() + 3.0, label, XS, Weight::REGULAR, color);
        x = rect.right() + 4.0;
    }
    // The field, 4 under them.
    y += h + 2.0 + 4.0;
    // The search field: 32 high, the width of the column.
    let rect = Rect::from_min_size(pos2(inner.left(), y), vec2(inner.width(), 32.0));
    let response = ui.interact(rect, ui.id().with("search"), Sense::click());
    let state = State::from_response(&response);
    kit.search_field.paint(&cx, &Part { rect, room: rect, state, label: None });
    pen.text(rect.left() + 12.0, rect.center().y - line(SM) / 2.0, "Search titles…", SM, Weight::REGULAR, p.muted);
}

pub(crate) fn badges_width(kit: &Kit, pen: &Pen<'_>, alerts: &Alerts) -> f32 {
    let mut w = 0.0;
    let mut n = 0;
    for (on, text) in [(alerts.critical, "!".to_string()), (alerts.decisions > 0, alerts.decisions.to_string()), (alerts.unread > 0, alerts.unread.to_string())] {
        if on {
            let g = pen.galley(&text, XS, Weight::REGULAR, Color32::PLACEHOLDER, None);
            w += kit.indicator_badge.measure(g.size()).x;
            n += 1;
        }
    }
    w + 4.0 * (n.max(1) - 1) as f32
}

/// The badges of what tickets ask of the user, from `at` (its x, the
/// line's middle), 4 apart: the critical "!", the decisions, the unread.
pub(crate) fn draw_badges(kit: &Kit, pen: &Pen<'_>, cx: &KitCx<'_>, at: Pos2, alerts: &Alerts) -> f32 {
    let mut x = at.x;
    let items = [
        (alerts.critical, "!".to_string(), Tone::Danger),
        (alerts.decisions > 0, alerts.decisions.to_string(), Tone::Warning),
        (alerts.unread > 0, alerts.unread.to_string(), Tone::Accent),
    ];
    let mut first = true;
    for (on, text, tone) in items {
        if !on {
            continue;
        }
        if !first {
            x += 4.0;
        }
        first = false;
        let galley = pen.galley(&text, XS, Weight::REGULAR, Color32::PLACEHOLDER, None);
        let size = kit.indicator_badge.measure(galley.size());
        // On a whole pixel, as GPUI places a box centred on a half one.
        let rect = Rect::from_min_size(pos2(x, (at.y - size.y / 2.0).floor()), size);
        kit.indicator_badge.paint_part(cx, &Part { rect, room: rect, state: State::REST.toned(tone), label: Some(&galley) });
        x = rect.right();
    }
    x
}

/// A ticket's row: 12 from the sides, 8 above and under, a line under it
/// at half the border's strength; the stripe, the glyph's column (18),
/// then the title's line, the snippet and the facts, 2 apart.
#[allow(clippy::too_many_arguments)]
fn row(ui: &mut egui::Ui, kit: &Kit, pen: &Pen<'_>, p: &Palette, list: Rect, top: f32, ticket: &Ticket, state: RowState, user: &str) -> f32 {
    let snippet = ticket.snippet.as_deref().map(|s| s.replace('\n', " ")).filter(|s| !s.trim().is_empty());
    let content = line(BASE) + snippet.as_ref().map_or(0.0, |_| 2.0 + line(XS)) + 2.0 + line(XS);
    let rect = Rect::from_min_size(pos2(list.left(), top), vec2(list.width(), 8.0 + content + 8.0 + 1.0));
    // Its place in the list too: the critical ticket is listed twice.
    let response = ui.interact(rect, ui.id().with(("row", &ticket.project, ticket.id, top as i32)), Sense::click());
    let cx = KitCx { painter: pen.painter, ppp: ui.ctx().pixels_per_point() };
    let body = Rect::from_min_max(rect.min, pos2(rect.right(), rect.bottom() - 1.0));
    kit.list_row.paint(&cx, &Part { rect: body, room: body, state: State::from_response(&response), label: None });
    pen.painter.hline(rect.x_range(), rect.bottom() - 0.5, egui::Stroke::new(1.0, p.border.gamma_multiply(0.5)));

    let x0 = rect.left() + 12.0;
    let (ctop, cbottom) = (top + 8.0, top + 8.0 + content);
    edge(pen.painter, p, &state, x0, ctop, cbottom);
    let yours = state.turn == Turn::You;
    if let Some(glyph) = state.glyph {
        let at = Rect::from_min_size(pos2(x0 + 3.0 + 8.0, ctop + 2.0), vec2(18.0, 18.0));
        if let Some(icon) = crate::icons::of_glyph(glyph) {
            icon.paint(pen.painter, at, p.glyph(glyph, yours));
        }
    }
    let x = x0 + 3.0 + 8.0 + 18.0 + 8.0;
    let right = rect.right() - 12.0;

    // The title's line: the number, the project, the title, then at the
    // end the priority, the comments and how long ago.
    let mut end = right;
    let mid = ctop + line(BASE) / 2.0;
    if let Some(when) = ticket.last_activity.as_deref().and_then(board::ago) {
        let w = pen.width(&when, XS, Weight::REGULAR);
        end -= w;
        pen.text(end, mid - line(XS) / 2.0, &when, XS, Weight::REGULAR, p.muted);
        end -= 8.0;
    }
    if let Some((icon, color, n)) = comments(p, ticket, user) {
        let w = pen.width(&n, XS, Weight::REGULAR);
        end -= w;
        pen.text(end, mid - line(XS) / 2.0, &n, XS, Weight::REGULAR, color);
        end -= 4.0 + 12.0;
        if let Some(icon) = icon {
            icon.paint(pen.painter, Rect::from_min_size(pos2(end, mid - 6.0), vec2(12.0, 12.0)), color);
        }
        end -= 8.0;
    }
    if let Some(priority) = ticket.priority.as_deref().filter(|p| *p != "normal") {
        let w = pen.width(priority, XS, Weight::REGULAR);
        end -= w;
        pen.text(end, mid - line(XS) / 2.0, priority, XS, Weight::REGULAR, p.muted);
        end -= 4.0 + 14.0;
        if let Some(icon) = crate::icons::of_priority(priority) {
            icon.paint(pen.painter, Rect::from_min_size(pos2(end, mid - 7.0), vec2(14.0, 14.0)), p.priority(priority));
        }
        end -= 8.0;
    }
    let mut x1 = x;
    x1 += pen.text(x1, ctop, &format!("#{}", ticket.id), BASE, Weight::REGULAR, p.muted) + 8.0;
    x1 += pen.text(x1, mid - line(XS) / 2.0, &ticket.project, XS, Weight::REGULAR, p.accent) + 8.0;
    let (weight, color) = if ticket.unread { (Weight::BOLD, p.text) } else { (Weight::REGULAR, p.text.gamma_multiply(0.72)) };
    pen.text_max(x1, ctop, &ticket.title, BASE, weight, color, Some(end - x1));

    let mut y = ctop + line(BASE) + 2.0;
    if let Some(snippet) = &snippet {
        pen.text_max(x, y, snippet, XS, Weight::REGULAR, p.muted, Some(right - x));
        y += line(XS) + 2.0;
    }
    facts(pen, p, x, y, right, ticket, &state, user);
    rect.bottom()
}

/// The row's left edge: the construction tape of a ticket to moderate,
/// else whose turn it is.
fn edge(painter: &egui::Painter, p: &Palette, state: &RowState, x: f32, top: f32, bottom: f32) {
    let rect = Rect::from_min_max(pos2(x, top), pos2(x + 3.0, bottom));
    if state.band == Band::Moderate {
        hazard(painter, Rect::from_min_max(rect.min, pos2(x + 5.0, bottom)));
        return;
    }
    let color = p.stripe(state);
    match state.stripe {
        Stripe::Solid | Stripe::Neutral => {
            painter.rect_filled(rect, 2.0, color);
        }
        Stripe::Dashed => dashes(painter, x + 1.5, top, bottom, 3.0, color),
        Stripe::Waiting => dashes(painter, x + 0.5, top, bottom, 1.0, p.muted.gamma_multiply(0.6)),
        Stripe::None => {}
    }
}

/// A dashed line down, as a dashed border: dashes as long as twice its
/// width, as far apart.
fn dashes(painter: &egui::Painter, x: f32, top: f32, bottom: f32, width: f32, color: Color32) {
    let dash = 2.0 * width;
    let mut y = top;
    while y < bottom {
        painter.vline(x, egui::Rangef::new(y, (y + dash).min(bottom)), egui::Stroke::new(width, color));
        y += 2.0 * dash;
    }
}

/// Yellow and black diagonal bands, 5 wide.
fn hazard(painter: &egui::Painter, rect: Rect) {
    const BAND: f32 = 4.0;
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    painter.rect_filled(rect, 2.0, Color32::from_rgb(0x1c, 0x1c, 0x1c));
    let (left, right) = (rect.left(), rect.right());
    let w = rect.width();
    let yellow = Color32::from_rgb(0xf2, 0xc2, 0x30);
    let mut y = rect.top() - w;
    while y < rect.bottom() + w {
        painter.add(egui::Shape::convex_polygon(
            vec![pos2(left, y + w), pos2(right, y), pos2(right, y + BAND), pos2(left, y + w + BAND)],
            yellow,
            egui::Stroke::NONE,
        ));
        y += 2.0 * BAND;
    }
}

/// The comment count: its icon, colour and number; none when nobody
/// spoke yet.
fn comments(p: &Palette, ticket: &Ticket, user: &str) -> Option<(Option<egui_pop::core::icon::Icon>, Color32, String)> {
    if ticket.pending_comment_count > 0 {
        return Some((crate::icons::pending_comments(), p.warning, ticket.pending_comment_count.to_string()));
    }
    if ticket.comment_count == 0 && ticket.last_speaker.is_none() {
        return None;
    }
    let mine = ticket.last_speaker.as_deref() == Some(user);
    let color = if ticket.unread {
        p.accent
    } else if mine {
        p.muted.gamma_multiply(0.55)
    } else {
        p.text
    };
    let icon = if mine && !ticket.unread { crate::icons::comments_mine() } else { crate::icons::comments() };
    Some((icon, color, ticket.comment_count.to_string()))
}

/// The facts under the snippet: the critical pill, the flame, the tags,
/// then what is said of it, joined by " · ".
#[allow(clippy::too_many_arguments)]
fn facts(pen: &Pen<'_>, p: &Palette, x: f32, top: f32, right: f32, ticket: &Ticket, state: &RowState, user: &str) {
    let who = |name: &str| if name == user { "you".to_string() } else { name.to_string() };
    let mut said: Vec<String> = Vec::new();
    if state.glyph == Some(Glyph::Rejected) {
        said.push("rejected".into());
    }
    if let Some(intent) = &ticket.intent {
        said.push(intent.clone());
    }
    if let Some(level) = ticket.level.as_deref().filter(|l| *l != "task") {
        said.push(level.to_string());
    }
    if let Some(holder) = &ticket.holder {
        let assigned = ticket.held_as.as_deref() == Some("assigned");
        said.push(format!("{} {}", if assigned { "assigned to" } else { "claimed by" }, who(holder)));
    }
    if let Some(speaker) = &ticket.last_speaker {
        said.push(if speaker == user { "you spoke last".into() } else { format!("{speaker} spoke last") });
    }
    if ticket.blocked {
        said.push("blocked".into());
    }
    if let Some(until) = ticket.postponed_until.as_deref() {
        said.push(format!("snoozed until {}", until.get(..10).unwrap_or(until)));
    }
    if ticket.has_payload {
        said.push("payload".into());
    }
    if let Some(scope) = ticket.scope.as_deref().filter(|s| *s != "default") {
        said.push(scope.to_string());
    }
    said.push(format!("by {} · {}", who(&ticket.by_agent), ticket.created_at.get(..10).unwrap_or("")));

    let mid = top + line(XS) / 2.0;
    let mut x = x;
    if let Some(critical) = &ticket.critical {
        // A pill: 6 from its sides, the icon (12) and the count 2 apart,
        // dark ink on the danger colour.
        let n = critical.quiet().map_or(critical.holds.to_string(), |q| format!("{} · {q}", critical.holds));
        let w = 6.0 + 12.0 + 2.0 + pen.width(&n, XS, Weight::REGULAR) + 6.0;
        let rect = Rect::from_min_size(pos2(x, top), vec2(w, line(XS)));
        pen.painter.rect_filled(rect, 2.0, p.danger);
        let ink = Color32::from_rgb(0x14, 0x14, 0x14);
        if let Some(icon) = crate::icons::critical() {
            icon.paint(pen.painter, Rect::from_min_size(pos2(x + 6.0, mid - 6.0), vec2(12.0, 12.0)), ink);
        }
        pen.text(x + 20.0, top, &n, XS, Weight::REGULAR, ink);
        x = rect.right() + 8.0;
    }
    if ticket.hot {
        if let Some(icon) = crate::icons::hot() {
            icon.paint(pen.painter, Rect::from_min_size(pos2(x, mid - 6.0), vec2(12.0, 12.0)), p.warning);
        }
        x += 12.0 + 8.0;
    }
    for tag in &ticket.tags {
        let w = pen.width(&tag.name, XS, Weight::REGULAR) + 8.0 + 2.0;
        let rect = Rect::from_min_size(pos2(x, top), vec2(w, line(XS) + 2.0));
        pen.painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, p.border), egui::StrokeKind::Inside);
        pen.text(x + 5.0, top + 1.0, &tag.name, XS, Weight::REGULAR, p.muted);
        x = rect.right() + 8.0;
    }
    pen.text_max(x, top, &said.join(" · "), XS, Weight::REGULAR, p.muted, Some(right - x));
}
