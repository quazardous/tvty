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
    /// Stopped, and its agent has a loop that runs, or a later one.
    #[serde(default)]
    pub superseded: bool,
    /// On the host, where a client attaches to it.
    #[serde(default)]
    pub attach: Option<crate::aiball::AttachPoint>,
    /// The clients attached to it.
    #[serde(default)]
    pub clients: Option<u32>,
}

/// What to start: in `cwd`, for `agent` (else the folder's own), as a crew
/// agent when `crew` — or, `again`, a loop this machine knows, started again
/// where it ran, its conversation resumed. `mode`: `tmux` or `host` when the
/// user chose; none, the folder's own (its `claude_loop.session`, as aiball
/// reads it).
#[derive(Clone, Debug, PartialEq)]
pub struct Start {
    pub cwd: String,
    pub project: Option<String>,
    pub agent: Option<String>,
    pub crew: bool,
    pub again: Option<String>,
    pub mode: Option<&'static str>,
    /// Which conversation its Claude takes up, in a folder where Claude Code
    /// already has some that no loop follows.
    pub resume: Resume,
}

/// A first start's conversation, in a folder where Claude Code has some.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Resume {
    /// Not asked yet: tvty asks when there is something to resume.
    #[default]
    Ask,
    /// This one (an id), or the folder's latest (`latest`).
    Conversation(String),
    /// A new one.
    Fresh,
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

/// The agent of another project whose folder `cwd` is, if one is: a
/// loop started there resumes the conversation it finds, that agent's. Never
/// for `agent` in its own folder (the one aiball knows it by), whoever else
/// is registered there (a test agent long gone, say).
pub fn stranger<'a>(cwd: &str, agent: Option<&str>, project: Option<&str>, homes: &'a [(String, Option<String>, String)]) -> Option<&'a str> {
    let same = |a: &str, b: &str| a.trim_end_matches('/') == b.trim_end_matches('/');
    let project = project?;
    if homes.iter().any(|(who, _, home)| Some(who.as_str()) == agent && same(home, cwd)) {
        return None;
    }
    homes
        .iter()
        .find(|(_, of, home)| same(home, cwd) && of.as_deref().is_some_and(|of| of != project))
        .map(|(agent, _, _)| agent.as_str())
}

/// Starts a loop in its directory, where its folder says (or `mode`) — or,
/// `again`, starts a known one again where it ran. Answers the session to
/// open.
pub fn start(aiball: &Aiball, start: &Start) -> anyhow::Result<String> {
    if let Some(name) = &start.again {
        // Its recorded conversation gone, the one chosen instead.
        if let Resume::Conversation(id) = &start.resume {
            return restart_on(aiball, name, id);
        }
        return restart(aiball, name, None);
    }
    if !std::path::Path::new(&start.cwd).is_dir() {
        bail!(crate::t!("misc-not-a-directory", folder = start.cwd.clone()));
    }
    let mut params = json!({ "cwd": start.cwd });
    if let Some(mode) = start.mode {
        params["mode"] = json!(mode);
    }
    if let Some(project) = &start.project {
        params["project"] = json!(project);
    }
    if let Some(agent) = &start.agent {
        params[if start.crew { "crew" } else { "agent" }] = json!(agent);
    }
    if let Resume::Conversation(id) = &start.resume {
        params["resume"] = json!(id);
    }
    let view: Value = aiball.call_starting("session.start", params)?;
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
    aiball.call_starting::<Value>("loop.restart", json!({ "name": name, "mode": mode, "force": true })).map(drop).map_err(said)
}

/// The loop's other tmux clients (claude-loop's terminal) made read-only
/// copies: `keep` is the pid of the one taking the controls.
pub fn others_to_copies(aiball: &Aiball, name: &str, keep: u32) -> anyhow::Result<()> {
    aiball.call::<Value>("loop.clients_readonly", json!({ "name": name, "keep_pid": keep })).map(drop)
}

/// Detaches the other tmux clients of a loop (claude-loop's terminal,
/// another tvty): `keep`'s, this terminal's, stays. Answers the clients left.
pub fn detach_others(aiball: &Aiball, name: &str, keep: u32) -> anyhow::Result<u64> {
    let left: Value = aiball.call("loop.clients_detach", json!({ "name": name, "keep_pid": keep }))?;
    Ok(left.get("clients").and_then(Value::as_u64).unwrap_or(0))
}

/// Stops a loop, keeping its state: it stays restartable.
pub fn stop(aiball: &Aiball, known: &KnownLoop) -> anyhow::Result<()> {
    let agent = known.agent().with_context(|| format!("{}: no agent to stop it by", known.name))?;
    stop_agent(aiball, agent)
}

/// Stops an agent's loop, wherever it runs (tmux or aiball's host): it
/// stays restartable.
pub fn stop_agent(aiball: &Aiball, agent: &str) -> anyhow::Result<()> {
    let answer: Value = aiball.call("consumer.stop_loop", json!({ "consumer_id": agent }))?;
    stopped(agent, &answer)
}

/// aiball's answer to a stop: it does not refuse one no loop received, it
/// says so (`delivered: false`) — not stopped, then.
fn stopped(agent: &str, answer: &Value) -> anyhow::Result<()> {
    match answer.get("delivered").and_then(Value::as_bool) {
        Some(false) => anyhow::bail!(crate::t!("misc-stop-not-received", agent = agent)),
        _ => Ok(()),
    }
}

/// Starts a stopped loop again where it ran, its conversation resumed.
/// Answers the session to open.
/// Starts a stopped loop again on a conversation of its folder (`latest` or
/// an id), not the one it recorded (gone).
pub fn restart_on(aiball: &Aiball, name: &str, conversation: &str) -> anyhow::Result<String> {
    let view: KnownLoop = aiball.call_starting("loop.restart", json!({ "name": name, "resume": conversation })).map_err(said)?;
    Ok(view.session())
}

/// `afk`: the hold it starts in (`off`, `wait_inf`); none: its agent's,
/// as aiball keeps it.
pub fn restart(aiball: &Aiball, name: &str, afk: Option<&str>) -> anyhow::Result<String> {
    let mut params = json!({ "name": name });
    if let Some(afk) = afk {
        params["afk"] = json!(afk);
    }
    let view: KnownLoop = aiball.call_starting("loop.restart", params).map_err(said)?;
    Ok(view.session())
}

/// aiball's refusal said for the user: a Claude at work is not moved.
fn said(error: anyhow::Error) -> anyhow::Error {
    if format!("{error:#}").contains("NOT_IDLE") {
        anyhow::anyhow!(crate::t!("misc-not-idle"))
    } else {
        error
    }
}

#[cfg(test)]
mod tests {
    use super::{KnownLoop, session_of, stopped, stranger};
    use serde_json::json;

    /// aiball's answers to a stop, as `consumer.stop_loop` gives them.
    #[test]
    fn a_stop_no_loop_received_is_a_failure() {
        assert!(stopped("w-claude", &json!({ "consumer_id": "w-claude", "action": "kill", "delivered": true })).is_ok());
        let none = stopped("w-claude", &json!({ "consumer_id": "w-claude", "action": "kill", "delivered": false }));
        assert!(none.unwrap_err().to_string().contains("no loop of it received the stop"));
    }

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

    #[test]
    fn a_folder_of_another_projects_agent_is_a_strangers() {
        let homes = vec![
            ("aiball-dev".to_string(), Some("aiball".to_string()), "/w/aiball".to_string()),
            ("book".to_string(), Some("book".to_string()), "/w/book".to_string()),
            ("aiball-crew".to_string(), Some("aiball".to_string()), "/w/aiball-crew".to_string()),
        ];
        assert_eq!(stranger("/w/aiball/", Some("book"), Some("book"), &homes), Some("aiball-dev"));
        assert_eq!(stranger("/w/book", Some("book"), Some("book"), &homes), None);
        // A crew agent next to its project's main loop is at home.
        assert_eq!(stranger("/w/aiball", Some("aiball-crew"), Some("aiball"), &homes), None);
        // No project known: nothing to compare.
        assert_eq!(stranger("/w/aiball", Some("x"), None, &homes), None);
        // In its own folder an agent is at home, whoever else is registered
        // there: a test agent of another project, long gone.
        let mut homes = homes;
        homes.insert(0, ("testuser".to_string(), Some("test".to_string()), "/w/aiball".to_string()));
        assert_eq!(stranger("/w/aiball", Some("aiball-dev"), Some("aiball"), &homes), None);
        // Another project's agent started there is still astray.
        assert!(stranger("/w/aiball", Some("book"), Some("book"), &homes).is_some());
    }
}
