//! The shell: the title bar's own part, the sessions on the start side,
//! the terminal's place in the centre, the tickets on the end side.

use egui_pop::egui;
use egui_pop::prelude::*;

use egui_pop::core::fonts::Weight;

use crate::row::{From, Row, font};
use crate::scene::Scene;

pub const NAME: &str = "Terminal Velocity";

pub struct Shell {
    scene: Option<Scene>,
}

impl Shell {
    pub fn new(scene: Scene) -> Self {
        Self { scene: Some(scene) }
    }
}

impl Plugin for Shell {
    fn id(&self) -> PluginId {
        PluginId::from_static("tvty.shell")
    }

    fn connect(&mut self, ctx: &mut Ctx<'_>) -> Result<(), PluginError> {
        crate::menu::declare(ctx)?;
        let scene = std::rc::Rc::new(self.scene.take().unwrap_or_default());
        let user = if scene.user.is_empty() {
            "you".to_string()
        } else {
            scene.user.clone()
        };
        let s = scene.clone();
        ctx.block(
            zones::SIDE_BAR,
            Block::add(BlockId::from_static("tvty.sessions"), 0, move |ui, app| crate::sidebar::show(ui, &s, app)),
        );
        ctx.block(
            zones::MAIN,
            Block::add(BlockId::from_static("tvty.terminal"), 0, |ui, app| {
                // Tips off, as the tests' tvty has them: TVTY_EGUI_TIPS=0.
                let tips = std::env::var_os("TVTY_EGUI_TIPS").is_none_or(|v| v != "0");
                if tips && !crate::fulllist::is_open(ui.ctx()) {
                    crate::tip::show(ui.ctx());
                }
                // Where a terminal would be, before one is picked: centred
                // on the terminal's place, which in the GPUI build runs
                // under the folded sessions' strip and 4 points under the
                // tickets panel's edge, where its sash sits.
                let muted = Tokens::of(ui.ctx()).text.muted;
                let area = ui.max_rect();
                let left = ui.ctx().content_rect().left() + 1.0;
                let right = app
                    .area_rect(egui_pop::model::panels::Area::EndSide)
                    .map_or(area.right(), |r| r.left() + 4.0);
                ui.painter().text(
                    egui::pos2((left + right) / 2.0, area.center().y),
                    egui::Align2::CENTER_CENTER,
                    "Pick a terminal on the left · ctrl+shift+space shows them all",
                    font(16.0, Weight::REGULAR),
                    muted,
                );
            }),
        );
        let panel_scene = scene.clone();
        ctx.block(
            zones::INSPECTOR,
            Block::add(BlockId::from_static("tvty.tickets"), 0, move |ui, app| {
                let project = crate::panel::project(&panel_scene);
                tickets_header(ui, &panel_scene, project.as_deref(), app);
                if let Some(name) = &project {
                    match crate::detail::shown(ui.ctx()) {
                        Some(id) => crate::detail::show(ui, &panel_scene, name, id),
                        None => crate::panel::show(ui, &panel_scene, name),
                    }
                    return;
                }
                // No terminal is picked: no project to show.
                let muted = Tokens::of(ui.ctx()).text.muted;
                egui::Frame::new().inner_margin(12).show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("No aiball project on this terminal.")
                            .size(14.0)
                            .color(muted),
                    );
                });
            }),
        );
        // The full list, over the window under its title bar, while open.
        ctx.block(
            zones::OVERLAYS,
            Block::add(BlockId::from_static("tvty.full-list"), 0, move |ui, _| {
                crate::fulllist::show(ui.ctx(), &scene)
            }),
        );
        // tvty's icon where the shell puts its menu button and name: the
        // window's title follows it (the kit's `title-bar.align: start`).
        ctx.block(
            zones::CAPTION_START,
            Block::replace(
                egui_pop::api::frame::BLOCK_MENU,
                BlockId::from_static("tvty.app-icon"),
                |ui, app| {
                    // GPUI's: 20 points, from 16 inside the window; a
                    // click opens tvty's menu under it.
                    ui.add_space(8.0);
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
                    if response.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text("Menu: about, help, restart, quit · F1").clicked() {
                        crate::menu::open(app);
                    }
                    if let Some(icon) = crate::icons::app() {
                        icon.paint(ui.painter(), rect, egui::Color32::WHITE);
                    }
                },
            ),
        );
        ctx.block(
            zones::CAPTION,
            Block::add(BlockId::from_static("tvty.caption"), 0, move |ui, app| {
                caption(ui, &user, app)
            }),
        );
        Ok(())
    }
}

/// The tickets panel's header, as the GPUI build lays it out: 36 high with
/// a line under it, 12 from the sides, 8 between items; the fold chevron,
/// the pin, the title, then at the end the megaphone, "+ New", a separator
/// and the full list.
fn tickets_header(ui: &mut egui::Ui, scene: &crate::scene::Scene, project: Option<&str>, app: &egui_pop::api::AppHandle) {
    let t = Tokens::of(ui.ctx());
    ui.spacing_mut().item_spacing.y = 0.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 36.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    let line = egui::Stroke::new(1.0, t.border.default);
    painter.hline(rect.x_range(), rect.bottom() - 0.5, line);
    let (regular, bold) = (Weight::REGULAR, Weight::BOLD);
    let mut row = Row::new(painter, rect, 12.0, rect.top() + 17.5);
    let muted = t.text.muted;
    // Each button's hover fill goes under it: its place kept now, filled
    // once the button's rectangle is known.
    let under = || painter.add(egui::Shape::Noop);
    // › folds the panel to its strip.
    let slot = under();
    let fold = row.glyph(From::Start, "›", font(14.0, regular), muted, 8.0);
    if crate::row::button(ui, painter, slot, fold.expand(4.0), "fold").on_hover_text("Fold the ticket panel").clicked() {
        app.fold_area(egui_pop::model::panels::Area::EndSide, true);
    }
    row.icon(From::Start, crate::icons::pinned(), t.accent.default, 8.0);
    match project {
        Some(name) => crate::panel::header_title(ui, scene, name, row.start(), row.mid()),
        None => {
            row.text(From::Start, "Tickets", font(14.0, bold), t.text.default, 8.0);
        }
    }
    let slot = under();
    let full = row.glyph(From::End, "⤢", font(14.0, regular), muted, 0.0);
    if crate::row::button(ui, painter, slot, full.expand(4.0), "full-list").on_hover_text("The ticket list, full screen · ctrl+shift+space").clicked() {
        crate::fulllist::set_open(ui.ctx(), true);
    }
    // The separator: 1 wide, 4 on each side, 12 high.
    let sep = row.room(From::End, 9.0, 8.0);
    let mid = row.mid();
    painter.vline(sep.center().x, egui::Rangef::new(mid - 6.0, mid + 6.0), line);
    // "+ New": a chip, 6 and a border on each side of its label.
    let label = painter.layout_no_wrap("+ New".into(), font(12.0, regular), t.accent.default);
    let chip = row
        .room(From::End, label.size().x.ceil() + 14.0, 8.0)
        .with_min_y(mid - 10.5)
        .with_max_y(mid + 10.5);
    let slot = under();
    crate::row::button(ui, painter, slot, chip, "new").on_hover_text("A new ticket · Ctrl+Shift+N");
    painter.rect_stroke(chip, 4.0, line, egui::StrokeKind::Inside);
    painter.galley(
        egui::pos2(chip.left() + 7.0, mid - label.size().y / 2.0),
        label,
        t.accent.default,
    );
    let slot = under();
    let megaphone = row.icon(From::End, crate::icons::megaphone(), muted, 8.0);
    if crate::row::button(ui, painter, slot, megaphone.expand(4.0), "message-all").on_hover_text("The project's standing instruction and wake focus").clicked() {
        crate::message::toggled(ui.ctx());
    }
}


/// The title bar's items before the window buttons, as the GPUI build has
/// them: the ticket to go to (a field), the theme, the language, a message
/// to every agent, the settings and who tvty acts as.
fn caption(ui: &mut egui::Ui, user: &str, app: &egui_pop::api::AppHandle) {
    let t = Tokens::of(ui.ctx());
    let regular = Weight::REGULAR;
    let muted = t.text.muted;
    // The theme's name as GPUI says it: tvty's own is "Default Dark".
    let theme = app.theme();
    let shown = theme.shown(ui.ctx());
    let name = if shown == crate::THEME { "Default Dark".to_string() } else { shown.clone() };
    let label = format!("◐ {name}");
    let label_width = ui.painter().layout_no_wrap(label.clone(), font(12.0, regular), muted).size().x.ceil();
    // As wide as the GPUI build's items, from the field to the user.
    let width = 96.0 + 8.0 + label_width + 8.0 + 25.0 + 8.0 + 23.0 + 4.0 + 22.0 + 8.0;
    let user_width = ui
        .painter()
        .layout_no_wrap(user.into(), font(14.0, regular), muted)
        .size()
        .x;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width + user_width, ui.available_height()),
        egui::Sense::hover(),
    );
    let painter = ui.painter().clone();
    let mut row = Row::new(&painter, rect, 0.0, rect.center().y);
    // Its own 8 to the buttons are the title bar's gap before them.
    row.text(From::End, user, font(14.0, regular), muted, 0.0);
    let under = || painter.add(egui::Shape::Noop);
    // ⚙: the settings.
    let slot = under();
    let cog = row.glyph(From::End, "⚙", font(14.0, regular), muted, 8.0);
    if crate::row::button(ui, &painter, slot, cog.expand(4.0), "settings").on_hover_text("Settings · Ctrl+,").clicked() {
        crate::settings::set_open(ui.ctx(), true);
    }
    // The megaphone: the message to every agent, a popover.
    let slot = under();
    let megaphone = row.icon(From::End, crate::icons::message_all(), muted, 4.0);
    if crate::row::button(ui, &painter, slot, megaphone.shrink2(egui::vec2(2.0, 0.0)), "message-all").on_hover_text("A message to every running agent: send, send & hold, release holds").clicked() {
        crate::message::toggled(ui.ctx());
    }
    let slot = under();
    let language = row.glyph(From::End, "EN", font(12.0, regular), muted, 8.0);
    crate::row::button(ui, &painter, slot, language.expand(4.0), "language");
    // The theme: a click lists the themes, egui-pop's own after tvty's.
    let slot = under();
    let chip = row.glyph(From::End, &label, font(12.0, regular), muted, 8.0);
    let response = crate::row::button(ui, &painter, slot, chip.expand2(egui::vec2(4.0, 4.0)), "theme").on_hover_text("The colour themes — the next one");
    egui::Popup::menu(&response).show(|ui| {
        let mut names: Vec<String> = theme.themes().names().map(str::to_string).collect();
        names.sort_by_key(|n| (n != crate::THEME, n.clone()));
        for n in names {
            let said = if n == crate::THEME { "Default Dark".to_string() } else { n.clone() };
            if ui.add(egui_pop::core::widgets::SelectableRow::new(n == shown, said)).clicked() {
                theme.pick(ui.ctx(), &n);
                ui.close();
            }
        }
    });
    // The ticket to go to: 96 wide, 20 high, a light fill even at rest;
    // a number and Enter open it in the panel.
    let field = row.room(From::End, 96.0, 8.0);
    painter.rect_filled(field, 6.0, t.surface.hover);
    let id = ui.id().with("goto");
    let mut text = ui.data(|d| d.get_temp::<String>(id)).unwrap_or_default();
    let edit = egui::TextEdit::singleline(&mut text)
        .frame(egui::Frame::NONE)
        .hint_text(egui::RichText::new("#...").color(muted))
        .font(font(12.0, regular))
        .text_color(t.text.default)
        .margin(egui::vec2(4.0, 2.0));
    let output = ui.put(field, edit).on_hover_text("Go to a ticket: its number or a comment's #C. link, Enter · ctrl+shift+g");
    if output.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Ok(n) = text.trim().trim_start_matches('#').parse::<u64>() {
            crate::detail::set_shown(ui.ctx(), Some(n));
        }
        text.clear();
    }
    ui.data_mut(|d| d.insert_temp(id, text));
    crate::message::show(ui.ctx());
}
