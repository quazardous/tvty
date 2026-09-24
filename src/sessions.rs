//! Which terminals there are: the tmux sessions on this machine, grouped by
//! project. A claude-loop session (`cl-*`) is matched to its aiball agent by
//! its directory, which gives it a project and a name; any other session
//! lands in a "tmux" group.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub struct Terminal {
    /// The tmux session to attach to.
    pub session: String,
    /// The agent's name, or the session's when no agent owns it.
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub name: String,
    pub terminals: Vec<Terminal>,
}

#[derive(Deserialize)]
struct Consumer {
    consumer_id: String,
    kind: String,
    cwd: Option<String>,
    project: Option<String>,
}

const LOOP_PREFIX: &str = "cl-";
const OTHER_GROUP: &str = "tmux";

/// Blocking: runs `tmux ls` and asks aiball. Call it off the UI thread.
pub fn discover() -> Vec<Project> {
    let consumers = consumers().unwrap_or_default();
    let mut projects: BTreeMap<String, Vec<Terminal>> = BTreeMap::new();
    for (session, path) in tmux_sessions() {
        let agent = session
            .starts_with(LOOP_PREFIX)
            .then(|| {
                consumers
                    .iter()
                    .find(|c| c.kind == "agent" && c.cwd.as_deref() == Some(path.as_str()))
            })
            .flatten();
        let (project, label) = match agent {
            Some(c) => (
                c.project.clone().unwrap_or_else(|| basename(&path)),
                c.consumer_id.clone(),
            ),
            None if session.starts_with(LOOP_PREFIX) => (basename(&path), session.clone()),
            None => (OTHER_GROUP.to_string(), session.clone()),
        };
        projects
            .entry(project)
            .or_default()
            .push(Terminal { session, label });
    }
    // Plain tmux sessions last: the projects are what tvty is for.
    let other = projects.remove(OTHER_GROUP);
    let mut list: Vec<Project> = projects
        .into_iter()
        .map(|(name, terminals)| Project { name, terminals })
        .collect();
    list.sort_by_key(|p| p.name.to_lowercase());
    if let Some(terminals) = other {
        list.push(Project {
            name: OTHER_GROUP.into(),
            terminals,
        });
    }
    list
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// `(session name, start directory)` of every tmux session.
fn tmux_sessions() -> Vec<(String, String)> {
    let Ok(output) = Command::new("tmux")
        .args(["ls", "-F", "#{session_name}\t#{session_path}"])
        .env_remove("TMUX")
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(name, path)| (name.to_string(), path.to_string()))
        .collect()
}

/// aiball's agents, over its local socket (`$AIBALL_SOCK`, or the default
/// path). The socket is trusted by file permissions: no token.
#[cfg(unix)]
fn consumers() -> anyhow::Result<Vec<Consumer>> {
    use std::os::unix::net::UnixStream;

    let path = std::env::var("AIBALL_SOCK").ok().filter(|p| !p.is_empty()).unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/.local/share/aiball/sock")
    });
    let mut stream = UnixStream::connect(path)?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.write_all(b"GET /api/consumers HTTP/1.0\r\nHost: aiball\r\n\r\n")?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    let response = String::from_utf8_lossy(&response);
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| anyhow::anyhow!("malformed response"))?;
    anyhow::ensure!(head.starts_with("HTTP/1.1 200") || head.starts_with("HTTP/1.0 200"), "{}", head.lines().next().unwrap_or(""));
    Ok(serde_json::from_str(body)?)
}

#[cfg(not(unix))]
fn consumers() -> anyhow::Result<Vec<Consumer>> {
    anyhow::bail!("aiball's socket is Unix only for now")
}
