//! Acting on several tickets at once, from the full list: which actions a
//! selection allows and on how many of its rows (aiball's web UI's rules),
//! and doing them, one ticket after the other, off the UI thread.

use crate::aiball::{Aiball, TicketRow};

/// A gesture on the selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Approve,
    Reject,
    Close,
    Reopen,
    MarkRead,
    MarkUnread,
    Snooze,
    Unsnooze,
    Step,
    Link,
}

impl Action {
    /// In the order the panel lists them.
    pub const ALL: [Action; 10] = [
        Action::Approve,
        Action::Reject,
        Action::Close,
        Action::Reopen,
        Action::MarkRead,
        Action::MarkUnread,
        Action::Snooze,
        Action::Unsnooze,
        Action::Step,
        Action::Link,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Action::Approve => "Approve",
            Action::Reject => "Reject",
            Action::Close => "Close",
            Action::Reopen => "Reopen",
            Action::MarkRead => "Mark read",
            Action::MarkUnread => "Mark unread",
            Action::Snooze => "Snooze 3 days",
            Action::Unsnooze => "Unsnooze",
            Action::Step => "Mark as step",
            Action::Link => "Link",
        }
    }

    /// What it says it did, in the summary.
    fn done(self) -> &'static str {
        match self {
            Action::Approve => "approved",
            Action::Reject => "rejected",
            Action::Close => "closed",
            Action::Reopen => "reopened",
            Action::MarkRead => "marked read",
            Action::MarkUnread => "marked unread",
            Action::Snooze => "snoozed",
            Action::Unsnooze => "unsnoozed",
            Action::Step => "marked as step",
            Action::Link => "linked",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            Action::Approve | Action::Reject => "tickets waiting for moderation",
            Action::Close => "open tickets",
            Action::Reopen => "closed tickets",
            Action::MarkRead => "tickets with something new",
            Action::MarkUnread => "tickets read",
            Action::Snooze => "open tickets not snoozed; back in 3 days",
            Action::Unsnooze => "snoozed tickets",
            Action::Step => "open tickets whose last word is not a step already",
            Action::Link => "the newest gets a relates_to link to each of the others",
        }
    }

    /// Asked before it goes.
    pub fn confirmed(self) -> bool {
        matches!(self, Action::Close | Action::Reject)
    }

    /// Whether it applies to `row`, as aiball's web UI decides.
    pub fn applies(self, row: &TicketRow) -> bool {
        let pending = row.status == "pending";
        let rejected = row.status == "rejected";
        let open = !row.closed && !pending && !rejected;
        match self {
            Action::Approve | Action::Reject => pending,
            Action::Close => open,
            Action::Reopen => row.closed,
            Action::MarkRead => row.unread,
            Action::MarkUnread => !row.unread,
            Action::Snooze => !rejected && !row.closed && row.postponed_until.is_none(),
            Action::Unsnooze => row.postponed_until.is_some(),
            Action::Step => open && !row.latest_is_step,
            Action::Link => !rejected,
        }
    }

    /// On how many of `rows` it applies; a link needs two.
    pub fn count(self, rows: &[&TicketRow]) -> usize {
        let n = rows.iter().filter(|r| self.applies(r)).count();
        if self == Action::Link && n < 2 { 0 } else { n }
    }
}

/// A link's star: the newest ticket, and the others it links to.
pub fn link_star(rows: &[&TicketRow]) -> Option<(u64, Vec<u64>)> {
    let mut eligible: Vec<&&TicketRow> = rows.iter().filter(|r| Action::Link.applies(r)).collect();
    if eligible.len() < 2 {
        return None;
    }
    eligible.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let newest = eligible[0].id;
    Some((newest, eligible[1..].iter().map(|r| r.id).collect()))
}

/// Does `action` on the rows it applies to; blocking. Answers one line
/// for a notification, and whether anything failed.
pub fn run(aiball: &Aiball, action: Action, rows: &[TicketRow]) -> (String, bool) {
    let refs: Vec<&TicketRow> = rows.iter().collect();
    let mut failures: Vec<String> = Vec::new();
    let mut done = 0usize;
    if action == Action::Link {
        if let Some((newest, others)) = link_star(&refs) {
            for other in others {
                match aiball.relate(newest, other, "relates_to") {
                    Ok(()) => done += 1,
                    Err(error) => failures.push(format!("#{other}: {error:#}")),
                }
            }
        }
    } else {
        let until = (chrono::Utc::now() + chrono::Duration::days(3)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        for row in refs.iter().filter(|r| action.applies(r)) {
            let result = match action {
                Action::Approve => aiball.moderate(row.id, true),
                Action::Reject => aiball.moderate(row.id, false),
                Action::Close => aiball.set_closed(&row.project, row.id, true),
                Action::Reopen => aiball.set_closed(&row.project, row.id, false),
                Action::MarkRead => aiball.mark_read(row.id),
                Action::MarkUnread => aiball.mark_unread(row.id),
                Action::Snooze => aiball.snooze(row.id, Some(&until)),
                Action::Unsnooze => aiball.snooze(row.id, None),
                Action::Step => aiball.step_ticket(row.id),
                Action::Link => unreachable!("linked above"),
            };
            match result {
                Ok(()) => done += 1,
                Err(error) => failures.push(format!("#{}: {error:#}", row.id)),
            }
        }
    }
    let mut line = format!("{done} {}", action.done());
    if let Some(first) = failures.first() {
        line.push_str(&format!(", {} refused ({first}{})", failures.len(), if failures.len() > 1 { "…" } else { "" }));
    }
    (line, !failures.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{Action, link_star};
    use crate::aiball::TicketRow;

    fn row(id: u64, status: &str, closed: bool, unread: bool, created: &str) -> TicketRow {
        serde_json::from_value(serde_json::json!({
            "id": id, "project": "demo", "title": "t", "status": status, "by_agent": "a", "closed": closed,
            "unread": unread, "created_at": created, "priority": "normal", "claimant": null, "assignee": null
        }))
        .unwrap()
    }

    #[test]
    fn each_action_counts_the_rows_it_applies_to() {
        let (a, b, c, d) = (
            row(1, "pending", false, true, "2026-09-01"),
            row(2, "approved", false, false, "2026-09-02"),
            row(3, "approved", true, false, "2026-09-03"),
            row(4, "approved", false, true, "2026-09-04"),
        );
        let rows = [&a, &b, &c, &d];
        assert_eq!(Action::Approve.count(&rows), 1);
        assert_eq!(Action::Close.count(&rows), 2);
        assert_eq!(Action::Reopen.count(&rows), 1);
        assert_eq!(Action::MarkRead.count(&rows), 2);
        assert_eq!(Action::MarkUnread.count(&rows), 2);
        assert_eq!(Action::Snooze.count(&rows), 3);
        assert_eq!(Action::Unsnooze.count(&rows), 0);
        assert_eq!(Action::Step.count(&rows), 2);
        assert_eq!(Action::Link.count(&rows[..1]), 0, "a link needs two");
        assert!(Action::Close.confirmed() && !Action::MarkRead.confirmed());
    }

    #[test]
    fn a_link_goes_from_the_newest_to_each_other() {
        let (a, b, c) = (row(1, "approved", false, false, "2026-09-01"), row(2, "rejected", false, false, "2026-09-05"), row(3, "approved", true, false, "2026-09-03"));
        assert_eq!(link_star(&[&a, &b, &c]), Some((3, vec![1])));
        assert_eq!(link_star(&[&a]), None);
    }
}
