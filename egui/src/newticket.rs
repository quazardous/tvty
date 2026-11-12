//! A new ticket, full screen, as the GPUI build lays it out
//! (`src/newticket.rs`): over the window under its title bar, a header
//! ("New ticket", "✕ Esc"), a side column of fields (where, fields,
//! people, links) and the words (the title, Write / Preview, the body,
//! the keys, "File and exit", "File the ticket").
//!
//! tvty-egui reads a scene, not aiball: a ticket filed here joins the
//! scene in memory (`merged`), listed and opened as any other, and goes
//! no further.

use std::rc::Rc;
use std::sync::Arc;

use egui_pop::core::fonts::Weight;
use egui_pop::core::kit::{Kit, KitCx, Part, State, Tone};
use egui_pop::core::tooltip::TooltipExt as _;
use egui_pop::core::ui::UiPop;
use egui_pop::egui::{self, Rect, Sense, pos2, vec2};
use egui_pop::i18n::tr;
use egui_pop::prelude::Tokens;

use crate::board::Ticket;
use crate::fulllist::{Palette, Pen, SM, XS, line};
use crate::row::font;
use crate::scene::Scene;

const LG: f32 = 18.0;
const INTENTS: [&str; 5] = ["request", "question", "fyi", "feature", "panic"];
const PRIORITIES: [&str; 4] = ["urgent", "high", "normal", "low"];
const LEVELS: [&str; 3] = ["task", "milestone", "roadmap"];
const SCOPES: [&str; 3] = ["internal", "default", "broadcast"];

/// The draft: kept when the form closes (Esc keeps it, as GPUI's).
#[derive(Clone, PartialEq)]
struct Draft {
    open: bool,
    project: String,
    intent: &'static str,
    priority: &'static str,
    level: &'static str,
    scope: &'static str,
    assign: Option<String>,
    parent: String,
    title: String,
    body: String,
    preview: bool,
    /// "A title first.", after a try without one.
    error: Option<String>,
}

impl Default for Draft {
    fn default() -> Self {
        Self {
            open: false,
            project: String::new(),
            intent: "request",
            priority: "normal",
            level: "task",
            scope: "default",
            assign: None,
            parent: String::new(),
            title: String::new(),
            body: String::new(),
            preview: false,
            error: None,
        }
    }
}

const DRAFT: &str = "tvty.new-ticket";
const FILED: &str = "tvty.new-ticket.filed";

fn draft(ctx: &egui::Context) -> Draft {
    ctx.data(|d| d.get_temp::<Draft>(egui::Id::unique(DRAFT))).unwrap_or_default()
}

fn set_draft(ctx: &egui::Context, draft: Draft) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique(DRAFT), draft));
}

/// Opens the form, on `project` when one is given.
pub fn open(ctx: &egui::Context, project: Option<&str>) {
    let mut d = draft(ctx);
    d.open = true;
    if let Some(p) = project {
        d.project = p.to_string();
    }
    set_draft(ctx, d);
}

/// The tickets filed here, with their threads: (project, ticket, thread).
type Filed = Arc<Vec<(String, Ticket, serde_json::Value)>>;

fn filed(ctx: &egui::Context) -> Filed {
    ctx.data(|d| d.get_temp::<Filed>(egui::Id::unique(FILED))).unwrap_or_default()
}

/// The scene with the tickets filed here: the scene itself while none was.
pub fn merged(ctx: &egui::Context, scene: &Rc<Scene>) -> Rc<Scene> {
    let filed = filed(ctx);
    if filed.is_empty() {
        return scene.clone();
    }
    let mut s = (**scene).clone();
    for (project, ticket, thread) in filed.iter() {
        if let Some(p) = s.projects.iter_mut().find(|p| p.name == *project) {
            p.tickets.insert(0, ticket.clone());
            p.threads.insert(ticket.id.to_string(), thread.clone());
        }
    }
    Rc::new(s)
}

/// Now, as aiball writes it: `2026-11-12T14:05:09.000Z`.
fn now_iso() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Days to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.000Z", rem / 3600, rem / 60 % 60, rem % 60)
}

/// Files the draft into the scene in memory; Err: what is missing.
fn file(ctx: &egui::Context, scene: &Scene, d: &Draft) -> Result<(String, u64), String> {
    let title = d.title.trim();
    if title.is_empty() {
        return Err(tr!("tickets-title-first"));
    }
    let filed = filed(ctx);
    let id = scene
        .projects
        .iter()
        .filter(|p| p.name == d.project)
        .flat_map(|p| p.tickets.iter().map(|t| t.id))
        .chain(filed.iter().filter(|(p, ..)| *p == d.project).map(|(_, t, _)| t.id))
        .max()
        .unwrap_or(0)
        + 1;
    let now = now_iso();
    let snippet = d.body.lines().find(|l| !l.trim().is_empty()).map(str::to_string);
    let ticket = Ticket {
        id,
        project: d.project.clone(),
        title: title.to_string(),
        snippet,
        by_agent: scene.user.clone(),
        created_at: now.clone(),
        intent: Some(d.intent.to_string()),
        priority: Some(d.priority.to_string()),
        level: Some(d.level.to_string()),
        scope: Some(d.scope.to_string()),
        last_activity: Some(now.clone()),
        last_speaker: Some(scene.user.clone()),
        holder: d.assign.clone(),
        held_as: d.assign.as_ref().map(|_| "assigned".to_string()),
        band_name: Some("open".to_string()),
        ..Default::default()
    };
    let thread = serde_json::json!({
        "ticket": {
            "id": id, "project": d.project, "title": title, "body": d.body, "by_agent": scene.user,
            "created_at": now, "intent": d.intent, "priority": d.priority, "level": d.level, "scope": d.scope,
            "holder": d.assign, "held_as": d.assign.as_ref().map(|_| "assigned"),
        },
        "comments": [],
    });
    let mut all = (*filed).clone();
    all.push((d.project.clone(), ticket, thread));
    ctx.data_mut(|data| data.insert_temp(egui::Id::unique(FILED), Arc::new(all)));
    Ok((d.project.clone(), id))
}

pub fn show(ctx: &egui::Context, scene: &Scene) {
    let mut d = draft(ctx);
    if !d.open {
        return;
    }
    // Esc closes and keeps the draft (a list open over it closes first).
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !egui::Popup::is_any_open(ctx) {
        d.open = false;
        set_draft(ctx, d);
        return;
    }
    if d.project.is_empty() {
        d.project = crate::sidebar::current(ctx).map(|(p, _)| p).or_else(|| crate::panel::project(scene)).or_else(|| scene.projects.first().map(|p| p.name.clone())).unwrap_or_default();
    }
    let before = d.clone();
    let screen = ctx.content_rect();
    let rect = Rect::from_min_max(pos2(screen.left() + 1.0, screen.top() + 41.0), pos2(screen.right() - 1.0, screen.bottom() - 1.0));
    let area = egui::Area::new(egui::Id::unique("tvty.new-ticket.area")).fixed_pos(rect.min);
    egui_pop::layers::area(ctx, area, egui_pop::layers::Rank::Overlay, |ui| {
        ui.set_clip_rect(rect);
        ui.allocate_exact_size(rect.size(), Sense::click());
        draw(ui, rect, scene, &mut d);
    });
    if d != before {
        set_draft(ctx, d);
    }
}

fn draw(ui: &mut egui::Ui, rect: Rect, scene: &Scene, d: &mut Draft) {
    let t = Tokens::of(ui.ctx());
    let kit = Kit::of(ui.ctx());
    let p = Palette::of(&t);
    let painter = ui.painter().clone();
    painter.rect_filled(rect, 0.0, p.bg);
    let pen = Pen { painter: &painter };
    let cx = KitCx { painter: &painter, ppp: ui.ctx().pixels_per_point() };

    // ── The header: 40 high, the title 16 in, "✕ Esc" at the end.
    let header = Rect::from_min_size(rect.min, vec2(rect.width(), 40.0));
    painter.hline(header.x_range(), header.bottom() + 0.5, egui::Stroke::new(1.0, p.border));
    let mid = header.center().y;
    let title = if d.parent.trim().is_empty() {
        tr!("newticket-title")
    } else {
        tr!("newticket-title-sub", parent = d.parent.trim().trim_start_matches('#'))
    };
    pen.text(header.left() + 16.0, mid - line(LG) / 2.0, &title, LG, Weight::BOLD, p.text);
    let esc = "✕  Esc";
    let w = pen.width(esc, SM, Weight::REGULAR);
    let at = pos2(header.right() - 16.0 - w, mid - line(SM) / 2.0);
    let slot = painter.add(egui::Shape::Noop);
    pen.text(at.x, at.y, esc, SM, Weight::REGULAR, p.accent);
    if crate::row::button(ui, &painter, slot, Rect::from_min_size(at - vec2(4.0, 2.0), vec2(w + 8.0, line(SM) + 4.0)), "new-ticket-esc").clicked() {
        d.open = false;
    }

    // ── The side column (281 wide, GPUI's), its 5 wide edge, the words.
    let body = Rect::from_min_max(pos2(rect.left(), header.bottom() + 1.0), rect.max);
    let side = Rect::from_min_size(body.min, vec2(281.0, body.height()));
    painter.rect_filled(Rect::from_min_size(pos2(side.right(), body.top()), vec2(5.0, body.height())), 0.0, p.border);
    fields(ui, &pen, &p, side, scene, d);
    let words = Rect::from_min_max(pos2(side.right() + 5.0 + 16.0, body.top() + 16.0), pos2(body.right() - 16.0, body.bottom() - 16.0));
    write(ui, &kit, &cx, &pen, &p, words, scene, d);
}

/// A field's line: its label 16 in, its control 104 in, 147 wide, 32 high.
fn field_row(ui: &mut egui::Ui, pen: &Pen<'_>, p: &Palette, side: Rect, y: &mut f32, label: &str, add: impl FnOnce(&mut egui::Ui, Rect)) {
    let control = Rect::from_min_size(pos2(side.left() + 104.0, *y), vec2(147.0, 32.0));
    pen.text_max(side.left() + 16.0, *y + 4.0, label, SM, Weight::REGULAR, p.muted, Some(84.0));
    add(ui, control);
    *y += 40.0;
}

/// A list of values in a combo box, the kit's (`combo-box`, `combo-arrow`).
fn choose(ui: &mut egui::Ui, at: Rect, id: &str, shown: String, values: &[(String, String)], chosen: &mut String) {
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(at));
    // GPUI's selects: 32 high.
    child.spacing_mut().interact_size.y = at.height();
    let combo = egui_pop::core::widgets::combo_box(("tvty.new-ticket", id)).selected_text(shown).width(at.width() - 8.0).height(240.0);
    child.combo_box(combo, |ui| {
        for (value, said) in values {
            if ui.add(egui_pop::core::widgets::SelectableRow::new(chosen == value, said.as_str())).clicked() {
                *chosen = value.clone();
            }
        }
    });
}

fn choose_static(ui: &mut egui::Ui, at: Rect, id: &str, prefix: &str, values: &[&'static str], chosen: &mut &'static str) {
    let said = |v: &str| egui_pop::i18n::global().t(&format!("fields-{prefix}-{v}"), &[]);
    let list: Vec<(String, String)> = values.iter().map(|v| (v.to_string(), said(v))).collect();
    let mut value = chosen.to_string();
    choose(ui, at, id, said(chosen), &list, &mut value);
    if let Some(v) = values.iter().find(|v| **v == value) {
        *chosen = v;
    }
}

fn group(pen: &Pen<'_>, p: &Palette, x: f32, y: &mut f32, title: &str) {
    *y += 12.0;
    pen.text(x, *y, &title.to_uppercase(), XS, Weight::BOLD, p.muted);
    *y += line(XS) + 4.0;
}

fn fields(ui: &mut egui::Ui, pen: &Pen<'_>, p: &Palette, side: Rect, scene: &Scene, d: &mut Draft) {
    let x = side.left() + 16.0;
    let mut y = side.top() + 4.0;
    group(pen, p, x, &mut y, &tr!("newticket-where"));
    let projects: Vec<(String, String)> = scene.projects.iter().map(|q| (q.name.clone(), q.name.clone())).collect();
    field_row(ui, pen, p, side, &mut y, &tr!("tickets-field-project"), |ui, at| {
        let shown = d.project.clone();
        choose(ui, at, "project", shown, &projects, &mut d.project);
    });
    group(pen, p, x, &mut y, &tr!("tickets-group-fields"));
    field_row(ui, pen, p, side, &mut y, &tr!("tickets-field-intent"), |ui, at| choose_static(ui, at, "intent", "intent", &INTENTS, &mut d.intent));
    field_row(ui, pen, p, side, &mut y, &tr!("tickets-field-priority"), |ui, at| choose_static(ui, at, "priority", "priority", &PRIORITIES, &mut d.priority));
    field_row(ui, pen, p, side, &mut y, &tr!("tickets-field-level"), |ui, at| choose_static(ui, at, "level", "level", &LEVELS, &mut d.level));
    field_row(ui, pen, p, side, &mut y, &tr!("tickets-field-scope"), |ui, at| choose_static(ui, at, "scope", "scope", &SCOPES, &mut d.scope));
    // Under the scope, whom it notifies: small, muted, wrapped.
    let note = egui_pop::i18n::global().t(&format!("fields-scope-note-{}", d.scope), &[]);
    let galley = pen.painter.layout(note, font(XS, Weight::REGULAR), p.muted, 147.0);
    let h = galley.size().y;
    pen.painter.galley(pos2(side.left() + 104.0, y - 4.0), galley, p.muted);
    y += h + 4.0;
    group(pen, p, x, &mut y, &tr!("tickets-group-people"));
    let consumers: Vec<(String, String)> = std::iter::once((String::new(), tr!("tickets-pick-nobody")))
        .chain(scene.consumers.iter().filter(|c| c.project.as_deref().is_none_or(|p| p == d.project)).map(|c| (c.consumer_id.clone(), c.consumer_id.clone())))
        .collect();
    field_row(ui, pen, p, side, &mut y, &tr!("newticket-assign-to"), |ui, at| {
        let mut who = d.assign.clone().unwrap_or_default();
        let shown = d.assign.clone().unwrap_or_else(|| tr!("tickets-pick-nobody"));
        choose(ui, at, "assign", shown, &consumers, &mut who);
        d.assign = (!who.is_empty()).then_some(who);
    });
    group(pen, p, x, &mut y, &tr!("tickets-group-links"));
    field_row(ui, pen, p, side, &mut y, &tr!("tickets-sub-ticket-of"), |ui, at| {
        let at = Rect::from_min_size(at.min, vec2(120.0, at.height()));
        let edit = egui::TextEdit::singleline(&mut d.parent).hint_text("#ticket").font(font(SM, Weight::REGULAR)).margin(egui::vec2(10.0, 7.0)).desired_width(at.width());
        ui.put(at, edit);
    });
}

#[allow(clippy::too_many_arguments)]
fn write(ui: &mut egui::Ui, kit: &Kit, cx: &KitCx<'_>, pen: &Pen<'_>, p: &Palette, words: Rect, scene: &Scene, d: &mut Draft) {
    // The title: 33 high, the width of the words.
    let title = Rect::from_min_size(words.min, vec2(words.width(), 33.0));
    let edit = egui::TextEdit::singleline(&mut d.title)
        .id(egui::Id::unique("tvty.new-ticket.title"))
        .hint_text(tr!("fields-title"))
        .font(font(SM, Weight::REGULAR))
        .margin(egui::vec2(10.0, 8.0))
        .desired_width(title.width());
    let response = ui.put(title, edit);
    // Opened, the title has the keyboard.
    if !ui.memory(|m| m.focused().is_some()) {
        response.request_focus();
    }

    // Write / Preview: tabs 8 under, a line under them.
    let mut x = words.left();
    let ty = title.bottom() + 12.0;
    for (preview, label) in [(false, tr!("fields-write")), (true, tr!("fields-preview"))] {
        let w = pen.width(&label, SM, Weight::REGULAR) + 24.0;
        let tab = Rect::from_min_size(pos2(x, ty), vec2(w, line(SM) + 6.0));
        let r = ui.interact(tab, ui.scope_id().with(("tab", preview)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        if r.clicked() {
            d.preview = preview;
        }
        let state = State::from_response(&r).with(State::SELECTED, d.preview == preview);
        kit.tab.paint(cx, &Part { rect: tab, room: tab, state, label: None });
        pen.text(x + 12.0, ty + 2.0, &label, SM, Weight::REGULAR, kit.tab.style(state).text);
        x = tab.right();
    }
    let tabs_bottom = ty + line(SM) + 6.0;
    pen.painter.hline(words.x_range(), tabs_bottom + 0.5, egui::Stroke::new(1.0, p.border));

    // The body: 218 high, GPUI's.
    let body = Rect::from_min_size(pos2(words.left(), tabs_bottom + 9.0), vec2(words.width(), 218.0));
    if d.preview {
        kit.field.paint(cx, &Part { rect: body, room: body, state: State::REST, label: None });
        let galley = pen.painter.layout(d.body.clone(), font(SM, Weight::REGULAR), p.text, body.width() - 20.0);
        pen.painter.galley(body.min + vec2(10.0, 10.0), galley, p.text);
    } else {
        let edit = egui::TextEdit::multiline(&mut d.body)
            .hint_text(tr!("fields-body"))
            .font(font(SM, Weight::REGULAR))
            .margin(egui::vec2(10.0, 10.0))
            .desired_width(body.width());
        ui.put(body, edit);
    }

    // The keys, then "File and exit" and "File the ticket" at the end.
    let row = body.bottom() + 9.0;
    let mut status = tr!("newticket-keys-exit");
    let mut color = p.muted;
    if let Some(error) = &d.error {
        status = error.clone();
        color = p.danger;
    }
    pen.text(words.left(), row + 3.0, &status, XS, Weight::REGULAR, color);
    let button = |ui: &mut egui::Ui, end: f32, text: &str, tone: Tone, id: &str| -> (egui::Response, f32) {
        let w = pen.width(text, SM, Weight::REGULAR) + 16.0;
        let r = Rect::from_min_size(pos2(end - w, row), vec2(w, line(SM) + 1.0));
        let response = ui.interact(r, ui.scope_id().with(id), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        let state = State::from_response(&response).toned(tone);
        kit.button.paint(cx, &Part { rect: r, room: r, state, label: None });
        pen.text(r.left() + 8.0, r.top(), text, SM, Weight::REGULAR, kit.button.style(state).text);
        (response, r.left() - 8.0)
    };
    let (file_open, end) = button(ui, words.right(), &tr!("newticket-file"), Tone::Accent, "file");
    let (file_exit, _) = button(ui, end, &tr!("newticket-file-exit"), Tone::Warning, "file-exit");
    let file_exit = file_exit.tip(tr!("newticket-file-exit-tip"));
    let ctrl_enter = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter));
    if file_open.clicked() || file_exit.clicked() || ctrl_enter {
        match file(ui.ctx(), scene, d) {
            Ok((_, id)) => {
                let project = d.project.clone();
                *d = Draft { project, ..Draft::default() };
                // "File the ticket" opens it in the panel; the others go back.
                if file_open.clicked() {
                    crate::fulllist::set_open(ui.ctx(), false);
                    crate::detail::set_shown(ui.ctx(), Some(id));
                }
            }
            Err(e) => d.error = Some(e),
        }
    }
}
