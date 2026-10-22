//! What the window shows, frozen: the sessions and tickets of a board as
//! read once from aiball (`scene.json`), so that tvty and tvty-egui draw
//! the same data when their screenshots are compared.

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Scene {
    /// Who tvty acts as.
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub projects: Vec<Project>,
    /// aiball's consumers (`consumer.list`).
    #[serde(default)]
    pub consumers: Vec<Consumer>,
    /// The tmux sessions: name and start folder.
    #[serde(default)]
    pub sessions: Vec<(String, String)>,
    /// The terminals tvty keeps open, and each project's order of them.
    #[serde(default)]
    pub open: Vec<String>,
    #[serde(default)]
    pub order: std::collections::HashMap<String, Vec<String>>,
}

/// An agent or a human, as aiball lists them.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Consumer {
    pub consumer_id: String,
    #[serde(default)]
    pub kind: String,
    pub project: Option<String>,
    pub cwd: Option<String>,
    pub state: Option<String>,
    pub present: Option<bool>,
    pub ping_unseen: Option<u64>,
    pub counters: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Project {
    pub name: String,
    /// Its tickets as the list reads them, closed ones too.
    #[serde(default)]
    pub tickets: Vec<crate::board::Ticket>,
}

impl Scene {
    /// `TVTY_EGUI_SCENE`, else an empty board.
    pub fn load() -> Self {
        let Some(path) = std::env::var_os("TVTY_EGUI_SCENE") else {
            return Self::default();
        };
        match std::fs::read_to_string(&path).map(|t| serde_json::from_str(&t)) {
            Ok(Ok(scene)) => scene,
            Ok(Err(e)) => {
                log::warn!("scene: {e}");
                Self::default()
            }
            Err(e) => {
                log::warn!("scene: {e}");
                Self::default()
            }
        }
    }
}
