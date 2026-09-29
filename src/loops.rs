//! The agents' loops as tvty sees them, to open, start, stop or move one —
//! all through aiball's bus, never claude-loop's command (tvty relies on
//! aiball; claude-loop is the terminal's side of it): `loop.list` says the
//! loops this machine knows, running or not; `session.start` starts one,
//! `loop.restart` starts one again or moves it, `consumer.stop_loop` stops
//! it and keeps its state. Each call blocks: off the UI thread.

use anyhow::{Context as _, bail};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::aiball::Aiball;

/// A loop this machine knows, running or not, as `loop.list` gives it.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct KnownLoop {
    /// Its name (claude-loop's, the key aiball knows it by).
    pub name: String,
    /// Where its Claude works.
    pub cwd: String,
    /// Its aiball agent.
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    /// `crew`, or none for a project's main loop.
    #[serde(default)]
    pub role: Option<String>,
    /// Where it runs, or ran: `host` (aiball's session host) or `tmux`.
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub running: bool,
    /// In tmux, the session to attach to.
    #[serde(default)]
    pub tmux: Option<String>,
    /// Its Claude's Remote Control: `false`, `true` (named as its agent) or
    /// the name it is found by on claude.ai.
    #[serde(default)]
    pub remote_control: Value,
}

/// What to start: in `cwd`, for `agent` (else the folder's own), as a crew
/// agent when `crew` — or, `again`, a loop this machine knows, started again
/// where it ran, its conversation resumed.
#[derive(Clone, Debug, PartialEq)]
pub struct Start {
    pub cwd: String,
    pub project: Option<String>,
    pub agent: Option<String>,
    pub crew: bool,
    pub again: Option<String>,
}

impl KnownLoop {
    pub fn agent(&self) -> Option<&str> {
        self.agent.as_deref()
    }

    /// It runs (or ran) on aiball's session host, not in tmux.
    pub fn on_host(&self) -> bool {
        self.mode == "host"
    }

    /// The session tvty opens it by: its agent's on aiball's host, its
    /// tmux session otherwise.
    pub fn session(&self) -> String {
        match (self.on_host(), &self.agent) {
            (true, Some(agent)) => format!("{}{agent}", crate::sessions::HOSTED_PREFIX),
            _ => self.tmux.clone().unwrap_or_else(|| self.name.clone()),
        }
    }
}

/// The loops of this machine; none when aiball does not answer (said in
/// the log, once until it changes: the list is asked every few seconds).
pub fn known(aiball: &Aiball) -> Vec<KnownLoop> {
    static SAID: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
    let answer = aiball.call::<Vec<KnownLoop>>("loop.list", json!({}));
    let error = answer.as_ref().err().map(|e| format!("{e:#}"));
    if let Ok(mut said) = SAID.lock()
        && *said != error
    {
        match &error {
            Some(error) => log::warn!("loops: {error}"),
            None if said.is_some() => log::info!("loops: listed again"),
            None => {}
        }
        *said = error;
    }
    match answer {
        Ok(mut loops) => {
            loops.sort_by(|a, b| a.name.cmp(&b.name));
            loops
        }
        Err(_) => Vec::new(),
    }
}

/// Starts a loop in tmux, in its directory — or, `again`, starts a known
/// one again where it ran. Answers the session to open.
pub fn start(aiball: &Aiball, start: &Start) -> anyhow::Result<String> {
    if let Some(name) = &start.again {
        return restart(aiball, name);
    }
    if !std::path::Path::new(&start.cwd).is_dir() {
        bail!("{} is not a directory", start.cwd);
    }
    let mut params = json!({ "cwd": start.cwd, "mode": "tmux" });
    if let Some(project) = &start.project {
        params["project"] = json!(project);
    }
    if let Some(agent) = &start.agent {
        params[if start.crew { "crew" } else { "agent" }] = json!(agent);
    }
    let view: Value = aiball.call("session.start", params)?;
    session_of(&view).context("session.start: no session in the answer")
}

/// The session a `session.start` answer names: the tmux one, else the
/// agent's on the host.
fn session_of(view: &Value) -> Option<String> {
    if let Some(tmux) = view.get("tmux").and_then(Value::as_str) {
        return Some(tmux.to_string());
    }
    view.get("agent").and_then(Value::as_str).map(|agent| format!("{}{agent}", crate::sessions::HOSTED_PREFIX))
}

/// Moves a running loop onto aiball's host (`to_host`) or into tmux, its
/// conversation resumed. The user confirmed it, told that a Claude at work
/// is interrupted: forced, as aiball otherwise waits for it to be idle.
pub fn move_to(aiball: &Aiball, name: &str, to_host: bool) -> anyhow::Result<()> {
    let mode = if to_host { "host" } else { "tmux" };
    aiball.call::<Value>("loop.restart", json!({ "name": name, "mode": mode, "force": true })).map(drop).map_err(said)
}

/// Stops a loop, keeping its state: it stays restartable.
pub fn stop(aiball: &Aiball, known: &KnownLoop) -> anyhow::Result<()> {
    let agent = known.agent().with_context(|| format!("{}: no agent to stop it by", known.name))?;
    aiball.call::<Value>("consumer.stop_loop", json!({ "consumer_id": agent })).map(drop)
}

/// Starts a stopped loop again where it ran, its conversation resumed.
/// Answers the session to open.
pub fn restart(aiball: &Aiball, name: &str) -> anyhow::Result<String> {
    let view: KnownLoop = aiball.call("loop.restart", json!({ "name": name })).map_err(said)?;
    Ok(view.session())
}

/// aiball's refusal said for the user: a Claude at work is not moved.
fn said(error: anyhow::Error) -> anyhow::Error {
    if format!("{error:#}").contains("NOT_IDLE") {
        anyhow::anyhow!("its Claude is working: once it is idle")
    } else {
        error
    }
}

#[cfg(test)]
mod tests {
    use super::{KnownLoop, session_of};
    use serde_json::json;

    #[test]
    fn a_loop_opens_by_its_agent_on_the_host_and_its_session_in_tmux() {
        let on_host = KnownLoop { name: "cl-w-1".into(), cwd: "/w".into(), agent: Some("w-claude".into()), mode: "host".into(), ..Default::default() };
        assert_eq!(on_host.session(), format!("{}w-claude", crate::sessions::HOSTED_PREFIX));
        let in_tmux = KnownLoop { mode: "tmux".into(), tmux: Some("cl-w".into()), ..on_host.clone() };
        assert_eq!(in_tmux.session(), "cl-w");
        let stopped = KnownLoop { tmux: None, ..in_tmux };
        assert_eq!(stopped.session(), "cl-w-1");
    }

    #[test]
    fn a_started_session_is_named_by_the_answer() {
        assert_eq!(session_of(&json!({ "agent": "a", "host": "tmux", "tmux": "cl-a" })).as_deref(), Some("cl-a"));
        assert_eq!(session_of(&json!({ "agent": "a" })), Some(format!("{}a", crate::sessions::HOSTED_PREFIX)));
        assert_eq!(session_of(&json!({})), None);
    }

    #[test]
    fn a_listed_loop_reads() {
        let listed: KnownLoop = serde_json::from_value(json!({
            "name": "cl-demo", "cwd": "/w", "agent": "demo-claude", "project": "demo",
            "role": null, "mode": "tmux", "running": true, "tmux": "cl-demo", "attach": null
        }))
        .unwrap();
        assert!(listed.running && !listed.on_host());
        assert_eq!(listed.session(), "cl-demo");
    }
}
