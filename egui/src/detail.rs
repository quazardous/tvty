//! A ticket opened in the tickets panel, as the GPUI build lays it out
//! (`panel.rs`'s `detail` there): the way back, the title, where it stands
//! said once, the thread, then a pending plan's choice and the reply box.
//! Drawn with the kit's components (`button`, `radio`, `field`, `card`,
//! `tag-chip`), set to GPUI's look by the theme.

use egui_pop::i18n::tr;
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

/// What the reader changed in a ticket's view: the order, what is unfolded,
/// the reply being written, the plan's option chosen.
#[derive(Clone, Default)]
struct View {
    newest_first: bool,
    card_open: bool,
    /// The entries unfolded, by their place in the thread.
    open: std::collections::BTreeSet<usize>,
    reply: String,
    quiet: bool,
    option: Option<String>,
}

pub fn show(ui: &mut egui::Ui, scene: &Scene, project: &str, id: u64, app: &egui_pop::api::AppHandle) {
    let Some(thread) = scene.projects.iter().find(|p| p.name == project).and_then(|p| p.threads.get(&id.to_string())) else {
        return;
    };
    let t = Tokens::of(ui.ctx());
    let kit = Kit::of(ui.ctx());
    let p = Palette::of(&t);
    let user = scene.user.as_str();
    let ticket = &thread["ticket"];
    let view_id = egui::Id::unique(("tvty.detail.view", project, id));
    let mut view: View = ui.data(|d| d.get_temp(view_id)).unwrap_or_default();
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
    let w = pen.text(left + 4.0, y, &tr!("tickets-back"), SM, Weight::REGULAR, p.accent);
    let back = Rect::from_min_size(pos2(left, y), vec2(w + 8.0, line(SM)));
    if ui.interact(back, ui.scope_id().with("back"), Sense::click()).clicked() {
        set_shown(ui.ctx(), None);
    }
    let more = "⤢ more";
    let w = pen.width(more, XS, Weight::REGULAR);
    let slot = painter.add(egui::Shape::Noop);
    pen.text(right - w, y + 2.0, more, XS, Weight::REGULAR, p.accent);
    if crate::row::button(ui, &painter, slot, Rect::from_min_size(pos2(right - w - 4.0, y), vec2(w + 8.0, line(XS) + 4.0)), "detail-more").clicked() {
        crate::notice::not_here(app, "The ticket full screen");
    }
    let order = if view.newest_first { "⇅ newest first" } else { "⇅ newest last" };
    let w2 = pen.width(order, XS, Weight::REGULAR);
    let slot = painter.add(egui::Shape::Noop);
    pen.text(right - w - 16.0 - w2, y + 2.0, order, XS, Weight::REGULAR, p.accent);
    if crate::row::button(ui, &painter, slot, Rect::from_min_size(pos2(right - w - 16.0 - w2 - 4.0, y), vec2(w2 + 8.0, line(XS) + 4.0)), "detail-order").clicked() {
        view.newest_first = !view.newest_first;
    }
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
        x += pen.text(x, y, &tr!("tickets-held-by", holder = holder), XS, Weight::REGULAR, p.muted) + 12.0;
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
    let snapshot = thread["comments"].as_array().and_then(|cs| cs.iter().rev().find_map(|c| {
        let m = meta_of(c)?;
        let said = m.get("summary_until")?.as_str()?.to_string();
        Some((c["by_agent"].as_str().unwrap_or("").to_string(), said))
    }));
    if let Some((by, said)) = snapshot {
        // Unfolded, the card holds the snapshot, wrapped, under its line.
        let text = view.card_open.then(|| painter.layout(said, crate::row::font(SM, Weight::REGULAR), p.text, right - left - 22.0 - 9.0));
        let h = 33.0 + text.as_ref().map_or(0.0, |g| g.size().y + 9.0);
        let card = Rect::from_min_size(pos2(left, y), vec2(right - left, h));
        let response = ui.interact(Rect::from_min_size(card.min, vec2(card.width(), 33.0)), ui.scope_id().with("snapshot"), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.clicked() {
            view.card_open = !view.card_open;
        }
        kit.card.paint(&cx, &Part { rect: card, room: card, state: State::from_response(&response), label: None });
        let mid = card.top() + 33.0 / 2.0;
        pen.text(card.left() + 9.0, mid - line(XS) / 2.0, if view.card_open { "▾" } else { "▸" }, XS, Weight::REGULAR, p.accent);
        pen.text(card.left() + 22.0, mid - line(XS) / 2.0, &tr!("tickets-where-it-stands", who = by.as_str()), XS, Weight::REGULAR, p.muted);
        if let Some(g) = text {
            painter.galley(pos2(card.left() + 22.0, card.top() + 33.0), g, p.text);
        }
        y = card.bottom() + 8.0;
    }
    painter.hline(rect.x_range(), y + 0.5, egui::Stroke::new(1.0, p.border));
    y += 1.0;

    // ── The thread: every entry but the last folded to a line.
    // The newest stays open; a click on another's line unfolds it.
    let last = entries.len().saturating_sub(1);
    let mut order: Vec<usize> = (0..entries.len()).collect();
    if view.newest_first {
        order.reverse();
    }
    for (n, &i) in order.iter().enumerate() {
        let entry = &entries[i];
        let folded = i != last && !view.open.contains(&i);
        // The open entry sits tighter under the folded ones.
        y += if n == 0 { 10.0 } else if folded { 8.0 } else { 3.0 };
        let mut x = left;
        if i != last {
            let head = Rect::from_min_max(pos2(left - 4.0, y - 3.0), pos2(right, y + line(XS) + 3.0));
            let r = ui.interact(head, ui.scope_id().with(("entry", i)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
            if r.clicked() && !view.open.remove(&i) {
                view.open.insert(i);
            }
            pen.text(x, y, if folded { "▸" } else { "▾" }, XS, Weight::REGULAR, p.accent);
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
        if folded {
            let body = entry.body.lines().next().unwrap_or("").to_string();
            pen.text_max(left, y, &body, SM, Weight::REGULAR, p.muted, Some(right - left));
            y += line(SM) + 10.0;
        } else {
            // Open: the whole body, wrapped.
            let g = painter.layout(entry.body.trim_end().to_string(), crate::row::font(SM, Weight::REGULAR), p.text, right - left);
            let h = g.size().y.max(line(SM));
            painter.galley(pos2(left, y + (line(SM) - SM) / 2.0 - 1.0), g, p.text);
            y += h + 10.0;
        }
        if folded {
            painter.hline(egui::Rangef::new(left, right), y + 0.5, egui::Stroke::new(1.0, p.border));
        }
        let _ = entry.meta;
    }

    // ── The composer, from the bottom up: what it sends and how.
    let bottom = rect.bottom();
    // A button: the kit's, in its state under the pointer; its click.
    let button = |ui: &mut egui::Ui, r: Rect, text: &str, state: State, size: f32| -> egui::Response {
        let response = ui.interact(r, ui.scope_id().with(("button", text)), Sense::click());
        let response = if state.contains(State::DISABLED) { response } else { response.on_hover_cursor(egui::CursorIcon::PointingHand) };
        let state = if state.contains(State::DISABLED) { state } else { state.with(State::HOVER, response.hovered()).with(State::ACTIVE, response.is_pointer_button_down_on()) };
        kit.button.paint(&cx, &Part { rect: r, room: r, state, label: None });
        let ink = if size == XS && !state.contains(State::SELECTED) { p.muted } else { kit.button.style(state).text };
        let w = pen.width(text, size, Weight::REGULAR);
        pen.text(r.center().x - w / 2.0, r.center().y - line(size) / 2.0, text, size, Weight::REGULAR, ink);
        response
    };
    // GPUI's widths in English, grown or shrunk by the words' own.
    let wider = |en: &str, said: &str, size: f32| pen.width(said, size, Weight::REGULAR) - pen.width(en, size, Weight::REGULAR);
    let said = format!("{}  ▾", tr!("tickets-reply"));
    let reply = Rect::from_min_max(pos2(right - 77.0 - wider("Reply  ▾", &said, SM), bottom - 37.0), pos2(right, bottom - 14.0));
    if button(ui, reply, &said, State::REST.toned(Tone::Accent), SM).clicked() {
        crate::notice::not_here(app, "Replying");
    }
    let quiet = tr!("tickets-without-notifying");
    let quiet_rect = Rect::from_min_max(pos2(reply.left() - 113.0 - wider("without notifying", &quiet, XS), bottom - 37.0), pos2(reply.left() - 5.0, bottom - 14.0));
    if button(ui, quiet_rect, &quiet, State::REST.with(State::SELECTED, view.quiet), XS).clicked() {
        view.quiet = !view.quiet;
    }
    let row_y = bottom - 69.0;
    let mut bx = left;
    for (en, text, w) in [("Close", tr!("tickets-close"), 56.0), ("Snooze ▾", tr!("tickets-snooze"), 79.0), ("Assign ▾", tr!("tickets-assign"), 74.0)] {
        let w = w + wider(en, &text, SM);
        if button(ui, Rect::from_min_size(pos2(bx, row_y), vec2(w, 24.0)), &text, State::REST, SM).clicked() {
            crate::notice::not_here(app, en.trim_end_matches(" ▾"));
        }
        bx += w + 8.0;
    }
    // The reply: a text field, the kit's box, the text 10 in.
    let field = Rect::from_min_max(pos2(left, row_y - 11.0 - 57.0), pos2(right, row_y - 11.0));
    let edit_id = egui::Id::unique(("tvty.detail.reply", project, id));
    let focused = ui.memory(|m| m.has_focus(edit_id));
    kit.field.paint(&cx, &Part { rect: field, room: field, state: State::REST.with(State::FOCUS, focused).with(State::HOVER, ui.rect_contains_pointer(field)), label: None });
    let edit = egui::TextEdit::multiline(&mut view.reply)
        .id(edit_id)
        .frame(egui::Frame::NONE)
        .margin(egui::Margin::ZERO)
        .font(crate::row::font(SM, Weight::REGULAR))
        .text_color(p.text)
        .hint_text(egui::RichText::new(tr!("tickets-reply-placeholder")).color(p.muted).font(crate::row::font(SM, Weight::REGULAR)))
        .desired_width(field.width() - 20.0)
        .desired_rows(2);
    ui.put(field.shrink2(vec2(10.0, 9.0)), edit);
    if focused && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter)) {
        crate::notice::not_here(app, "Replying");
    }

    // A pending plan with options: the choice, above the reply box.
    let Some((by, options)) = pending_plan(thread) else {
        ui.data_mut(|d| d.insert_temp(view_id, view));
        return;
    };
    let hint_y = field.top() - 6.0 - line(XS);
    pen.text(left, hint_y, "To reject, say why below first.", XS, Weight::REGULAR, p.muted);
    let mut oy = hint_y - 2.0 - options.len() as f32 * 31.0;
    let top_of_options = oy;
    for option in &options {
        let key = option["key"].as_str().unwrap_or("");
        let label = option["label"].as_str().unwrap_or(key);
        let recommended = option["recommended"].as_bool().unwrap_or(false);
        let chosen = view.option.as_deref().map_or(recommended, |o| o == key);
        let radio = Rect::from_min_size(pos2(left + 4.0, oy + 4.0), vec2(22.0, 22.0));
        let line_rect = Rect::from_min_max(pos2(left, oy), pos2(right, oy + 30.0));
        let picked = ui.interact(line_rect, ui.scope_id().with(("option", key)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked();
        if ui.put(radio, RadioButton::new(chosen, ())).clicked() || picked {
            view.option = Some(key.to_string());
        }
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
    if button(ui, accept, "Accept → go", State::REST.toned(Tone::Success), SM).clicked() {
        crate::notice::not_here(app, "Accepting the plan");
    }
    let reject = Rect::from_min_max(pos2(accept.left() - 8.0 - 59.0, bar_mid - 12.0), pos2(accept.left() - 8.0, bar_mid + 12.0));
    // Reject asks a reason first: written in the reply, it can be pressed.
    let state = if view.reply.trim().is_empty() { State::DISABLED } else { State::REST };
    if button(ui, reject, "Reject", state, SM).clicked() && !view.reply.trim().is_empty() {
        crate::notice::not_here(app, "Rejecting the plan");
    }
    ui.data_mut(|d| d.insert_temp(view_id, view));
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
