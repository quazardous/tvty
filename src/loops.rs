//! claude-loop's loops as tvty sees them, to open a session: the loops this
//! machine knows (a state directory with its `plate.json`: where it works,
//! for which agent and project), and starting one — always in its own
//! working directory, never tvty's. claude-loop does the starting; tvty only
//! calls it (`claude-loop start --no-attach`), then opens the session.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context as _, bail};
use serde::Deserialize;

/// A loop this machine knows, running or not.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct KnownLoop {
    /// Its name, also its tmux session's.
    pub name: String,
    /// Where its Claude works.
    pub cwd: String,
    /// Its aiball agent.
    #[serde(default)]
    pub consumer: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    /// `crew`, or none for a project's main loop.
    #[serde(default)]
    pub role: Option<String>,
}

/// Where claude-loop keeps its loops' state (`$CLAUDE_LOOP_STATE_ROOT`,
/// else `~/.claude-loop`).
pub fn state_root() -> PathBuf {
    std::env::var_os("CLAUDE_LOOP_STATE_ROOT")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".claude-loop"))
}

/// The loops of this machine, read from their `plate.json`.
pub fn known() -> Vec<KnownLoop> {
    let Ok(entries) = std::fs::read_dir(state_root()) else {
        return Vec::new();
    };
    let mut loops: Vec<KnownLoop> = entries
        .flatten()
        .filter_map(|e| std::fs::read(e.path().join("plate.json")).ok())
        .filter_map(|bytes| serde_json::from_slice(&bytes).ok())
        .collect();
    loops.sort_by(|a, b| a.name.cmp(&b.name));
    loops
}

/// What to start: in `cwd`, for `agent` (else the folder's own), as a crew
/// agent when `crew`.
#[derive(Clone, Debug, PartialEq)]
pub struct Start {
    pub cwd: String,
    pub project: Option<String>,
    pub agent: Option<String>,
    pub crew: bool,
}

/// The arguments of `claude-loop` for `start`.
pub fn start_args(start: &Start) -> Vec<String> {
    let mut args = vec!["start".to_string(), "--no-attach".to_string()];
    if let Some(project) = &start.project {
        args.extend(["--project".to_string(), project.clone()]);
    }
    if let Some(agent) = &start.agent {
        args.extend([if start.crew { "--crew" } else { "--agent" }.to_string(), agent.clone()]);
    }
    args
}

/// Starts a loop, detached, in its directory. Blocking: call it off the UI
/// thread. Answers the loop's name, found by where it works and for whom.
pub fn start(start: &Start) -> anyhow::Result<String> {
    let cwd = Path::new(&start.cwd);
    if !cwd.is_dir() {
        bail!("{} is not a directory", start.cwd);
    }
    let output = Command::new("claude-loop")
        .args(start_args(start))
        .current_dir(cwd)
        .env_remove("TMUX")
        .output()
        .context("claude-loop")?;
    let said = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        bail!("claude-loop start: {}", last_line(&said));
    }
    // The loop's plate says where it works and for whom: its name is there.
    known()
        .into_iter()
        .find(|l| Path::new(&l.cwd) == cwd && (start.agent.is_none() || l.consumer == start.agent))
        .map(|l| l.name)
        .with_context(|| format!("claude-loop start: {}", last_line(&said)))
}

fn last_line(text: &str) -> &str {
    text.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("no output").trim()
}

#[cfg(test)]
mod tests {
    use super::{Start, start_args};

    #[test]
    fn a_loop_starts_detached_for_its_agent() {
        let s = Start { cwd: "/w".into(), project: Some("demo".into()), agent: Some("demo-app".into()), crew: false };
        assert_eq!(start_args(&s), ["start", "--no-attach", "--project", "demo", "--agent", "demo-app"]);
        let crew = Start { crew: true, project: None, ..s };
        assert_eq!(start_args(&crew), ["start", "--no-attach", "--crew", "demo-app"]);
    }
}
