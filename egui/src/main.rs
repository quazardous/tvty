//! tvty's window on egui-pop: the shell's regions as egui-pop zones, its
//! look from tokens. An experiment (docs/EGUI.md): nothing is painted
//! around egui-pop; what it cannot express is changed in egui-pop.

use egui_pop::api::frame::FramePlugin;
use egui_pop::eframe;
use egui_pop::egui;
use egui_pop::prelude::*;

/// The theme tvty starts in: the kit's Default Dark, as GPUI draws it.
const THEME: &str = "tvty-default-dark";

mod icons;
mod row;
mod tip;
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
    // Registered before the theme is installed: its textures are made then.
    icons::register();
    let mut themes = egui_pop::tokens::Themes::default();
    themes
        .add_text(
            THEME,
            include_str!("../themes/tvty-default-dark.tokens.yaml"),
        )
        .expect("tvty's theme reads");
    App::builder()
        .name(shell::NAME)
        .themes(themes)
        .theme(ThemeSpec::new(THEME))
        .menu_bar(false)
        // As tvty starts: the sessions folded to their strip, the tickets
        // panel at its width.
        .folded(&[egui_pop::model::panels::Area::StartSide])
        // GPUI's 428 count the window's outline, inside which egui-pop lays out.
        .shell_panel_size(egui_pop::api::workbench::PANEL_INSPECTOR, 427.0)
        .plugins(MinimalPlugins)
        .plugin(FramePlugin::own())
        .plugin(shell::Shell::new(scene))
        .persist(dir)
        .run_native(shell::NAME, options)
}
