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

/// A row of the ticket list, as `/api/inbox` builds it for [`Aiball::user`]
/// — the same row aiball's web UI shows.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TicketRow {
    pub id: u64,
    pub project: String,
    pub title: String,
    /// Moderation: `approved`, `pending`, `rejected`.
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub resolved: bool,
    pub priority: Option<String>,
    pub claimant: Option<String>,
    pub assignee: Option<String>,
    /// Something new on it for the user.
    #[serde(default)]
    pub unread: bool,
    /// An agent was active on it lately.
    #[serde(default)]
    pub hot: bool,
    /// Who wrote the last message.
    pub last_speaker: Option<String>,
    pub last_activity: Option<String>,
    #[serde(default)]
    pub comment_count: u32,
    /// Set on the project's critical ticket: how many open tickets it holds.
    pub critical: Option<Critical>,
    #[serde(default)]
    pub pending_plan: bool,
    #[serde(default)]
    pub pending_resolution: bool,
    #[serde(default)]
    pub pending_wontfix: bool,
    #[serde(default)]
    pub pending_escalation: bool,
    /// Comments waiting for moderation.
    #[serde(default)]
    pub pending_comment_count: u32,
    /// The pending decision is still the last message.
    #[serde(default)]
    pub pending_decision_is_latest: bool,
    /// The last message is a step (`then: continue`).
    #[serde(default)]
    pub latest_is_step: bool,
    /// ... and it went quiet.
    #[serde(default)]
    pub stalled_step: bool,
    #[serde(default)]
    pub latest_plan_rejected: bool,
    #[serde(default)]
    pub latest_resolution_rejected: bool,
    /// Computed by aiball for the reader when asked with `v=tvty`: whose
    /// turn (`you`, `them`, `none`), the sorting band (0 to 5) and the state
    /// glyph's name. Absent from an aiball that predates them.
    pub turn: Option<String>,
    pub band: Option<u8>,
    pub state_glyph: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Critical {
    #[serde(default)]
    pub holds: u32,
}

impl TicketRow {
    /// The agent working on it: its assignee, else its claimant.
    pub fn holder(&self) -> Option<&str> {
        self.assignee.as_deref().or(self.claimant.as_deref())
    }

    /// A plan, resolution, wontfix or escalation awaits a decision.
    pub fn pending_decision(&self) -> bool {
        self.pending_plan || self.pending_resolution || self.pending_wontfix || self.pending_escalation
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
    /// Moderation: `approved`, `pending`, `rejected`.
    #[serde(default)]
    pub status: String,
    pub closed: bool,
    #[serde(default)]
    pub resolved: bool,
    pub resolved_by: Option<String>,
    pub priority: Option<String>,
    pub claimant: Option<String>,
    pub assignee: Option<String>,
    /// The claim is still held (a lapsed one stays in `claimant`).
    #[serde(default)]
    pub is_claim: bool,
    /// The live step, when the last word is a `then: continue`.
    pub step: Option<Step>,
    pub critical: Option<Critical>,
    pub token_usage: Option<TokenUsage>,
    /// The ticket's own meta: a decision when it was filed with one.
    pub meta: Option<String>,
    #[serde(default)]
    pub relations: Vec<Relation>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Step {
    pub resume_at: Option<String>,
    pub resume_on_ticket: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TokenUsage {
    #[serde(default)]
    pub tokens_in: u64,
    #[serde(default)]
    pub tokens_out: u64,
    #[serde(default)]
    pub cache_w: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Relation {
    pub target_ticket_id: u64,
    /// `depends_on`, `blocks`, `relates_to`, `duplicates`, `parent_of`…
    pub kind: String,
    /// `open`, `closed`…
    pub target_stage: Option<String>,
    /// Seen from the other ticket: this one is its target.
    #[serde(default)]
    pub reciprocal: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub kind: String,
    pub by_agent: String,
    pub body: Option<String>,
    pub meta: Option<String>,
    pub created_at: String,
    /// Moderation: `approved`, `pending`, `rejected`.
    #[serde(default)]
    pub status: String,
    /// For an event: the ticket it comes from or points to.
    pub source_ticket_id: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    /// `plan`, `resolution`, `wontfix`, `escalation`.
    pub kind: String,
    /// `pending`, `accepted`, `rejected`.
    pub status: String,
    pub decided_by: Option<String>,
}

fn meta_of(meta: Option<&str>) -> Option<Value> {
    serde_json::from_str(meta?).ok()
}

fn decision_of(meta: Option<&str>) -> Option<Decision> {
    let meta = meta_of(meta)?;
    let decision = meta.get("decision")?;
    Some(Decision {
        kind: decision.get("kind")?.as_str()?.to_string(),
        status: decision.get("status")?.as_str()?.to_string(),
        decided_by: decision.get("decided_by").and_then(Value::as_str).map(str::to_string),
    })
}

impl TicketHeader {
    /// The decision the ticket was filed with, if any.
    pub fn decision(&self) -> Option<Decision> {
        decision_of(self.meta.as_deref())
    }

    /// Who works on it: its assignee, else whoever still holds the claim.
    pub fn holder(&self) -> Option<&str> {
        self.assignee
            .as_deref()
            .or(self.claimant.as_deref().filter(|_| self.is_claim))
    }
}

impl Comment {
    pub fn decision(&self) -> Option<Decision> {
        decision_of(self.meta.as_deref())
    }

    /// The one-line state of the ticket its author left with it.
    pub fn summary_until(&self) -> Option<String> {
        let meta = meta_of(self.meta.as_deref())?;
        let summary = meta.get("summary_until")?.as_str()?.trim();
        (!summary.is_empty()).then(|| summary.to_string())
    }

    /// A step: `then: continue`.
    pub fn is_step(&self) -> bool {
        meta_of(self.meta.as_deref())
            .and_then(|m| m.get("step").and_then(Value::as_bool))
            .unwrap_or(false)
    }

    /// The commits the comment cites, as short SHAs.
    pub fn commits(&self) -> Vec<String> {
        let Some(meta) = meta_of(self.meta.as_deref()) else {
            return Vec::new();
        };
        let Some(commits) = meta.get("commits").and_then(Value::as_array) else {
            return Vec::new();
        };
        commits
            .iter()
            .filter_map(|c| c.as_str().or_else(|| c.get("sha").and_then(Value::as_str)))
            .map(|sha| sha.chars().take(7).collect())
            .collect()
    }
}

impl Aiball {
    /// [`socket_path`]; acting as `$TVTY_USER`,
    /// else as the local owner until [`Aiball::find_user`] knows better.
    pub fn from_env() -> Self {
        let socket = socket_path();
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

    /// The open tickets of a project, as the web UI's list rows.
    pub fn open_tickets(&self, project: &str) -> anyhow::Result<Vec<TicketRow>> {
        self.get(&format!("/api/inbox?project={}&open=1&v=tvty&sort=band", encode(project)))
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

    /// Moderation: approve or reject a pending ticket or comment.
    pub fn moderate(&self, message: u64, approve: bool) -> anyhow::Result<()> {
        let verb = if approve { "approve" } else { "reject" };
        self.post(&format!("/api/messages/{message}/{verb}"), json!({})).map(drop)
    }

    /// Closes or reopens a ticket.
    pub fn set_closed(&self, project: &str, ticket: u64, closed: bool) -> anyhow::Result<()> {
        self.post(
            "/api/messages",
            json!({
                "project": project,
                "kind": if closed { "ticket_closed" } else { "ticket_reopened" },
                "ticket_id": ticket,
                "parent_id": ticket,
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

/// aiball's local socket: `$AIBALL_SOCK`, else its default place.
pub fn socket_path() -> PathBuf {
    std::env::var("AIBALL_SOCK")
        .ok()
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(home).join(".local/share/aiball/sock")
        })
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
