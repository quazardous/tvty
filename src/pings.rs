//! The user's pings, as aiball pushes them on its bus (`user.<me>.pings`,
//! see [`crate::live`]): an agent answered, mentioned the user, asks a
//! decision, a ticket waits for moderation. Each becomes a tvty notification;
//! opening its ticket marks it read in aiball, so aiball's web UI and tvty
//! agree.

#[derive(Debug, PartialEq)]
pub struct PingInfo {
    pub ticket: u64,
    pub project: String,
    pub title: String,
    /// Who wrote what pinged, and what it is ("a plan", "a comment"…).
    pub from: String,
    pub what: String,
    /// The message that pinged (the comment, the new ticket).
    pub message: Option<u64>,
    /// The start of what was written (the comment, the new ticket's body),
    /// when the ping carries it.
    pub excerpt: String,
    /// The ticket's intent is `panic`.
    pub urgent: bool,
    /// The ticket waits for moderation.
    pub pending: bool,
    /// It proposes a decision (a plan, a resolution…): the user's to take.
    pub proposal: bool,
    /// When it came, as aiball writes times (ISO 8601, UTC): they compare
    /// as text.
    pub at: String,
}

/// The pings that came after `seen` (the newest one tvty knew of), oldest
/// first — none when tvty never knew one: a first start does not replay
/// the whole inbox.
pub fn missed(mut pings: Vec<PingInfo>, seen: Option<&str>) -> Vec<PingInfo> {
    let Some(seen) = seen else { return Vec::new() };
    pings.retain(|p| p.at.as_str() > seen);
    pings.sort_by(|a, b| a.at.cmp(&b.at));
    pings
}

/// One notification for the pings missed while tvty was closed: the
/// newest said, the others counted.
pub fn missed_text(missed: &[PingInfo]) -> Option<String> {
    let newest = missed.last()?;
    let what = if newest.title.is_empty() { newest.what.clone() } else { format!("{} — {}", newest.title, newest.what) };
    Some(match missed.len() {
        1 => format!("{what} · while tvty was closed"),
        2 => format!("{what} · and 1 more while tvty was closed"),
        n => format!("{what} · and {} more while tvty was closed", n - 1),
    })
}

/// What pinged, in a few words.
pub fn what_it_is(kind: &str, decision: Option<String>) -> String {
    match (kind, decision.as_deref()) {
        (_, Some("plan")) => "proposes a plan".into(),
        (_, Some("resolution")) => "proposes to close".into(),
        (_, Some("wontfix")) => "proposes to close without a fix".into(),
        (_, Some("escalation")) => "escalates".into(),
        ("ticket_created", _) => "a new ticket".into(),
        _ => "a new comment".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{PingInfo, missed, missed_text, what_it_is};

    fn ping(ticket: u64, at: &str) -> PingInfo {
        PingInfo {
            ticket,
            project: "demo".into(),
            title: format!("t{ticket}"),
            from: "demo-crew".into(),
            what: "proposes to close".into(),
            message: None,
            excerpt: String::new(),
            urgent: false,
            pending: false,
            proposal: true,
            at: at.into(),
        }
    }

    #[test]
    fn what_came_while_closed_is_one_notification() {
        let pings = vec![ping(3, "2026-09-27T18:00:00.000Z"), ping(1, "2026-09-27T17:00:00.000Z"), ping(2, "2026-09-27T17:30:00.000Z")];
        // A first start knows nothing: nothing replayed.
        assert!(missed(pings.iter().map(|p| ping(p.ticket, &p.at)).collect(), None).is_empty());
        let found = missed(pings, Some("2026-09-27T17:10:00.000Z"));
        assert_eq!(found.iter().map(|p| p.ticket).collect::<Vec<_>>(), [2, 3]);
        assert_eq!(missed_text(&found).unwrap(), "t3 — proposes to close · and 1 more while tvty was closed");
        assert_eq!(missed_text(&found[..1]).unwrap(), "t2 — proposes to close · while tvty was closed");
        assert!(missed_text(&[]).is_none());
    }

    #[test]
    fn a_ping_says_what_it_is() {
        assert_eq!(what_it_is("comment_added", Some("plan".into())), "proposes a plan");
        assert_eq!(what_it_is("comment_added", None), "a new comment");
        assert_eq!(what_it_is("ticket_created", None), "a new ticket");
    }
}
