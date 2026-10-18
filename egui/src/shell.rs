//! The shell: the title bar's own part, the sessions on the start side,
//! the terminal's place in the centre, the tickets on the end side.

use egui_pop::egui;
use egui_pop::prelude::*;

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
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        egui::Stroke::new(1.0, t.border.default),
    );
    let mid = rect.top() + 17.5;
    let font = |size: f32, weight| {
        egui_pop::core::fonts::font_id(egui_pop::core::fonts::Role::Ui, weight, size)
    };
    let regular = egui_pop::core::fonts::Weight::REGULAR;
    let text = |x: f32, align: egui::Align2, s: &str, f: egui::FontId, c| {
        painter.text(egui::pos2(x, mid), align, s, f, c)
    };
    let icon = |left: f32, icon: Option<egui_pop::core::icon::Icon>, c| {
        let r = egui::Rect::from_min_size(egui::pos2(left, mid - 7.5), egui::vec2(15.0, 15.0));
        if let Some(i) = icon {
            i.paint(painter, r, c);
        }
    };
    // From the start: a 20 wide button, an icon button (4 + 15 + 4), the title.
    let mut x = rect.left() + 12.0;
    text(x + 10.0, egui::Align2::CENTER_CENTER, "›", font(14.0, regular), t.text.muted);
    x += 20.0 + 8.0;
    icon(x + 4.0, crate::icons::pinned(), t.accent.default);
    x += 23.0 + 8.0;
    text(
        x,
        egui::Align2::LEFT_CENTER,
        "Tickets",
        font(14.0, egui_pop::core::fonts::Weight::BOLD),
        t.text.default,
    );
    // From the end.
    let mut x = rect.right() - 12.0;
    // A glyph button: at least 20 wide, 4 on each side of its glyph.
    let full = painter.layout_no_wrap("⤢".into(), font(14.0, regular), t.text.muted);
    let width = (full.size().x + 8.0).max(20.0);
    text(x - width / 2.0, egui::Align2::CENTER_CENTER, "⤢", font(14.0, regular), t.text.muted);
    x -= width + 8.0;
    painter.vline(
        x - 4.5,
        egui::Rangef::new(mid - 6.0, mid + 6.0),
        egui::Stroke::new(1.0, t.border.default),
    );
    x -= 9.0 + 8.0;
    let label = painter.layout_no_wrap("+ New".into(), font(12.0, regular), t.accent.default);
    let chip = egui::Rect::from_min_max(
        egui::pos2(x - label.size().x - 14.0, mid - 10.5),
        egui::pos2(x, mid + 10.5),
    );
    painter.rect_stroke(
        chip,
        4.0,
        egui::Stroke::new(1.0, t.border.default),
        egui::StrokeKind::Inside,
    );
    painter.galley(
        egui::pos2(chip.left() + 7.0, mid - label.size().y / 2.0),
        label,
        t.accent.default,
    );
    x = chip.left() - 8.0;
    icon(x - 19.0, crate::icons::megaphone(), t.text.muted);
}
