//! aiball's live feed (`/ws`): every change on the board — a message
//! posted, a decision taken, an agent's state — so tvty reads the board again
//! when it moves instead of every few seconds. Read-only; aiball serves it on
//! its TCP port only (`$AIBALL_URL`, default `http://127.0.0.1:7777`).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;

/// Whether the feed is up: while it is not, the board is polled.
#[derive(Clone, Default)]
pub struct Feed {
    connected: Arc<AtomicBool>,
}

impl Feed {
    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Listens on its own thread, sends `()` on every board event, and
    /// reconnects after a drop.
    pub fn start(changed: UnboundedSender<()>) -> Self {
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
                            let _ = changed.unbounded_send(());
                            while let Ok(message) = socket.read() {
                                let tungstenite::Message::Text(text) = message else {
                                    continue;
                                };
                                if text.contains("\"type\":\"hello\"") {
                                    continue;
                                }
                                if changed.unbounded_send(()).is_err() {
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
