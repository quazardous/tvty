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
    /// Its events not seen yet.
    #[serde(default)]
    pub ping_unseen: Option<u32>,
    /// Its wait credit, per project.
    #[serde(default)]
    pub wait_credit: Option<Vec<WaitCredit>>,
}

/// An agent's loop bar, as its loop pushes it to aiball: what claude-loop
/// paints in tmux's status line, as facts and absolute times.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct AgentBar {
    /// `boot`, `idle` or `busy`.
    pub phase: String,
    /// `boot`, `stop` (a human types), `wait` (held) or `loop`.
    pub presence: String,
    pub afk: BarAfk,
    pub prompt: BarPrompt,
    pub human_typing: bool,
    pub marker: BarMarker,
    pub alerts: BarAlerts,
    pub proxy_alive: bool,
    pub zen: bool,
    pub counters: Option<BarCounters>,
    pub next_wake_at: Option<String>,
    pub boot: Option<BarBoot>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarAfk {
    /// `off`, `wait_10m` or `wait_inf`.
    pub mode: String,
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarPrompt {
    pub visible: bool,
    pub has_input: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarMarker {
    /// A passing word: `retry 3`, `compacting`, `resuming`…
    pub info: Option<String>,
    pub health_prompt: bool,
    pub resume_picker: bool,
    pub resume_mode_picker: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarAlerts {
    pub link_down: bool,
    pub daemon_down: bool,
    pub not_logged_in: bool,
    pub trust_dialog: bool,
    pub api_unreachable: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarCounters {
    pub open: Option<u32>,
    pub backlog: Option<u32>,
    pub events: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarBoot {
    pub started_at: String,
    pub deadline_at: Option<String>,
}

/// The last bar an agent's loop pushed; `stale` once the loop is gone.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarRead {
    pub bar: AgentBar,
    pub stale: bool,
}

/// A message (a ticket or a comment), as `GET /api/messages/:id` gives it.
#[derive(Clone, Debug, Deserialize)]
pub struct Message {
    pub kind: String,
    pub by_agent: String,
    #[serde(default)]
    pub project: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub meta: Option<String>,
    /// Moderation: `approved`, `pending`, `rejected`.
    #[serde(default)]
    pub status: String,
}

impl Message {
    /// The decision it carries (`plan`, `resolution`…), if any.
    pub fn decision_kind(&self) -> Option<String> {
        let meta: Value = serde_json::from_str(self.meta.as_deref()?).ok()?;
        meta.pointer("/decision/kind")?.as_str().map(str::to_string)
    }
}

/// An agent's own backlog, as it sees it.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentBacklog {
    #[serde(default)]
    pub rows: Vec<BacklogRow>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BacklogRow {
    pub id: u64,
    pub project: String,
    pub title: String,
    /// -1 critical, 0 hot, 1 actionable, 2 follow-up, 3 waiting on them,
    /// 4 blocked.
    pub backlog_tier: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WaitCredit {
    pub project: String,
    pub balance: i64,
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
    /// Who holds it now, and how (aiball's rule: see [`Holding`]).
    #[serde(flatten)]
    pub held: Holding,
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
    /// Computed by aiball for the reader when asked with `view=turn`: whose
    /// turn (`you`, `them`, `none`), the sorting band (0 to 5) and the state
    /// glyph's name.
    pub turn: Option<String>,
    pub band: Option<u8>,
    pub state_glyph: Option<String>,
    // What the full-screen list shows besides.
    /// The start of the ticket's body or summary.
    pub snippet: Option<String>,
    #[serde(default)]
    pub by_agent: String,
    #[serde(default)]
    pub created_at: String,
    pub intent: Option<String>,
    pub level: Option<String>,
    pub scope: Option<String>,
    #[serde(default)]
    pub blocked: bool,
    pub milestone: Option<Milestone>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    pub token_usage: Option<TokenUsage>,
    /// Snoozed until then.
    pub postponed_until: Option<String>,
    #[serde(default)]
    pub has_payload: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Tag {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Milestone {
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Critical {
    #[serde(default)]
    pub holds: u32,
}

/// Who holds a ticket now, as aiball says it on rows and headers alike.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Holding {
    /// The assignee, else the claimant while the claim is live.
    pub holder: Option<String>,
    /// `assigned`, `claim`, `lapsed_claim` (a claimant on record, no holder).
    pub held_as: Option<String>,
}

impl Holding {
    pub fn assigned(&self) -> bool {
        self.held_as.as_deref() == Some("assigned")
    }

    pub fn lapsed(&self) -> bool {
        self.held_as.as_deref() == Some("lapsed_claim")
    }
}

impl TicketRow {
    /// The agent working on it now.
    pub fn holder(&self) -> Option<&str> {
        self.held.holder.as_deref()
    }

    /// A plan, resolution, wontfix or escalation awaits a decision.
    pub fn pending_decision(&self) -> bool {
        self.pending_plan || self.pending_resolution || self.pending_wontfix || self.pending_escalation
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Thread {
    pub ticket: TicketHeader,
    #[serde(default)]
    pub comments: Vec<Comment>,
    /// The uploads its texts cite, as aiball resolves them.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// An upload a thread cites (`/uploads/<sha>.<ext>`).
#[derive(Clone, Debug, Deserialize)]
pub struct Attachment {
    /// The path the texts cite.
    #[serde(rename = "ref")]
    pub reference: String,
    pub content_type: Option<String>,
    pub bytes: Option<u64>,
    /// `file://…` where aiball keeps it, and whether that is this machine.
    pub uri: Option<String>,
    #[serde(default)]
    pub local: bool,
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
    #[serde(flatten)]
    pub held: Holding,
    /// The live step, when the last word is a `then: continue`.
    pub step: Option<Step>,
    pub critical: Option<Critical>,
    pub token_usage: Option<TokenUsage>,
    /// The ticket's own meta: a decision when it was filed with one.
    pub meta: Option<String>,
    /// Snoozed until then.
    pub postponed_until: Option<String>,
    #[serde(default)]
    pub relations: Vec<Relation>,
    // The invariants the full-screen detail shows.
    pub intent: Option<String>,
    pub level: Option<String>,
    pub scope: Option<String>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    pub milestone: Option<Milestone>,
    pub claim_until: Option<String>,
    pub parent_ticket_id: Option<u64>,
    #[serde(default)]
    pub sub_tickets: Vec<Value>,
    #[serde(default)]
    pub has_payload: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Step {
    pub resume_at: Option<String>,
    pub resume_on_ticket: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TokenUsage {
    #[serde(default)]
    pub tokens_in: u64,
    #[serde(default)]
    pub tokens_out: u64,
    #[serde(default)]
    pub cache_w: u64,
    #[serde(default)]
    pub cache_r: u64,
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
    /// Its short reference, `#C.<hashid>`.
    pub hashid: Option<String>,
    pub votes_summary: Option<Votes>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Votes {
    #[serde(default)]
    pub up: u32,
    #[serde(default)]
    pub down: u32,
    /// The reader's own vote: 1, -1, or none.
    pub mine: Option<i64>,
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

    /// Who works on it now.
    pub fn holder(&self) -> Option<&str> {
        self.held.holder.as_deref()
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
        self.get(&format!("/api/inbox?project={}&open=1&view=turn&sort=band", encode(project)))
    }

    /// A project's tickets, closed ones too: the most pressing first, at
    /// most `limit`.
    pub fn all_tickets(&self, project: &str, limit: usize) -> anyhow::Result<Vec<TicketRow>> {
        self.get(&format!(
            "/api/inbox?project={}&view=turn&sort=band&limit={limit}",
            encode(project)
        ))
    }

    pub fn thread(&self, ticket: u64) -> anyhow::Result<Thread> {
        self.get(&format!("/api/tickets/{ticket}?full=1&limit=9999"))
    }

    pub fn mark_read(&self, ticket: u64) -> anyhow::Result<()> {
        self.post(&format!("/api/tickets/{ticket}/mark-read"), json!({}))
            .map(drop)
    }

    /// Posts a comment; `quiet`: without notifying anyone (scope
    /// internal). Answers the comment's id.
    pub fn reply(&self, project: &str, ticket: u64, body: &str, quiet: bool) -> anyhow::Result<u64> {
        let mut message = json!({
            "project": project,
            "kind": "comment_added",
            "ticket_id": ticket,
            "parent_id": ticket,
            "body": body,
        });
        if quiet {
            message["scope"] = json!("internal");
        }
        let answer = self.post("/api/messages", message)?;
        answer.get("id").and_then(Value::as_u64).context("the new comment has no id")
    }

    /// Files a new ticket, labels included; answers its id.
    pub fn create(&self, ticket: &NewTicket) -> anyhow::Result<u64> {
        let body = new_ticket_message(ticket).to_string();
        // Like every client that files tickets: aiball tags the platform.
        let answer = self.request_bytes(
            "POST",
            "/api/messages",
            "application/json",
            &[("x-aiball-platform", std::env::consts::OS)],
            body.as_bytes(),
        )?;
        let answer: Value = serde_json::from_str(&answer).unwrap_or(Value::Null);
        answer.get("id").and_then(Value::as_u64).context("the new ticket has no id")
    }

    /// An agent's loop bar; `None` when its loop never pushed one (a loop
    /// started before aiball served bars, or no loop at all).
    pub fn agent_bar(&self, agent: &str) -> anyhow::Result<Option<BarRead>> {
        match self.request("GET", &format!("/api/consumers/{}/bar", encode(agent)), None) {
            Ok(body) => Ok(Some(serde_json::from_str(&body).context("the agent's bar")?)),
            Err(error) if format!("{error}").contains(": 404") => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// A message: a ticket or a comment.
    pub fn message(&self, id: u64) -> anyhow::Result<Message> {
        self.get(&format!("/api/messages/{id}"))
    }

    /// An agent's own backlog, in `project`.
    pub fn agent_backlog(&self, agent: &str, project: &str) -> anyhow::Result<AgentBacklog> {
        self.get(&format!("/api/consumers/{}/backlog?project={}", encode(agent), encode(project)))
    }

    /// Holds or frees an agent's loop (claude-loop's AFK): `toggle`, `off`
    /// (autonomous), `arm_10m` (held ten minutes), `arm_inf` (held).
    pub fn afk(&self, agent: &str, action: &str) -> anyhow::Result<()> {
        self.post(&format!("/api/agents/{}/afk", encode(agent)), json!({ "action": action })).map(drop)
    }

    /// Marks a question (`- [ ]` in a comment) answered by a comment.
    pub fn answer_question(&self, message: u64, question: &str, answered_in: u64) -> anyhow::Result<()> {
        self.post(
            &format!("/api/messages/{message}/questions/{}/answer", encode(question)),
            json!({ "answered_in": answered_in }),
        )
        .map(drop)
    }

    /// Snoozes a ticket until `until` (ISO 8601), or wakes it (`None`).
    pub fn snooze(&self, ticket: u64, until: Option<&str>) -> anyhow::Result<()> {
        match until {
            Some(until) => self.post(&format!("/api/tickets/{ticket}/postpone"), json!({ "until": until })),
            None => self.post(&format!("/api/tickets/{ticket}/unsnooze"), json!({})),
        }
        .map(drop)
    }

    pub fn set_priority(&self, ticket: u64, priority: &str) -> anyhow::Result<()> {
        self.post(&format!("/api/messages/{ticket}/edit"), json!({ "priority": priority }))
            .map(drop)
    }

    /// Edits a ticket's fields: `{"title": …}`, `{"priority": …}`,
    /// `{"intent": …}`, `{"level": …}`, `{"scope": …}`, `{"body": …}`.
    pub fn edit(&self, message: u64, fields: Value) -> anyhow::Result<()> {
        self.post(&format!("/api/messages/{message}/edit"), fields).map(drop)
    }

    pub fn add_tag(&self, ticket: u64, tag: &str) -> anyhow::Result<()> {
        self.post(&format!("/api/messages/{ticket}/tags"), json!({ "tag": tag }))
            .map(drop)
    }

    pub fn remove_tag(&self, ticket: u64, tag: &str) -> anyhow::Result<()> {
        self.request("DELETE", &format!("/api/messages/{ticket}/tags/{}", encode(tag)), None)
            .map(drop)
    }

    /// The tags a project's tickets can carry.
    pub fn tag_catalog(&self, project: &str) -> anyhow::Result<Vec<String>> {
        let tags: Vec<Tag> = self.get(&format!("/api/tags?project={}", encode(project)))?;
        Ok(tags.into_iter().map(|t| t.name).collect())
    }

    /// A project's milestones not yet released: (id, title).
    pub fn milestones(&self, project: &str) -> anyhow::Result<Vec<(u64, String)>> {
        let answer: Value = self.get(&format!("/api/projects/{}/milestones", encode(project)))?;
        Ok(answer
            .get("milestones")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|m| !m.get("released").and_then(Value::as_bool).unwrap_or(false))
            .filter_map(|m| {
                let id = m.get("id")?.as_u64()?;
                let title = m.get("title").and_then(Value::as_str).unwrap_or("").to_string();
                Some((id, title))
            })
            .collect())
    }

    pub fn set_milestone(&self, ticket: u64, milestone: Option<u64>) -> anyhow::Result<()> {
        self.post(&format!("/api/tickets/{ticket}/milestone"), json!({ "milestone_id": milestone }))
            .map(drop)
    }

    /// Assigns the ticket to `who`, or releases it (`None`).
    pub fn assign(&self, ticket: u64, who: Option<&str>) -> anyhow::Result<()> {
        match who {
            Some(who) => self.post(&format!("/api/tickets/{ticket}/assign"), json!({ "assignee": who })),
            None => self.post(&format!("/api/tickets/{ticket}/release"), json!({})),
        }
        .map(drop)
    }

    /// Makes `who` the ticket's reporter (its owner).
    pub fn set_owner(&self, ticket: u64, who: &str) -> anyhow::Result<()> {
        self.post(&format!("/api/tickets/{ticket}/owner"), json!({ "owner": who })).map(drop)
    }

    /// Relates the ticket to `target`; `ignored` removes the relation.
    pub fn relate(&self, ticket: u64, target: u64, kind: &str) -> anyhow::Result<()> {
        self.post(
            &format!("/api/tickets/{ticket}/relations"),
            json!({ "target_ticket_id": target, "kind": kind }),
        )
        .map(drop)
    }

    pub fn move_ticket(&self, ticket: u64, project: &str) -> anyhow::Result<()> {
        self.post(&format!("/api/tickets/{ticket}/move"), json!({ "project": project })).map(drop)
    }

    /// Deletes a comment (aiball keeps a tombstone).
    pub fn delete_comment(&self, comment: u64) -> anyhow::Result<()> {
        self.post(&format!("/api/messages/{comment}/delete"), json!({})).map(drop)
    }

    /// Makes a comment a pending decision of `kind`.
    pub fn classify(&self, comment: u64, kind: &str) -> anyhow::Result<()> {
        self.post(&format!("/api/messages/{comment}/promote"), json!({ "kind": kind })).map(drop)
    }

    /// Takes a pending decision off a comment.
    pub fn untag(&self, comment: u64) -> anyhow::Result<()> {
        self.post(&format!("/api/messages/{comment}/untag"), json!({})).map(drop)
    }

    /// Marks an agent's comment as a step, or unmarks it.
    pub fn set_step(&self, comment: u64, step: bool) -> anyhow::Result<()> {
        let verb = if step { "step" } else { "unstep" };
        self.post(&format!("/api/messages/{comment}/{verb}"), json!({})).map(drop)
    }

    /// Votes on a comment: 1, -1, or 0 to take the vote back.
    pub fn vote(&self, comment: u64, value: i64) -> anyhow::Result<()> {
        self.post(&format!("/api/messages/{comment}/vote"), json!({ "value": value })).map(drop)
    }

    /// Makes a comment unread again for those it notified.
    pub fn resurface(&self, comment: u64) -> anyhow::Result<()> {
        self.post(&format!("/api/messages/{comment}/resurface"), json!({})).map(drop)
    }

    /// Who can be @mentioned: projects, then agents.
    pub fn mention_suggestions(&self) -> anyhow::Result<Vec<String>> {
        let (projects, agents) = self.projects_and_agents()?;
        Ok(projects.into_iter().chain(agents).collect())
    }

    /// The board's projects and agents.
    pub fn projects_and_agents(&self) -> anyhow::Result<(Vec<String>, Vec<String>)> {
        #[derive(Deserialize)]
        struct Suggestions {
            projects: Vec<String>,
            agents: Vec<String>,
        }
        let s: Suggestions = self.get("/api/mention-suggestions")?;
        Ok((s.projects, s.agents))
    }

    /// Stores a file in aiball's uploads; answers its URL, to cite in a
    /// comment.
    pub fn upload(&self, bytes: &[u8], content_type: &str, name: &str) -> anyhow::Result<String> {
        let answer = self.request_bytes(
            "POST",
            "/api/uploads",
            content_type,
            &[("x-aiball-upload-name", name)],
            bytes,
        )?;
        let answer: Value = serde_json::from_str(&answer)?;
        answer.get("url").and_then(Value::as_str).map(str::to_string).context("the upload has no url")
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
            }),
        )
        .map(drop)
    }

    /// Accept or reject the decision a comment carries.
    pub fn decide(&self, comment: u64, accept: bool) -> anyhow::Result<()> {
        let status = if accept { "accepted" } else { "rejected" };
        self.post(&format!("/api/messages/{comment}/decide"), json!({ "status": status }))
            .map(drop)
    }

    /// A read whose answer is taken as it comes.
    pub fn get_value(&self, path: &str) -> anyhow::Result<Value> {
        self.get(path)
    }

    fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let body = self.request("GET", path, None)?;
        serde_json::from_str(&body).with_context(|| format!("GET {path}"))
    }

    fn post(&self, path: &str, body: Value) -> anyhow::Result<Value> {
        let answer = self.request("POST", path, Some(body))?;
        Ok(serde_json::from_str(&answer).unwrap_or(Value::Null))
    }

    fn request(&self, method: &str, path: &str, body: Option<Value>) -> anyhow::Result<String> {
        let body = body.map(|b| b.to_string()).unwrap_or_default();
        self.request_bytes(method, path, "application/json", &[], body.as_bytes())
    }

    /// An upload's bytes, through the socket (`/api/uploads/<sha>`).
    pub fn upload_bytes(&self, reference: &str) -> anyhow::Result<Vec<u8>> {
        self.request_raw("GET", reference, "application/octet-stream", &[], &[])
    }

    fn request_bytes(
        &self,
        method: &str,
        path: &str,
        content_type: &str,
        headers: &[(&str, &str)],
        body: &[u8],
    ) -> anyhow::Result<String> {
        let answer = self.request_raw(method, path, content_type, headers, body)?;
        Ok(String::from_utf8_lossy(&answer).into_owned())
    }

    #[cfg(unix)]
    fn request_raw(
        &self,
        method: &str,
        path: &str,
        content_type: &str,
        headers: &[(&str, &str)],
        body: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        use std::os::unix::net::UnixStream;

        let mut stream = UnixStream::connect(&self.socket)
            .with_context(|| format!("aiball's socket {}", self.socket.display()))?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        // HTTP/1.0: the daemon closes the connection after the answer, so
        // reading to the end reads exactly one answer, never chunked.
        let extra: String = headers.iter().map(|(k, v)| format!("{k}: {v}\r\n")).collect();
        let head = format!(
            "{method} {path} HTTP/1.0\r\nHost: aiball\r\nx-aiball-consumer: {user}\r\n\
             content-type: {content_type}\r\n{extra}content-length: {len}\r\n\r\n",
            user = self.user,
            len = body.len(),
        );
        stream.write_all(head.as_bytes())?;
        stream.write_all(body)?;
        let mut answer = Vec::new();
        stream.read_to_end(&mut answer)?;
        let split = answer
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .context("malformed answer from aiball")?;
        let (head, body) = (String::from_utf8_lossy(&answer[..split]), &answer[split + 4..]);
        let status = head.split(' ').nth(1).unwrap_or("");
        if !status.starts_with('2') {
            bail!("{method} {path}: {status} {}", String::from_utf8_lossy(body).trim());
        }
        Ok(body.to_vec())
    }

    #[cfg(not(unix))]
    fn request_raw(&self, _: &str, _: &str, _: &str, _: &[(&str, &str)], _: &[u8]) -> anyhow::Result<Vec<u8>> {
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
pub(crate) fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// A ticket to file: what goes in the one call that creates it.
#[derive(Clone, Debug, PartialEq)]
pub struct NewTicket {
    pub project: String,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub intent: String,
    pub priority: String,
    pub scope: String,
    pub parent: Option<u64>,
    /// Tag names.
    pub tags: Vec<String>,
    pub assignee: Option<String>,
    /// A milestone's id.
    pub milestone: Option<u64>,
    pub level: String,
}

/// The message that files `ticket` whole, labels included: aiball checks
/// everything first and files it at once, or refuses it at once. The
/// defaults (normal priority, default scope, a task) and empty fields left
/// out; its author is the caller, whom aiball knows from the connection.
pub fn new_ticket_message(ticket: &NewTicket) -> Value {
    let mut message = json!({
        "project": ticket.project,
        "kind": "ticket_created",
        "title": ticket.title.trim(),
        "body": ticket.body,
        "intent": ticket.intent,
    });
    if !ticket.summary.trim().is_empty() {
        message["summary"] = json!(ticket.summary.trim());
    }
    if ticket.priority != "normal" {
        message["priority"] = json!(ticket.priority);
    }
    if ticket.scope != "default" {
        message["scope"] = json!(ticket.scope);
    }
    if let Some(parent) = ticket.parent {
        message["parent_id"] = json!(parent);
    }
    if !ticket.tags.is_empty() {
        message["tags"] = json!(ticket.tags);
    }
    if let Some(assignee) = &ticket.assignee {
        message["assignee"] = json!(assignee);
    }
    if let Some(milestone) = ticket.milestone {
        message["milestone"] = json!(milestone);
    }
    if ticket.level != "task" {
        message["level"] = json!(ticket.level);
    }
    message
}

#[cfg(test)]
mod new_ticket_tests {
    use super::{NewTicket, new_ticket_message};
    use serde_json::json;

    fn ticket() -> NewTicket {
        NewTicket {
            project: "demo".into(),
            title: "  A title ".into(),
            summary: " ".into(),
            body: "words".into(),
            intent: "request".into(),
            priority: "normal".into(),
            scope: "default".into(),
            parent: None,
            tags: Vec::new(),
            assignee: None,
            milestone: None,
            level: "task".into(),
        }
    }

    #[test]
    fn the_defaults_stay_out() {
        assert_eq!(
            new_ticket_message(&ticket()),
            json!({ "project": "demo", "kind": "ticket_created", "title": "A title", "body": "words", "intent": "request" })
        );
    }

    #[test]
    fn what_is_set_goes_in() {
        let t = NewTicket { summary: "short".into(), priority: "high".into(), scope: "internal".into(), parent: Some(12), ..ticket() };
        let m = new_ticket_message(&t);
        assert_eq!((m["summary"].clone(), m["priority"].clone(), m["scope"].clone(), m["parent_id"].clone()), (json!("short"), json!("high"), json!("internal"), json!(12)));
    }

    #[test]
    fn the_labels_go_in_the_same_message() {
        let t = NewTicket {
            tags: vec!["ui".into()],
            assignee: Some("demo-crew".into()),
            milestone: Some(7),
            level: "roadmap".into(),
            ..ticket()
        };
        let m = new_ticket_message(&t);
        assert_eq!(
            (m["tags"].clone(), m["assignee"].clone(), m["milestone"].clone(), m["level"].clone()),
            (json!(["ui"]), json!("demo-crew"), json!(7), json!("roadmap"))
        );
    }
}
