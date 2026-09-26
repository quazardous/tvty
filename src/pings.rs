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
    /// The ticket's intent is `panic`.
    pub urgent: bool,
    /// The ticket waits for moderation.
    pub pending: bool,
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
    use super::what_it_is;

    #[test]
    fn a_ping_says_what_it_is() {
        assert_eq!(what_it_is("comment_added", Some("plan".into())), "proposes a plan");
        assert_eq!(what_it_is("comment_added", None), "a new comment");
        assert_eq!(what_it_is("ticket_created", None), "a new ticket");
    }
}
