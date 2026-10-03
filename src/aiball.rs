//! aiball's HTTP API, where aiball is (tvty-ipc's `aiball::locate`,
//! docs/IPC.md): its socket, which trusts the same user, or TCP with the
//! human's token. Either way the identity rides in the `x-aiball-consumer`
//! header — over TCP aiball honours it for a human's token. tvty acts as the
//! human it runs for.
//!
//! Every call blocks: run them off the UI thread.

use anyhow::{Context as _, anyhow, bail};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tvty_ipc::aiball::{LocateError, Location};

#[derive(Clone, Debug)]
pub struct Aiball {
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
    /// It works on another machine: its `cwd` is that machine's.
    #[serde(default)]
    pub remote: Option<bool>,
    /// The machine its loop is connected from, as aiball names them: `hub`,
    /// `node:<label>`; none without a loop connected.
    #[serde(default)]
    pub machine: Option<String>,
    /// Its events not seen yet.
    #[serde(default)]
    pub ping_unseen: Option<u32>,
    /// The session aiball's host runs for it, when its Claude runs there
    /// (not in claude-loop's tmux).
    #[serde(default)]
    pub session: Option<HostedSession>,
    /// Its counters as the daemon computes them, loop or not; none until
    /// computed once.
    #[serde(default)]
    pub counters: Option<AgentCounters>,
}

/// aiball's managed config, as one layer sees it (the board's, or a
/// project's): every setting, described.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ManagedConfig {
    #[serde(default)]
    pub project: Option<String>,
    pub config: Vec<ConfigEntry>,
}

/// One setting of aiball's config.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    /// `global`, `project` or `global+project`: the layers it has.
    pub scope: String,
    /// `boolean`, `enum`, `number`, `duration` (seconds), `string`.
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub protected: bool,
    pub label: String,
    #[serde(default)]
    pub description: String,
    /// Its section, a dotted path (`tickets.steps`).
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub default: Value,
    /// The board's own value, and the project's (null: not set there).
    #[serde(default)]
    pub global: Value,
    #[serde(default)]
    pub project: Value,
    /// The value in force in this layer.
    #[serde(default)]
    pub value: Value,
    /// Where aiball reads it: `db` (what `config.set` writes), `file` (a
    /// project's `.aiball.yaml`); `["db"]` when unsaid, as aiball has it.
    #[serde(default = "db_only")]
    pub sources: Vec<String>,
}

fn db_only() -> Vec<String> {
    vec!["db".into()]
}

impl ConfigEntry {
    /// `config.set` can write it (it is not read from a file only).
    pub fn writable(&self) -> bool {
        self.sources.iter().any(|s| s == "db")
    }

    /// It can be set in a project's layer (`in_project`) or the board's.
    pub fn settable_in(&self, in_project: bool) -> bool {
        self.writable() && if in_project { self.has_project() } else { self.has_global() }
    }

    pub fn has_global(&self) -> bool {
        self.scope.split('+').any(|s| s == "global")
    }

    pub fn has_project(&self) -> bool {
        self.scope.split('+').any(|s| s == "project")
    }
}

/// An agent's counters, as claude-loop's line has them (o: b: e:).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct AgentCounters {
    #[serde(default)]
    pub open: Option<u32>,
    #[serde(default)]
    pub backlog: Option<u32>,
    #[serde(default)]
    pub events: Option<u32>,
}

/// An agent's session on aiball's host.
/// A folder's Claude Code conversations (`session.conversations`).
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Conversations {
    /// The one a loop of the folder follows already, if it still exists.
    #[serde(default)]
    pub tracked: Option<String>,
    #[serde(default)]
    pub conversations: Vec<Conversation>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Conversation {
    pub id: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    /// The user's first words in it, cut short.
    #[serde(default)]
    pub first_prompt: Option<String>,
    /// Another agent's loop that runs on it.
    #[serde(default)]
    pub held_by: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct HostedSession {
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub attach: Option<AttachPoint>,
    /// Its loop's tmux session, when it runs in claude-loop's tmux.
    #[serde(default)]
    pub tmux: Option<String>,
    /// The clients attached to it: terminals, tvty's included.
    #[serde(default)]
    pub clients: Option<u32>,
    /// Those of them with the controls (a tmux loop's; a host does not say).
    #[serde(default)]
    pub interactive: Option<u32>,
    /// The machine that holds it, as aiball names them: attached to from
    /// that machine only.
    #[serde(default)]
    pub machine: Option<String>,
}

/// Where a client attaches: a socket on this machine, or none (`reason`).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct AttachPoint {
    pub socket: Option<String>,
}

impl HostedSession {
    /// Its attach socket, while it runs.
    pub fn socket(&self) -> Option<&str> {
        self.attach.as_ref().and_then(|a| a.socket.as_deref()).filter(|_| self.running)
    }
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
    /// While its usage limit is reached: when it resets, as Claude Code
    /// says it, and as a date when it could be read.
    #[serde(default)]
    pub limit_resets: Option<LimitResets>,
    /// Whether its Claude is in Remote Control, whatever turned it on (its
    /// folder's setting, or `/rc` typed): none from an older loop.
    #[serde(default)]
    pub remote_control: Option<BarRemoteControl>,
    /// The model its Claude ran its last turn on, with its price and a
    /// newer one of its family, as aiball's model list says: none before
    /// the first turn ends, or from an older loop.
    #[serde(default)]
    pub model: Option<BarModel>,
    /// The tool calls Claude Code refused it (its auto mode's classifier,
    /// a deny rule): a refused agent stops there. None after an hour without
    /// one, or from a loop older than aiball 0.55.
    #[serde(default)]
    pub denials: Option<BarDenials>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarDenials {
    #[serde(default)]
    pub last_hour: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub last_at: Option<String>,
    /// As Claude Code gives it ("Auto-Mode Bypass").
    #[serde(default)]
    pub last_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarModel {
    /// `claude-opus-5-5`.
    pub id: String,
    /// `Opus 5.5`.
    pub name: String,
    /// USD per million tokens.
    #[serde(default)]
    pub cost: Option<ModelCost>,
    #[serde(default)]
    pub newer: Option<NewerModel>,
    /// The list the price and the newer model come from.
    #[serde(default)]
    pub catalog: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct ModelCost {
    pub input: f64,
    pub output: f64,
}

impl ModelCost {
    /// `$15 / $75 per M tokens`.
    pub fn said(&self) -> String {
        let usd = |n: f64| if n.fract() == 0. { format!("${n:.0}") } else { format!("${n}") };
        format!("{} / {} per M tokens", usd(self.input), usd(self.output))
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct NewerModel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub cost: Option<ModelCost>,
}

impl BarModel {
    /// What its tip says: its id, its price, a newer one of its family.
    pub fn said(&self) -> String {
        let mut said = self.id.clone();
        if let Some(cost) = &self.cost {
            said.push_str(&format!(" · {}", cost.said()));
        }
        if let Some(newer) = &self.newer {
            said.push_str(&format!("\n{} is out", newer.name));
            if let Some(cost) = &newer.cost {
                said.push_str(&format!(" ({})", cost.said()));
            }
        }
        if let Some(catalog) = &self.catalog {
            said.push_str(&format!("\nprices from {catalog}"));
        }
        said
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarRemoteControl {
    pub on: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LimitResets {
    pub text: String,
    pub at: Option<String>,
}

impl AgentBar {
    /// "usage limit reached", and when it resets if said.
    /// Its refused tool calls in the last hour, said: how many, how long
    /// since the last, and why. None without any.
    pub fn denials_said(&self) -> Option<String> {
        let d = self.denials.as_ref().filter(|d| d.last_hour > 0)?;
        let ago = d
            .last_at
            .as_deref()
            .and_then(crate::status::parse_time)
            .map(|at| format!(", the last {} ago", crate::status::ago(crate::status::now().saturating_sub(at))))
            .unwrap_or_default();
        let why = d.last_reason.as_deref().filter(|r| !r.is_empty()).map(|r| format!(": {r}")).unwrap_or_default();
        Some(format!(
            "{} tool call{} refused by Claude Code in the last hour{ago}{why} — a refused agent stops there",
            d.last_hour,
            if d.last_hour == 1 { "" } else { "s" }
        ))
    }

    pub fn limit_said(&self) -> Option<String> {
        self.alerts.limit_reached.then(|| match &self.limit_resets {
            Some(resets) if !resets.text.is_empty() => format!("usage limit reached · resets {}", resets.text),
            _ => "usage limit reached".to_string(),
        })
    }
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
    /// Its Claude Code installed an update and waits for a restart.
    #[serde(default)]
    pub restart_needed: bool,
    /// A restart was asked and waits for its Claude's next idle.
    #[serde(default)]
    pub restart_pending: bool,
    /// Its Claude hit a usage limit (weekly, session, monthly spend…): the
    /// loop holds it until let go.
    #[serde(default)]
    pub limit_reached: bool,
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
    /// When tvty saw its phase change (seconds since the epoch): the bar
    /// says the phase, not since when; none when it has not changed since
    /// tvty started.
    #[serde(skip)]
    pub phase_since: Option<u64>,
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
    /// Sunk: a backlog wake gave it to the agent, and its loop will not
    /// bring it up again before this, unless the thread moves.
    #[serde(default)]
    pub backlog_cooled_until: Option<String>,
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
    /// When the step's agent said it resumes.
    #[serde(default)]
    pub step_resume_at: Option<String>,
    #[serde(default)]
    pub latest_plan_rejected: bool,
    #[serde(default)]
    pub latest_resolution_rejected: bool,
    /// Computed by aiball for the reader when asked with `view=turn`: whose
    /// turn (`you`, `them`, `none`), the sorting band's name and the state
    /// glyph's name.
    pub turn: Option<String>,
    pub band_name: Option<String>,
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
    /// How long nothing moved on it ("9 d"), when it went quiet.
    #[serde(default)]
    pub quiet: Option<String>,
}

impl Critical {
    /// How long it went quiet, when it did (aiball says "" when it did not).
    pub fn quiet(&self) -> Option<&str> {
        self.quiet.as_deref().map(str::trim).filter(|q| !q.is_empty())
    }

    /// Its quiet span in minutes, to rank by ("45 m", "3 h", "9 d", "2 w").
    pub fn quiet_minutes(&self) -> u64 {
        let Some(quiet) = self.quiet() else { return 0 };
        let (number, unit) = quiet.split_once(' ').unwrap_or((quiet, "m"));
        let n: u64 = number.parse().unwrap_or(0);
        n * match unit.chars().next() {
            Some('h') => 60,
            Some('d') => 60 * 24,
            Some('w') => 60 * 24 * 7,
            _ => 1,
        }
    }

    /// What it holds back, and for how long: "holds 2 · quiet 9 d".
    pub fn said(&self) -> String {
        match self.quiet() {
            Some(quiet) => format!("holds {} · quiet {quiet}", self.holds),
            None => format!("holds {}", self.holds),
        }
    }
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

/// Someone's own choice about a ticket: to follow it, or to mute it.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Subscriber {
    pub consumer_id: String,
    #[serde(default)]
    pub muted: bool,
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

    /// When a step's agent resumes: at a time (`resume_on.timer`), or when
    /// another ticket moves (`resume_on.ticket`), whichever comes first.
    pub fn step_resume(&self) -> (Option<String>, Option<u64>) {
        let Some(meta) = meta_of(self.meta.as_deref()) else { return (None, None) };
        let at = meta.get("step_resume_at").and_then(Value::as_str).map(str::to_string);
        (at, meta.get("step_resume_on_ticket").and_then(Value::as_u64))
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

/// How long a loop's start is waited for: Claude resuming its conversation
/// takes up to a minute.
const LOOP_START_WAIT: std::time::Duration = std::time::Duration::from_secs(90);

/// aiball's bus, shared by every copy of [`Aiball`]: the calls aiball has
/// made methods go through it.
static WIRE: std::sync::OnceLock<crate::wire::Wire> = std::sync::OnceLock::new();

/// Opens aiball's bus, once, as `user` for now; its notifications go to
/// `notices`.
pub fn start_wire(user: &str, notices: futures::channel::mpsc::UnboundedSender<crate::wire::Notification>) -> crate::wire::Wire {
    WIRE.get_or_init(|| crate::wire::Wire::start(user.to_string(), notices)).clone()
}

impl Aiball {
    /// A method of aiball's bus: its answer, read as `T`.
    fn rpc<T: DeserializeOwned>(&self, method: &str, params: Value) -> anyhow::Result<T> {
        let wire = WIRE.get().context("aiball's bus is not open")?;
        let answer = wire.call(method, params)?;
        serde_json::from_value(answer).with_context(|| format!("{method}: an answer tvty does not read"))
    }

    /// A method of aiball's bus, its answer read as `T`: for a module that
    /// speaks one part of the bus itself (the loops).
    pub fn call<T: DeserializeOwned>(&self, method: &str, params: Value) -> anyhow::Result<T> {
        self.rpc(method, params)
    }

    /// A method that starts a loop (`session.start`, `loop.restart`): the
    /// daemon answers once the loop runs, which takes up to a minute while
    /// Claude resumes its conversation. On a connection of its own, waited
    /// for as long: on the shared one it would hold up every other call
    /// behind it, and give up before the loop is there. Blocking: off the UI
    /// thread.
    pub fn call_starting<T: DeserializeOwned>(&self, method: &str, params: Value) -> anyhow::Result<T> {
        let wire = WIRE.get().context("aiball's bus is not open")?;
        let answer = wire.call_alone(method, params, LOOP_START_WAIT)?;
        serde_json::from_value(answer).with_context(|| format!("{method}: an answer tvty does not read"))
    }

    /// A method of aiball's bus whose answer does not matter.
    fn rpc_do(&self, method: &str, params: Value) -> anyhow::Result<()> {
        self.rpc::<Value>(method, params).map(drop)
    }

    /// aiball where [`location`] finds it, at each call; acting as `$TVTY_USER`, else as
    /// the local owner until [`Aiball::find_user`] knows better.
    pub fn from_env() -> Self {
        let user = std::env::var("TVTY_USER").unwrap_or_else(|_| "human".into());
        Self { user }
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
        // The bus runs as the user too: the author of a write is the caller.
        if let Some(wire) = WIRE.get() {
            wire.set_user(&self.user);
        }
    }

    pub fn consumers(&self) -> anyhow::Result<Vec<Consumer>> {
        self.rpc("consumer.list", json!({}))
    }

    /// A project's tickets, closed ones too: the most pressing first, at
    /// most `limit`.
    pub fn all_tickets(&self, project: &str, limit: usize) -> anyhow::Result<Vec<TicketRow>> {
        self.inbox(json!({ "project": project, "view": "turn", "sort": "band", "limit": limit }))
    }

    /// `inbox.list`: its rows (it also says the total, which tvty does not show).
    fn inbox(&self, params: Value) -> anyhow::Result<Vec<TicketRow>> {
        #[derive(Deserialize)]
        struct Inbox {
            rows: Vec<TicketRow>,
        }
        Ok(self.rpc::<Inbox>("inbox.list", params)?.rows)
    }

    pub fn thread(&self, ticket: u64) -> anyhow::Result<Thread> {
        self.rpc("ticket.get", json!({ "id": ticket, "full": true, "limit": 9999 }))
    }

    pub fn mark_read(&self, ticket: u64) -> anyhow::Result<()> {
        self.rpc_do("ticket.mark_read", json!({ "id": ticket }))
    }

    /// What a loop started in `cwd` would start with, each value with where
    /// it comes from, as aiball resolves it (`project.settings`). tvty never
    /// reads the folder's `.aiball.yaml` itself.
    pub fn project_settings(&self, cwd: &str) -> anyhow::Result<ProjectSettings> {
        self.rpc("project.settings", json!({ "cwd": cwd }))
    }

    /// Sets where the folder's loops run (`session`) and its Claude's Remote
    /// Control (`remote_control`) in the `.aiball.yaml` they read
    /// (`project.settings_set`): the keys `patch` gives, a null one removed
    /// (the layer below applies again). Answers the settings as they now are.
    pub fn project_settings_set(&self, cwd: &str, patch: Value) -> anyhow::Result<ProjectSettings> {
        let mut params = patch;
        params["cwd"] = json!(cwd);
        self.rpc("project.settings_set", params)
    }

    /// A project's standing instruction and wake focus
    /// (`project.standing_prompt`).
    pub fn standing(&self, project: &str) -> anyhow::Result<Standing> {
        self.rpc("project.standing_prompt", json!({ "project": project }))
    }

    /// Every project's standing, in one call (`project.list { detailed }`):
    /// its instruction and whether a focus applies, as said — not the
    /// focus's tickets nor its end, which [`Aiball::standing`] gives.
    pub fn standings(&self) -> anyhow::Result<Vec<Standing>> {
        #[derive(Deserialize)]
        struct Row {
            name: String,
            #[serde(flatten)]
            standing: Standing,
        }
        let rows: Vec<Row> = self.rpc("project.list", json!({ "detailed": true }))?;
        Ok(rows.into_iter().map(|r| Standing { project: r.name, ..r.standing }).collect())
    }

    /// Sets a project's standing instruction (none: cleared) and its wake
    /// focus: the tickets (none: cleared) and until when (an ISO date).
    pub fn set_standing(&self, project: &str, prompt: Option<&str>, focus: Option<&str>, until: Option<&str>) -> anyhow::Result<Standing> {
        self.rpc(
            "project.set_standing_prompt",
            json!({ "project": project, "standing_prompt": prompt, "focus_tickets": focus, "focus_until": until }),
        )
    }

    /// Types `message` into every running agent loop now; `hold`: holds
    /// them too (not AFK ∞) until released.
    pub fn message_all(&self, message: &str, hold: bool) -> anyhow::Result<Vec<LoopHold>> {
        let answer: LoopHolds = self.rpc("loops.message_all", json!({ "message": message, "hold": hold }))?;
        Ok(answer.results)
    }

    /// Lifts the hold on every running agent loop.
    pub fn release_all(&self) -> anyhow::Result<Vec<LoopHold>> {
        let answer: LoopHolds = self.rpc("loops.release_all", json!({}))?;
        Ok(answer.results)
    }

    /// Makes a folder an aiball project (`.mcp.json`, `.aiball.yaml`), as
    /// `aiball init` would; `dry_run`: says what it would do, writes nothing.
    pub fn project_init(&self, ask: &InitAsk, dry_run: bool) -> anyhow::Result<InitDone> {
        // The choices left out unless made, as the command's flags: a
        // lead's .aiball.yaml says no role.
        let mut params = json!({ "cwd": ask.cwd, "project": ask.project, "agent": ask.agent, "dry_run": dry_run });
        if ask.crew {
            params["role"] = json!("crew");
        }
        if ask.private {
            params["private"] = json!(true);
        }
        if ask.no_claim {
            params["no_claim"] = json!(true);
        }
        self.rpc("project.init", params)
    }

    /// What a message (a ticket, a comment) says: its body, if any.
    pub fn message_body(&self, id: u64) -> anyhow::Result<Option<String>> {
        let message: Value = self.rpc("message.get", json!({ "id": id }))?;
        Ok(message.get("body").and_then(Value::as_str).map(str::to_string))
    }

    pub fn mark_unread(&self, ticket: u64) -> anyhow::Result<()> {
        self.rpc_do("ticket.mark_unread", json!({ "id": ticket }))
    }

    /// Marks a ticket's last word (an agent's) as a step.
    pub fn step_ticket(&self, ticket: u64) -> anyhow::Result<()> {
        self.rpc_do("ticket.step", json!({ "id": ticket }))
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
        let answer: Value = self.rpc("message.post", message)?;
        answer.get("id").and_then(Value::as_u64).context("the new comment has no id")
    }

    /// Files a new ticket, labels included; answers its id.
    pub fn create(&self, ticket: &NewTicket) -> anyhow::Result<u64> {
        // aiball tags the platform, which the bus connection declares.
        let answer: Value = self.rpc("message.post", new_ticket_message(ticket))?;
        answer.get("id").and_then(Value::as_u64).context("the new ticket has no id")
    }

    /// An agent's own backlog, in `project`: every row (as its loop reads
    /// it), each sunk one saying until when — with the pause its loop
    /// announced to aiball.
    pub fn agent_backlog(&self, agent: &str, project: &str) -> anyhow::Result<AgentBacklog> {
        self.rpc("consumer.backlog", json!({ "consumer_id": agent, "project": project, "limit": "500" }))
    }

    /// The agents whose session runs now, as aiball's sessions say.
    pub fn running_agents(&self) -> anyhow::Result<Vec<String>> {
        let sessions: Vec<Value> = self.rpc("session.list", json!({}))?;
        Ok(sessions
            .iter()
            .filter(|s| s.get("running").and_then(Value::as_bool) == Some(true))
            .filter_map(|s| s.get("agent").and_then(Value::as_str).map(String::from))
            .collect())
    }

    /// Whether aiball's host runs `agent`'s session, its program up — which
    /// aiball knows before the agent itself says it is there.
    pub fn host_runs(&self, agent: &str) -> anyhow::Result<bool> {
        let sessions: Vec<Value> = self.rpc("session.list", json!({}))?;
        Ok(sessions.iter().any(|s| {
            s.get("agent").and_then(Value::as_str) == Some(agent) && s.get("running").and_then(Value::as_bool) == Some(true)
        }))
    }

    /// The user's unread pings, newest first: each its message, as aiball
    /// gives it.
    pub fn unread_pings(&self, limit: u32) -> anyhow::Result<Vec<Value>> {
        let answer: Value = self.rpc("ping.list", json!({ "unread": true, "limit": limit }))?;
        Ok(answer
            .get("pings")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|p| p.get("message").cloned())
            .collect())
    }

    /// The ticket a reference names — a ticket's number, or a comment's
    /// hashid (its thread) — and its project.
    pub fn resolve_ticket(&self, reference: &str) -> anyhow::Result<(u64, String)> {
        let answer: Value = self.rpc("ticket.get", json!({ "id": reference }))?;
        let ticket = answer.get("ticket").unwrap_or(&answer);
        let id = ticket.get("id").and_then(Value::as_u64).context("ticket.get: no id")?;
        let project = ticket.get("project").and_then(Value::as_str).unwrap_or_default().to_string();
        Ok((id, project))
    }

    /// The version of the daemon tvty speaks to.
    pub fn daemon_version(&self) -> anyhow::Result<String> {
        let info: Value = self.rpc("daemon.info", json!({}))?;
        info.get("version").and_then(Value::as_str).map(str::to_string).context("daemon.info: no version")
    }

    /// Where aiball's web UI is served, as its GNOME extension opens it:
    /// the local address (tvty speaks to a daemon of this machine), else
    /// its public one.
    pub fn web_ui(&self) -> anyhow::Result<String> {
        let info: Value = self.rpc("daemon.info", json!({}))?;
        let url = |key: &str| info.get(key).and_then(Value::as_str).filter(|u| !u.is_empty()).map(str::to_string);
        url("web_url").or_else(|| url("public_url")).context("daemon.info: no web address")
    }

    /// A ticket's title, project and whether it is closed: its header alone
    /// (`ticket.get` without the thread), for a reference's tooltip.
    pub fn ticket_brief(&self, ticket: u64) -> anyhow::Result<(String, String, bool)> {
        let answer: Value = self.rpc("ticket.get", json!({ "id": ticket }))?;
        let t = answer.get("ticket").unwrap_or(&answer);
        let text = |key: &str| t.get(key).and_then(Value::as_str).unwrap_or_default().to_string();
        Ok((text("title"), text("project"), t.get("closed").and_then(Value::as_bool).unwrap_or(false)))
    }

    /// Holds or frees an agent's loop (claude-loop's AFK): `toggle`, `off`
    /// (autonomous), `arm_10m` (held ten minutes), `arm_inf` (held).
    pub fn afk(&self, agent: &str, action: &str) -> anyhow::Result<()> {
        self.rpc_do("consumer.afk", json!({ "name": agent, "action": action }))
    }

    /// Starts an agent's loop on the daemon's host, in `cwd`: for `agent` (or
    /// the folder's own), as a crew agent of that name when `crew`. Answers
    /// the agent.
    pub fn start_agent(&self, cwd: &str, project: Option<&str>, agent: Option<&str>, crew: bool) -> anyhow::Result<String> {
        // On the host, asked: whatever the folder says.
        let mut params = json!({ "cwd": cwd, "mode": "host" });
        if let Some(project) = project {
            params["project"] = json!(project);
        }
        if let Some(agent) = agent {
            params[if crew { "crew" } else { "agent" }] = json!(agent);
        }
        let answer: Value = self.call_starting("session.start", params)?;
        answer.get("agent").and_then(Value::as_str).map(str::to_string).context("session.start: no agent in the answer")
    }

    /// Starts a terminal the daemon's host holds: `argv` in `cwd`, under
    /// `name`. It lives on without tvty.
    pub fn start_terminal(&self, name: &str, argv: &[String], cwd: &str) -> anyhow::Result<()> {
        self.call_starting::<Value>("session.start", json!({ "name": name, "argv": argv, "cwd": cwd })).map(drop)
    }

    /// Claude Code's conversations in `cwd`, newest first, and the one a
    /// loop of it follows already (`tracked`) — for a crew agent, `crew`.
    pub fn conversations(&self, cwd: &str, crew: Option<&str>) -> anyhow::Result<Conversations> {
        let mut params = json!({ "cwd": cwd, "limit": 5 });
        if let Some(crew) = crew {
            params["crew"] = json!(crew);
        }
        self.rpc("session.conversations", params)
    }

    /// Forgets a stopped loop (as `claude-loop rm`): aiball no longer lists
    /// it; its folder, its `.aiball.yaml` and the tickets stay.
    pub fn forget_loop(&self, name: &str) -> anyhow::Result<()> {
        self.rpc_do("loop.remove", json!({ "name": name }))
    }

    /// Forgets an agent without a loop: its record goes; its tickets stay.
    pub fn forget_agent(&self, agent: &str) -> anyhow::Result<()> {
        self.rpc_do("consumer.delete", json!({ "consumer_id": agent }))
    }

    /// Names a terminal the daemon's host holds (`None`: its own name back);
    /// its key stays. Every client hears the new name.
    pub fn label_terminal(&self, name: &str, label: Option<&str>) -> anyhow::Result<()> {
        self.rpc_do("session.label", json!({ "name": name, "label": label }))
    }

    /// Removes a terminal the daemon's host holds (its program ended, or not).
    pub fn stop_terminal(&self, name: &str) -> anyhow::Result<()> {
        // The daemon answers once the host is gone, which it gives up to
        // 15 s (a program that ignores the hangup).
        // On a connection of its own: the other calls do not wait behind it.
        let wire = WIRE.get().context("aiball's bus is not open")?;
        wire.call_alone("session.stop", json!({ "name": name }), std::time::Duration::from_secs(20)).map(drop)
    }

    /// aiball's managed config, as the board (`None`) or a project sees it.
    pub fn config_managed(&self, project: Option<&str>) -> anyhow::Result<ManagedConfig> {
        let params = match project {
            Some(project) => json!({ "project": project }),
            None => json!({}),
        };
        self.rpc("config.managed", params)
    }

    /// A setting of aiball's config set, on the board or in a project.
    pub fn config_set(&self, key: &str, value: Value, project: Option<&str>) -> anyhow::Result<()> {
        let mut params = json!({ "key": key, "value": value });
        if let Some(project) = project {
            params["project"] = json!(project);
        }
        self.rpc_do("config.set", params)
    }

    /// A setting of aiball's config back to the layer below.
    pub fn config_clear(&self, key: &str, project: Option<&str>) -> anyhow::Result<()> {
        let mut params = json!({ "key": key });
        if let Some(project) = project {
            params["project"] = json!(project);
        }
        self.rpc_do("config.clear", params)
    }

    /// The daemon computes an agent's counters now; they come back through
    /// its state, to every client.
    pub fn refresh_counters(&self, agent: &str) -> anyhow::Result<()> {
        self.rpc_do("consumer.counters", json!({ "consumer_id": agent }))
    }

    /// Restarts an agent's Claude Code as soon as it is idle (at once when
    /// it is), resuming its conversation: its loop waits, however long, and
    /// its bar says a restart is pending meanwhile.
    pub fn restart_claude(&self, agent: &str) -> anyhow::Result<()> {
        self.rpc_do("consumer.restart_claude", json!({ "name": agent, "when_idle": true }))
    }

    /// Marks a question (`- [ ]` in a comment) answered by a comment.
    pub fn answer_question(&self, message: u64, question: &str, answered_in: u64) -> anyhow::Result<()> {
        self.rpc_do("message.answer_question", json!({ "id": message, "qid": question, "answered_in": answered_in }))
    }

    /// Snoozes a ticket until `until` (ISO 8601), or wakes it (`None`).
    pub fn snooze(&self, ticket: u64, until: Option<&str>) -> anyhow::Result<()> {
        match until {
            Some(until) => self.rpc_do("ticket.postpone", json!({ "id": ticket, "until": until })),
            None => self.rpc_do("ticket.unsnooze", json!({ "id": ticket })),
        }
    }

    pub fn set_priority(&self, ticket: u64, priority: &str) -> anyhow::Result<()> {
        self.rpc_do("message.edit", json!({ "id": ticket, "priority": priority }))
    }

    /// Edits a ticket's fields: `{"title": …}`, `{"priority": …}`,
    /// `{"intent": …}`, `{"level": …}`, `{"scope": …}`, `{"body": …}`.
    pub fn edit(&self, message: u64, mut fields: Value) -> anyhow::Result<()> {
        fields["id"] = json!(message);
        self.rpc_do("message.edit", fields)
    }

    /// Who follows a ticket, or muted it, by their own choice: the owners
    /// notified by their role are not among them.
    pub fn ticket_subscribers(&self, ticket: u64) -> anyhow::Result<Vec<Subscriber>> {
        #[derive(Deserialize)]
        struct Answer {
            subscriptions: Vec<Subscriber>,
        }
        let answer: Answer = self.rpc("ticket.subscribers", json!({ "id": ticket }))?;
        Ok(answer.subscriptions)
    }

    /// `consumer` follows the ticket from now on.
    pub fn subscribe(&self, ticket: u64, consumer: &str) -> anyhow::Result<()> {
        self.rpc_do("ticket.subscribe", json!({ "ticket_id": ticket, "consumer_id": consumer }))
    }

    /// `consumer` no longer follows (or mutes) the ticket: its role decides again.
    pub fn unsubscribe(&self, ticket: u64, consumer: &str) -> anyhow::Result<()> {
        self.rpc_do("ticket.unsubscribe", json!({ "ticket_id": ticket, "consumer_id": consumer }))
    }

    pub fn add_tag(&self, ticket: u64, tag: &str) -> anyhow::Result<()> {
        self.rpc_do("message.add_tag", json!({ "id": ticket, "tag": tag }))
    }

    pub fn remove_tag(&self, ticket: u64, tag: &str) -> anyhow::Result<()> {
        self.rpc_do("message.remove_tag", json!({ "id": ticket, "tag": tag }))
    }

    /// The tags a project's tickets can carry.
    pub fn tag_catalog(&self, project: &str) -> anyhow::Result<Vec<String>> {
        let tags: Vec<Tag> = self.rpc("tag.list", json!({ "project": project }))?;
        Ok(tags.into_iter().map(|t| t.name).collect())
    }

    /// A project's milestones not yet released: (id, title).
    pub fn milestones(&self, project: &str) -> anyhow::Result<Vec<(u64, String)>> {
        let answer: Value = self.rpc("project.milestones", json!({ "project": project }))?;
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
        self.rpc_do("ticket.set_milestone", json!({ "id": ticket, "milestone_id": milestone }))
    }

    /// Assigns the ticket to `who`, or releases it (`None`).
    pub fn assign(&self, ticket: u64, who: Option<&str>) -> anyhow::Result<()> {
        match who {
            Some(who) => self.rpc_do("ticket.assign", json!({ "id": ticket, "assignee": who })),
            None => self.rpc_do("ticket.release", json!({ "id": ticket })),
        }
    }

    /// Makes `who` the ticket's reporter (its owner).
    pub fn set_owner(&self, ticket: u64, who: &str) -> anyhow::Result<()> {
        self.rpc_do("ticket.set_owner", json!({ "id": ticket, "owner": who }))
    }

    /// Relates the ticket to `target`; `ignored` removes the relation.
    pub fn relate(&self, ticket: u64, target: u64, kind: &str) -> anyhow::Result<()> {
        self.rpc_do("ticket.relate", json!({ "id": ticket, "target_ticket_id": target, "kind": kind }))
    }

    pub fn move_ticket(&self, ticket: u64, project: &str) -> anyhow::Result<()> {
        self.rpc_do("ticket.move", json!({ "id": ticket, "project": project }))
    }

    /// Deletes a comment (aiball keeps a tombstone).
    pub fn delete_comment(&self, comment: u64) -> anyhow::Result<()> {
        self.rpc_do("message.delete", json!({ "id": comment }))
    }

    /// Makes a comment a pending decision of `kind`.
    pub fn classify(&self, comment: u64, kind: &str) -> anyhow::Result<()> {
        self.rpc_do("message.promote", json!({ "id": comment, "kind": kind }))
    }

    /// Takes a pending decision off a comment.
    pub fn untag(&self, comment: u64) -> anyhow::Result<()> {
        self.rpc_do("message.untag", json!({ "id": comment }))
    }

    /// Marks an agent's comment as a step, or unmarks it.
    pub fn set_step(&self, comment: u64, step: bool) -> anyhow::Result<()> {
        let method = if step { "message.step" } else { "message.unstep" };
        self.rpc_do(method, json!({ "id": comment }))
    }

    /// Votes on a comment: 1, -1, or 0 to take the vote back.
    pub fn vote(&self, comment: u64, value: i64) -> anyhow::Result<()> {
        self.rpc_do("message.vote", json!({ "id": comment, "value": value }))
    }

    /// Makes a comment unread again for those it notified.
    pub fn resurface(&self, comment: u64) -> anyhow::Result<()> {
        self.rpc_do("message.resurface", json!({ "id": comment }))
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
        let s: Suggestions = self.rpc("mention.suggestions", json!({}))?;
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
        let method = if approve { "message.approve" } else { "message.reject" };
        self.rpc_do(method, json!({ "id": message }))
    }

    /// Closes or reopens a ticket.
    pub fn set_closed(&self, project: &str, ticket: u64, closed: bool) -> anyhow::Result<()> {
        self.rpc_do(
            "message.post",
            json!({
                "project": project,
                "kind": if closed { "ticket_closed" } else { "ticket_reopened" },
                "ticket_id": ticket,
                "parent_id": ticket,
            }),
        )
    }

    /// Accept or reject the decision a comment carries.
    pub fn decide(&self, comment: u64, accept: bool) -> anyhow::Result<()> {
        let status = if accept { "accepted" } else { "rejected" };
        self.rpc_do("message.decide", json!({ "id": comment, "status": status }))
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

    fn request_raw(
        &self,
        method: &str,
        path: &str,
        content_type: &str,
        headers: &[(&str, &str)],
        body: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        use std::io::{Read, Write};
        use std::time::Duration;

        // Found again at each call: aiball may have come up since (its
        // socket made by a start tvty asked for), as the bus finds it again
        // at each reconnection.
        let at = location().map_err(|e| anyhow!("{e}"))?;
        at.credentials.check(&at.endpoint)?;
        let mut stream = at.endpoint.connect().with_context(|| format!("aiball at {}", at.endpoint))?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        // HTTP/1.0: the daemon closes the connection after the answer, so
        // reading to the end reads exactly one answer, never chunked.
        let auth: String = at.credentials.header().map(|(k, v)| format!("{k}: {v}\r\n")).unwrap_or_default();
        let extra: String = headers.iter().map(|(k, v)| format!("{k}: {v}\r\n")).collect();
        let head = format!(
            "{method} {path} HTTP/1.0\r\nHost: {host}\r\n{auth}x-aiball-consumer: {user}\r\n\
             content-type: {content_type}\r\n{extra}content-length: {len}\r\n\r\n",
            host = at.endpoint.http_host(),
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
}

/// Where aiball is, found as aiball's own clients find it (docs/IPC.md).
pub fn location() -> Result<Location, LocateError> {
    tvty_ipc::aiball::locate()
}

/// Where aiball is, said to a person (Options > About, errors).
pub fn location_said() -> String {
    match location() {
        Ok(at) => at.endpoint.to_string(),
        Err(error) => error.to_string(),
    }
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

/// A folder's configuration as aiball resolves it (`project.settings`): what
/// a loop started there takes, each value with where it comes from.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProjectSettings {
    /// The `.aiball.yaml` that applies (the nearest up the tree), if any.
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub consumer: FolderConsumer,
    /// Where its loops run: `host` or `tmux`.
    #[serde(default)]
    pub session: Setting<String>,
    /// Its Claude's Remote Control: `false`, `true` or a name.
    #[serde(default)]
    pub remote_control: Setting<Value>,
    /// Every setting of the folder a client may show and change, described:
    /// a key aiball adds shows with no code of tvty's own.
    #[serde(default)]
    pub settings: Vec<FolderSetting>,
}

/// A folder setting as aiball describes it: its key in `.aiball.yaml`, its
/// type (`enum`: one of `options`; `boolean_or_name`: true, false or a
/// name), its default and value, where the value comes from.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct FolderSetting {
    pub key: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub default: Value,
    #[serde(default)]
    pub value: Value,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: String,
}

/// The identity a loop started in the folder takes.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct FolderConsumer {
    #[serde(default)]
    pub project: Setting<String>,
    #[serde(default)]
    pub agent: Setting<String>,
    /// `crew`, or none for the project's lead.
    #[serde(default)]
    pub role: Setting<Option<String>>,
}

/// A value, and where it comes from: `file` (the folder's `.aiball.yaml`),
/// `global` (the machine's config), `mcp`, `env`, or `default`.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Setting<T> {
    pub value: T,
    #[serde(default)]
    pub from: String,
}

impl<T> Setting<T> {
    /// Not aiball's default: something set it.
    pub fn set(&self) -> bool {
        !self.from.is_empty() && self.from != "default"
    }

    /// Where it comes from, said to the user.
    pub fn said(&self) -> &str {
        match self.from.as_str() {
            "file" => "from .aiball.yaml",
            "global" => "from aiball's global config",
            "mcp" => "from .mcp.json",
            "env" => "from aiball's environment",
            _ => "default",
        }
    }
}

/// A project's standing instruction (read at the head of every wake of its
/// agents) and its wake focus (only these tickets wake them).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Standing {
    #[serde(default)]
    pub project: String,
    #[serde(default)]
    pub standing_prompt: Option<String>,
    #[serde(default)]
    pub focus_tickets: Option<String>,
    #[serde(default)]
    pub focus_until: Option<String>,
    #[serde(default)]
    pub focus_active: bool,
    /// The focus as aiball reads it, said.
    #[serde(default)]
    pub focus_line: Option<String>,
}

impl Standing {
    /// Something steers the project's agents now.
    pub fn active(&self) -> bool {
        self.standing_prompt.as_deref().is_some_and(|p| !p.trim().is_empty()) || self.focus_active
    }
}

#[derive(Deserialize)]
struct LoopHolds {
    #[serde(default)]
    results: Vec<LoopHold>,
}

/// What became of one loop: its message typed (`delivered`) or queued
/// (`spooled`), its hold `armed`, `released` or `failed`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LoopHold {
    pub consumer_id: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub hold: Option<String>,
    #[serde(default)]
    pub hold_error: Option<String>,
}

/// A folder to make an aiball project of, and how.
#[derive(Clone, Debug, PartialEq)]
pub struct InitAsk {
    pub cwd: String,
    pub project: String,
    pub agent: String,
    pub crew: bool,
    pub private: bool,
    pub no_claim: bool,
}

/// What `project.init` did to a folder (or would do).
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct InitDone {
    #[serde(default)]
    pub steps: Vec<InitStep>,
    /// The project was on the board already: the folder joins it.
    #[serde(default)]
    pub project_exists: bool,
    /// aiball's skill for Claude Code: "installed" or "missing".
    #[serde(default)]
    pub skill: String,
}

/// One file of it: what became of it, in aiball's words.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct InitStep {
    pub file: String,
    pub action: String,
    #[serde(default)]
    pub message: String,
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

#[cfg(test)]
mod bar_tests {
    use super::AgentBar;
    use serde_json::json;

    fn bar() -> serde_json::Value {
        json!({ "phase": "idle", "presence": "loop", "afk": { "mode": "off", "expires_at": null },
                "prompt": { "visible": true, "has_input": false }, "human_typing": false,
                "marker": { "info": null, "health_prompt": false, "resume_picker": false, "resume_mode_picker": false },
                "alerts": { "link_down": false, "daemon_down": false, "not_logged_in": false, "trust_dialog": false, "api_unreachable": false },
                "proxy_alive": true, "zen": false, "counters": null, "next_wake_at": null, "boot": null, "host": "external" })
    }

    #[test]
    fn a_bar_says_when_claude_waits_for_a_restart() {
        let plain: AgentBar = serde_json::from_value(bar()).unwrap();
        assert!(!plain.alerts.restart_needed);
        let mut updated = bar();
        updated["alerts"]["restart_needed"] = json!(true);
        let updated: AgentBar = serde_json::from_value(updated).unwrap();
        assert!(updated.alerts.restart_needed);
    }

    #[test]
    fn a_bar_says_its_model_its_price_and_a_newer_one() {
        let plain: AgentBar = serde_json::from_value(bar()).unwrap();
        assert!(plain.model.is_none());
        let mut with = bar();
        with["model"] = json!({ "id": "claude-opus-5-5", "name": "Opus 5.5", "cost": { "input": 15, "output": 75 },
                                "newer": { "id": "claude-opus-5-6", "name": "Opus 5.6", "cost": { "input": 12.5, "output": 60 } },
                                "catalog": "models.dev" });
        let model = serde_json::from_value::<AgentBar>(with).unwrap().model.unwrap();
        assert_eq!(model.said(), "claude-opus-5-5 · $15 / $75 per M tokens\nOpus 5.6 is out ($12.5 / $60 per M tokens)\nprices from models.dev");
        // As a loop pushes it, before the daemon adds the price.
        let mut bare = bar();
        bare["model"] = json!({ "id": "claude-opus-5-5", "name": "Opus 5.5" });
        assert_eq!(serde_json::from_value::<AgentBar>(bare).unwrap().model.unwrap().said(), "claude-opus-5-5");
    }
}

#[cfg(test)]
mod critical_tests {
    use super::Critical;

    #[test]
    fn a_critical_ticket_says_how_long_it_went_quiet() {
        let quiet = |q: Option<&str>| Critical { holds: 2, quiet: q.map(str::to_string) };
        assert_eq!(quiet(Some("9 d")).said(), "holds 2 · quiet 9 d");
        assert_eq!(quiet(None).said(), "holds 2");
        assert_eq!(quiet(Some("9 d")).quiet_minutes(), 9 * 24 * 60);
        assert_eq!(quiet(Some("3 h")).quiet_minutes(), 180);
        assert!(quiet(Some("2 w")).quiet_minutes() > quiet(Some("9 d")).quiet_minutes());
        assert_eq!(quiet(None).quiet_minutes(), 0);
        assert_eq!(quiet(Some("")).said(), "holds 2");
    }
}
