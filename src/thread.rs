//! What a ticket's thread tells at a glance, before it is drawn: where it
//! stands in one sentence, the one decision that can still be taken, and
//! which comments to show in full. The reading follows aiball's `brief`
//! mode — an agent and the user read a thread from the same place.

use crate::aiball::{Comment, Decision, Thread};
use crate::rowstate::{Glyph, RowState, Turn};

/// The decision a thread waits on: the latest one, while still pending.
#[derive(Clone, Debug, PartialEq)]
pub struct Active {
    /// The message carrying it: a comment, or the ticket itself.
    pub message: u64,
    pub kind: String,
    pub by: String,
}

/// A decision chip, as a comment shows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionState {
    Pending,
    Accepted,
    Rejected,
    /// Still pending, but a newer decision replaced it: it can no longer
    /// be taken.
    Superseded,
}

impl DecisionState {
    fn of(decision: &Decision, latest: bool) -> Self {
        match decision.status.as_str() {
            "accepted" => Self::Accepted,
            "rejected" => Self::Rejected,
            _ if latest => Self::Pending,
            _ => Self::Superseded,
        }
    }
}

/// How one entry of the thread is drawn.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// Its whole body.
    Full,
    /// One line — its own `summary_until`, else its first line — before
    /// the thread's latest snapshot; a click unfolds it.
    Folded(String),
    /// A lifecycle or relation event: one grey line, `count` of them when
    /// the same author did the same thing in a row.
    Event { verb: String, count: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: u64,
    pub shape: Shape,
    pub decision: Option<(String, DecisionState)>,
    /// A step; `true` for the latest one, the only one that still runs.
    pub step: Option<bool>,
    /// Waits for moderation.
    pub pending: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub active: Option<Active>,
    /// The latest `summary_until`, and who left it: where the ticket stands.
    pub summary: Option<(String, String)>,
    /// The ticket's own decision, when it was filed with one.
    pub ticket_decision: Option<(String, DecisionState)>,
    pub entries: Vec<Entry>,
}

/// Events of the same kind and author closer than this are one line.
const MERGE_WITHIN: u64 = 60;

pub fn read(thread: &Thread) -> Reading {
    let comments: Vec<&Comment> = thread
        .comments
        .iter()
        .filter(|c| c.status != "rejected")
        .filter(|c| c.kind != "comment_added" || c.body.is_some() || c.summary_until().is_some())
        .collect();

    // The latest decision wins; the ticket's own comes first.
    let mut latest = thread.ticket.decision().map(|d| (thread.ticket.id, thread.ticket.by_agent.clone(), d));
    for comment in &comments {
        if let Some(decision) = comment.decision() {
            latest = Some((comment.id, comment.by_agent.clone(), decision));
        }
    }
    let latest_id = latest.as_ref().map(|(id, _, _)| *id);
    let active = latest
        .filter(|(_, _, d)| d.status == "pending")
        .map(|(message, by, d)| Active { message, kind: d.kind, by });
    let ticket_decision = thread
        .ticket
        .decision()
        .map(|d| (d.kind.clone(), DecisionState::of(&d, latest_id == Some(thread.ticket.id))));

    let latest_step = comments.iter().rev().find(|c| c.is_step()).map(|c| c.id);
    let pivot = comments.iter().rposition(|c| c.kind == "comment_added" && c.summary_until().is_some());
    let summary = pivot.and_then(|at| {
        let c = comments[at];
        c.summary_until().map(|s| (s, c.by_agent.clone()))
    });

    let mut entries: Vec<Entry> = Vec::new();
    let mut last_event: Option<(&str, &str, u64)> = None;
    for (at, comment) in comments.iter().enumerate() {
        if comment.kind != "comment_added" {
            let when = crate::status::parse_time(&comment.created_at).unwrap_or(0);
            let same = last_event.is_some_and(|(kind, by, then)| {
                kind == comment.kind && by == comment.by_agent && when.abs_diff(then) <= MERGE_WITHIN
            });
            if same {
                if let Some(Entry { shape: Shape::Event { count, .. }, .. }) = entries.last_mut() {
                    *count += 1;
                }
            } else {
                entries.push(Entry {
                    id: comment.id,
                    shape: Shape::Event { verb: verb(comment), count: 1 },
                    decision: None,
                    step: None,
                    pending: comment.status == "pending",
                });
            }
            last_event = Some((&comment.kind, &comment.by_agent, when));
            continue;
        }
        last_event = None;
        let folded = pivot.is_some_and(|pivot| at < pivot);
        let shape = if folded {
            Shape::Folded(comment.summary_until().unwrap_or_else(|| first_line(comment.body.as_deref())))
        } else {
            Shape::Full
        };
        entries.push(Entry {
            id: comment.id,
            shape,
            decision: comment
                .decision()
                .map(|d| (d.kind.clone(), DecisionState::of(&d, latest_id == Some(comment.id)))),
            step: comment.is_step().then(|| latest_step == Some(comment.id)),
            pending: comment.status == "pending",
        });
    }

    Reading { active, summary, ticket_decision, entries }
}

/// The first line of a body, cut short.
pub fn first_line(body: Option<&str>) -> String {
    let line = body
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().trim_start_matches('#').trim())
        .find(|l| !l.is_empty())
        .unwrap_or_default();
    if line.chars().count() > 140 {
        format!("{}…", line.chars().take(140).collect::<String>())
    } else {
        line.to_string()
    }
}

fn verb(event: &Comment) -> String {
    let source = event.source_ticket_id.map(|id| format!(" #{id}")).unwrap_or_default();
    match event.kind.as_str() {
        "ticket_closed" => "closed the ticket".into(),
        "ticket_reopened" => "reopened the ticket".into(),
        "ticket_resolved" => "marked it resolved".into(),
        "ticket_blocked" => "flagged it to be decided".into(),
        "claim_taken_over" => "took over the claim".into(),
        "ticket_sub_added" => format!("added a sub-ticket{source}"),
        "ticket_referenced" => format!("referenced it from{source}"),
        "dependency_closed" => format!("a dependency closed{source}"),
        "dependency_rejected" => format!("a dependency was rejected{source}"),
        "related_closed" => format!("a related ticket closed{source}"),
        "ticket_relation" => format!("linked{source}"),
        other => other.replace('_', " "),
    }
}

/// What a decision kind asks of the one who takes it.
pub fn kind_noun(kind: &str) -> &str {
    match kind {
        "wontfix" => "closing without a fix",
        "escalation" => "escalation",
        "resolution" => "resolution",
        _ => "plan",
    }
}

/// The glyph of a thread when the list has no row for it (a closed ticket).
pub fn glyph(thread: &Thread, reading: &Reading) -> Option<Glyph> {
    if thread.ticket.closed {
        return Some(if thread.ticket.resolved { Glyph::ClosedResolved } else { Glyph::Closed });
    }
    if let Some(active) = &reading.active {
        return Some(match active.kind.as_str() {
            "resolution" => Glyph::Resolution,
            "wontfix" => Glyph::Wontfix,
            "escalation" => Glyph::Escalation,
            _ => Glyph::Plan,
        });
    }
    thread.ticket.step.as_ref().map(|_| Glyph::Step)
}

/// Where the ticket stands and who moves next, in one sentence. `row` is
/// the list's reading of the ticket, when it has one; `now` in seconds.
pub fn sentence(thread: &Thread, reading: &Reading, row: Option<&RowState>, stalled: bool, user: &str, now: u64) -> String {
    let ticket = &thread.ticket;
    let who = |name: &str| if name == user { "you".to_string() } else { name.to_string() };
    if ticket.closed {
        return match (ticket.resolved, ticket.resolved_by.as_deref()) {
            (true, Some(by)) => format!("Closed, resolved by {}", who(by)),
            (true, None) => "Closed, resolved".into(),
            (false, _) => "Closed without a resolution".into(),
        };
    }
    if ticket.status == "pending" {
        return "Yours: this ticket waits for moderation".into();
    }
    if let Some(active) = &reading.active {
        let noun = kind_noun(&active.kind);
        return if active.by == user {
            format!("Your {noun} waits for a decision")
        } else if active.kind == "escalation" {
            format!("Yours: {} escalates — act, then accept", active.by)
        } else {
            format!("Yours: accept or reject {}'s {noun}", active.by)
        };
    }
    let holder = ticket.holder().map(who);
    if let Some(step) = &ticket.step {
        let agent = holder.clone().unwrap_or_else(|| "an agent".into());
        if stalled {
            return format!("{agent}'s step went quiet");
        }
        let mut line = format!("{agent} is on a step");
        if let Some(at) = step.resume_at.as_deref().and_then(crate::status::parse_time) {
            if at > now {
                line.push_str(&format!(" · resumes in {}", span(at - now)));
            }
        }
        if let Some(ticket) = step.resume_on_ticket {
            line.push_str(&format!(" · waits on #{ticket}"));
        }
        return line;
    }
    let last = thread
        .comments
        .iter()
        .rev()
        .find(|c| c.kind == "comment_added" && c.status != "rejected")
        .map(|c| c.by_agent.as_str());
    match row.map(|r| r.turn) {
        Some(Turn::You) => match last {
            Some(speaker) if speaker != user => format!("Yours: answer {speaker}"),
            _ => "Yours: nobody else is on it".into(),
        },
        Some(Turn::Them) => match holder {
            Some(holder) => format!("{holder}'s turn: you spoke last"),
            None => "Their turn: you spoke last".into(),
        },
        _ => String::new(),
    }
}

fn span(seconds: u64) -> String {
    match seconds {
        0..60 => "under a minute".into(),
        60..3600 => format!("{} min", seconds / 60),
        _ => format!("{} h", seconds / 3600),
    }
}

#[cfg(test)]
mod tests {
    use super::{DecisionState, Shape, read, sentence};
    use crate::aiball::Thread;
    use crate::rowstate::{Band, RowState, Stripe, Turn};

    fn thread(comments: serde_json::Value) -> Thread {
        serde_json::from_value(serde_json::json!({
            "ticket": {
                "id": 1, "title": "t", "body": "b", "by_agent": "david",
                "created_at": "2026-09-24T10:00:00.000Z", "status": "approved",
                "closed": false, "claimant": "demo-claude", "is_claim": true,
                "assignee": null, "priority": "normal", "meta": null
            },
            "comments": comments,
        }))
        .unwrap()
    }

    fn comment(id: u64, by: &str, meta: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "id": id, "kind": "comment_added", "by_agent": by, "body": format!("body {id}"),
            "meta": meta.to_string(), "created_at": "2026-09-24T10:00:00.000Z", "status": "approved"
        })
    }

    fn turn(turn: Turn) -> RowState {
        RowState { band: Band::Open, turn, glyph: None, stripe: Stripe::None }
    }

    #[test]
    fn a_newer_decision_supersedes_the_older() {
        let t = thread(serde_json::json!([
            comment(2, "demo-claude", serde_json::json!({"decision": {"kind": "plan", "status": "pending"}})),
            comment(3, "demo-claude", serde_json::json!({"decision": {"kind": "plan", "status": "pending"}})),
        ]));
        let r = read(&t);
        assert_eq!(r.entries[0].decision, Some(("plan".into(), DecisionState::Superseded)));
        assert_eq!(r.entries[1].decision, Some(("plan".into(), DecisionState::Pending)));
        assert_eq!(r.active.as_ref().map(|a| a.message), Some(3));
        assert_eq!(sentence(&t, &r, None, false, "david", 0), "Yours: accept or reject demo-claude's plan");
        // Your own proposal is not yours to take.
        assert_eq!(sentence(&t, &r, None, false, "demo-claude", 0), "Your plan waits for a decision");
    }

    #[test]
    fn a_decided_latest_leaves_nothing_to_take() {
        let t = thread(serde_json::json!([
            comment(2, "demo-claude", serde_json::json!({"decision": {"kind": "plan", "status": "pending"}})),
            comment(3, "demo-claude", serde_json::json!({"decision": {"kind": "resolution", "status": "accepted"}})),
        ]));
        let r = read(&t);
        assert_eq!(r.active, None);
        assert_eq!(r.entries[0].decision.as_ref().map(|d| d.1), Some(DecisionState::Superseded));
    }

    #[test]
    fn comments_before_the_latest_snapshot_fold_to_it() {
        let t = thread(serde_json::json!([
            comment(2, "demo-claude", serde_json::json!({"summary_until": "first state"})),
            comment(3, "david", serde_json::json!(null)),
            comment(4, "demo-claude", serde_json::json!({"summary_until": "where it stands"})),
            comment(5, "david", serde_json::json!(null)),
        ]));
        let r = read(&t);
        let shapes: Vec<&Shape> = r.entries.iter().map(|e| &e.shape).collect();
        assert_eq!(
            shapes,
            [&Shape::Folded("first state".into()), &Shape::Folded("body 3".into()), &Shape::Full, &Shape::Full]
        );
        assert_eq!(r.summary, Some(("where it stands".into(), "demo-claude".into())));
    }

    #[test]
    fn only_the_latest_step_runs() {
        let t = thread(serde_json::json!([
            comment(2, "demo-claude", serde_json::json!({"step": true})),
            comment(3, "demo-claude", serde_json::json!({"step": true})),
        ]));
        let r = read(&t);
        assert_eq!((r.entries[0].step, r.entries[1].step), (Some(false), Some(true)));
    }

    #[test]
    fn events_of_one_author_in_a_row_are_one_line() {
        let event = |id: u64| {
            serde_json::json!({
                "id": id, "kind": "ticket_relation", "by_agent": "david", "body": "",
                "meta": null, "created_at": "2026-09-24T10:00:00.000Z", "status": "approved",
                "source_ticket_id": 7
            })
        };
        let r = read(&thread(serde_json::json!([event(2), event(3)])));
        assert_eq!(r.entries.len(), 1);
        assert_eq!(r.entries[0].shape, Shape::Event { verb: "linked #7".into(), count: 2 });
    }

    #[test]
    fn whose_turn_in_words() {
        let t = thread(serde_json::json!([comment(2, "demo-claude", serde_json::json!(null))]));
        let r = read(&t);
        assert_eq!(sentence(&t, &r, Some(&turn(Turn::You)), false, "david", 0), "Yours: answer demo-claude");
        let t = thread(serde_json::json!([comment(2, "david", serde_json::json!(null))]));
        let r = read(&t);
        assert_eq!(sentence(&t, &r, Some(&turn(Turn::Them)), false, "david", 0), "demo-claude's turn: you spoke last");
    }

    #[test]
    fn a_step_says_when_it_resumes() {
        let mut t = thread(serde_json::json!([comment(2, "demo-claude", serde_json::json!({"step": true}))]));
        t.ticket.step = Some(crate::aiball::Step { resume_at: None, resume_on_ticket: Some(9) });
        let r = read(&t);
        assert_eq!(sentence(&t, &r, None, false, "david", 0), "demo-claude is on a step · waits on #9");
        assert_eq!(sentence(&t, &r, None, true, "david", 0), "demo-claude's step went quiet");
    }
}
