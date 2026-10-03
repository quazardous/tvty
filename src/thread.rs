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
            Shape::Folded(
                comment
                    .summary_until()
                    .map(|s| s.lines().map(plain_line).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" "))
                    .unwrap_or_else(|| first_line(comment.body.as_deref())),
            )
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

/// A question asked in a comment — a `- [ ]` line aiball marked with
/// `<!-- q:id -->` — still open.
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    pub id: String,
    pub text: String,
}

/// The open questions of a body.
pub fn questions(body: Option<&str>) -> Vec<Question> {
    body.unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start().strip_prefix(['-', '*', '+'])?.trim_start();
            let rest = rest.strip_prefix("[ ]")?.trim_start();
            let rest = rest.strip_prefix("<!--")?.trim_start().strip_prefix("q:")?;
            let (id, text) = rest.split_once("-->")?;
            let id = id.trim();
            let valid = !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            valid.then(|| Question { id: id.to_string(), text: text.trim().to_string() })
        })
        .collect()
}

/// The first line of a body, cut short, as plain text: what a folded
/// body or comment shows.
pub fn first_line(body: Option<&str>) -> String {
    let line = body
        .unwrap_or_default()
        .lines()
        .map(plain_line)
        .find(|l| !l.is_empty())
        .unwrap_or_default();
    if line.chars().count() > 140 {
        format!("{}…", line.chars().take(140).collect::<String>())
    } else {
        line
    }
}

/// The start of a body as plain words, its lines run together, cut at
/// `max` characters: what a notification quotes. Its emphasis marks and
/// code quotes go too.
pub fn excerpt(body: Option<&str>, max: usize) -> String {
    let words = body
        .unwrap_or_default()
        .lines()
        .map(|l| plain_line(l).replace("**", "").replace("__", "").replace('`', ""))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if words.chars().count() > max {
        format!("{}…", words.chars().take(max).collect::<String>().trim_end())
    } else {
        words
    }
}

/// A markdown line as plain words: a heading's or a quote's mark gone, an
/// image said as "🖼 image" (never its markdown), a link as its text.
pub fn plain_line(line: &str) -> String {
    let line = line.trim().trim_start_matches(['#', '>']).trim();
    let mut out = String::new();
    let mut images = 0;
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        let image = open > 0 && rest[..open].ends_with('!');
        let Some((text, after)) = rest[open + 1..].split_once("](") else { break };
        let Some((_, tail)) = after.split_once(')') else { break };
        if text.contains(['[', ']']) {
            out.push_str(&rest[..=open]);
            rest = &rest[open + 1..];
            continue;
        }
        if image {
            out.push_str(&rest[..open - 1]);
            images += 1;
            // Images side by side are said once, counted.
            if !tail.trim_start().starts_with("![") {
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push_str(&crate::t!("tickets-images", count = images));
                images = 0;
            }
            rest = tail.trim_start_matches(|c: char| c == ' ' && tail.trim_start().starts_with("!["));
        } else {
            out.push_str(&rest[..open]);
            out.push_str(text);
            rest = tail;
        }
    }
    out.push_str(rest);
    out.trim().to_string()
}

fn verb(event: &Comment) -> String {
    let source = event.source_ticket_id.map(|id| format!(" #{id}")).unwrap_or_default();
    match event.kind.as_str() {
        "ticket_closed" => crate::t!("tickets-event-closed"),
        "ticket_reopened" => crate::t!("tickets-event-reopened"),
        "ticket_resolved" => crate::t!("tickets-event-resolved"),
        "ticket_blocked" => crate::t!("tickets-event-blocked"),
        "claim_taken_over" => crate::t!("tickets-event-taken-over"),
        "ticket_sub_added" => crate::t!("tickets-event-sub-added", source = source),
        "ticket_referenced" => crate::t!("tickets-event-referenced", source = source),
        "dependency_closed" => crate::t!("tickets-event-dependency-closed", source = source),
        "dependency_rejected" => crate::t!("tickets-event-dependency-rejected", source = source),
        "related_closed" => crate::t!("tickets-event-related-closed", source = source),
        "ticket_relation" => crate::t!("tickets-event-linked", source = source),
        other => other.replace('_', " "),
    }
}

/// What a decision kind asks of the one who takes it.
pub fn kind_noun(kind: &str) -> String {
    crate::t!(match kind {
        "wontfix" => "tickets-kind-wontfix",
        "escalation" => "tickets-kind-escalation",
        "resolution" => "tickets-kind-resolution",
        _ => "tickets-kind-plan",
    })
}

/// A decision kind's id suffix (`plan`, `resolution`, `wontfix`,
/// `escalation`): the words that differ by kind are whole sentences.
pub fn kind_key(kind: &str) -> &'static str {
    match kind {
        "wontfix" => "wontfix",
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
    let you = crate::t!("tickets-you");
    let who = |name: &str| if name == user { you.clone() } else { name.to_string() };
    if ticket.closed {
        return match (ticket.resolved, ticket.resolved_by.as_deref()) {
            (true, Some(by)) => crate::t!("tickets-closed-resolved-by", who = who(by)),
            (true, None) => crate::t!("tickets-closed-resolved"),
            (false, _) => crate::t!("tickets-closed-unresolved"),
        };
    }
    if ticket.status == "pending" {
        return crate::t!("tickets-yours-moderation");
    }
    if let Some(active) = &reading.active {
        let kind = kind_key(&active.kind);
        return if active.by == user {
            crate::t!(&format!("tickets-your-proposal-{kind}"))
        } else if active.kind == "escalation" {
            crate::t!("tickets-yours-escalates", who = active.by.clone())
        } else {
            crate::t!(&format!("tickets-yours-decide-{kind}"), who = active.by.clone())
        };
    }
    let holder = ticket.holder().map(who);
    if let Some(step) = &ticket.step {
        let agent = holder.clone().unwrap_or_else(|| crate::t!("tickets-an-agent"));
        if stalled {
            return crate::t!("tickets-step-quiet", agent = agent);
        }
        let mut line = crate::t!("tickets-on-step", agent = agent);
        if let Some(at) = step.resume_at.as_deref().and_then(crate::status::parse_time) {
            if at > now {
                line.push_str(&crate::t!("tickets-step-resumes-in", span = span(at - now)));
            }
        }
        if let Some(ticket) = step.resume_on_ticket {
            line.push_str(&crate::t!("tickets-step-waits-on", ticket = ticket));
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
        Some(Turn::You) => match (last, holder.as_deref()) {
            (Some(speaker), _) if speaker != user => crate::t!("tickets-yours-answer", who = speaker),
            // Someone else holds it (assigned, claimed): not nobody.
            (_, Some(holder)) if holder != you.as_str() => crate::t!("tickets-holds-you-spoke", holder = holder),
            _ => crate::t!("tickets-yours-nobody"),
        },
        Some(Turn::Them) => match holder {
            Some(holder) => crate::t!("tickets-their-turn-of", holder = holder),
            None => crate::t!("tickets-their-turn"),
        },
        _ => String::new(),
    }
}

fn span(seconds: u64) -> String {
    match seconds {
        0..60 => crate::t!("tickets-span-under-minute"),
        60..3600 => crate::t!("tickets-span-minutes", n = seconds / 60),
        _ => crate::t!("tickets-span-hours", n = seconds / 3600),
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
                "closed": false, "claimant": "demo-claude", "holder": "demo-claude", "held_as": "claim",
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
        // Held by an agent, though the board says it is yours: it is not nobody's.
        assert_eq!(sentence(&t, &r, Some(&turn(Turn::You)), false, "david", 0), "demo-claude holds it: you spoke last");
    }

    #[test]
    fn open_questions_are_found() {
        let body = "Two things:\n- [ ] <!-- q:a3f2c1 --> Is the API change fine?\n- [x] <!-- q:b7e891 --> Main?\n  * [ ]<!-- q:c1 --> Tabs or spaces?\n- [ ] not marked";
        let found = super::questions(Some(body));
        assert_eq!(
            found,
            [
                super::Question { id: "a3f2c1".into(), text: "Is the API change fine?".into() },
                super::Question { id: "c1".into(), text: "Tabs or spaces?".into() },
            ]
        );
    }

    #[test]
    fn a_step_says_when_it_resumes() {
        let mut t = thread(serde_json::json!([comment(2, "demo-claude", serde_json::json!({"step": true}))]));
        t.ticket.step = Some(crate::aiball::Step { resume_at: None, resume_on_ticket: Some(9) });
        let r = read(&t);
        assert_eq!(sentence(&t, &r, None, false, "david", 0), "demo-claude is on a step · waits on #9");
        assert_eq!(sentence(&t, &r, None, true, "david", 0), "demo-claude's step went quiet");
    }

    #[test]
    fn an_excerpt_runs_the_lines_together_and_cuts() {
        let body = "## Plan\n\nRead **the** `load`.\n> quoted\n![shot](/uploads/x.png)";
        assert_eq!(super::excerpt(Some(body), 200), "Plan Read the load. quoted 🖼 image");
        assert_eq!(super::excerpt(Some("one two three"), 7), "one two…");
        assert_eq!(super::excerpt(None, 10), "");
    }

    #[test]
    fn a_folded_line_says_images_never_their_markdown() {
        use super::{first_line, plain_line};
        assert_eq!(first_line(Some("![pasted](/uploads/751f.png)")), "🖼 image");
        assert_eq!(first_line(Some("\n\n![a](/uploads/a.png) ![b](/uploads/b.png)\n\nthen")), "🖼 2 images");
        assert_eq!(plain_line("> Look: ![shot](/uploads/a.png)"), "Look: 🖼 image");
        assert_eq!(plain_line("see the [log](/uploads/log.txt) here"), "see the log here");
        assert_eq!(plain_line("## A title"), "A title");
        assert_eq!(plain_line("an [odd] bracket"), "an [odd] bracket");
    }
}
