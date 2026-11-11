//! A ticket opened in the tickets panel, as the GPUI build lays it out
//! (`panel.rs`'s `detail` there): the way back, the title, where it stands
//! said once, the thread, then a pending plan's choice and the reply box.
//! Drawn with the kit's components (`button`, `radio`, `field`, `card`,
//! `tag-chip`), set to GPUI's look by the theme.

use egui_pop::core::fonts::Weight;
use egui_pop::core::kit::{Kit, KitCx, Part, State, Tone};
use egui_pop::core::widgets::RadioButton;
use egui_pop::egui::{self, Color32, Rect, Sense, pos2, vec2};
use egui_pop::prelude::Tokens;
use serde_json::Value;

use crate::board::{self, Glyph};
use crate::fulllist::{Palette, Pen, SM, XS, line};
use crate::scene::Scene;

/// GPUI's large text (`text_lg`).
const LG: f32 = 18.0;

const SHOWN: &str = "tvty.ticket.shown";

/// The ticket the panel shows: the one clicked, else at start
/// `TVTY_EGUI_TICKET` (to compare).
pub fn shown(ctx: &egui::Context) -> Option<u64> {
    let id = egui::Id::unique(SHOWN);
    ctx.data(|d| d.get_temp::<Option<u64>>(id))
        .unwrap_or_else(|| std::env::var("TVTY_EGUI_TICKET").ok().and_then(|v| v.parse().ok()))
}

/// Opens a ticket in the panel, or goes back to the list (`None`).
pub fn set_shown(ctx: &egui::Context, ticket: Option<u64>) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique(SHOWN), ticket));
}

/// One entry of the thread: the ticket's body, then each comment.
struct Entry<'a> {
    who: String,
    when: Option<String>,
    body: String,
    /// Its decision chip: "plan · pending" and the like; pending is lit.
    chip: Option<(String, bool)>,
    meta: Option<&'a Value>,
}

fn meta_of(c: &Value) -> Option<Value> {
    match c.get("meta")? {
        Value::String(s) => serde_json::from_str(s).ok(),
        v @ Value::Object(_) => Some(v.clone()),
        _ => None,
    }
}

pub fn show(ui: &mut egui::Ui, scene: &Scene, project: &str, id: u64) {
    let Some(thread) = scene.projects.iter().find(|p| p.name == project).and_then(|p| p.threads.get(&id.to_string())) else {
        return;
    };
    let t = Tokens::of(ui.ctx());
    let kit = Kit::of(ui.ctx());
    let p = Palette::of(&t);
    let user = scene.user.as_str();
    let ticket = &thread["ticket"];
    let row = scene.projects.iter().find(|q| q.name == project).and_then(|q| q.tickets.iter().find(|t| t.id == id));
    let state = row.map(|r| board::state_of(r, user));
    // Under the panel's header.
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter().clone();
    let pen = Pen { painter: &painter };
    let cx = KitCx { painter: &painter, ppp: ui.ctx().pixels_per_point() };
    let (left, right) = (rect.left() + 12.0, rect.right() - 12.0);

    // ── The way back, and the thread's own buttons on its line.
    let mut y = rect.top() + 8.0;
    let w = pen.text(left + 4.0, y, "← Tickets", SM, Weight::REGULAR, p.accent);
    let back = Rect::from_min_size(pos2(left, y), vec2(w + 8.0, line(SM)));
    if ui.interact(back, ui.scope_id().with("back"), Sense::click()).clicked() {
        set_shown(ui.ctx(), None);
    }
    let more = "⤢ more";
    let w = pen.width(more, XS, Weight::REGULAR);
    pen.text(right - w, y + 2.0, more, XS, Weight::REGULAR, p.accent);
    let order = "⇅ newest last";
    let w2 = pen.width(order, XS, Weight::REGULAR);
    pen.text(right - w - 16.0 - w2, y + 2.0, order, XS, Weight::REGULAR, p.accent);
    y += line(SM) + 6.0;

    // ── The title: its glyph (20), "#N title" large and bold, wrapping.
    let yours = state.as_ref().is_some_and(|s| s.turn == board::Turn::You);
    let glyph = state.as_ref().and_then(|s| s.glyph);
    let tx = left + 20.0 + 8.0;
    let title = format!("#{} {}", id, ticket["title"].as_str().unwrap_or(""));
    let galley = painter.layout(title, crate::row::font(LG, Weight::BOLD), p.text, right - tx);
    let rows = galley.rows.len().max(1) as f32;
    let title_h = rows * line(LG);
    if let Some(icon) = glyph.and_then(crate::icons::of_glyph) {
        icon.paint(&painter, Rect::from_center_size(pos2(left + 10.0, y + title_h / 2.0), vec2(20.0, 20.0)), p.glyph(glyph.unwrap(), yours));
    }
    // Each row in its line box, as GPUI lays them.
    for (i, row) in galley.rows.iter().enumerate() {
        let text: String = row.row.glyphs.iter().map(|g| g.chr).collect();
        pen.text(tx, y + i as f32 * line(LG), text.trim_end(), LG, Weight::BOLD, p.text);
    }
    y += title_h + 6.0;

    // ── Where it stands, said once: a bar in the turn's colour.
    let said = sentence(thread, user);
    if let Some(state) = state.as_ref().filter(|_| !said.is_empty()) {
        let bar = Rect::from_min_size(pos2(left, y), vec2(3.0, line(SM)));
        painter.rect_filled(bar, 0.0, p.stripe(state));
        pen.text(left + 3.0 + 8.0, y, &said, SM, Weight::REGULAR, p.text);
        y += line(SM) + 6.0;
    }
    // Held by, and the priority.
    let mut x = left;
    if let Some(holder) = ticket["holder"].as_str() {
        x += pen.text(x, y, &format!("held by {holder}"), XS, Weight::REGULAR, p.muted) + 12.0;
    }
    if let Some(priority) = ticket["priority"].as_str().filter(|p| *p != "normal") {
        if let Some(icon) = crate::icons::of_priority(priority) {
            icon.paint(&painter, Rect::from_min_size(pos2(x, y + line(XS) / 2.0 - 6.0), vec2(12.0, 12.0)), p.priority(priority));
        }
        pen.text(x + 12.0 + 4.0, y, priority, XS, Weight::REGULAR, p.muted);
    }
    y += line(XS) + 6.0;
    // The last snapshot, folded: a card.
    let entries = entries(thread, user);
    if let Some(by) = thread["comments"].as_array().and_then(|cs| cs.iter().rev().find(|c| meta_of(c).is_some_and(|m| m.get("summary_until").is_some()))).and_then(|c| c["by_agent"].as_str()) {
        let card = Rect::from_min_size(pos2(left, y), vec2(right - left, 33.0));
        kit.card.paint(&cx, &Part { rect: card, room: card, state: State::REST, label: None });
        let mid = card.center().y;
        pen.text(card.left() + 9.0, mid - line(XS) / 2.0, "▸", XS, Weight::REGULAR, p.accent);
        pen.text(card.left() + 22.0, mid - line(XS) / 2.0, &format!("Where it stands · {by}"), XS, Weight::REGULAR, p.muted);
        y = card.bottom() + 8.0;
    }
    painter.hline(rect.x_range(), y + 0.5, egui::Stroke::new(1.0, p.border));
    y += 1.0;

    // ── The thread: every entry but the last folded to a line.
    let last = entries.len().saturating_sub(1);
    for (i, entry) in entries.iter().enumerate() {
        let folded = i < last;
        // The open entry sits tighter under the folded ones.
        y += if i == 0 { 10.0 } else if folded { 8.0 } else { 3.0 };
        let mut x = left;
        if folded {
            pen.text(x, y, "▸", XS, Weight::REGULAR, p.accent);
            x += 18.0;
        }
        x += pen.text(x, y, &entry.who, XS, Weight::REGULAR, p.muted) + 8.0;
        if let Some(when) = &entry.when {
            x += pen.text(x, y, when, XS, Weight::REGULAR, p.muted) + 8.0;
        }
        if let Some((chip, lit)) = &entry.chip {
            let label = pen.galley(chip, XS, Weight::REGULAR, Color32::PLACEHOLDER, None);
            let size = vec2(label.size().x + 12.0 + if *lit { 14.0 } else { 0.0 }, line(XS) + 2.0);
            let r = Rect::from_min_size(pos2(x, y - 3.0), size + vec2(0.0, 2.0));
            let state = if *lit { State::REST.toned(Tone::Warning) } else { State::REST };
            kit.tag_chip.paint(&cx, &Part { rect: r, room: r, state, label: None });
            let ink = kit.tag_chip.style(state).text;
            let mut lx = r.left() + 6.0;
            if *lit {
                if let Some(icon) = crate::icons::of_glyph(Glyph::Plan) {
                    icon.paint(&painter, Rect::from_min_size(pos2(lx, r.center().y - 6.0), vec2(12.0, 12.0)), ink);
                }
                lx += 14.0;
            }
            pen.text(lx, y, chip, XS, Weight::REGULAR, ink);
        }
        y += line(XS) + if folded { 8.0 } else { 5.0 };
        let body = entry.body.lines().next().unwrap_or("").to_string();
        let color = if folded { p.muted } else { p.text };
        pen.text_max(left, y, &body, SM, Weight::REGULAR, color, Some(right - left));
        y += line(SM) + 10.0;
        if folded {
            painter.hline(egui::Rangef::new(left, right), y + 0.5, egui::Stroke::new(1.0, p.border));
        }
        let _ = entry.meta;
    }

    // ── The composer, from the bottom up: what it sends and how.
    let bottom = rect.bottom();
    let button = |r: Rect, text: &str, state: State| {
        kit.button.paint(&cx, &Part { rect: r, room: r, state, label: None });
        let ink = kit.button.style(state).text;
        let w = pen.width(text, SM, Weight::REGULAR);
        pen.text(r.center().x - w / 2.0, r.center().y - line(SM) / 2.0, text, SM, Weight::REGULAR, ink);
    };
    let small = |r: Rect, text: &str| {
        kit.button.paint(&cx, &Part { rect: r, room: r, state: State::REST, label: None });
        let w = pen.width(text, XS, Weight::REGULAR);
        pen.text(r.center().x - w / 2.0, r.center().y - line(XS) / 2.0, text, XS, Weight::REGULAR, p.muted);
    };
    let reply = Rect::from_min_max(pos2(right - 77.0, bottom - 37.0), pos2(right, bottom - 14.0));
    button(reply, "Reply  ▾", State::REST.toned(Tone::Accent));
    small(Rect::from_min_max(pos2(reply.left() - 113.0, bottom - 37.0), pos2(reply.left() - 5.0, bottom - 14.0)), "without notifying");
    let row_y = bottom - 69.0;
    let mut bx = left;
    for (text, w) in [("Close", 56.0), ("Snooze ▾", 79.0), ("Assign ▾", 74.0)] {
        button(Rect::from_min_size(pos2(bx, row_y), vec2(w, 24.0)), text, State::REST);
        bx += w + 8.0;
    }
    let field = Rect::from_min_max(pos2(left, row_y - 11.0 - 57.0), pos2(right, row_y - 11.0));
    kit.field.paint(&cx, &Part { rect: field, room: field, state: State::REST, label: None });
    pen.text(field.left() + 10.0, field.top() + 9.0, "Reply… (ctrl+enter sends)", SM, Weight::REGULAR, p.muted);

    // A pending plan with options: the choice, above the reply box.
    let Some((by, options)) = pending_plan(thread) else { return };
    let hint_y = field.top() - 6.0 - line(XS);
    pen.text(left, hint_y, "To reject, say why below first.", XS, Weight::REGULAR, p.muted);
    let mut oy = hint_y - 2.0 - options.len() as f32 * 31.0;
    let top_of_options = oy;
    for option in &options {
        let key = option["key"].as_str().unwrap_or("");
        let label = option["label"].as_str().unwrap_or(key);
        let recommended = option["recommended"].as_bool().unwrap_or(false);
        let radio = Rect::from_min_size(pos2(left + 4.0, oy + 4.0), vec2(22.0, 22.0));
        ui.put(radio, RadioButton::new(recommended, ()));
        let mut lx = left + 26.0;
        let confidence = option["confidence"].as_str();
        let suggested = match confidence {
            Some("high") => p.success,
            Some("medium") => p.warning,
            Some("low") => p.danger,
            _ => p.success,
        };
        if confidence.is_some() {
            painter.circle_filled(pos2(lx + 4.0, oy + 4.0 + line(SM) / 2.0 - 1.0), 4.0, suggested);
            lx += 16.0;
        }
        let color = if recommended { suggested } else { p.text };
        pen.text(lx, oy + 4.0, label, SM, Weight::REGULAR, color);
        if recommended {
            let w = pen.width("★", SM, Weight::REGULAR);
            pen.text(right - w, oy + 4.0, "★", SM, Weight::REGULAR, suggested);
        }
        oy += 31.0;
    }
    let choose_y = top_of_options + 1.0 - line(XS);
    pen.text(left, choose_y, "Choose one:", XS, Weight::REGULAR, p.muted);
    // The decision's line: who proposes, Reject and Accept → go.
    let bar_mid = choose_y - 16.0;
    painter.hline(rect.x_range(), bar_mid - 24.0 + 0.5, egui::Stroke::new(1.0, p.border));
    if let Some(icon) = crate::icons::of_glyph(Glyph::Plan) {
        icon.paint(&painter, Rect::from_center_size(pos2(left + 10.0, bar_mid), vec2(16.0, 16.0)), p.warning);
    }
    pen.text(left + 20.0, bar_mid - line(SM) / 2.0, &format!("{by} proposes a plan"), SM, Weight::REGULAR, p.text);
    let accept = Rect::from_min_max(pos2(right - 102.0, bar_mid - 12.0), pos2(right, bar_mid + 12.0));
    button(accept, "Accept → go", State::REST.toned(Tone::Success));
    let reject = Rect::from_min_max(pos2(accept.left() - 8.0 - 59.0, bar_mid - 12.0), pos2(accept.left() - 8.0, bar_mid + 12.0));
    button(reject, "Reject", State::DISABLED);
}

/// The entries: the body as the reporter's, then each comment.
fn entries<'a>(thread: &'a Value, user: &str) -> Vec<Entry<'a>> {
    let who = |name: &str| if name == user { "you".to_string() } else { name.to_string() };
    let ticket = &thread["ticket"];
    let mut out = vec![Entry {
        who: who(ticket["by_agent"].as_str().unwrap_or("")),
        when: ticket["created_at"].as_str().and_then(board::ago),
        body: ticket["body"].as_str().unwrap_or("").to_string(),
        chip: None,
        meta: None,
    }];
    let comments: Vec<&Value> = thread["comments"].as_array().map(|c| c.iter().filter(|c| c["kind"] == "comment_added").collect()).unwrap_or_default();
    let latest = comments.iter().rposition(|c| meta_of(c).is_some_and(|m| m.get("decision").is_some()));
    for (i, c) in comments.iter().enumerate() {
        let chip = meta_of(c).and_then(|m| {
            let d = m.get("decision")?.clone();
            let kind = d["kind"].as_str()?.to_string();
            let status = d["status"].as_str()?.to_string();
            let pending = status == "pending";
            let said = if pending && Some(i) != latest { "superseded".to_string() } else { status };
            Some((format!("{kind} · {said}"), pending && Some(i) == latest))
        });
        out.push(Entry {
            who: who(c["by_agent"].as_str().unwrap_or("")),
            when: c["created_at"].as_str().and_then(board::ago),
            body: c["body"].as_str().unwrap_or("").to_string(),
            chip,
            meta: Some(c),
        });
    }
    out
}

/// The latest decision, when it is a pending plan with options: who
/// proposes it, and its options.
fn pending_plan(thread: &Value) -> Option<(String, Vec<Value>)> {
    let comments = thread["comments"].as_array()?;
    let c = comments.iter().rev().find(|c| meta_of(c).is_some_and(|m| m.get("decision").is_some()))?;
    let m = meta_of(c)?;
    let d = m.get("decision")?;
    if d["kind"] != "plan" || d["status"] != "pending" {
        return None;
    }
    let options = d["options"].as_array()?.clone();
    Some((c["by_agent"].as_str()?.to_string(), options))
}

/// "Yours: accept or reject X's plan", for a pending plan the user decides.
fn sentence(thread: &Value, _user: &str) -> String {
    match pending_plan(thread) {
        Some((by, _)) => format!("Yours: accept or reject {by}'s plan"),
        None => String::new(),
    }
}
