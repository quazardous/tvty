//! tvty's window on egui-pop: the shell's regions as egui-pop zones, its
//! look from tokens. An experiment (docs/EGUI.md): nothing is painted
//! around egui-pop; what it cannot express is changed in egui-pop.

use egui_pop::api::frame::FramePlugin;
use egui_pop::eframe;
use egui_pop::egui;
use egui_pop::prelude::*;

mod scene;
mod shell;

fn main() -> eframe::Result {
    let scene = scene::Scene::load();
    let dir = std::env::var_os("TVTY_EGUI_STATE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("tvty-egui"));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(shell::NAME)
            .with_inner_size([1280.0, 800.0])
            .with_decorations(false),
        ..Default::default()
    };
    App::builder()
        .name(shell::NAME)
        .plugins(MinimalPlugins)
        .plugin(FramePlugin::own())
        .plugin(shell::Shell::new(scene))
        .persist(dir)
        .run_native(shell::NAME, options)
}
