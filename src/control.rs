//! The debug control: a test drives tvty and reads it by name, not by
//! pixels — `scripts/tvty-ctl`, docs/TESTING.md. In a development build
//! only (`debug_assertions`: a release build has none, whatever its
//! environment), and there only when `TVTY_DEBUG_CONTROL` is set (the test
//! env does), never in the user's tvty. One JSON request a line, one JSON
//! answer a line, at a rendez-vous in the state directory (a test tvty's
//! own, apart from the user's), as tvty's instance has its own
//! (`crate::instance`): a Unix socket (`tvty-control.sock`) where there are
//! some; elsewhere (Windows) a loopback TCP port written with a secret in
//! `tvty-control.addr`, the secret said first (`secret …`).
//!
//! The requests are carried out on the UI thread, by the shell
//! ([`crate::shell`]'s `control`): what is on screen is `crate::inspect`'s,
//! the clicks and keys go in through the window, as the user's would.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Duration;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use serde_json::{Value, json};

use tvty_ipc::{Conn, Endpoint, Listener};

use crate::config;
use tvty_config::Place;

/// A request, and where its answer goes.
pub struct Request {
    pub command: Value,
    pub reply: std::sync::mpsc::Sender<Value>,
}

/// How long a request may take on the UI thread before its caller hears so.
const ANSWER_WITHIN: Duration = Duration::from_secs(30);

/// The control on a TCP port: where there are no Unix sockets, or asked
/// (`TVTY_CONTROL_TCP`, to test Windows' way on Linux).
fn over_tcp() -> bool {
    !cfg!(unix) || std::env::var_os("TVTY_CONTROL_TCP").is_some_and(|v| !v.is_empty())
}

/// Where the control waits: a socket where there are Unix sockets, else
/// the file saying the TCP port and its secret.
pub fn rendezvous() -> Option<PathBuf> {
    let dir = config::dir(Place::State)?;
    Some(dir.join(if over_tcp() { "tvty-control.addr" } else { "tvty-control.sock" }))
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
    let path = rendezvous()?;
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

fn listen(path: &std::path::Path, tx: UnboundedSender<Request>) -> anyhow::Result<()> {
    let _ = std::fs::remove_file(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let (listener, secret) = if !over_tcp() {
        (Listener::bind(&Endpoint::Unix(path.to_path_buf()))?, None)
    } else {
        let listener = Listener::bind(&Endpoint::Tcp(([127, 0, 0, 1], 0).into()))?;
        let secret = tvty_ipc::fresh_secret();
        // Written whole or not at all: a client never reads an address
        // without its secret.
        let partial = path.with_extension("addr.partial");
        std::fs::write(&partial, format!("{}\n{secret}\n", listener.local()))?;
        // The user's alone (Windows' own folder is already).
        #[cfg(unix)]
        std::fs::set_permissions(&partial, std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
        std::fs::rename(&partial, path)?;
        (listener, Some(secret))
    };
    std::thread::Builder::new().name("control".into()).spawn(move || {
        loop {
            match listener.accept() {
                Ok(conn) => {
                    let (tx, secret) = (tx.clone(), secret.clone());
                    let _ = std::thread::Builder::new().name("control-client".into()).spawn(move || serve(conn, secret, tx));
                }
                Err(error) => log::warn!("debug control: {error}"),
            }
            if tx.is_closed() {
                return;
            }
        }
    })?;
    Ok(())
}

/// One client: the secret first when there is one, then its requests, one
/// a line, each answered before the next.
fn serve(conn: Conn, secret: Option<String>, tx: UnboundedSender<Request>) {
    let mut writer = match conn.try_clone() {
        Ok(writer) => writer,
        Err(_) => return,
    };
    let mut lines = BufReader::new(conn).lines();
    if let Some(secret) = secret {
        let said = lines.next().and_then(Result::ok);
        if said.as_deref().map(str::trim).and_then(|l| l.strip_prefix("secret ")) != Some(secret.as_str()) {
            log::warn!("debug control: a connection without the secret, closed");
            return;
        }
    }
    for line in lines {
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
