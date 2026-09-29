//! One tvty per state directory: a second launch hands itself over to the
//! one running — which comes forward, and opens the session asked for — and
//! exits. The rendez-vous lives in the state directory, so a test tvty
//! (scripts/test-env, wbox: a state directory of its own) never meets the
//! user's. `TVTY_NEW_INSTANCE=1` starts one more anyway. docs/IPC.md, "One
//! tvty per state directory".

use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Duration;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use tvty_ipc::{Conn, Endpoint, Listener};

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

/// Where the running tvty waits for later launches.
#[derive(Clone, Debug)]
enum Rendezvous {
    /// A Unix socket (`tvty.sock`): the operating system lets only the user
    /// in.
    Socket(PathBuf),
    /// A loopback TCP port, written with a secret in a file (`tvty.addr`)
    /// only the user reads: anyone on the machine reaches the port, only the
    /// user knows what to say first.
    Tcp(PathBuf),
}

impl Rendezvous {
    /// This state directory's: a socket where there are Unix sockets.
    fn here() -> Option<Self> {
        let dir = config::dir(Place::State)?;
        Some(if cfg!(unix) { Self::Socket(dir.join("tvty.sock")) } else { Self::Tcp(dir.join("tvty.addr")) })
    }

    fn file(&self) -> &PathBuf {
        match self {
            Self::Socket(path) | Self::Tcp(path) => path,
        }
    }

    /// Where a running tvty would be, and the secret it expects.
    fn find(&self) -> Option<(Endpoint, Option<String>)> {
        match self {
            Self::Socket(path) => Some((Endpoint::Unix(path.clone()), None)),
            Self::Tcp(path) => {
                let text = std::fs::read_to_string(path).ok()?;
                let mut lines = text.lines();
                let endpoint = Endpoint::parse(lines.next()?).ok()?;
                Some((endpoint, Some(lines.next()?.trim().to_string())))
            }
        }
    }

    /// Takes the rendez-vous: a listener, and the secret it will ask for.
    /// Whatever a tvty that died left there goes first.
    fn open(&self) -> io::Result<(Listener, Option<String>)> {
        let _ = std::fs::remove_file(self.file());
        if let Some(parent) = self.file().parent() {
            std::fs::create_dir_all(parent)?;
        }
        match self {
            Self::Socket(path) => Ok((Listener::bind(&Endpoint::Unix(path.clone()))?, None)),
            Self::Tcp(path) => {
                let listener = Listener::bind(&Endpoint::Tcp(([127, 0, 0, 1], 0).into()))?;
                let secret = tvty_ipc::fresh_secret();
                // Written whole or not at all: a later launch never reads
                // an address without its secret.
                let partial = path.with_extension("addr.partial");
                std::fs::write(&partial, format!("{}\n{secret}\n", listener.local()))?;
                std::fs::rename(&partial, path)?;
                Ok((listener, Some(secret)))
            }
        }
    }
}

/// Whether this tvty holds the rendez-vous (the first one).
static HOLDS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Lets the rendez-vous go, before a restart: the tvty that follows claims
/// it instead of handing over to this one. Only the one holding it does.
pub fn release() {
    if HOLDS.swap(false, std::sync::atomic::Ordering::Relaxed)
        && let Some(rendezvous) = Rendezvous::here()
    {
        let _ = std::fs::remove_file(rendezvous.file());
    }
}

/// Hands over to the running tvty, or becomes the one: before the window.
pub fn claim(session: Option<&str>) -> Claim {
    if std::env::var_os("TVTY_NEW_INSTANCE").is_some() {
        return Claim::Alone;
    }
    let Some(rendezvous) = Rendezvous::here() else { return Claim::Alone };
    claim_at(&rendezvous, session)
}

fn claim_at(rendezvous: &Rendezvous, session: Option<&str>) -> Claim {
    if hand_over(rendezvous, session) {
        return Claim::Handed;
    }
    let (listener, secret) = match rendezvous.open() {
        Ok(opened) => opened,
        Err(error) => {
            log::warn!("instance: {}: {error}", rendezvous.file().display());
            return Claim::Alone;
        }
    };
    let (tx, rx) = unbounded();
    std::thread::Builder::new()
        .name("instance".into())
        .spawn(move || listen(listener, secret, tx))
        .map(|_| {
            HOLDS.store(true, std::sync::atomic::Ordering::Relaxed);
            Claim::First(rx)
        })
        .unwrap_or(Claim::Alone)
}

/// Tells the running tvty to come forward; whether it answered.
fn hand_over(rendezvous: &Rendezvous, session: Option<&str>) -> bool {
    let Some((endpoint, secret)) = rendezvous.find() else { return false };
    let Ok(mut conn) = endpoint.connect() else { return false };
    // A port a dead tvty left may be someone else's now: wait a little, not
    // for ever.
    let _ = conn.set_read_timeout(Some(Duration::from_secs(2)));
    if let Some(secret) = secret
        && writeln!(conn, "secret {secret}").is_err()
    {
        return false;
    }
    if writeln!(conn, "raise {}", session.unwrap_or("")).is_err() {
        return false;
    }
    let mut answer = String::new();
    BufReader::new(conn).read_line(&mut answer).is_ok() && answer.trim() == "ok"
}

fn listen(listener: Listener, secret: Option<String>, tx: UnboundedSender<Raise>) {
    loop {
        let mut conn = match listener.accept() {
            Ok(conn) => conn,
            Err(error) => {
                log::warn!("instance: {error}");
                continue;
            }
        };
        // One slow client must not hold the next up.
        let _ = conn.set_read_timeout(Some(Duration::from_secs(2)));
        if let Some(raise) = request(&conn, secret.as_deref()) {
            log::info!("instance: a later launch asks to come forward ({:?})", raise.session);
            let _ = tx.unbounded_send(raise);
            let _ = conn.write_all(b"ok\n");
        }
        if tx.is_closed() {
            return;
        }
    }
}

/// What a connection asks, once it said the secret (when there is one).
fn request(conn: &Conn, secret: Option<&str>) -> Option<Raise> {
    let mut reader = BufReader::new(conn.try_clone().ok()?);
    let mut line = String::new();
    if let Some(secret) = secret {
        reader.read_line(&mut line).ok()?;
        if line.trim().strip_prefix("secret ") != Some(secret) {
            log::warn!("instance: a connection without the secret, closed");
            return None;
        }
        line.clear();
    }
    reader.read_line(&mut line).ok()?;
    parse(&line)
}

/// `raise [SESSION]`.
fn parse(line: &str) -> Option<Raise> {
    let rest = line.trim().strip_prefix("raise")?;
    let session = rest.trim();
    Some(Raise { session: (!session.is_empty()).then(|| session.to_string()) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_later_launch_asks_to_raise_and_maybe_a_session() {
        assert_eq!(parse("raise \n"), Some(Raise { session: None }));
        assert_eq!(parse("raise cl-app-1\n"), Some(Raise { session: Some("cl-app-1".into()) }));
        assert_eq!(parse("hello"), None);
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tvty-instance-{}-{name}-{}", std::process::id(), tvty_ipc::fresh_secret()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The whole hand-over, on each rendez-vous this platform can hold.
    fn places() -> Vec<Rendezvous> {
        let mut places = vec![Rendezvous::Tcp(scratch("tcp").join("tvty.addr"))];
        if cfg!(unix) {
            places.push(Rendezvous::Socket(scratch("sock").join("tvty.sock")));
        }
        places
    }

    #[test]
    fn a_second_launch_hands_over_to_the_first() {
        for place in places() {
            let Claim::First(mut raises) = claim_at(&place, None) else { panic!("{place:?}: not first") };
            assert!(matches!(claim_at(&place, Some("cl-app")), Claim::Handed), "{place:?}");
            let raise = futures::executor::block_on(futures::StreamExt::next(&mut raises));
            assert_eq!(raise, Some(Raise { session: Some("cl-app".into()) }), "{place:?}");
        }
    }

    #[test]
    fn over_tcp_the_secret_is_asked_first() {
        let place = Rendezvous::Tcp(scratch("secret").join("tvty.addr"));
        let Claim::First(_raises) = claim_at(&place, None) else { panic!("not first") };
        let (endpoint, secret) = place.find().unwrap();
        assert!(secret.is_some());
        // Someone who read the port but not the file.
        let mut stranger = endpoint.connect().unwrap();
        stranger.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        writeln!(stranger, "raise evil").unwrap();
        let mut answer = String::new();
        let _ = BufReader::new(stranger).read_line(&mut answer);
        assert_ne!(answer.trim(), "ok");
    }

    #[test]
    fn a_rendezvous_a_dead_tvty_left_is_taken_over() {
        for place in places() {
            // What a crash leaves: the file, nobody behind it.
            match &place {
                Rendezvous::Tcp(path) => std::fs::write(path, "tcp://127.0.0.1:9\nstale\n").unwrap(),
                Rendezvous::Socket(path) => std::fs::write(path, "").unwrap(),
            }
            assert!(matches!(claim_at(&place, None), Claim::First(_)), "{place:?}");
        }
    }
}
