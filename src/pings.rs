//! The user's pings, as aiball pushes them (`GET /api/events?consumer_id=`,
//! the stream aiball's web badge and the loops follow): an agent answered,
//! mentioned the user, asks a decision, a ticket waits for moderation. Each
//! becomes a tvty notification; opening its ticket marks it read in aiball
//! (`/api/tickets/:id/mark-read`), so aiball's web UI and tvty agree.

use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use serde_json::Value;

use crate::aiball::Aiball;

/// What the stream says.
#[derive(Debug, PartialEq)]
pub enum Push {
    /// Connected; this many pings wait unread.
    Hello { unread: u32 },
    /// A new ping, with what tvty read of it.
    Ping(PingInfo),
}

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
}

/// Follows the user's pings on a thread of its own, reconnecting when the
/// stream drops, until nobody listens.
pub fn follow(aiball: Aiball, pushes: UnboundedSender<Push>) {
    std::thread::Builder::new()
        .name("aiball-pings".into())
        .spawn(move || {
            loop {
                if let Err(error) = listen(&aiball, &pushes) {
                    log::debug!("aiball pings: {error:#}");
                }
                if pushes.is_closed() {
                    return;
                }
                std::thread::sleep(Duration::from_secs(5));
            }
        })
        .expect("failed to start the pings thread");
}

#[cfg(unix)]
fn listen(aiball: &Aiball, pushes: &UnboundedSender<Push>) -> anyhow::Result<()> {
    let mut stream = std::os::unix::net::UnixStream::connect(crate::aiball::socket_path())?;
    // HTTP/1.0: the answer is not chunked, the stream is the body itself.
    write!(
        stream,
        "GET /api/events?consumer_id={user}&source=ui HTTP/1.0\r\nHost: aiball\r\nx-aiball-consumer: {user}\r\n\r\n",
        user = aiball.user
    )?;
    let mut event = String::new();
    for line in BufReader::new(stream).lines() {
        let line = line?;
        if let Some(name) = line.strip_prefix("event:") {
            event = name.trim().to_string();
        } else if let Some(data) = line.strip_prefix("data:") {
            if let Some(push) = parse(&event, data.trim(), aiball) {
                if pushes.unbounded_send(push).is_err() {
                    return Ok(());
                }
            }
        } else if line.is_empty() {
            event.clear();
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn listen(_: &Aiball, _: &UnboundedSender<Push>) -> anyhow::Result<()> {
    anyhow::bail!("aiball's socket is Unix only for now")
}

/// One event of the stream, read into what tvty shows.
fn parse(event: &str, data: &str, aiball: &Aiball) -> Option<Push> {
    let data: Value = serde_json::from_str(data).ok()?;
    match event {
        "hello" => Some(Push::Hello { unread: data.get("unread")?.as_u64()? as u32 }),
        "ping" => {
            let ticket = data.get("ticket_id")?.as_u64()?;
            let urgent = data.get("intent").and_then(Value::as_str) == Some("panic");
            // A ticket is a message too: its title, project and reporter.
            let head = aiball.message(ticket).ok()?;
            let comment = data.get("comment_id").and_then(Value::as_u64);
            let (from, what) = match comment.and_then(|c| aiball.message(c).ok()) {
                Some(message) => (message.by_agent.clone(), what_it_is(&message.kind, message.decision_kind())),
                None => (head.by_agent.clone(), what_it_is(&head.kind, head.decision_kind())),
            };
            let title = head.title.clone().unwrap_or_default();
            Some(Push::Ping(PingInfo { ticket, project: head.project, title, from, what, urgent }))
        }
        _ => None,
    }
}

/// What pinged, in a few words.
fn what_it_is(kind: &str, decision: Option<String>) -> String {
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
