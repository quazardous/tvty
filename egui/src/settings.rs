//! tvty's settings, the page ⚙ opens over the window (`settings.rs` in the
//! GPUI build). Not drawn yet: only whether it is open.

use egui_pop::egui;

const OPEN: &str = "tvty.settings.open";

pub fn is_open(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::unique(OPEN))).unwrap_or(false)
}

pub fn set_open(ctx: &egui::Context, open: bool) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique(OPEN), open));
}
