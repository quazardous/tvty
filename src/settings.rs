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
    /// The projects' list, on the left.
    pub sidebar_open: bool,
    /// The colour theme's name; `None`: the kit's default dark theme.
    pub theme: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            panel_width: None,
            panel_open: true,
            sidebar_open: true,
            theme: None,
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
