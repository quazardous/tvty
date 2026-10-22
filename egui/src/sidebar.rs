//! The sessions list, open, as the GPUI build lays it out (`sidebar` and
//! `project_block` in `src/shell.rs`, `sessions_list` in
//! `src/shell/loopstabs.rs`): its header (the Sessions and Workspaces
//! tabs, the order, "+ project", ">_", the fold), the filter, then three
//! sections — the live sessions by project, the stopped loops, the agents
//! without one.
//!
//! The tabs, the field, the rows and the counters are egui-pop's kit
//! components (`tab`, `field`, `list-row`, `indicator-badge`, `pill`);
//! the sections' headers are painted: the kit has no such component.

use std::collections::BTreeMap;

use egui_pop::core::fonts::Weight;
use egui_pop::core::kit::{Kit, KitCx, Part, State, Tone};
use egui_pop::egui::{self, Color32, Rect, Sense, pos2, vec2};
use egui_pop::prelude::Tokens;

use crate::board::{Alerts, Ticket};
use crate::fulllist::{Palette, Pen, SM, XS, draw_badges, badges_width, line};
use crate::scene::{Consumer, Scene};

/// The list's width: GPUI's, the widest its header's groups make it.
pub const WIDTH: f32 = 344.0;

/// GPUI's heights: the header, a section's title, a project's heading, a
/// session's row (with its state line).
const HEADER: f32 = 36.0;
const TITLE: f32 = 28.0;
const HEADING: f32 = 30.0;
const ROW: f32 = 51.0;

/// A session of the live list.
struct Session<'a> {
    label: String,
    agent: Option<&'a Consumer>,
    open: bool,
}

/// A project of the live list, its sessions in the user's order.
struct Project<'a> {
    name: String,
    on_board: bool,
    sessions: Vec<Session<'a>>,
}

/// The tmux sessions by project, as tvty groups them: a loop's session
/// (`cl-`) goes with the agent working in its folder, and that agent's
/// project.
fn live(scene: &Scene) -> Vec<Project<'_>> {
    let mut projects: BTreeMap<String, Project<'_>> = BTreeMap::new();
    for (session, path) in &scene.sessions {
        let agent = session
            .starts_with("cl-")
            .then(|| scene.consumers.iter().find(|c| c.kind == "agent" && c.cwd.as_deref() == Some(path.as_str())))
            .flatten();
        let basename = || path.rsplit('/').next().unwrap_or(path).to_string();
        let (project, on_board, label) = match agent {
            Some(c) => match &c.project {
                Some(p) => (p.clone(), true, c.consumer_id.clone()),
                None => (basename(), false, c.consumer_id.clone()),
            },
            None => ("tmux".to_string(), false, session.clone()),
        };
        let entry = projects.entry(project.clone()).or_insert_with(|| Project { name: project.clone(), on_board, sessions: Vec::new() });
        entry.on_board |= on_board;
        entry.sessions.push(Session { label, agent, open: scene.open.contains(session) });
        // The user's order of a project's terminals, as kept.
        if let Some(order) = scene.order.get(&project) {
            let rank = |s: &Session<'_>| {
                let name = scene.sessions.iter().find(|(_, p)| s.agent.and_then(|a| a.cwd.as_deref()) == Some(p.as_str())).map(|(n, _)| n.clone());
                name.and_then(|n| order.iter().position(|o| *o == n)).unwrap_or(usize::MAX)
            };
            entry.sessions.sort_by_key(rank);
        }
    }
    let mut list: Vec<Project<'_>> = projects.into_values().collect();
    list.sort_by_key(|p| p.name.to_lowercase());
    list
}

pub fn show(ui: &mut egui::Ui, scene: &Scene) {
    let t = Tokens::of(ui.ctx());
    let kit = Kit::of(ui.ctx());
    let p = Palette::of(&t);
    let rect = ui.max_rect();
    let painter = ui.painter().clone();
    let pen = Pen { painter: &painter };
    let cx = KitCx { painter: &painter, ppp: ui.ctx().pixels_per_point() };
    painter.rect_filled(rect, 0.0, p.bg);

    // ── The header: 36 high, 12 from the sides, a line under it.
    let header = Rect::from_min_size(rect.min, vec2(rect.width(), HEADER));
    painter.hline(header.x_range(), header.bottom() - 0.5, egui::Stroke::new(1.0, p.border));
    let mid = header.top() + (HEADER - 1.0) / 2.0;
    let mut x = header.left() + 12.0;
    for (label, on) in [("Sessions", true), ("Workspaces", false)] {
        // A tab: 4 from its sides, its text and the 2 of its underline.
        let w = pen.width(label, SM, Weight::REGULAR) + 8.0;
        let h = line(SM) + 2.0;
        let tab = Rect::from_min_size(pos2(x, mid - h / 2.0), vec2(w, h));
        let response = ui.interact(tab, ui.id().with(("tab", label)), Sense::click());
        let state = State::from_response(&response).with(State::SELECTED, on);
        kit.tab.paint(&cx, &Part { rect: tab, room: tab, state, label: None });
        pen.text(x + 4.0, tab.top(), label, SM, Weight::BOLD, kit.tab.style(state).text);
        x = tab.right() + 4.0;
    }
    // From the end: the fold, a separator, ">_", "+ project", the order.
    let mut end = header.right() - 12.0;
    let glyph = |end: &mut f32, s: &str, size: f32, color: Color32, least: f32| {
        let w = (pen.width(s, size, Weight::REGULAR) + 8.0).max(least);
        *end -= w;
        let tw = pen.width(s, size, Weight::REGULAR);
        pen.text(*end + (w - tw) / 2.0, mid - line(size) / 2.0, s, size, Weight::REGULAR, color);
        *end -= 4.0;
    };
    glyph(&mut end, "‹", 16.0, p.muted, 20.0);
    end -= 4.0;
    painter.vline(end - 0.5, egui::Rangef::new(mid - 6.0, mid + 6.0), egui::Stroke::new(1.0, p.border));
    end -= 1.0 + 4.0 + 4.0;
    glyph(&mut end, ">_", XS, p.accent, 0.0);
    glyph(&mut end, "+ project", XS, p.accent, 0.0);
    glyph(&mut end, "⇅", XS, p.accent, 0.0);

    // ── The filter: a small field, 8 from the sides, 6 above and under.
    let filter = Rect::from_min_size(pos2(rect.left(), header.bottom()), vec2(rect.width(), 6.0 + 24.0 + 6.0 + 1.0));
    painter.hline(filter.x_range(), filter.bottom() - 0.5, egui::Stroke::new(1.0, p.border));
    let field = Rect::from_min_size(pos2(rect.left() + 8.0, filter.top() + 6.0), vec2(rect.width() - 16.0, 24.0));
    let response = ui.interact(field, ui.id().with("filter"), Sense::click());
    kit.field.paint(&cx, &Part { rect: field, room: field, state: State::from_response(&response), label: None });
    pen.text(field.left() + 8.0, field.center().y - line(SM) / 2.0, "Filter…  ctrl+shift+f", SM, Weight::REGULAR, p.muted);

    // ── The sections.
    let projects = live(scene);
    let running: usize = projects.iter().map(|p| p.sessions.len()).sum();
    let placed: Vec<&str> = projects.iter().flat_map(|p| p.sessions.iter().filter_map(|s| s.agent.map(|a| a.consumer_id.as_str()))).collect();
    let shut: Vec<&Consumer> = scene
        .consumers
        .iter()
        .filter(|c| c.kind == "agent" && c.project.is_some() && !placed.contains(&c.consumer_id.as_str()))
        .collect();
    let mut y = filter.bottom();
    y = section(&pen, &p, rect, y, "live", running);
    for project in &projects {
        y = project_block(ui, &kit, &pen, &cx, &p, rect, y, project, scene);
    }
    y = section(&pen, &p, rect, y, "idle", 0);
    y = empty(&pen, &p, rect, y, "No stopped loop");
    y = section(&pen, &p, rect, y, "shut", shut.len());
    if shut.is_empty() {
        empty(&pen, &p, rect, y, "No agent without a loop");
    }
}

/// A section's title: 28 high with a line above and under, its chevron
/// (10 wide), its name in capitals and its count on a round pill, 6 apart.
fn section(pen: &Pen<'_>, p: &Palette, rect: Rect, top: f32, title: &str, count: usize) -> f32 {
    let band = Rect::from_min_size(pos2(rect.left(), top), vec2(rect.width(), TITLE));
    let stroke = egui::Stroke::new(1.0, p.border);
    pen.painter.hline(band.x_range(), band.top() + 0.5, stroke);
    pen.painter.hline(band.x_range(), band.bottom() - 0.5, stroke);
    let mid = band.center().y;
    let x = band.left() + 12.0;
    pen.text(x, mid - line(XS) / 2.0, "▾", XS, Weight::REGULAR, p.accent);
    let x = x + 10.0 + 6.0;
    let x = x + pen.text(x, mid - line(XS) / 2.0, &title.to_uppercase(), XS, Weight::BOLD, p.text) + 6.0;
    let n = count.to_string();
    let w = pen.width(&n, XS, Weight::REGULAR) + 12.0;
    let pill = Rect::from_min_size(pos2(x, mid - line(XS) / 2.0), vec2(w, line(XS)));
    pen.painter.rect_filled(pill, line(XS) / 2.0, p.border);
    pen.text(x + 6.0, pill.top(), &n, XS, Weight::REGULAR, p.text);
    band.bottom()
}

/// A section's line saying it has nothing: muted, 12 from the side, at
/// least 24 high.
fn empty(pen: &Pen<'_>, p: &Palette, rect: Rect, top: f32, said: &str) -> f32 {
    pen.text(rect.left() + 12.0, top, said, SM, Weight::REGULAR, p.muted);
    top + line(SM).max(24.0)
}

/// A project's heading (its name, its badges, "+ session", its options,
/// ">_"), then its sessions.
#[allow(clippy::too_many_arguments)]
fn project_block(ui: &mut egui::Ui, kit: &Kit, pen: &Pen<'_>, cx: &KitCx<'_>, p: &Palette, rect: Rect, top: f32, project: &Project<'_>, scene: &Scene) -> f32 {
    let tickets: Vec<&Ticket> = scene.projects.iter().filter(|q| q.name == project.name).flat_map(|q| q.tickets.iter()).filter(|t| !t.closed).collect();
    let alerts = Alerts::of(tickets.iter().copied());
    // 8 above, 4 under: the middle of what is left.
    let mid = top + 8.0 + (HEADING - 12.0) / 2.0;
    let mut end = rect.right() - 12.0;
    if project.on_board {
        let link = |end: &mut f32, s: &str, color: Color32, least: f32| {
            let tw = pen.width(s, XS, Weight::REGULAR);
            let w = (tw + 8.0).max(least);
            *end -= w;
            pen.text(*end + (w - tw) / 2.0, mid - line(XS) / 2.0, s, XS, Weight::REGULAR, color);
            *end -= 4.0;
        };
        link(&mut end, ">_", p.accent, 0.0);
        link(&mut end, "⚙", p.muted, 20.0);
        link(&mut end, "+ session", p.accent, 0.0);
    }
    let w = badges_width(kit, pen, &alerts);
    if w > 0.0 {
        draw_badges(kit, pen, cx, pos2(end - w, mid), &alerts);
        end -= w + 4.0;
    }
    let x = rect.left() + 12.0;
    pen.text_max(x, mid - line(XS) / 2.0, &project.name.to_uppercase(), XS, Weight::BOLD, p.muted, Some(end - x));
    let mut y = top + HEADING;
    let critical_holder = tickets.iter().find(|t| t.critical.is_some()).and_then(|t| t.holder.clone());
    for session in &project.sessions {
        y = session_row(ui, kit, pen, cx, p, rect, y, session, critical_holder.as_deref());
    }
    y
}

/// A session's row: 51 high; its loop's colour (3 wide), the green dot of
/// a terminal open in tvty, its name, ⇄ for a loop in tmux, its counters;
/// under them its state.
#[allow(clippy::too_many_arguments)]
fn session_row(ui: &mut egui::Ui, kit: &Kit, pen: &Pen<'_>, cx: &KitCx<'_>, p: &Palette, rect: Rect, top: f32, session: &Session<'_>, critical: Option<&str>) -> f32 {
    let row = Rect::from_min_size(pos2(rect.left(), top), vec2(rect.width(), ROW));
    let response = ui.interact(row, ui.id().with(("session", &session.label)), Sense::click());
    kit.list_row.paint(cx, &Part { rect: row, room: row, state: State::from_response(&response), label: None });
    let online = session.agent.and_then(|a| a.present).unwrap_or(false);
    // The loop's state: working, starting, idle; nothing offline.
    if let Some(color) = session.agent.filter(|_| online).map(|a| match a.state.as_deref() {
        Some("busy") => p.accent,
        Some("boot") => p.warning,
        _ => p.muted,
    }) {
        pen.painter.rect_filled(Rect::from_min_max(pos2(row.left() + 4.0, top + 4.0), pos2(row.left() + 7.0, row.bottom() - 4.0)), 2.0, color);
    }
    let x = row.left() + 4.0 + 3.0 + 8.0;
    let line_top = top + 4.0;
    let mid = line_top + line(SM) / 2.0;
    if session.open {
        // A dot 7 wide, 6 down in its 13 high box, on whole pixels.
        let dot = Rect::from_min_size(pos2(x, (mid - 6.5 + 6.0).floor()), vec2(7.0, 7.0));
        pen.painter.rect_filled(dot, 3.5, p.success);
    }
    let mut end = row.right() - 12.0;
    if let Some(agent) = session.agent {
        end = counters(kit, pen, cx, end, mid, agent, critical == Some(agent.consumer_id.as_str()));
        let w = pen.width("⇄", XS, Weight::REGULAR);
        end -= 8.0 + w;
        pen.text(end, mid - line(XS) / 2.0, "⇄", XS, Weight::REGULAR, p.muted);
    }
    let label_x = x + 7.0 + 8.0;
    pen.text_max(label_x, line_top, &session.label, SM, Weight::REGULAR, p.text, Some(end - 8.0 - label_x));
    if session.agent.is_some() && !online {
        pen.text(x, line_top + line(SM) + 2.0, "offline", 11.0, Weight::REGULAR, p.muted);
    }
    row.bottom()
}

/// An agent's counters, from `end` towards the start, 4 apart: the
/// critical "!", its backlog (`b`) and its events (`e`), each lit in its
/// colour when not zero; where they start.
fn counters(kit: &Kit, pen: &Pen<'_>, cx: &KitCx<'_>, end: f32, mid: f32, agent: &Consumer, critical: bool) -> f32 {
    let backlog = agent.counters.as_ref().and_then(|c| c.get("backlog")).and_then(|v| v.as_u64());
    let events = agent.ping_unseen.unwrap_or(0);
    let mut lights: Vec<(&str, Option<u64>, Tone)> = vec![("e", Some(events), Tone::Accent), ("b", backlog, Tone::Info)];
    let mut x = end;
    for (i, (letter, value, tone)) in lights.drain(..).enumerate() {
        if i > 0 {
            x -= 4.0;
        }
        let on = value.is_some_and(|v| v > 0);
        let n = value.map_or("-".to_string(), |v| v.to_string());
        // 4 from its sides, the letter and the value 2 apart.
        let (lw, nw) = (pen.width(letter, XS, Weight::REGULAR), pen.width(&n, XS, Weight::REGULAR));
        let w = 4.0 + lw + 2.0 + nw + 4.0;
        x -= w;
        let rect = Rect::from_min_size(pos2(x, mid - line(XS) / 2.0), vec2(w, line(XS)));
        let state = if on { State::REST.toned(tone) } else { State::REST };
        kit.pill.paint(cx, &Part { rect, room: rect, state, label: None });
        let ink = kit.pill.style(state).text;
        pen.text(x + 4.0, rect.top(), letter, XS, Weight::REGULAR, ink.gamma_multiply(0.7));
        pen.text(x + 4.0 + lw + 2.0, rect.top(), &n, XS, Weight::REGULAR, ink);
    }
    if critical {
        let alerts = Alerts { critical: true, decisions: 0, unread: 0 };
        let w = badges_width(kit, pen, &alerts);
        x -= 4.0 + w;
        draw_badges(kit, pen, cx, pos2(x, mid), &alerts);
    }
    x
}
