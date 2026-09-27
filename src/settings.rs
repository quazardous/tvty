//! tvty's sets of settings, each kept by [`crate::config`]:
//!
//! - [`Preferences`], the user's, in `~/.config/tvty/settings.toml` — the
//!   options page writes it, the user may edit it, tvty reads it again;
//! - [`Layout`] and [`Workspace`], what tvty remembers on its own, in
//!   `~/.local/state/tvty/`.
//!
//! An older tvty kept them all in `~/.config/tvty/state.json`: they are
//! taken from there once, and the file is renamed `state.json.migrated`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::{Place, Stored, dir};

// ── The user's preferences ───────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub appearance: Appearance,
    pub notifications: Notifications,
    pub scroll: Scroll,
    pub tickets: Tickets,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    /// The colour theme's name; none: the kit's default dark theme.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// The terminals' own colour theme; none: the window's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_theme: Option<String>,
    /// The terminals' font size and the window's text size, in pixels;
    /// none: the defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_font_size: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Notifications {
    /// Shown at most, and for how many seconds; none: the defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds: Option<u64>,
    /// The user's own gestures (closed, replied, decided…) notified too.
    pub own: bool,
}

impl Default for Notifications {
    fn default() -> Self {
        Self { max: None, seconds: None, own: true }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Scroll {
    /// The mouse wheel's speed, times tvty's own (1: a notch is about three
    /// ticket rows, five lines of a terminal's history).
    pub speed: f32,
}

impl Default for Scroll {
    fn default() -> Self {
        Self { speed: 1. }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tickets {
    /// A ticket's thread shows its newest word first.
    pub newest_first: bool,
}

impl Stored for Preferences {
    const PLACE: Place = Place::Config;
    const FILE: &'static str = "settings.toml";
    const EDITED: bool = true;

    fn migrate() -> Option<Self> {
        let old = old_state()?;
        let (str_of, f32_of) = (|k: &str| old.get(k)?.as_str().map(String::from), |k: &str| old.get(k)?.as_f64().map(|v| v as f32));
        Some(Self {
            appearance: Appearance {
                theme: str_of("theme"),
                terminal_theme: str_of("terminal_theme"),
                terminal_font_size: f32_of("terminal_font_size"),
                window_font_size: f32_of("window_font_size"),
            },
            notifications: Notifications {
                max: old.get("notify_max").and_then(Value::as_u64).map(|v| v as usize),
                seconds: old.get("notify_seconds").and_then(Value::as_u64),
                own: old.get("notify_own").and_then(Value::as_bool).unwrap_or(true),
            },
            scroll: Scroll { speed: f32_of("scroll_speed").unwrap_or(1.) },
            tickets: Tickets { newest_first: old.get("thread_newest_first").and_then(Value::as_bool).unwrap_or(false) },
        })
    }
}

// ── What tvty remembers ──────────────────────────────────────────────

/// The window's arrangement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Layout {
    /// The ticket panel's width, in pixels; none until the user drags it:
    /// a third of the window.
    pub panel_width: Option<f32>,
    pub panel_open: bool,
    /// The projects' list, on the left, over the terminal.
    pub sidebar_open: bool,
    /// Its width, in pixels; none: the default.
    pub sidebar_width: Option<f32>,
    /// The sessions list's folded sections (`live`, `idle`, `shut`).
    pub sessions_folded: Vec<String>,
    /// The accordions' sections given a height by hand (`list/section`).
    pub section_heights: HashMap<String, f32>,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            panel_width: None,
            panel_open: true,
            sidebar_open: true,
            sidebar_width: None,
            sessions_folded: vec!["idle".into(), "shut".into()],
            section_heights: HashMap::new(),
        }
    }
}

impl Stored for Layout {
    const PLACE: Place = Place::State;
    const FILE: &'static str = "layout.json";

    fn migrate() -> Option<Self> {
        let old = old_state()?;
        let default = Self::default();
        Some(Self {
            panel_width: old.get("panel_width").and_then(Value::as_f64).map(|v| v as f32),
            panel_open: old.get("panel_open").and_then(Value::as_bool).unwrap_or(default.panel_open),
            sidebar_open: old.get("sidebar_open").and_then(Value::as_bool).unwrap_or(default.sidebar_open),
            sidebar_width: old.get("sidebar_width").and_then(Value::as_f64).map(|v| v as f32),
            sessions_folded: old.get("sessions_folded").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or(default.sessions_folded),
            section_heights: old.get("section_heights").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
        })
    }
}

/// The work as it was left.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    /// The terminals open, the one used last first, and the one shown;
    /// opened again at start.
    pub open_terminals: Vec<String>,
    pub shown_terminal: Option<String>,
    /// The project the last new ticket went to.
    pub last_ticket_project: Option<String>,
}

impl Stored for Workspace {
    const PLACE: Place = Place::State;
    const FILE: &'static str = "workspace.json";

    fn migrate() -> Option<Self> {
        let old = old_state()?;
        Some(Self {
            open_terminals: old.get("open_terminals").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
            shown_terminal: old.get("shown_terminal").and_then(Value::as_str).map(String::from),
            last_ticket_project: old.get("last_ticket_project").and_then(Value::as_str).map(String::from),
        })
    }
}

// ── The older file ───────────────────────────────────────────────────

fn old_state_path() -> Option<std::path::PathBuf> {
    Some(dir(Place::Config)?.join("state.json"))
}

/// What an older tvty kept in `~/.config/tvty/state.json`.
fn old_state() -> Option<serde_json::Map<String, Value>> {
    let text = std::fs::read_to_string(old_state_path()?).ok()?;
    serde_json::from_str::<Value>(&text).ok()?.as_object().cloned()
}

/// Once its content is taken: the older file goes aside, never read again.
pub fn retire_old_state() {
    let Some(path) = old_state_path().filter(|p| p.exists()) else { return };
    let aside = path.with_extension("json.migrated");
    match std::fs::rename(&path, &aside) {
        Ok(()) => log::info!("settings: {} taken apart, kept as {}", path.display(), aside.display()),
        Err(e) => log::warn!("settings: {}: {e}", path.display()),
    }
}

/// The three sets, as the shell holds them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub prefs: Preferences,
    pub layout: Layout,
    pub workspace: Workspace,
}

impl Settings {
    /// The sets as they are kept now.
    pub fn current(cx: &gpui_kit::App) -> Self {
        Self {
            prefs: crate::config::get::<Preferences>(cx).clone(),
            layout: crate::config::get::<Layout>(cx).clone(),
            workspace: crate::config::get::<Workspace>(cx).clone(),
        }
    }

    /// Keeps the sets: each written once its changes rest; one unchanged
    /// is not written.
    pub fn save(&self, cx: &mut gpui_kit::App) {
        crate::config::update::<Preferences>(cx, |p| *p = self.prefs.clone());
        crate::config::update::<Layout>(cx, |l| *l = self.layout.clone());
        crate::config::update::<Workspace>(cx, |w| *w = self.workspace.clone());
    }
}

/// Says what was wrong with a file read at start, once the window listens.
pub fn report_errors(cx: &mut gpui_kit::App) {
    crate::config::report::<Preferences>(cx);
    crate::config::report::<Layout>(cx);
    crate::config::report::<Workspace>(cx);
    crate::config::report::<crate::keymap::KeymapFile>(cx);
}

/// Registers the sets, taking an older tvty's file apart the first time.
pub fn init(cx: &mut gpui_kit::App) {
    crate::config::register::<Preferences>(cx);
    crate::config::register::<Layout>(cx);
    crate::config::register::<Workspace>(cx);
    retire_old_state();
}

#[cfg(test)]
mod tests {
    use super::Preferences;
    use crate::config::{parse, render};

    #[test]
    fn preferences_read_as_sections() {
        let prefs: Preferences = parse("[appearance]\ntheme = \"Everforest Dark\"\n\n[scroll]\nspeed = 2.0\n").unwrap();
        assert_eq!(prefs.appearance.theme.as_deref(), Some("Everforest Dark"));
        assert_eq!(prefs.scroll.speed, 2.);
        // Left out: the defaults.
        assert!(prefs.notifications.own);
        let text = render(&prefs).unwrap();
        assert!(text.contains("[appearance]") && text.contains("[notifications]"), "{text}");
    }
}
