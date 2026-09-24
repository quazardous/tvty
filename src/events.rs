//! aiball's live feed (`/ws`): every change on the board — a message
//! posted, a decision taken, an agent's state — so tvty reads again what
//! moved, when it moves, instead of everything every few seconds. Read-only;
//! aiball serves it on its TCP port only (`$AIBALL_URL`, default
//! `http://127.0.0.1:7777`).

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
            "consumer_changed" => Some(Change::Consumers),
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
    /// reconnects after a drop.
    pub fn start(changed: UnboundedSender<Change>) -> Self {
        let feed = Self::default();
        let connected = feed.connected.clone();
        let url = ws_url();
        std::thread::Builder::new()
            .name("aiball-feed".into())
            .spawn(move || {
                loop {
                    match tungstenite::connect(&url) {
                        Ok((mut socket, _)) => {
                            log::info!("aiball feed: connected to {url}");
                            connected.store(true, Ordering::Relaxed);
                            // Catch up on whatever changed while it was down.
                            let _ = changed.unbounded_send(Change::All);
                            while let Ok(message) = socket.read() {
                                let tungstenite::Message::Text(text) = message else {
                                    continue;
                                };
                                let Some(change) = Change::of(&text) else {
                                    continue;
                                };
                                if changed.unbounded_send(change).is_err() {
                                    return; // the window is gone
                                }
                            }
                            connected.store(false, Ordering::Relaxed);
                            log::info!("aiball feed: dropped");
                        }
                        Err(error) => log::debug!("aiball feed: {error}"),
                    }
                    if changed.is_closed() {
                        return;
                    }
                    std::thread::sleep(Duration::from_secs(5));
                }
            })
            .expect("failed to start the feed thread");
        feed
    }
}

fn ws_url() -> String {
    let base = std::env::var("AIBALL_URL").unwrap_or_else(|_| "http://127.0.0.1:7777".into());
    let base = base.trim_end_matches('/');
    let base = base
        .strip_prefix("https://")
        .map(|rest| format!("wss://{rest}"))
        .or_else(|| base.strip_prefix("http://").map(|rest| format!("ws://{rest}")))
        .unwrap_or_else(|| base.to_string());
    format!("{base}/ws")
}
