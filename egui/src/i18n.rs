//! tvty's words in the language chosen, through egui-pop's translations
//! (`egui_pop::i18n`): the GPUI build's catalogues
//! (`assets/locales/<lang>/*.ftl`, built in by `build.rs`), English the
//! reference. The title bar's language menu and `TVTY_LANG` choose, as in
//! the GPUI build; the choice is egui-pop's `appearance.language` setting.

use egui_pop::api::AppHandle;
use egui_pop::i18n::{Catalogue, I18nPlugin, plugin};
use egui_pop::model::values::Value;

include!(concat!(env!("OUT_DIR"), "/locales.rs"));

/// The languages tvty speaks: their code and their name in themselves.
pub const LANGS: &[(&str, &str)] = &[("en", "English"), ("fr", "Français"), ("es", "Español"), ("de", "Deutsch")];

pub fn plugin() -> I18nPlugin {
    I18nPlugin::new().catalogues(CATALOGUES).env("TVTY_LANG")
}

/// The setting: `system` or a language's code.
pub fn chosen(app: &AppHandle) -> String {
    let path = format!("settings/{}", plugin::SETTING_LANGUAGE);
    match egui_pop::core::lock(&app.store()).get(&path).map(|e| e.value.clone()) {
        Some(Value::Choice(c)) if !c.is_empty() => c,
        _ => plugin::SYSTEM.to_string(),
    }
}

pub fn choose(app: &AppHandle, value: &str) {
    if let Err(e) = plugin::set_language(app, value) {
        log::warn!("language {value}: {e}");
    }
}

/// The language the system speaks, as tvty can: its index in `LANGS`.
pub fn system() -> usize {
    let code = egui_pop::i18n::locale::system().unwrap_or_default().get(..2).unwrap_or("").to_ascii_lowercase();
    LANGS.iter().position(|(c, _)| *c == code).unwrap_or(0)
}

/// The language spoken now: its code, two letters ("en").
pub fn spoken(ctx: &egui_pop::egui::Context) -> String {
    egui_pop::i18n::plugin::of(ctx).map_or_else(|| "en".to_string(), |i| i.locale().get(..2).unwrap_or("en").to_string())
}
