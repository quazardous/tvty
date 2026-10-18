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
        let s = scene.clone();
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
                ui.centered_and_justified(|ui| ui.weak("Pick a session"));
            }),
        );
        let s = scene;
        ctx.block(
            zones::INSPECTOR,
            Block::add(BlockId::from_static("tvty.tickets"), 0, move |ui, _| {
                for project in &s.projects {
                    for ticket in &project.tickets {
                        ui.label(format!("{}  {}", ticket.id, ticket.title));
                    }
                }
            }),
        );
        let _ = egui::Id::NULL;
        Ok(())
    }
}
