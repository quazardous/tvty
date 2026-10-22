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
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub sessions: Vec<Session>,
    /// Its tickets as the list reads them, closed ones too.
    #[serde(default)]
    pub tickets: Vec<crate::board::Ticket>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Session {
    pub name: String,
    /// `busy`, `idle` or `boot`.
    #[serde(default)]
    pub state: String,
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
