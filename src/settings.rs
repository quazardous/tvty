//! What tvty remembers between runs, in `$XDG_CONFIG_HOME/tvty/state.json`
//! (`~/.config/tvty/state.json`).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The ticket panel's width, in pixels; `None` until the user drags it:
    /// a third of the window.
    pub panel_width: Option<f32>,
    pub panel_open: bool,
    /// The projects' list, on the left, over the terminal.
    pub sidebar_open: bool,
    /// Its width, in pixels; `None`: the default.
    pub sidebar_width: Option<f32>,
    /// The colour theme's name; `None`: the kit's default dark theme.
    pub theme: Option<String>,
    /// The terminals' own colour theme; `None`: the window's.
    pub terminal_theme: Option<String>,
    /// A ticket's thread shows its newest word first (top-down).
    pub thread_newest_first: bool,
    /// The mouse wheel's speed, times tvty's own (1: a notch is about three
    /// ticket rows, five lines of a terminal's history).
    pub scroll_speed: f32,
    /// The terminals' font size and the window's text size, in pixels;
    /// `None`: the defaults.
    pub terminal_font_size: Option<f32>,
    pub window_font_size: Option<f32>,
    /// Notifications shown at most, and for how many seconds; `None`: the
    /// defaults.
    pub notify_max: Option<usize>,
    pub notify_seconds: Option<u64>,
    /// The user's own gestures (closed, replied, decided…) notified too.
    pub notify_own: bool,
    /// The project the last new ticket went to.
    pub last_ticket_project: Option<String>,
    /// The sessions list's folded sections (`live`, `idle`, `shut`).
    pub sessions_folded: Vec<String>,
    /// The workspace as tvty was left: the terminals open in it, the one
    /// used last first, and the one shown; opened again at start.
    pub open_terminals: Vec<String>,
    pub shown_terminal: Option<String>,
    /// The accordions' sections given a height by hand (`list/section`).
    pub section_heights: std::collections::HashMap<String, f32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            panel_width: None,
            panel_open: true,
            sidebar_open: true,
            sidebar_width: None,
            theme: None,
            terminal_theme: None,
            thread_newest_first: false,
            scroll_speed: 1.,
            last_ticket_project: None,
            notify_max: None,
            notify_seconds: None,
            notify_own: true,
            terminal_font_size: None,
            window_font_size: None,
            sessions_folded: vec!["idle".into(), "shut".into()],
            open_terminals: Vec::new(),
            shown_terminal: None,
            section_heights: Default::default(),
        }
    }
}

fn path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("tvty").join("state.json"))
}

impl Settings {
    pub fn load() -> Self {
        path()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = path() else { return };
        let write = || -> std::io::Result<()> {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(&path, serde_json::to_vec_pretty(self)?)
        };
        if let Err(error) = write() {
            log::warn!("settings: {}: {error}", path.display());
        }
    }
}
