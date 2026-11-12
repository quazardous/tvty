//! tvty's window on egui-pop: the shell's regions as egui-pop zones, its
//! look from tokens. An experiment (docs/EGUI.md): nothing is painted
//! around egui-pop; what it cannot express is changed in egui-pop.

use egui_pop::api::frame::FramePlugin;
use egui_pop::eframe;
use egui_pop::egui;
use egui_pop::prelude::*;

/// The theme tvty starts in: the kit's Default Dark, as GPUI draws it.
pub(crate) const THEME: &str = "tvty-default-dark";

mod audit;
mod board;
mod detail;
mod fulllist;
mod i18n;
mod panel;
mod icons;
mod menu;
mod notice;
mod newticket;
mod message;
mod row;
mod tip;
mod scene;
mod settings;
mod shell;
mod sidebar;

/// Whether section stacks fill their panel (egui-pop's default) rather than
/// size by their content as GPUI's do: `TVTY_EGUI_FILL=1`, to compare.
pub(crate) fn sections_fill() -> bool {
    std::env::var_os("TVTY_EGUI_FILL").is_some_and(|v| v == "1")
}

fn main() -> eframe::Result {
    let scene = scene::Scene::load();
    let dir = std::env::var_os("TVTY_EGUI_STATE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("tvty-egui"));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(shell::NAME)
            // The launcher's StartupWMClass: the dock groups the window under it.
            .with_app_id("tvty-egui")
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
        // As tvty starts: the sessions folded to their strip (open with
        // TVTY_EGUI_SIDEBAR, to compare), lying over the centre when
        // open; the tickets panel at its width.
        .folded(if std::env::var_os("TVTY_EGUI_SIDEBAR").is_some() { &[] } else { &[egui_pop::model::panels::Area::StartSide] })
        .over(&[egui_pop::model::panels::Area::StartSide])
        // Folded, the sessions keep only their grip, as GPUI's strip.
        .bare_strips(&[egui_pop::model::panels::Area::StartSide, egui_pop::model::panels::Area::EndSide])
        // GPUI's bounds: the sessions 230 to 60 % of the window, the
        // tickets at least 260, the centre at least 320.
        .shell_panel_min_size(egui_pop::api::workbench::PANEL_SIDE_BAR, 230.0)
        .shell_panel_max_size(egui_pop::api::workbench::PANEL_SIDE_BAR, egui_pop::api::workbench::ShellSize::Share(0.6))
        .shell_panel_min_size(egui_pop::api::workbench::PANEL_INSPECTOR, 260.0)
        .centre_min_size(320.0)
        .shell_panel_size(egui_pop::api::workbench::PANEL_SIDE_BAR, sidebar::WIDTH)
        // GPUI's 428 count the window's outline, inside which egui-pop lays out.
        .shell_panel_size(egui_pop::api::workbench::PANEL_INSPECTOR, 427.0)
        .plugins(MinimalPlugins)
        .plugin(FramePlugin::own())
        .plugin(i18n::plugin())
        .plugin(egui_pop::api::toasts::ToastsPlugin)
        .plugin(shell::Shell::new(scene))
        .persist(dir)
        .run_native(shell::NAME, options)
}
