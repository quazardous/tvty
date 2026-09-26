//! The board as aiball pushes it: tvty subscribes on aiball's bus to every
//! project's open tickets, every agent's state and loop bar, and the user's
//! pings, and keeps what they say here. Each change comes as data: the board
//! is built again from this, with nothing read back from aiball.
//!
//! Subscribing happens on every greeting of the bus (a first connection, a
//! reconnection, the user now known): with `since`, the daemon replays what
//! was missed, or sends the values whole again.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde_json::{Value, json};

use crate::aiball::{BarRead, Consumer, TicketRow};
use crate::pings::PingInfo;

/// What a subscription is to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Tickets,
    State,
    Bar,
    Pings,
    /// The sessions the daemon's host holds (terminals without an agent too).
    Sessions,
}

impl Kind {
    fn subject(self, user: &str) -> String {
        match self {
            Kind::Tickets => "project.*.tickets".into(),
            Kind::State => "agent.*.state".into(),
            Kind::Bar => "agent.*.bar".into(),
            Kind::Pings => format!("user.{user}.pings"),
            Kind::Sessions => "session.*.state".into(),
        }
    }
}

/// What the state learnt, for the shell to act on.
#[derive(Debug, PartialEq)]
pub enum Update {
    /// The board changed: build it again.
    Board,
    /// A ticket came onto a project's list: it was filed.
    Filed(Filed),
    /// A ping for the user.
    Ping(PingInfo),
    /// The user's unread pings, when subscribing.
    Unread(u32),
}

/// A ticket filed, from the row that brought it.
#[derive(Clone, Debug, PartialEq)]
pub struct Filed {
    pub project: String,
    pub ticket: u64,
    pub title: String,
    pub by: String,
    /// It waits for moderation.
    pub pending: bool,
}

/// Subscribing: what to call, and how to take the answers back.
pub struct Plan {
    calls: Vec<(Kind, Value)>,
}

#[derive(Default)]
pub struct Live {
    /// The agents, as `agent.*.state` has them, by id.
    consumers: BTreeMap<String, Value>,
    bars: HashMap<String, BarRead>,
    /// Open tickets by project.
    tickets: BTreeMap<String, Vec<TicketRow>>,
    /// The daemon's epoch and the last event's seq: what `since` resumes.
    since: Option<(Value, u64)>,
    /// The host's sessions, by name, as `session.*.state` has them.
    sessions: BTreeMap<String, Value>,
    /// Subscriptions by id.
    subscriptions: HashMap<String, Kind>,
    /// Events whose subscription's answer is not in yet.
    early: Vec<Value>,
    /// Whether the tickets were ever received (their first value files
    /// nothing).
    tickets_seen: bool,
    /// Whether the agents were ever received.
    agents_seen: bool,
    /// Who the subscriptions run as: a ticket row is its reader's (unread,
    /// whose turn), so another user's rows are another board.
    user: String,
}

impl Live {
    /// What to subscribe to now; the user's pings when the connection runs
    /// as a human (they are one's own).
    pub fn plan(&mut self, user: &str) -> Plan {
        self.subscriptions.clear();
        self.early.clear();
        if self.user != user {
            // Another user: another board and other pings, subscribed
            // afresh rather than resumed. The rows come again, as this user
            // reads them: nothing in them is news, and the board is not
            // theirs until they are in.
            self.user = user.to_string();
            self.tickets_seen = false;
            self.since = None;
        }
        let mut kinds = vec![Kind::Tickets, Kind::State, Kind::Bar, Kind::Sessions];
        if !user.is_empty() {
            kinds.push(Kind::Pings);
        }
        let since = self.since.as_ref().map(|(epoch, seq)| json!({ "epoch": epoch, "seq": seq }));
        let calls = kinds
            .into_iter()
            .map(|kind| {
                let mut params = json!({ "subject": kind.subject(user) });
                if kind == Kind::Tickets {
                    params["open"] = json!(true);
                }
                if let Some(since) = &since {
                    params["since"] = since.clone();
                }
                (kind, params)
            })
            .collect();
        Plan { calls }
    }

    /// Takes the subscriptions' answers in: their values, or what was missed.
    pub fn subscribed(&mut self, plan: Plan, answers: Vec<anyhow::Result<Value>>) -> Vec<Update> {
        let mut updates = vec![Update::Board];
        for ((kind, _), answer) in plan.calls.into_iter().zip(answers) {
            let answer = match answer {
                Ok(answer) => answer,
                Err(error) => {
                    log::warn!("aiball bus: subscribing to {kind:?}: {error:#}");
                    continue;
                }
            };
            let Some(id) = answer.get("id").and_then(id_of) else { continue };
            self.subscriptions.insert(id, kind);
            if let (Some(epoch), Some(seq)) = (answer.get("epoch"), answer.get("seq").and_then(Value::as_u64)) {
                self.advance(epoch.clone(), seq);
            }
            if answer.get("replayed").and_then(Value::as_bool) == Some(true) {
                for event in answer.get("events").and_then(Value::as_array).into_iter().flatten() {
                    let subject = event.get("subject").and_then(Value::as_str).unwrap_or_default();
                    updates.extend(self.apply(kind, subject, event.get("data").unwrap_or(&Value::Null)));
                }
            } else if let Some(value) = answer.get("value") {
                updates.extend(self.set_value(kind, value));
            }
            // Events that came before this answer.
            for event in std::mem::take(&mut self.early) {
                updates.extend(self.event(&event));
            }
        }
        // Those of no subscription of this connection.
        self.early.clear();
        updates
    }

    /// One `bus.event`.
    pub fn event(&mut self, params: &Value) -> Vec<Update> {
        let Some(id) = params.get("subscription").and_then(id_of) else { return Vec::new() };
        let Some(kind) = self.subscriptions.get(&id).copied() else {
            self.early.push(params.clone());
            return Vec::new();
        };
        if let (Some(seq), Some((epoch, _))) = (params.get("seq").and_then(Value::as_u64), self.since.clone()) {
            self.advance(epoch, seq);
        }
        let subject = params.get("subject").and_then(Value::as_str).unwrap_or_default();
        self.apply(kind, subject, params.get("data").unwrap_or(&Value::Null))
    }

    fn advance(&mut self, epoch: Value, seq: u64) {
        let seq = match &self.since {
            Some((known, last)) if *known == epoch => seq.max(*last),
            _ => seq,
        };
        self.since = Some((epoch, seq));
    }

    /// A subscription's whole value.
    fn set_value(&mut self, kind: Kind, value: &Value) -> Vec<Update> {
        match kind {
            Kind::Tickets => {
                let known: HashSet<u64> = self.tickets.values().flatten().map(|t| t.id).collect();
                let mut tickets = BTreeMap::new();
                for (project, rows) in value.as_object().into_iter().flatten() {
                    match serde_json::from_value::<Vec<TicketRow>>(rows.clone()) {
                        Ok(rows) => {
                            tickets.insert(project.clone(), rows);
                        }
                        Err(error) => log::warn!("aiball bus: {project}'s tickets: {error}"),
                    }
                }
                // Filed while away: rows not known before, once tvty had a list.
                let filed = if self.tickets_seen {
                    tickets.values().flatten().filter(|t| !known.contains(&t.id) && is_new(t)).map(filed_of).collect()
                } else {
                    Vec::new()
                };
                self.tickets = tickets;
                self.tickets_seen = true;
                filed.into_iter().map(Update::Filed).collect()
            }
            Kind::State => {
                self.consumers = value.as_object().map(|m| m.clone().into_iter().collect()).unwrap_or_default();
                self.agents_seen = true;
                Vec::new()
            }
            Kind::Bar => {
                self.bars = value
                    .as_object()
                    .into_iter()
                    .flatten()
                    .filter_map(|(agent, bar)| Some((agent.clone(), serde_json::from_value(bar.clone()).ok()?)))
                    .collect();
                Vec::new()
            }
            Kind::Pings => {
                let unread = value.get("unread").and_then(Value::as_u64).unwrap_or(0) as u32;
                vec![Update::Unread(unread)]
            }
            Kind::Sessions => {
                self.sessions = value.as_object().map(|m| m.clone().into_iter().collect()).unwrap_or_default();
                Vec::new()
            }
        }
    }

    /// One event of a subscription, `subject` naming what it is about.
    fn apply(&mut self, kind: Kind, subject: &str, data: &Value) -> Vec<Update> {
        match kind {
            Kind::Tickets => {
                let project = data
                    .get("project")
                    .and_then(Value::as_str)
                    .or_else(|| data.pointer("/row/project").and_then(Value::as_str))
                    .map(str::to_string)
                    .or_else(|| middle(subject));
                let Some(project) = project else { return Vec::new() };
                match data.get("op").and_then(Value::as_str) {
                    Some("upsert") => {
                        let row = match serde_json::from_value::<TicketRow>(data.get("row").cloned().unwrap_or_default()) {
                            Ok(row) => row,
                            Err(error) => {
                                log::warn!("aiball bus: a ticket row tvty does not read: {error}");
                                return Vec::new();
                            }
                        };
                        let rows = self.tickets.entry(project).or_default();
                        let mut updates = vec![Update::Board];
                        match rows.iter_mut().find(|t| t.id == row.id) {
                            Some(existing) => *existing = row,
                            None => {
                                if is_new(&row) {
                                    updates.push(Update::Filed(filed_of(&row)));
                                }
                                rows.push(row);
                            }
                        }
                        updates
                    }
                    Some("remove") => {
                        let Some(id) = data.get("id").and_then(Value::as_u64) else { return Vec::new() };
                        if let Some(rows) = self.tickets.get_mut(&project) {
                            rows.retain(|t| t.id != id);
                        }
                        vec![Update::Board]
                    }
                    _ => Vec::new(),
                }
            }
            Kind::State => {
                // The whole entry, as `consumer.list` builds it; `null` once
                // the agent is gone.
                let agent = middle(subject).or_else(|| data.get("consumer_id").and_then(Value::as_str).map(str::to_string));
                let Some(agent) = agent else { return Vec::new() };
                if data.is_null() {
                    self.consumers.remove(&agent);
                } else {
                    self.consumers.insert(agent, data.clone());
                }
                vec![Update::Board]
            }
            Kind::Bar => {
                let agent = middle(subject).or_else(|| data.get("consumer_id").and_then(Value::as_str).map(str::to_string));
                let Some(agent) = agent else { return Vec::new() };
                if data.is_null() {
                    self.bars.remove(&agent);
                } else {
                    match serde_json::from_value::<BarRead>(data.clone()) {
                        Ok(bar) => {
                            self.bars.insert(agent, bar);
                        }
                        Err(error) => log::debug!("aiball bus: a bar tvty does not read: {error}"),
                    }
                }
                vec![Update::Board]
            }
            Kind::Pings => self.ping(data).map(Update::Ping).into_iter().collect(),
            // `{ name, session }`: the session as `session.list` gives it,
            // or `null` once it stopped.
            Kind::Sessions => {
                let name = data.get("name").and_then(Value::as_str).map(str::to_string).or_else(|| middle(subject));
                let Some(name) = name else { return Vec::new() };
                match data.get("session") {
                    Some(session) if !session.is_null() => {
                        self.sessions.insert(name, session.clone());
                    }
                    _ => {
                        self.sessions.remove(&name);
                    }
                }
                vec![Update::Board]
            }
        }
    }

    /// A ping, from the message it carries; its ticket's title from the
    /// rows when the message is a comment.
    fn ping(&self, data: &Value) -> Option<PingInfo> {
        let message = data.get("message")?;
        let text = |key: &str| message.get(key).and_then(Value::as_str).unwrap_or_default().to_string();
        let ticket = message
            .get("ticket_id")
            .and_then(Value::as_u64)
            .or_else(|| message.get("id").and_then(Value::as_u64))?;
        let kind = text("kind");
        let decision = message
            .pointer("/decision/kind")
            .or_else(|| message.get("decision"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let title = Some(text("title"))
            .filter(|t| !t.is_empty())
            .or_else(|| self.tickets.values().flatten().find(|t| t.id == ticket).map(|t| t.title.clone()))
            .unwrap_or_default();
        let pending = kind == "ticket_created" && text("status") == "pending";
        Some(PingInfo {
            ticket,
            project: text("project"),
            title,
            from: Some(text("by_agent")).filter(|s| !s.is_empty()).unwrap_or_else(|| "aiball".into()),
            what: if pending { "a new ticket to moderate".into() } else { crate::pings::what_it_is(&kind, decision) },
            urgent: data.get("intent").and_then(Value::as_str) == Some("panic"),
            pending,
        })
    }

    /// The agents, read as tvty reads them.
    pub fn consumers(&self) -> Vec<Consumer> {
        self.consumers.values().filter_map(|c| serde_json::from_value(c.clone()).ok()).collect()
    }

    /// The tickets and the agents were received: the board says what
    /// aiball has, not what has not arrived yet.
    pub fn ready(&self) -> bool {
        self.tickets_seen && self.agents_seen
    }

    /// The host's running sessions without an agent, once they can be
    /// attached to: (name, attach socket).
    pub fn terminals(&self) -> Vec<(String, String)> {
        self.sessions
            .iter()
            .filter(|(_, s)| s.get("agent").is_none_or(Value::is_null))
            .filter(|(_, s)| s.get("running").and_then(Value::as_bool) != Some(false))
            .filter_map(|(name, s)| Some((name.clone(), s.pointer("/attach/socket")?.as_str()?.to_string())))
            .collect()
    }

    /// Every session the host knows without an agent, running or not: the
    /// names a new one must not take.
    pub fn session_names(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }

    /// How many subscriptions are up.
    pub fn subscriptions(&self) -> usize {
        self.subscriptions.len()
    }

    pub fn bars(&self) -> &HashMap<String, BarRead> {
        &self.bars
    }

    pub fn tickets(&self) -> &BTreeMap<String, Vec<TicketRow>> {
        &self.tickets
    }
}

/// A subscription id, a string or a number.
fn id_of(v: &Value) -> Option<String> {
    v.as_str().map(str::to_string).or_else(|| v.as_u64().map(|n| n.to_string()))
}

/// `agent.worker.bar` → `worker`.
fn middle(subject: &str) -> Option<String> {
    let mut parts = subject.splitn(3, '.');
    parts.next()?;
    let name = parts.next()?;
    parts.next()?;
    Some(name.to_string())
}

/// A row that comes onto the list was filed, rather than reopened or moved
/// in: nobody spoke on it yet.
fn is_new(row: &TicketRow) -> bool {
    row.comment_count == 0
}

fn filed_of(row: &TicketRow) -> Filed {
    Filed {
        project: row.project.clone(),
        ticket: row.id,
        title: row.title.clone(),
        by: row.by_agent.clone(),
        pending: row.status == "pending",
    }
}

/// Runs a plan on the bus: one call per subscription. Blocking.
pub fn subscribe(wire: &crate::wire::Wire, plan: &Plan) -> Vec<anyhow::Result<Value>> {
    plan.calls.iter().map(|(_, params)| wire.call("bus.subscribe", params.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::{Live, Update};
    use serde_json::json;

    fn row(id: u64, status: &str) -> serde_json::Value {
        json!({ "id": id, "project": "demo", "title": format!("t{id}"), "status": status, "by_agent": "demo-crew",
                "priority": "normal", "claimant": null, "assignee": null })
    }

    #[test]
    fn values_then_events_keep_the_lists() {
        let mut live = Live::default();
        let plan = live.plan("david");
        let answers = vec![
            Ok(json!({ "id": "t", "epoch": "e1", "seq": 10, "replayed": false, "value": { "demo": [row(1, "approved")] } })),
            Ok(json!({ "id": "s", "epoch": "e1", "seq": 10, "replayed": false, "value": { "demo-crew": { "consumer_id": "demo-crew", "kind": "agent" } } })),
            Ok(json!({ "id": "b", "epoch": "e1", "seq": 10, "replayed": false, "value": {} })),
            Ok(json!({ "id": "h", "epoch": "e1", "seq": 10, "replayed": false, "value": {} })),
            Ok(json!({ "id": "p", "epoch": "e1", "seq": 10, "replayed": false, "value": { "unread": 3 } })),
        ];
        let updates = live.subscribed(plan, answers);
        assert!(updates.contains(&Update::Unread(3)));
        assert_eq!(live.tickets()["demo"].len(), 1);
        assert_eq!(live.consumers().len(), 1);

        // A new row files a ticket; a remove takes one off.
        let filed = live.event(&json!({ "subscription": "t", "subject": "project.demo.tickets", "seq": 11,
                                        "data": { "op": "upsert", "row": row(2, "pending") } }));
        assert!(filed.iter().any(|u| matches!(u, Update::Filed(f) if f.ticket == 2 && f.pending)));
        live.event(&json!({ "subscription": "t", "subject": "project.demo.tickets", "seq": 12,
                            "data": { "op": "remove", "id": 1, "project": "demo" } }));
        assert_eq!(live.tickets()["demo"].iter().map(|t| t.id).collect::<Vec<_>>(), vec![2]);

        // An agent's entry, whole, then gone.
        live.event(&json!({ "subscription": "s", "subject": "agent.demo-crew.state", "seq": 13,
                            "data": { "consumer_id": "demo-crew", "kind": "agent", "present": true, "state": "busy",
                                      "state_since": "2026-09-26T14:00:00Z" } }));
        let crew = &live.consumers()[0];
        assert_eq!((crew.present, crew.state.as_deref()), (Some(true), Some("busy")));
        live.event(&json!({ "subscription": "s", "subject": "agent.demo-crew.state", "seq": 14, "data": null }));
        assert!(live.consumers().is_empty());

        // A ping says who and what, without reading the message.
        let ping = live.event(&json!({ "subscription": "p", "subject": "user.david.pings", "seq": 15,
            "data": { "ticket_id": 2, "message": { "id": 7, "kind": "comment_added", "status": "approved",
                "by_agent": "demo-crew", "project": "demo", "ticket_id": 2, "title": null, "decision": "plan" } } }));
        match &ping[..] {
            [Update::Ping(p)] => assert_eq!((p.ticket, p.title.as_str(), p.what.as_str()), (2, "t2", "proposes a plan")),
            other => panic!("{other:?}"),
        }

        // Resuming names the epoch and the last seq.
        let plan = live.plan("david");
        assert_eq!(plan.calls[0].1["since"], json!({ "epoch": "e1", "seq": 15 }));
    }

    #[test]
    fn another_user_is_another_board() {
        let mut live = Live::default();
        let answers = |seq: u64| vec![
            Ok(json!({ "id": "t", "epoch": "e1", "seq": seq, "replayed": false, "value": { "demo": [row(1, "approved")] } })),
            Ok(json!({ "id": "s", "epoch": "e1", "seq": seq, "replayed": false, "value": {} })),
            Ok(json!({ "id": "b", "epoch": "e1", "seq": seq, "replayed": false, "value": {} })),
        ];
        let plan = live.plan("");
        live.subscribed(plan, answers(10));
        assert!(live.ready());
        // Connected again as the user: not ready until their rows are in,
        // and none of them is filed.
        let plan = live.plan("david");
        assert!(!live.ready());
        assert!(plan.calls.iter().all(|(_, params)| params.get("since").is_none()));
        let mut theirs = answers(11);
        theirs[0] = Ok(json!({ "id": "t", "epoch": "e1", "seq": 11, "replayed": false,
                               "value": { "demo": [row(1, "approved"), row(2, "approved")] } }));
        theirs.push(Ok(json!({ "id": "h", "epoch": "e1", "seq": 11, "replayed": false, "value": {} })));
        theirs.push(Ok(json!({ "id": "p", "epoch": "e1", "seq": 11, "replayed": false, "value": { "unread": 0 } })));
        let updates = live.subscribed(plan, theirs);
        assert!(live.ready());
        assert!(!updates.iter().any(|u| matches!(u, Update::Filed(_))));
    }

    #[test]
    fn the_hosts_terminals_are_its_sessions_without_an_agent() {
        let mut live = Live::default();
        let plan = live.plan("");
        let answers = vec![
            Ok(json!({ "id": "t", "epoch": "e1", "seq": 1, "replayed": false, "value": {} })),
            Ok(json!({ "id": "s", "epoch": "e1", "seq": 1, "replayed": false, "value": {} })),
            Ok(json!({ "id": "b", "epoch": "e1", "seq": 1, "replayed": false, "value": {} })),
            Ok(json!({ "id": "h", "epoch": "e1", "seq": 1, "replayed": false, "value": {
                "plain": { "name": "plain", "agent": null, "running": true, "attach": { "socket": "/s/plain" } },
                "crew": { "name": "crew", "agent": "demo-crew", "running": true, "attach": { "socket": "/s/crew" } },
            } })),
        ];
        live.subscribed(plan, answers);
        assert_eq!(live.terminals(), vec![("plain".to_string(), "/s/plain".to_string())]);
        // Started later: `{ name, session }`; stopped: `session: null`.
        live.event(&json!({ "subscription": "h", "subject": "session.two.state", "seq": 2, "data": {
            "name": "two", "session": { "name": "two", "agent": null, "running": true, "attach": { "socket": "/s/two" } } } }));
        assert_eq!(live.terminals().len(), 2);
        live.event(&json!({ "subscription": "h", "subject": "session.plain.state", "seq": 3,
                            "data": { "name": "plain", "session": null } }));
        assert_eq!(live.terminals(), vec![("two".to_string(), "/s/two".to_string())]);
    }

    #[test]
    fn an_event_before_its_answer_waits_for_it() {
        let mut live = Live::default();
        let plan = live.plan("");
        assert!(live
            .event(&json!({ "subscription": "s", "subject": "agent.demo-crew.state", "seq": 11,
                            "data": { "consumer_id": "demo-crew", "kind": "agent", "state": "busy" } }))
            .is_empty());
        let answers = vec![
            Ok(json!({ "id": "t", "epoch": "e1", "seq": 10, "replayed": false, "value": {} })),
            Ok(json!({ "id": "s", "epoch": "e1", "seq": 10, "replayed": false,
                       "value": { "demo-crew": { "consumer_id": "demo-crew", "kind": "agent", "state": "idle" } } })),
            Ok(json!({ "id": "b", "epoch": "e1", "seq": 10, "replayed": false, "value": {} })),
        ];
        live.subscribed(plan, answers);
        assert_eq!(live.consumers()[0].state.as_deref(), Some("busy"));
    }
}
