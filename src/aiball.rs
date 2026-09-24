//! aiball's HTTP API, over its local socket. The socket trusts the same
//! user (file permissions), so there is no token: the identity rides in the
//! `x-aiball-consumer` header. tvty acts as the human it runs for.
//!
//! Every call blocks: run them off the UI thread.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context as _, bail};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub struct Aiball {
    socket: PathBuf,
    /// Who tvty acts as. Found with [`Aiball::find_user`].
    pub user: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Consumer {
    pub consumer_id: String,
    pub kind: String,
    pub cwd: Option<String>,
    pub project: Option<String>,
    pub last_seen_at: Option<String>,
    /// A loop's Claude: `busy`, `idle` or `boot`, as its loop pushes it.
    pub state: Option<String>,
    pub state_since: Option<String>,
    /// Who drives it: `loop` (autonomous), `wait` (held), `stop` (a human is
    /// typing), `boot`.
    pub state_human_word: Option<String>,
    /// The loop is connected to aiball right now.
    pub present: Option<bool>,
}

/// A row of a ticket listing, seen by [`Aiball::user`].
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TicketRow {
    pub id: u64,
    pub project: String,
    pub title: String,
    pub priority: Option<String>,
    pub claimant: Option<String>,
    pub assignee: Option<String>,
    /// A plan or resolution awaits a decision.
    #[serde(default)]
    pub pending_decision: bool,
    /// Something new on it for the user.
    #[serde(default)]
    pub unread: bool,
}

impl TicketRow {
    /// The agent working on it: its assignee, else its claimant.
    pub fn holder(&self) -> Option<&str> {
        self.assignee.as_deref().or(self.claimant.as_deref())
    }

    pub fn urgent(&self) -> bool {
        matches!(self.priority.as_deref(), Some("high" | "urgent"))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Thread {
    pub ticket: TicketHeader,
    #[serde(default)]
    pub comments: Vec<Comment>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TicketHeader {
    pub id: u64,
    pub title: String,
    pub body: Option<String>,
    pub by_agent: String,
    pub created_at: String,
    pub closed: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub kind: String,
    pub by_agent: String,
    pub body: Option<String>,
    pub meta: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    /// `plan`, `resolution`, `wontfix`, `escalation`.
    pub kind: String,
    /// `pending`, `accepted`, `rejected`.
    pub status: String,
}

impl Comment {
    pub fn decision(&self) -> Option<Decision> {
        let meta: Value = serde_json::from_str(self.meta.as_deref()?).ok()?;
        let decision = meta.get("decision")?;
        Some(Decision {
            kind: decision.get("kind")?.as_str()?.to_string(),
            status: decision.get("status")?.as_str()?.to_string(),
        })
    }
}

impl Thread {
    /// The comment carrying the thread's latest decision, when it still
    /// awaits one: only the latest decision of a thread can be taken.
    pub fn pending_decision(&self) -> Option<(&Comment, Decision)> {
        let (comment, decision) = self
            .comments
            .iter()
            .rev()
            .find_map(|c| c.decision().map(|d| (c, d)))?;
        (decision.status == "pending").then_some((comment, decision))
    }
}

impl Aiball {
    /// `$AIBALL_SOCK`, else aiball's default socket; acting as `$TVTY_USER`,
    /// else as the local owner until [`Aiball::find_user`] knows better.
    pub fn from_env() -> Self {
        let socket = std::env::var("AIBALL_SOCK")
            .ok()
            .filter(|p| !p.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_default();
                PathBuf::from(home).join(".local/share/aiball/sock")
            });
        let user = std::env::var("TVTY_USER").unwrap_or_else(|_| "human".into());
        Self { socket, user }
    }

    /// The human to act as: `$TVTY_USER` if set, else the named human seen
    /// last (the generic `human` identity only when there is no other).
    pub fn find_user(&mut self, consumers: &[Consumer]) {
        if std::env::var("TVTY_USER").is_ok() {
            return;
        }
        if let Some(human) = consumers
            .iter()
            .filter(|c| c.kind == "human" && c.consumer_id != "human")
            .max_by(|a, b| a.last_seen_at.cmp(&b.last_seen_at))
        {
            self.user = human.consumer_id.clone();
        }
    }

    pub fn consumers(&self) -> anyhow::Result<Vec<Consumer>> {
        self.get("/api/consumers")
    }

    /// The open tickets of a project.
    pub fn open_tickets(&self, project: &str) -> anyhow::Result<Vec<TicketRow>> {
        self.get(&format!("/api/tickets?project={}&open=1", encode(project)))
    }

    /// The ticket holding back the most open tickets of the project.
    pub fn critical(&self, project: &str) -> anyhow::Result<Option<u64>> {
        let answer: Value = self.get(&format!("/api/projects/{}/critical", encode(project)))?;
        Ok(match answer.get("critical") {
            Some(Value::Number(n)) => n.as_u64(),
            Some(Value::Object(o)) => o
                .get("id")
                .or_else(|| o.get("ticket_id"))
                .and_then(Value::as_u64),
            _ => None,
        })
    }

    pub fn thread(&self, ticket: u64) -> anyhow::Result<Thread> {
        self.get(&format!("/api/tickets/{ticket}?full=1&limit=9999"))
    }

    pub fn mark_read(&self, ticket: u64) -> anyhow::Result<()> {
        self.post(&format!("/api/tickets/{ticket}/mark-read"), json!({}))
            .map(drop)
    }

    pub fn reply(&self, project: &str, ticket: u64, body: &str) -> anyhow::Result<()> {
        self.post(
            "/api/messages",
            json!({
                "project": project,
                "kind": "comment_added",
                "ticket_id": ticket,
                "parent_id": ticket,
                "body": body,
                "by_agent": self.user,
            }),
        )
        .map(drop)
    }

    /// Accept or reject the decision a comment carries.
    pub fn decide(&self, comment: u64, accept: bool) -> anyhow::Result<()> {
        let status = if accept { "accepted" } else { "rejected" };
        self.post(&format!("/api/messages/{comment}/decide"), json!({ "status": status, "decided_by": self.user }))
            .map(drop)
    }

    fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let body = self.request("GET", path, None)?;
        serde_json::from_str(&body).with_context(|| format!("GET {path}"))
    }

    fn post(&self, path: &str, body: Value) -> anyhow::Result<Value> {
        let answer = self.request("POST", path, Some(body))?;
        Ok(serde_json::from_str(&answer).unwrap_or(Value::Null))
    }

    #[cfg(unix)]
    fn request(&self, method: &str, path: &str, body: Option<Value>) -> anyhow::Result<String> {
        use std::os::unix::net::UnixStream;

        let mut stream = UnixStream::connect(&self.socket)
            .with_context(|| format!("aiball's socket {}", self.socket.display()))?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let body = body.map(|b| b.to_string()).unwrap_or_default();
        // HTTP/1.0: the daemon closes the connection after the answer, so
        // reading to the end reads exactly one answer, never chunked.
        let request = format!(
            "{method} {path} HTTP/1.0\r\nHost: aiball\r\nx-aiball-consumer: {user}\r\n\
             content-type: application/json\r\ncontent-length: {len}\r\n\r\n{body}",
            user = self.user,
            len = body.len(),
        );
        stream.write_all(request.as_bytes())?;
        let mut answer = Vec::new();
        stream.read_to_end(&mut answer)?;
        let answer = String::from_utf8_lossy(&answer);
        let (head, body) = answer
            .split_once("\r\n\r\n")
            .context("malformed answer from aiball")?;
        let status = head.split(' ').nth(1).unwrap_or("");
        if !status.starts_with('2') {
            bail!("{method} {path}: {status} {}", body.trim());
        }
        Ok(body.to_string())
    }

    #[cfg(not(unix))]
    fn request(&self, _: &str, _: &str, _: Option<Value>) -> anyhow::Result<String> {
        bail!("aiball's socket is Unix only for now")
    }
}

/// Percent-encodes a path segment or query value.
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
