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
        let scene = std::rc::Rc::new(self.scene.take().unwrap_or_default());
        let user = if scene.user.is_empty() {
            "you".to_string()
        } else {
            scene.user.clone()
        };
        let s = scene;
        ctx.block(
            zones::SIDE_BAR,
            Block::add(BlockId::from_static("tvty.sessions"), 0, move |ui, _| {
                for project in &s.projects {
                    ui.strong(&project.name);
                    for session in &project.sessions {
                        ui.label(&session.name);
                    }
                }
            }),
        );
        ctx.block(
            zones::MAIN,
            Block::add(BlockId::from_static("tvty.terminal"), 0, |ui, _| {
                // Where a terminal would be, before one is picked.
                let muted = Tokens::of(ui.ctx()).text.muted;
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(
                            "Pick a terminal on the left · ctrl+shift+space shows them all",
                        )
                        .size(16.0)
                        .color(muted),
                    )
                });
            }),
        );
        ctx.block(
            zones::INSPECTOR,
            Block::add(BlockId::from_static("tvty.tickets"), 0, |ui, _| {
                tickets_header(ui);
                // No terminal is picked at start: no project to show.
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
        // tvty's icon where the shell puts its menu button and name: the
        // window's title follows it (the kit's `title-bar.align: start`).
        ctx.block(
            zones::CAPTION_START,
            Block::replace(
                egui_pop::api::frame::BLOCK_MENU,
                BlockId::from_static("tvty.app-icon"),
                |ui, _| {
                    ui.add_space(10.0);
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                    if let Some(icon) = crate::icons::app() {
                        icon.paint(ui.painter(), rect, egui::Color32::WHITE);
                    }
                },
            ),
        );
        ctx.block(
            zones::CAPTION,
            Block::add(BlockId::from_static("tvty.caption"), 0, move |ui, _| {
                caption(ui, &user)
            }),
        );
        Ok(())
    }
}

/// The tickets panel's header, as the GPUI build lays it out: 36 high with
/// a line under it, 12 from the sides, 8 between items; the fold chevron,
/// the pin, the title, then at the end the megaphone, "+ New", a separator
/// and the full list.
fn tickets_header(ui: &mut egui::Ui) {
    let t = Tokens::of(ui.ctx());
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
    row.glyph(From::Start, "›", font(14.0, regular), muted, 8.0);
    row.icon(From::Start, crate::icons::pinned(), t.accent.default, 8.0);
    row.text(From::Start, "Tickets", font(14.0, bold), t.text.default, 8.0);
    row.glyph(From::End, "⤢", font(14.0, regular), muted, 0.0);
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
    painter.rect_stroke(chip, 4.0, line, egui::StrokeKind::Inside);
    painter.galley(
        egui::pos2(chip.left() + 7.0, mid - label.size().y / 2.0),
        label,
        t.accent.default,
    );
    row.icon(From::End, crate::icons::megaphone(), muted, 8.0);
}

/// The title bar's items before the window buttons, as the GPUI build has
/// them: the ticket to go to (a field), the theme, the language, a message
/// to every agent, the settings and who tvty acts as.
fn caption(ui: &mut egui::Ui, user: &str) {
    let t = Tokens::of(ui.ctx());
    let regular = Weight::REGULAR;
    let muted = t.text.muted;
    // As wide as the GPUI build's items, from the field to the user.
    let width = 96.0 + 8.0 + 98.0 + 8.0 + 25.0 + 8.0 + 23.0 + 4.0 + 22.0 + 8.0;
    let user_width = ui
        .painter()
        .layout_no_wrap(user.into(), font(14.0, regular), muted)
        .size()
        .x;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width + user_width, ui.available_height()),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    let mut row = Row::new(painter, rect, 0.0, rect.center().y);
    // Its own 8 to the buttons are the title bar's gap before them.
    row.text(From::End, user, font(14.0, regular), muted, 0.0);
    row.glyph(From::End, "⚙", font(14.0, regular), muted, 8.0);
    row.icon(From::End, crate::icons::message_all(), muted, 4.0);
    row.glyph(From::End, "EN", font(12.0, regular), muted, 8.0);
    row.glyph(From::End, "◐ Default Dark", font(12.0, regular), muted, 8.0);
    // The field: 96 wide, 20 high, a light fill even at rest.
    let field = row.room(From::End, 96.0, 8.0);
    painter.rect_filled(field, 6.0, t.surface.hover);
    painter.text(
        egui::pos2(field.left() + 4.0, row.mid()),
        egui::Align2::LEFT_CENTER,
        "#...",
        font(12.0, regular),
        muted,
    );
}
