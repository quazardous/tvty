//! aiball's live feed (`/ws`): every change on the board — a message
//! posted, a decision taken, an agent's state — so tvty reads again what
//! moved, when it moves, instead of everything every few seconds. Read-only;
//! over aiball's local socket, where the same user is trusted.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use serde_json::Value;

/// What an event says moved.
#[derive(Debug)]
pub enum Change {
    /// A project's tickets.
    Project(String),
    /// The agents.
    Consumers,
    /// Something wider, or unknown: everything.
    All,
    /// A ticket was filed (its project's tickets moved too).
    Filed(Filed),
}

/// A new ticket, as the event carries it.
#[derive(Debug, Clone, PartialEq)]
pub struct Filed {
    pub project: String,
    pub ticket: u64,
    pub title: String,
    pub by: String,
    /// It waits for moderation.
    pub pending: bool,
}

impl Filed {
    /// The ticket a `message_created` event files, if it files one.
    pub fn of(event: &str) -> Option<Self> {
        let event: Value = serde_json::from_str(event).ok()?;
        if event.get("type")?.as_str()? != "message_created" {
            return None;
        }
        let data = event.get("data")?;
        if data.get("kind")?.as_str()? != "ticket_created" {
            return None;
        }
        let text = |key: &str| data.get(key).and_then(Value::as_str).unwrap_or_default().to_string();
        Some(Self {
            project: text("project"),
            ticket: data.get("id")?.as_u64()?,
            title: text("title"),
            by: text("by_agent"),
            pending: data.get("status").and_then(Value::as_str) == Some("pending"),
        })
    }
}

impl Change {
    fn of(event: &str) -> Option<Self> {
        let event: Value = serde_json::from_str(event).ok()?;
        let project = event
            .pointer("/data/project")
            .and_then(Value::as_str)
            .map(|p| Change::Project(p.to_string()));
        match event.get("type")?.as_str()? {
            "hello" => None,
            // An agent's state, or its loop bar.
            "consumer_changed" | "agent_bar" => Some(Change::Consumers),
            "message_created" | "message_decided" | "message_edited" | "message_noted" => {
                Some(project.unwrap_or(Change::All))
            }
            // Tags and rules do not change what tvty shows.
            "message_tagged" | "tag_changed" | "rule_changed" | "automation_rule_changed" => None,
            _ => Some(Change::All),
        }
    }
}

/// Whether the feed is up: while it is not, the board is polled.
#[derive(Clone, Default)]
pub struct Feed {
    connected: Arc<AtomicBool>,
}

impl Feed {
    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Listens on its own thread, sends what each board event changed, and
    /// reconnects after a drop. Tries aiball's local socket first (trusted,
    /// no token, like the API), then its TCP port.
    pub fn start(changed: UnboundedSender<Change>) -> Self {
        let feed = Self::default();
        let connected = feed.connected.clone();
        std::thread::Builder::new()
            .name("aiball-feed".into())
            .spawn(move || {
                loop {
                    let alive = match connect_local() {
                        Ok(socket) => listen(socket, "the local socket", &changed, &connected),
                        Err(error) => {
                            log::debug!("aiball feed: local socket: {error}");
                            true
                        }
                    };
                    if !alive || changed.is_closed() {
                        return; // the window is gone
                    }
                    std::thread::sleep(Duration::from_secs(5));
                }
            })
            .expect("failed to start the feed thread");
        feed
    }
}

/// Reads events until the connection drops; `false` once nobody listens.
fn listen<S: std::io::Read + std::io::Write>(
    mut socket: tungstenite::WebSocket<S>,
    from: &str,
    changed: &UnboundedSender<Change>,
    connected: &AtomicBool,
) -> bool {
    log::info!("aiball feed: connected to {from}");
    connected.store(true, Ordering::Relaxed);
    // Catch up on whatever changed while it was down.
    let _ = changed.unbounded_send(Change::All);
    let mut alive = true;
    while let Ok(message) = socket.read() {
        let tungstenite::Message::Text(text) = message else {
            continue;
        };
        // A new ticket says so itself: it carries what to notify.
        let Some(change) = Filed::of(&text).map(Change::Filed).or_else(|| Change::of(&text)) else {
            continue;
        };
        if changed.unbounded_send(change).is_err() {
            alive = false;
            break;
        }
    }
    connected.store(false, Ordering::Relaxed);
    log::info!("aiball feed: dropped");
    alive
}

/// `/ws` over aiball's local socket, where the same user is trusted.
#[cfg(unix)]
fn connect_local() -> anyhow::Result<tungstenite::WebSocket<std::os::unix::net::UnixStream>> {
    let stream = std::os::unix::net::UnixStream::connect(crate::aiball::socket_path())?;
    let (socket, _) = tungstenite::client("ws://aiball/ws", stream)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(socket)
}

#[cfg(not(unix))]
fn connect_local() -> anyhow::Result<tungstenite::WebSocket<std::net::TcpStream>> {
    anyhow::bail!("no local socket on this platform")
}

#[cfg(test)]
mod tests {
    use super::Filed;

    #[test]
    fn a_ticket_filed_is_read_from_its_event() {
        let event = r#"{"type":"message_created","data":{"id":14,"project":"demo","kind":"ticket_created","title":"A title","by_agent":"demo-crew","status":"pending"}}"#;
        assert_eq!(
            Filed::of(event),
            Some(Filed { project: "demo".into(), ticket: 14, title: "A title".into(), by: "demo-crew".into(), pending: true })
        );
        let comment = r#"{"type":"message_created","data":{"id":15,"project":"demo","kind":"comment_added","ticket_id":14}}"#;
        assert_eq!(Filed::of(comment), None);
    }
}
