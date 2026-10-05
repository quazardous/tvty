//! Workspaces: named groups of groups. A group is a project's terminals
//! (what the sessions list, the tabs and the slider show together); a
//! workspace keeps some of them, and for each of their agents' sessions how
//! it runs — on its own (`auto`) or held (`stop`). Kept in tvty's state
//! (`workspaces.json`), this tvty's alone.
//!
//! Here, what is kept and what is worked out of it, with no window: what a
//! workspace would stop when shut (the groups no other workspace has), what
//! it would start or let run when opened. The shell asks before doing any
//! of it (`shell::workspaces`, the sessions' picker).

use serde::{Deserialize, Serialize};
use tvty_config::{Place, Stored};

/// How a session runs, as a workspace keeps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Its loop runs on its own.
    Auto,
    /// Held until let go.
    Stop,
}

impl Mode {
    /// From aiball's AFK mode (`off`, `wait_10m`, `wait_inf`): on its own
    /// only when nothing holds it.
    pub fn of_afk(afk: Option<&str>) -> Self {
        match afk {
            Some("wait_10m") | Some("wait_inf") => Self::Stop,
            _ => Self::Auto,
        }
    }

    /// The AFK action that sets it.
    pub fn from_held(held: bool) -> Self {
        if held { Self::Stop } else { Self::Auto }
    }

    /// As aiball keeps an agent's hold.
    pub fn hold(self) -> &'static str {
        match self {
            Self::Auto => "off",
            Self::Stop => "wait_inf",
        }
    }

    /// Its glyph, as the agent bar's.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Auto => "▶",
            Self::Stop => "■",
        }
    }
}

/// An agent's session in a group, and how it runs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub agent: String,
    pub mode: Mode,
}

/// A group: a project's sessions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub project: String,
    pub sessions: Vec<Session>,
}

/// A workspace: its name, its groups.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    pub name: String,
    pub groups: Vec<Group>,
}

/// The workspaces kept.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Saved {
    pub workspaces: Vec<Workspace>,
}

impl Stored for Saved {
    const PLACE: Place = Place::State;
    const FILE: &'static str = "workspaces.json";
}

impl Saved {
    pub fn get(&self, name: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.name == name)
    }

    /// `workspace` kept under its name: replaced, or added last.
    pub fn put(&mut self, workspace: Workspace) {
        match self.workspaces.iter_mut().find(|w| w.name == workspace.name) {
            Some(kept) => *kept = workspace,
            None => self.workspaces.push(workspace),
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.workspaces.retain(|w| w.name != name);
    }

    /// A name no workspace has: `wanted`, or `wanted 2`, `wanted 3`…
    pub fn free_name(&self, wanted: &str) -> String {
        let wanted = if wanted.trim().is_empty() { "Workspace" } else { wanted.trim() };
        if self.get(wanted).is_none() {
            return wanted.to_string();
        }
        (2..).map(|n| format!("{wanted} {n}")).find(|name| self.get(name).is_none()).unwrap_or_else(|| wanted.to_string())
    }

    /// The other workspaces that have `project`'s group too.
    pub fn others_with(&self, name: &str, project: &str) -> Vec<&str> {
        self.workspaces
            .iter()
            .filter(|w| w.name != name && w.groups.iter().any(|g| g.project == project))
            .map(|w| w.name.as_str())
            .collect()
    }
}

/// What opening a workspace would do to one of its sessions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opening {
    /// Not running: started, then set as kept.
    Start(Mode),
    /// Kept on its own, running held: let go.
    Launch,
    /// Kept held, running on its own: listed, not done unless asked.
    Hold,
    /// As kept already.
    AsKept,
}

/// What opening would do to `session`, given how it is `now`.
pub fn opening(session: &Session, now: Option<Mode>) -> Opening {
    match (now, session.mode) {
        (None, mode) => Opening::Start(mode),
        (Some(Mode::Stop), Mode::Auto) => Opening::Launch,
        (Some(Mode::Auto), Mode::Stop) => Opening::Hold,
        (Some(_), _) => Opening::AsKept,
    }
}

impl Opening {
    /// Done unless the user unticks it.
    pub fn by_default(self) -> bool {
        matches!(self, Self::Start(_) | Self::Launch)
    }

    /// What it says beside the session.
    pub fn said(self) -> String {
        crate::t!(match self {
            Self::Start(Mode::Auto) => "workspaces-start-own",
            Self::Start(Mode::Stop) => "workspaces-start-held",
            Self::Launch => "workspaces-let-go",
            Self::Hold => "workspaces-to-hold",
            Self::AsKept => "workspaces-as-kept",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace(name: &str, projects: &[&str]) -> Workspace {
        Workspace {
            name: name.into(),
            groups: projects.iter().map(|p| Group { project: p.to_string(), sessions: vec![Session { agent: format!("{p}-claude"), mode: Mode::Auto }] }).collect(),
        }
    }

    #[test]
    fn a_group_in_another_workspace_too_is_shared() {
        let mut saved = Saved::default();
        saved.put(workspace("front", &["app", "ui"]));
        saved.put(workspace("back", &["api", "app"]));
        // Shutting "front": "ui" is its own, "app" is "back"'s too.
        assert!(saved.others_with("front", "ui").is_empty());
        assert_eq!(saved.others_with("front", "app"), vec!["back"]);
    }

    #[test]
    fn a_workspace_is_replaced_under_its_name_and_a_new_name_is_free() {
        let mut saved = Saved::default();
        saved.put(workspace("front", &["app"]));
        saved.put(workspace("front", &["ui"]));
        assert_eq!(saved.workspaces.len(), 1);
        assert_eq!(saved.get("front").unwrap().groups[0].project, "ui");
        assert_eq!(saved.free_name("front"), "front 2");
        assert_eq!(saved.free_name("  "), "Workspace");
        saved.remove("front");
        assert!(saved.workspaces.is_empty());
    }

    #[test]
    fn opening_starts_what_is_stopped_and_lets_go_what_is_kept_on_its_own() {
        let auto = Session { agent: "a".into(), mode: Mode::Auto };
        let held = Session { agent: "b".into(), mode: Mode::Stop };
        assert_eq!(opening(&auto, None), Opening::Start(Mode::Auto));
        assert_eq!(opening(&held, None), Opening::Start(Mode::Stop));
        // Kept on its own, found held: offered to let go.
        assert_eq!(opening(&auto, Some(Mode::Stop)), Opening::Launch);
        assert!(Opening::Launch.by_default());
        // Kept held, found on its own: listed, not done by default.
        assert_eq!(opening(&held, Some(Mode::Auto)), Opening::Hold);
        assert!(!Opening::Hold.by_default());
        assert_eq!(opening(&auto, Some(Mode::Auto)), Opening::AsKept);
    }

    #[test]
    fn a_hold_of_any_kind_is_kept_as_stop() {
        assert_eq!(Mode::of_afk(Some("off")), Mode::Auto);
        assert_eq!(Mode::of_afk(None), Mode::Auto);
        assert_eq!(Mode::of_afk(Some("wait_10m")), Mode::Stop);
        assert_eq!(Mode::of_afk(Some("wait_inf")), Mode::Stop);
        assert_eq!(Mode::Stop.hold(), "wait_inf");
    }
}
