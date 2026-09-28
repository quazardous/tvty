//! One tvty per state directory: a second launch hands itself over to the
//! one running — which comes forward, and opens the session asked for — and
//! exits. The rendez-vous is a socket in the state directory, so a test tvty
//! (scripts/test-env, wbox: a state directory of its own) never meets the
//! user's. `TVTY_NEW_INSTANCE=1` starts one more anyway.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::Duration;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};

use crate::config;
use tvty_config::Place;

/// What a later launch asks of the running tvty.
#[derive(Debug, PartialEq)]
pub struct Raise {
    /// The session to open (`tvty SESSION`), if any.
    pub session: Option<String>,
}

/// The first instance's requests, for the window to take (once).
pub struct Raises(pub Option<UnboundedReceiver<Raise>>);
impl gpui_kit::Global for Raises {}

/// How this launch starts.
pub enum Claim {
    /// It is the instance: later launches' requests come here.
    First(UnboundedReceiver<Raise>),
    /// Another one runs, and was told: this launch is done.
    Handed,
    /// No rendez-vous (no state directory, or one more instance asked for).
    Alone,
}

/// Whether this tvty holds the rendez-vous (the first one).
static HOLDS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Lets the rendez-vous go, before a restart: the tvty that follows claims
/// it instead of handing over to this one. Only the one holding it does.
pub fn release() {
    if HOLDS.swap(false, std::sync::atomic::Ordering::Relaxed)
        && let Some(path) = socket()
    {
        let _ = std::fs::remove_file(path);
    }
}

fn socket() -> Option<PathBuf> {
    Some(config::dir(Place::State)?.join("tvty.sock"))
}

/// Hands over to the running tvty, or becomes the one: before the window.
pub fn claim(session: Option<&str>) -> Claim {
    if std::env::var_os("TVTY_NEW_INSTANCE").is_some() {
        return Claim::Alone;
    }
    let Some(path) = socket() else { return Claim::Alone };
    if hand_over(&path, session) {
        return Claim::Handed;
    }
    // None answered: a socket left by one that died goes.
    let _ = std::fs::remove_file(&path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let listener = match UnixListener::bind(&path) {
        Ok(listener) => listener,
        Err(error) => {
            log::warn!("instance: {}: {error}", path.display());
            return Claim::Alone;
        }
    };
    let (tx, rx) = unbounded();
    std::thread::Builder::new()
        .name("instance".into())
        .spawn(move || listen(listener, tx))
        .map(|_| {
            HOLDS.store(true, std::sync::atomic::Ordering::Relaxed);
            Claim::First(rx)
        })
        .unwrap_or(Claim::Alone)
}

/// Tells the running tvty to come forward; whether it answered.
fn hand_over(path: &PathBuf, session: Option<&str>) -> bool {
    let Ok(mut stream) = UnixStream::connect(path) else { return false };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    if writeln!(stream, "raise {}", session.unwrap_or("")).is_err() {
        return false;
    }
    let mut answer = String::new();
    BufReader::new(stream).read_line(&mut answer).is_ok() && answer.trim() == "ok"
}

fn listen(listener: UnixListener, tx: UnboundedSender<Raise>) {
    for stream in listener.incoming().flatten() {
        let mut line = String::new();
        let mut reader = BufReader::new(&stream);
        if reader.read_line(&mut line).is_err() {
            continue;
        }
        if let Some(raise) = parse(&line) {
            log::info!("instance: a later launch asks to come forward ({:?})", raise.session);
            let _ = tx.unbounded_send(raise);
            let _ = (&stream).write_all(b"ok\n");
        }
    }
}

/// `raise [SESSION]`.
fn parse(line: &str) -> Option<Raise> {
    let rest = line.trim().strip_prefix("raise")?;
    let session = rest.trim();
    Some(Raise { session: (!session.is_empty()).then(|| session.to_string()) })
}

#[cfg(test)]
mod tests {
    use super::{Raise, parse};

    #[test]
    fn a_later_launch_asks_to_raise_and_maybe_a_session() {
        assert_eq!(parse("raise \n"), Some(Raise { session: None }));
        assert_eq!(parse("raise cl-app-1\n"), Some(Raise { session: Some("cl-app-1".into()) }));
        assert_eq!(parse("hello"), None);
    }
}
