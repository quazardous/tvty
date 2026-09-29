//! The debug control: a test drives tvty and reads it by name, not by
//! pixels — `scripts/tvty-ctl`, docs/TESTING.md. In a development build
//! only (`debug_assertions`: a release build has none, whatever its
//! environment), and there only when `TVTY_DEBUG_CONTROL` is set (the test
//! env does), never in the user's tvty. One JSON request a line, one JSON answer a line, on a socket in
//! the state directory (a test tvty's own, apart from the user's).
//!
//! The requests are carried out on the UI thread, by the shell
//! ([`crate::shell`]'s `control`): what is on screen is `crate::inspect`'s,
//! the clicks and keys go in through the window, as the user's would.

use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use serde_json::{Value, json};

use crate::config;
use tvty_config::Place;

/// A request, and where its answer goes.
pub struct Request {
    pub command: Value,
    pub reply: std::sync::mpsc::Sender<Value>,
}

/// How long a request may take on the UI thread before its caller hears so.
const ANSWER_WITHIN: Duration = Duration::from_secs(30);

/// The control's socket, when it is on.
pub fn socket() -> Option<std::path::PathBuf> {
    Some(config::dir(Place::State)?.join("tvty-control.sock"))
}

/// Opens the control when `TVTY_DEBUG_CONTROL` asks for it: the requests,
/// to carry out.
pub fn start() -> Option<UnboundedReceiver<Request>> {
    // Never in a release build: it drives the window and fakes failures.
    if !cfg!(debug_assertions) {
        return None;
    }
    std::env::var_os("TVTY_DEBUG_CONTROL").filter(|v| !v.is_empty())?;
    crate::inspect::enable();
    let path = socket()?;
    let (tx, rx) = unbounded();
    match listen(&path, tx) {
        Ok(()) => {
            log::info!("debug control: on, at {}", path.display());
            Some(rx)
        }
        Err(error) => {
            log::warn!("debug control: {}: {error:#}", path.display());
            None
        }
    }
}

#[cfg(unix)]
fn listen(path: &std::path::Path, tx: UnboundedSender<Request>) -> anyhow::Result<()> {
    use std::os::unix::net::UnixListener;
    let _ = std::fs::remove_file(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let listener = UnixListener::bind(path)?;
    std::thread::Builder::new().name("control".into()).spawn(move || {
        for stream in listener.incoming().flatten() {
            let tx = tx.clone();
            let _ = std::thread::Builder::new().name("control-client".into()).spawn(move || serve(stream, tx));
        }
    })?;
    Ok(())
}

#[cfg(not(unix))]
fn listen(_: &std::path::Path, _: UnboundedSender<Request>) -> anyhow::Result<()> {
    anyhow::bail!("the debug control waits for tvty-ipc's rendez-vous here")
}

/// One client: its requests, one a line, each answered before the next.
#[cfg(unix)]
fn serve(stream: std::os::unix::net::UnixStream, tx: UnboundedSender<Request>) {
    let mut writer = match stream.try_clone() {
        Ok(writer) => writer,
        Err(_) => return,
    };
    for line in BufReader::new(stream).lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let answer = match serde_json::from_str::<Value>(&line) {
            Ok(command) => {
                let (reply, answer) = std::sync::mpsc::channel();
                if tx.unbounded_send(Request { command, reply }).is_err() {
                    break;
                }
                answer.recv_timeout(ANSWER_WITHIN).unwrap_or_else(|_| json!({ "error": "no answer from the UI thread" }))
            }
            Err(error) => json!({ "error": format!("not JSON: {error}") }),
        };
        if writeln!(writer, "{answer}").is_err() {
            break;
        }
    }
}
