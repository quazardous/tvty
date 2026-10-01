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

use tvty_config::{Kind, Schema, Setting};

use crate::config::{Place, Stored, dir};

// ── The user's preferences ───────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub appearance: Appearance,
    pub notifications: Notifications,
    pub scroll: Scroll,
    pub mouse: Mouse,
    pub tickets: Tickets,
    pub sessions: Sessions,
    pub updates: Updates,
    pub tips: Tips,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mouse {
    /// Whether the pointer gives the keyboard: `click`, `hover`; none: as
    /// the window manager does.
    pub focus: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tips {
    /// "Did you know?" once after start and on first entering a surface.
    pub show: bool,
}

impl Default for Tips {
    fn default() -> Self {
        Self { show: true }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Updates {
    /// Asks GitHub, at start and once a day, whether a newer Terminal
    /// Velocity is out.
    pub check: bool,
}

impl Default for Updates {
    fn default() -> Self {
        Self { check: true }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sessions {
    /// The groups in the order they were used, the latest first (the
    /// slider's, ctrl+tab), rather than alphabetical.
    pub recent_first: bool,
    /// Behind a proxy node: the sessions of aiball's hub are listed too (to
    /// read, not to open).
    pub show_hub: bool,
    /// When tvty quits with loops of this machine running: `stop` them,
    /// `keep` them running; none: ask.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_quit: Option<String>,
    /// At start, the loops stopped when tvty quit: `restart` them as they
    /// were (their hold put back), restart them `fresh` (on their own),
    /// `leave` them; none: ask.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_start: Option<String>,
}

impl Default for Sessions {
    fn default() -> Self {
        Self { recent_first: true, show_hub: false, on_quit: None, on_start: None }
    }
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
    /// The terminals' background, in percent (the rest lets the desktop
    /// through); none: opaque.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_opacity: Option<f32>,
    /// Behind a see-through terminal, the desktop blurred (where the
    /// compositor can: KDE).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub terminal_blur: bool,
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
    /// A brief "copied" when text goes to the clipboard.
    pub copied: bool,
}

impl Default for Notifications {
    fn default() -> Self {
        Self { max: None, seconds: None, own: true, copied: true }
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
    /// A ticket's thread, full screen, shows its newest word first.
    pub newest_first: bool,
    /// The same in the panel, which has an order of its own.
    pub panel_newest_first: bool,
    /// The ticket panel lies over the terminal, as the sessions' list does,
    /// rather than beside it.
    pub panel_overlay: bool,
    /// In the panel, a ticket's "Where it stands" shows its text at once,
    /// rather than folded to its title.
    pub summary_open: bool,
}

/// Every preference, declared once: its key in `settings.toml`, the options
/// page and group that show it, what it says, its kind and bounds. The
/// options pages are built from it, and a hand-edited file checked by it.
pub const SCHEMA: Schema = Schema(&[
    Setting {
        key: "appearance.terminal_font_size",
        page: "Appearance",
        group: "Sizes",
        label: "Terminal font",
        about: "The terminals' text. Ctrl+Shift+= / Ctrl+Shift+− / Ctrl+Shift+0 too.",
        kind: Kind::Number { min: 8., max: 32., step: 1., default: 14., unit: "px", integer: false, slider: false },
    },
    Setting {
        key: "appearance.window_font_size",
        page: "Appearance",
        group: "Sizes",
        label: "Window text",
        about: "Everything around the terminals — lists, tickets, menus.",
        kind: Kind::Number { min: 12., max: 22., step: 1., default: 16., unit: "px", integer: false, slider: false },
    },
    Setting {
        key: "notifications.max",
        page: "Appearance",
        group: "Notifications",
        label: "Shown at most",
        about: "In the terminal's top right corner (bottom left over a full screen), the newest in the corner; older ones make room.",
        kind: Kind::Number { min: 1., max: 10., step: 1., default: 5., unit: "", integer: true, slider: false },
    },
    Setting {
        key: "notifications.seconds",
        page: "Appearance",
        group: "Notifications",
        label: "Seconds shown",
        about: "Then it goes, unless the pointer is on it.",
        kind: Kind::Number { min: 2., max: 30., step: 1., default: 6., unit: "s", integer: true, slider: false },
    },
    Setting {
        key: "notifications.own",
        page: "Appearance",
        group: "Notifications",
        label: "Your own gestures",
        about: "A ticket closed, a reply posted, a plan accepted: said once aiball has it. A refusal is always said.",
        kind: Kind::Toggle { default: true, on: "shown", off: "hidden" },
    },
    Setting {
        key: "notifications.copied",
        page: "Appearance",
        group: "Notifications",
        label: "Copied",
        about: "A brief \"Copied to clipboard\" at the bottom when text goes to the clipboard (not for a mere selection).",
        kind: Kind::Toggle { default: true, on: "shown", off: "hidden" },
    },
    Setting {
        key: "scroll.speed",
        page: "Appearance",
        group: "Mouse",
        label: "Wheel speed",
        about: "Times tvty's own: at 1, a notch is about three ticket rows, five lines of a terminal's history.",
        kind: Kind::Number { min: 0.2, max: 5., step: 0.2, default: 1., unit: "×", integer: false, slider: false },
    },
    Setting {
        key: "mouse.focus",
        page: "Appearance",
        group: "Mouse",
        label: "Focus",
        about: "What gives the keyboard to the terminal or to a box to write in: a click, or the pointer moving over it (as a window manager's focus follows the mouse). By default, as the system does.",
        kind: Kind::Choice,
    },
    Setting {
        key: "appearance.theme",
        page: "Appearance",
        group: "Colours",
        label: "Window",
        about: "The window's colour theme. Ctrl+Shift+K steps through them.",
        kind: Kind::Choice,
    },
    Setting {
        key: "appearance.terminal_theme",
        page: "Appearance",
        group: "Colours",
        label: "Terminal",
        about: "The terminals' own, or the window's — a dark terminal in a light window.",
        kind: Kind::Choice,
    },
    Setting {
        key: "appearance.terminal_opacity",
        page: "Appearance",
        group: "Colours",
        label: "Terminal opacity",
        about: "Below 100 %, the desktop shows through the terminals' background, live. The lists, the tickets and the colours a program sets stay opaque.",
        kind: Kind::Number { min: 50., max: 100., step: 5., default: 100., unit: "%", integer: true, slider: true },
    },
    Setting {
        key: "appearance.terminal_blur",
        page: "Appearance",
        group: "Colours",
        label: "Blur behind",
        about: "Behind a see-through terminal, the desktop blurred. Only where the compositor offers it (KDE); elsewhere it shows sharp.",
        kind: Kind::Toggle { default: false, on: "blurred", off: "sharp" },
    },
    Setting {
        key: "updates.check",
        page: "Layout",
        group: "Updates",
        label: "Check for updates",
        about: "At start and once a day, asks GitHub whether a newer Terminal Velocity is out, and says so once (nothing else is sent). The updater (the menu's Updates…) installs it.",
        kind: Kind::Toggle { default: true, on: "check", off: "never" },
    },
    Setting {
        key: "tips.show",
        page: "Layout",
        group: "Tips",
        label: "Show tips",
        about: "\"Did you know?\": a short tip once after start, and the first time a page is opened; never one for what you already use. The menu's Tips… shows them all.",
        kind: Kind::Toggle { default: true, on: "show", off: "never" },
    },
    Setting {
        key: "sessions.recent_first",
        page: "Layout",
        group: "Sessions",
        label: "Order",
        about: "The projects in the list, the slider and the gallery: the one used last first, as ctrl+tab goes — or alphabetical. Also the ⇅ in the list's header.",
        kind: Kind::Toggle { default: true, on: "most recent first", off: "alphabetical" },
    },
    Setting {
        key: "sessions.show_hub",
        page: "Layout",
        group: "Sessions",
        label: "The hub's sessions",
        about: "When this machine reaches aiball through a proxy node: list the sessions that run on aiball's hub too, apart, under \"on hub\". They are read from here (state, tickets), never opened: a session is attached from its own machine. The sessions of other nodes are never listed.",
        kind: Kind::Toggle { default: false, on: "listed", off: "not listed" },
    },
    Setting {
        key: "sessions.on_quit",
        page: "Layout",
        group: "Sessions",
        label: "On quit",
        about: "When tvty quits with Claude Code loops of this machine running: ask, stop them (through aiball: they stay restartable), or keep them running.",
        kind: Kind::Choice,
    },
    Setting {
        key: "sessions.on_start",
        page: "Layout",
        group: "Sessions",
        label: "On start",
        about: "The loops tvty stopped when it quit: ask, restart them as they were (resuming their conversation, a held one held again), restart them fresh (booting, then on their own), or leave them stopped.",
        kind: Kind::Choice,
    },
    Setting {
        key: "tickets.panel_overlay",
        page: "Layout",
        group: "Ticket panel",
        label: "Place",
        about: "Beside the terminal, which is then narrower — or over it, as the sessions' list on the left: the terminal keeps its whole width and the panel covers its right side while open.",
        kind: Kind::Toggle { default: false, on: "over the terminal", off: "beside the terminal" },
    },
    Setting {
        key: "tickets.newest_first",
        page: "Ticket list",
        group: "Thread",
        label: "Newest first, full screen",
        about: "A ticket full screen shows its thread's newest word first (also ⇅ in the ticket's header).",
        kind: Kind::Toggle { default: false, on: "newest first", off: "newest last" },
    },
    Setting {
        key: "tickets.panel_newest_first",
        page: "Ticket list",
        group: "Thread",
        label: "Newest first, in the panel",
        about: "A ticket in the panel shows its thread's newest word first; off, last, by the reply (also ⇅ there). The panel has an order of its own: the full screen's is not its.",
        kind: Kind::Toggle { default: false, on: "newest first", off: "newest last" },
    },
    Setting {
        key: "tickets.summary_open",
        page: "Ticket list",
        group: "Thread",
        label: "Where it stands, open",
        about: "In the ticket panel, a ticket's \"Where it stands\" shows its text at once; off, it is folded to its title, a click away. Full screen it always shows.",
        kind: Kind::Toggle { default: false, on: "open", off: "folded" },
    },
]);

impl Stored for Preferences {
    const PLACE: Place = Place::Config;
    const FILE: &'static str = "settings.toml";
    const EDITED: bool = true;

    fn check(&self) -> Result<(), String> {
        SCHEMA.check(self)
    }

    fn migrate() -> Option<Self> {
        let old = old_state()?;
        let (str_of, f32_of) = (|k: &str| old.get(k)?.as_str().map(String::from), |k: &str| old.get(k)?.as_f64().map(|v| v as f32));
        Some(Self {
            appearance: Appearance {
                theme: str_of("theme"),
                terminal_theme: str_of("terminal_theme"),
                terminal_font_size: f32_of("terminal_font_size"),
                window_font_size: f32_of("window_font_size"),
                terminal_opacity: None,
                terminal_blur: false,
            },
            notifications: Notifications {
                max: old.get("notify_max").and_then(Value::as_u64).map(|v| v as usize),
                seconds: old.get("notify_seconds").and_then(Value::as_u64),
                own: old.get("notify_own").and_then(Value::as_bool).unwrap_or(true),
                copied: true,
            },
            scroll: Scroll { speed: f32_of("scroll_speed").unwrap_or(1.) },
            mouse: Mouse::default(),
            tickets: Tickets { newest_first: old.get("thread_newest_first").and_then(Value::as_bool).unwrap_or(false), panel_overlay: false, summary_open: false, panel_newest_first: false },
            sessions: Sessions::default(),
            updates: Updates::default(),
            tips: Tips::default(),
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
    /// The left panel's tab: `workspaces`, or the sessions' (anything else).
    pub sidebar_tab: String,
    /// The workspaces folded in their tab, by name.
    pub workspaces_folded: Vec<String>,
    /// The accordions' sections given a height by hand (`list/section`).
    pub section_heights: HashMap<String, f32>,
    /// A full-screen ticket's fields column, in pixels; none until the user
    /// drags its border.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_width: Option<f32>,
    /// The window as it was left, opened so again; none: the default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowState>,
}

/// A window's state: windowed, maximized or full screen, and its size
/// when windowed (the size it goes back to).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowState {
    pub state: String,
    pub width: f32,
    pub height: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            panel_width: None,
            panel_open: true,
            sidebar_open: true,
            sidebar_width: None,
            sessions_folded: vec!["idle".into(), "shut".into()],
            sidebar_tab: String::new(),
            workspaces_folded: Vec::new(),
            section_heights: HashMap::new(),
            fields_width: None,
            window: None,
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
            sidebar_tab: String::new(),
            workspaces_folded: Vec::new(),
            section_heights: old.get("section_heights").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
            fields_width: None,
            window: None,
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
    /// When the newest ping tvty knew of came: those after it, at the next
    /// start, came while tvty was closed.
    pub pings_seen: Option<String>,
    /// The loops tvty stopped when it quit: offered to restart at start.
    #[serde(default)]
    pub stopped_on_quit: Vec<String>,
    /// Their AFK mode then (`off`, `wait_10m`, `wait_inf`), by loop: put
    /// back when they restart as they were.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub holds_on_quit: HashMap<String, String>,
    /// The loops asked to stop when tvty quit that still ran when it did:
    /// at start, said as such, or offered with the others if they stopped
    /// since.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub still_on_quit: Vec<String>,
    /// Each project's sessions in the order they came, as the list shows
    /// them: a new one last.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub terminal_order: HashMap<String, Vec<String>>,
    /// The standing instructions given last (the 📢), newest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub standing_history: Vec<String>,
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
            pings_seen: None,
            stopped_on_quit: Vec::new(),
            holds_on_quit: HashMap::new(),
            still_on_quit: Vec::new(),
            terminal_order: HashMap::new(),
            standing_history: Vec::new(),
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

/// What tvty remembers, as the shell holds it (the preferences are not
/// here: they change through the store only, see `Shell::set_pref`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub layout: Layout,
    pub workspace: Workspace,
    /// The workspaces kept (groups of groups).
    pub saved: crate::workspaces::Saved,
}

impl Settings {
    /// The sets as they are kept now.
    pub fn current(cx: &gpui_kit::App) -> Self {
        Self {
            layout: crate::config::get::<Layout>(cx).clone(),
            workspace: crate::config::get::<Workspace>(cx).clone(),
            saved: crate::config::get::<crate::workspaces::Saved>(cx).clone(),
        }
    }

    /// Keeps the sets: each written once its changes rest; one unchanged
    /// is not written.
    pub fn save(&self, cx: &mut gpui_kit::App) {
        crate::config::update::<Layout>(cx, |l| *l = self.layout.clone());
        crate::config::update::<Workspace>(cx, |w| *w = self.workspace.clone());
        crate::config::update::<crate::workspaces::Saved>(cx, |w| *w = self.saved.clone());
    }
}

/// Says what was wrong with a file read at start, once the window listens.
pub fn report_errors(cx: &mut gpui_kit::App) {
    crate::config::report::<Preferences>(cx);
    crate::config::report::<Layout>(cx);
    crate::config::report::<Workspace>(cx);
    crate::config::report::<crate::keymap::Shortcuts>(cx);
}

/// Writes every set now, what waits included: the tvty that follows (a
/// restart) reads them as they are.
pub fn flush(cx: &mut gpui_kit::App) {
    crate::config::flush::<Preferences>(cx);
    crate::config::flush::<Layout>(cx);
    crate::config::flush::<Workspace>(cx);
    crate::config::flush::<crate::keymap::Shortcuts>(cx);
}

/// Registers the sets, taking an older tvty's file apart the first time.
pub fn init(cx: &mut gpui_kit::App) {
    crate::config::register::<Preferences>(cx);
    crate::config::register::<Layout>(cx);
    crate::config::register::<Workspace>(cx);
    crate::config::register::<crate::workspaces::Saved>(cx);
    retire_old_state();
}

#[cfg(test)]
mod tests {
    use super::Preferences;
    use tvty_config::{parse, render};

    #[test]
    fn every_setting_reaches_its_field() {
        use super::SCHEMA;
        use tvty_config::{Kind, Value};
        let prefs = Preferences::default();
        for setting in SCHEMA.0 {
            let value = match setting.kind {
                // Away from its default: its top, or its bottom when the
                // top is the default (an opacity).
                Kind::Number { min, max, default, .. } => Value::Number(if max == default { min } else { max }),
                Kind::Toggle { default, .. } => Value::Toggle(!default),
                Kind::Choice => Value::Choice(Some("x".into())),
                Kind::Action { .. } => continue,
            };
            let changed = SCHEMA.with(&prefs, setting.key, value.clone()).unwrap_or_else(|e| panic!("{e}"));
            assert_ne!(changed, prefs, "{} reaches no field", setting.key);
            assert_eq!(SCHEMA.value(&changed, setting.key), Some(value), "{}", setting.key);
        }
    }

    #[test]
    fn a_hand_edit_out_of_bounds_is_refused() {
        assert_eq!(parse::<Preferences>("[appearance]\nterminal_font_size = 50\n").unwrap_err(), "appearance.terminal_font_size = 50: from 8 px to 32 px");
    }

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
