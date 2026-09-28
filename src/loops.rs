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
    /// The agent it runs as on aiball's session host, when it was moved
    /// there (none: it runs in tmux).
    #[serde(default)]
    pub host_agent: Option<String>,
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
/// agent when `crew` — or, `again`, a loop this machine knows, restarted
/// where it ran (on aiball's host, or in tmux), its conversation resumed.
#[derive(Clone, Debug, PartialEq)]
pub struct Start {
    pub cwd: String,
    pub project: Option<String>,
    pub agent: Option<String>,
    pub crew: bool,
    pub again: Option<String>,
}

impl KnownLoop {
    /// Its agent: its own, or the one it runs as on aiball's host.
    pub fn agent(&self) -> Option<&str> {
        self.consumer.as_deref().or(self.host_agent.as_deref())
    }

    /// The session tvty opens it by: its agent's on aiball's host, its
    /// tmux session's name otherwise.
    pub fn session(&self) -> String {
        match &self.host_agent {
            Some(agent) => format!("{}{agent}", crate::sessions::HOSTED_PREFIX),
            None => self.name.clone(),
        }
    }
}

/// The arguments of `claude-loop` for `start`.
pub fn start_args(start: &Start) -> Vec<String> {
    if let Some(name) = &start.again {
        return vec!["restart".to_string(), "--resume".to_string(), name.clone()];
    }
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
/// thread. Answers the session to open, found by where the loop works and
/// for whom.
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
    let verb = if start.again.is_some() { "restart" } else { "start" };
    if !output.status.success() {
        bail!("claude-loop {verb}: {}", last_line(&said));
    }
    // The loop's plate says where it works, for whom, and where it runs.
    known()
        .into_iter()
        .find(|l| match &start.again {
            Some(name) => l.name == *name,
            None => Path::new(&l.cwd) == cwd && (start.agent.is_none() || l.agent() == start.agent.as_deref()),
        })
        .map(|l| l.session())
        .with_context(|| format!("claude-loop {verb}: {}", last_line(&said)))
}

/// Moves a running loop onto aiball's session host (`to_host`) or into
/// tmux: claude-loop restarts it there, resuming its conversation. Blocking:
/// call it off the UI thread.
pub fn move_to(name: &str, to_host: bool) -> anyhow::Result<()> {
    let output = Command::new("claude-loop")
        .args(move_args(name, to_host))
        .env_remove("TMUX")
        .output()
        .context("claude-loop")?;
    if !output.status.success() {
        let said = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr);
        bail!("claude-loop restart: {}", last_line(&said));
    }
    Ok(())
}

/// Stops a loop, keeping its state: it stays restartable. Blocking.
pub fn stop(name: &str) -> anyhow::Result<()> {
    run(&["stop", name], "stop")
}

/// Starts a stopped loop again where it ran, resuming its conversation.
/// Blocking.
pub fn restart(name: &str) -> anyhow::Result<()> {
    run(&["restart", "--resume", name], "restart")
}

fn run(args: &[&str], verb: &str) -> anyhow::Result<()> {
    let output = Command::new("claude-loop").args(args).env_remove("TMUX").output().context("claude-loop")?;
    if !output.status.success() {
        let said = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr);
        bail!("claude-loop {verb}: {}", last_line(&said));
    }
    Ok(())
}

fn move_args(name: &str, to_host: bool) -> Vec<String> {
    let place = if to_host { "--host" } else { "--tmux" };
    vec!["restart".into(), "--resume".into(), place.into(), name.into()]
}

fn last_line(text: &str) -> &str {
    text.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("no output").trim()
}

#[cfg(test)]
mod tests {
    use super::{KnownLoop, Start, move_args, start_args};

    #[test]
    fn a_loop_moves_by_a_restart_that_resumes() {
        assert_eq!(move_args("cl-app-1", true), ["restart", "--resume", "--host", "cl-app-1"]);
        assert_eq!(move_args("cl-app-1", false), ["restart", "--resume", "--tmux", "cl-app-1"]);
    }

    #[test]
    fn a_loop_starts_detached_for_its_agent() {
        let s = Start { cwd: "/w".into(), project: Some("demo".into()), agent: Some("demo-app".into()), crew: false, again: None };
        assert_eq!(start_args(&s), ["start", "--no-attach", "--project", "demo", "--agent", "demo-app"]);
        let crew = Start { crew: true, project: None, ..s.clone() };
        assert_eq!(start_args(&crew), ["start", "--no-attach", "--crew", "demo-app"]);
        // A loop known here starts again where it ran, its talk resumed.
        let again = Start { again: Some("cl-demo-1".into()), ..s };
        assert_eq!(start_args(&again), ["restart", "--resume", "cl-demo-1"]);
    }

    #[test]
    fn a_loop_on_the_host_opens_as_its_agent() {
        let on_host = KnownLoop {
            name: "cl-w-1".into(),
            cwd: "/w".into(),
            consumer: None,
            project: None,
            role: None,
            host_agent: Some("w-claude".into()),
        };
        assert_eq!(on_host.agent(), Some("w-claude"));
        assert_eq!(on_host.session(), format!("{}w-claude", crate::sessions::HOSTED_PREFIX));
        let in_tmux = KnownLoop { host_agent: None, consumer: Some("w-claude".into()), ..on_host };
        assert_eq!(in_tmux.session(), "cl-w-1");
    }
}
