//! Attaching to a session a host holds, without tmux: aiball's LOOP-HOST
//! protocol (version 1, aiball's `docs/LOOP-HOST.md`) over the session's
//! `attach.sock`. The daemon's session host serves it, and claude-loop's
//! proxy will too.
//!
//! The socket is a path whatever the system. Where there are no Unix
//! sockets (Windows), the host listens on the loopback and writes where, and
//! a token, in a file beside the path (`attach.sock.addr`); the client says
//! the token in its `hello`.
//!
//! Frames both ways: `[type: u8][length: u32 BE][payload]`. tvty says
//! `hello` (interactive, the whole stream), then feeds the `snapshot` and
//! the `output` to its terminal as a PTY's bytes would be; its keys go as
//! `input`, its size as `resize`. The session's end (`exited` for good, or
//! `closed`) ends the view, as a PTY's child exiting does.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::Term;
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::vte::ansi;
use serde_json::{Value, json};

const HELLO: u8 = 0x01;
const WELCOME: u8 = 0x02;
const SNAPSHOT: u8 = 0x03;
const OUTPUT: u8 = 0x04;
const INPUT: u8 = 0x05;
const RESIZE: u8 = 0x06;
const FOCUS: u8 = 0x07;
const SIZE: u8 = 0x08;
const EXITED: u8 = 0x0a;
const CLOSED: u8 = 0x0b;
const ERROR: u8 = 0x0c;

/// The connection to a session (docs/IPC.md): its `attach.sock`, a Unix
/// socket, or the loopback port its address file names.
type Stream = tvty_ipc::Conn;

/// The history the snapshot brings above the screen.
const SCROLLBACK: u32 = 2000;

/// How long a session just started may take to listen on its socket.
const READY_TRIES: u32 = 30;
const READY_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);

/// The way to a session: what the view writes to it.
pub struct Attach {
    /// Once connected; what is written before is dropped (keys) or kept
    /// for the hello (the size).
    stream: Mutex<Option<Stream>>,
    size: Mutex<(u16, u16)>,
    closed: std::sync::atomic::AtomicBool,
    /// A client that types and whose size counts; a copy (an observer)
    /// sends neither keys nor sizes.
    interactive: bool,
}

impl Attach {
    /// Connects to `socket` as an interactive client of `size` (columns,
    /// lines), and feeds what comes to `term` on a thread of its own; the
    /// emulator's `listener` hears of each change, and of the end.
    /// The connection is made on that thread: a session just started may
    /// take a moment to listen, and the view does not wait for it.
    /// `interactive`: a client that types and whose size counts; else an
    /// observer, which only watches (a card), and never resizes the session.
    pub fn connect<L>(socket: &Path, size: (u16, u16), interactive: bool, term: Arc<FairMutex<Term<L>>>, listener: L) -> anyhow::Result<Arc<Self>>
    where
        L: EventListener + Clone + Send + 'static,
    {
        let attach = Arc::new(Self {
            stream: Mutex::new(None),
            size: Mutex::new(size),
            closed: std::sync::atomic::AtomicBool::new(false),
            interactive,
        });
        let (socket, this) = (socket.to_path_buf(), attach.clone());
        std::thread::Builder::new().name("attach".into()).spawn(move || {
            match this.open(&socket, interactive) {
                Ok(reader) => read(reader, term, listener),
                Err(error) => {
                    log::warn!("attach {}: {error:#}", socket.display());
                    listener.send_event(Event::Exit);
                }
            }
        })?;
        Ok(attach)
    }

    /// Connects (trying a while), says hello with the size wanted by then,
    /// and, interactive, takes the focus; answers the stream to read.
    fn open(&self, socket: &Path, interactive: bool) -> anyhow::Result<Stream> {
        let mut tries = 0;
        let (stream, token) = loop {
            match dial(socket) {
                Ok(dialled) => break dialled,
                Err(error) if tries < READY_TRIES && error.kind() != std::io::ErrorKind::Unsupported => {
                    tries += 1;
                    log::debug!("attach {}: not yet ({error})", socket.display());
                    std::thread::sleep(READY_PAUSE);
                }
                Err(error) => return Err(error.into()),
            }
        };
        let reader = stream.try_clone()?;
        *self.stream.lock().map_err(|_| anyhow::anyhow!("attach: poisoned"))? = Some(stream);
        if self.closed.load(std::sync::atomic::Ordering::Relaxed) {
            self.close();
        }
        let (columns, lines) = *self.size.lock().map_err(|_| anyhow::anyhow!("attach: poisoned"))?;
        let mut hello = json!({
            "version": 1,
            "client": "tvty",
            "mode": if interactive { "interactive" } else { "readonly" },
            "view": "stream",
            "scrollback": SCROLLBACK,
            "size": { "rows": lines, "cols": columns },
        });
        if let Some(token) = token {
            hello["token"] = Value::String(token);
        }
        self.send(HELLO, hello.to_string().as_bytes())?;
        // Opened to be shown: this client's size is the one to use.
        if interactive {
            self.focus();
        }
        Ok(reader)
    }

    /// Keys, as typed or pasted.
    pub fn input(&self, bytes: &[u8]) {
        if self.interactive {
            let _ = self.send(INPUT, bytes);
        }
    }

    /// The size this client would like (it takes it once it types).
    pub fn resize(&self, columns: u16, lines: u16) {
        if let Ok(mut size) = self.size.lock() {
            *size = (columns, lines);
        }
        if self.interactive {
            let _ = self.send(RESIZE, json!({ "rows": lines, "cols": columns }).to_string().as_bytes());
        }
    }

    /// This client took focus: its size is the one to use.
    pub fn focus(&self) {
        if self.interactive {
            let _ = self.send(FOCUS, b"{}");
        }
    }

    /// Leaves the session (it goes on without this client).
    pub fn close(&self) {
        self.closed.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Ok(stream) = self.stream.lock() {
            if let Some(stream) = stream.as_ref() {
                stream.shutdown();
            }
        }
    }

    fn send(&self, kind: u8, payload: &[u8]) -> anyhow::Result<()> {
        let mut frame = Vec::with_capacity(5 + payload.len());
        frame.push(kind);
        frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        frame.extend_from_slice(payload);
        let mut stream = self.stream.lock().map_err(|_| anyhow::anyhow!("attach: poisoned"))?;
        // Not connected yet: nothing to write to.
        let Some(stream) = stream.as_mut() else { return Ok(()) };
        stream.write_all(&frame)?;
        Ok(())
    }
}

/// The file a host that cannot listen on `socket` writes beside it: the
/// loopback port it listens on, and the token a client must say.
fn address_file(socket: &Path) -> PathBuf {
    let mut name = socket.as_os_str().to_os_string();
    name.push(".addr");
    PathBuf::from(name)
}

/// What an address file says: `{ "port": .., "token": ".." }`.
fn address(text: &str) -> Option<(tvty_ipc::Endpoint, String)> {
    let value: Value = serde_json::from_str(text).ok()?;
    let port = u16::try_from(value.get("port")?.as_u64()?).ok().filter(|port| *port != 0)?;
    let token = value.get("token")?.as_str().filter(|token| !token.is_empty())?;
    Some((tvty_ipc::Endpoint::Tcp(([127, 0, 0, 1], port).into()), token.to_string()))
}

/// The session's `attach.sock`, a path (aiball's contract): the port its
/// address file names, with the token to say, or else the Unix socket.
fn dial(socket: &Path) -> std::io::Result<(Stream, Option<String>)> {
    let file = address_file(socket);
    match std::fs::read_to_string(&file) {
        Ok(text) => {
            let (endpoint, token) = address(&text)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}: not a port and a token", file.display())))?;
            Ok((endpoint.connect()?, Some(token)))
        }
        // No Unix sockets here: the host has not written its address yet.
        Err(error) if !cfg!(unix) => Err(error),
        Err(_) => Ok((tvty_ipc::Endpoint::Unix(socket.to_path_buf()).connect()?, None)),
    }
}

/// The session's side, until it ends: frames to the terminal.
fn read<L: EventListener + Clone>(mut stream: Stream, term: Arc<FairMutex<Term<L>>>, listener: L) {
    let mut parser: ansi::Processor = ansi::Processor::new();
    let mut snapshots = 0usize;
    loop {
        let (kind, payload) = match frame(&mut stream) {
            Ok(frame) => frame,
            Err(error) => {
                log::info!("attach: the session's socket closed ({error})");
                break;
            }
        };
        match kind {
            WELCOME | SIZE => {
                // The session's size, which this client shows (cropping it
                // when smaller, as the protocol says).
                let value: Value = serde_json::from_slice(&payload).unwrap_or_default();
                let size = value.get("size").unwrap_or(&value);
                let (rows, cols) = (
                    size.get("rows").and_then(Value::as_u64).unwrap_or(0) as usize,
                    size.get("cols").and_then(Value::as_u64).unwrap_or(0) as usize,
                );
                if rows > 0 && cols > 0 {
                    let mut term = term.lock();
                    let was = { use alacritty_terminal::grid::Dimensions as _; (term.columns(), term.screen_lines()) };
                    if kind == SIZE && was != (cols, rows) {
                        // Another client took the size, or this one did.
                        log::info!("attach: the session's size is {cols}x{rows} (was {}x{})", was.0, was.1);
                    }
                    term.resize(TermSize::new(cols, rows));
                    drop(term);
                    listener.send_event(Event::Wakeup);
                }
            }
            SNAPSHOT | OUTPUT if payload.len() >= 8 => {
                let mut term = term.lock();
                // A later snapshot is a resync: it repaints a fresh terminal.
                if kind == SNAPSHOT {
                    if snapshots > 0 {
                        parser.advance(&mut *term, b"\x1bc");
                    }
                    snapshots += 1;
                }
                parser.advance(&mut *term, &payload[8..]);
                drop(term);
                listener.send_event(Event::Wakeup);
            }
            EXITED => {
                let value: Value = serde_json::from_slice(&payload).unwrap_or_default();
                // Restarting: the session goes on, a fresh snapshot follows.
                if value.get("restarting").and_then(Value::as_bool) != Some(true) {
                    log::info!("attach: the session's program ended ({value})");
                }
            }
            CLOSED => {
                log::info!("attach: the session closed ({})", String::from_utf8_lossy(&payload));
                break;
            }
            ERROR => log::warn!("attach: {}", String::from_utf8_lossy(&payload)),
            _ => {}
        }
    }
    listener.send_event(Event::Exit);
}

/// One frame: its type and payload.
fn frame(stream: &mut Stream) -> std::io::Result<(u8, Vec<u8>)> {
    let mut head = [0u8; 5];
    stream.read_exact(&mut head)?;
    let length = u32::from_be_bytes([head[1], head[2], head[3], head[4]]) as usize;
    let mut payload = vec![0u8; length];
    stream.read_exact(&mut payload)?;
    Ok((head[0], payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alacritty_terminal::grid::Dimensions as _;
    use alacritty_terminal::term::Config;
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc;
    use std::time::Duration;

    /// Hears of the view's end.
    #[derive(Clone)]
    struct Heard(mpsc::Sender<()>);

    impl EventListener for Heard {
        fn send_event(&self, event: Event) {
            if matches!(event, Event::Exit) {
                let _ = self.0.send(());
            }
        }
    }

    fn write(stream: &mut TcpStream, kind: u8, payload: &[u8]) {
        stream.write_all(&[kind]).unwrap();
        stream.write_all(&(payload.len() as u32).to_be_bytes()).unwrap();
        stream.write_all(payload).unwrap();
    }

    fn read_frame(stream: &mut TcpStream) -> std::io::Result<(u8, Vec<u8>)> {
        let mut head = [0u8; 5];
        stream.read_exact(&mut head)?;
        let mut payload = vec![0u8; u32::from_be_bytes([head[1], head[2], head[3], head[4]]) as usize];
        stream.read_exact(&mut payload)?;
        Ok((head[0], payload))
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tvty-attach-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A host on the loopback, its address written beside `socket`: it wants
    /// `token` in the hello, then shows a screen, echoes what is typed, and
    /// closes on a `q`. Answers what it was told: each hello, and the keys.
    fn host(socket: &Path, token: &'static str) -> mpsc::Receiver<Value> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::fs::write(address_file(socket), json!({ "port": port, "token": token }).to_string()).unwrap();
        let (told, tells) = mpsc::channel();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let Ok((HELLO, payload)) = read_frame(&mut stream) else { continue };
                let hello: Value = serde_json::from_slice(&payload).unwrap();
                let _ = told.send(hello.clone());
                if hello["token"] != token {
                    continue;
                }
                write(&mut stream, WELCOME, json!({ "version": 1, "size": { "rows": 5, "cols": 20 } }).to_string().as_bytes());
                write(&mut stream, SNAPSHOT, &[&[0u8; 8][..], b"held by a host"].concat());
                while let Ok((kind, payload)) = read_frame(&mut stream) {
                    if kind != INPUT {
                        continue;
                    }
                    let _ = told.send(json!({ "input": String::from_utf8_lossy(&payload) }));
                    if payload == b"q" {
                        write(&mut stream, CLOSED, b"{}");
                        break;
                    }
                    write(&mut stream, OUTPUT, &[&[0u8; 8][..], b"\r\n", &payload[..]].concat());
                }
            }
        });
        tells
    }

    /// A view's terminal, the way to it, and the word of its end.
    fn attached(socket: &Path, interactive: bool) -> (Arc<FairMutex<Term<Heard>>>, Arc<Attach>, mpsc::Receiver<()>) {
        let (heard, ended) = mpsc::channel();
        let heard = Heard(heard);
        let term = Arc::new(FairMutex::new(Term::new(Config::default(), &TermSize::new(80, 24), heard.clone())));
        let attach = Attach::connect(socket, (80, 24), interactive, term.clone(), heard).unwrap();
        (term, attach, ended)
    }

    fn line(term: &Arc<FairMutex<Term<Heard>>>, line: i32) -> String {
        use alacritty_terminal::index::{Column, Line};
        let term = term.lock();
        (0..term.columns()).map(|column| term.grid()[Line(line)][Column(column)].c).collect::<String>().trim_end().to_string()
    }

    /// Waits for the screen's first line to be `text`.
    fn shows(term: &Arc<FairMutex<Term<Heard>>>, text: &str) -> bool {
        for _ in 0..100 {
            if line(term, 0) == text {
                return true;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        false
    }

    const WAIT: Duration = Duration::from_secs(10);

    #[test]
    fn an_address_file_is_a_port_and_a_token() {
        assert_eq!(address_file(Path::new("/h/s/attach.sock")), PathBuf::from("/h/s/attach.sock.addr"));
        let (endpoint, token) = address(r#"{"port":4312,"token":"abc"}"#).unwrap();
        assert_eq!((endpoint.to_string().as_str(), token.as_str()), ("tcp://127.0.0.1:4312", "abc"));
        for text in ["", "{}", r#"{"port":0,"token":"abc"}"#, r#"{"port":4312}"#, r#"{"port":4312,"token":""}"#, r#"{"port":70000,"token":"abc"}"#] {
            assert!(address(text).is_none(), "{text}");
        }
    }

    #[test]
    fn a_session_is_attached_by_its_address_file_and_the_keys_held() {
        let socket = scratch("held").join("attach.sock");
        let tells = host(&socket, "the-token");
        let (term, attach, ended) = attached(&socket, true);
        let hello = tells.recv_timeout(WAIT).unwrap();
        assert_eq!((hello["token"].as_str(), hello["mode"].as_str()), (Some("the-token"), Some("interactive")));
        assert!(shows(&term, "held by a host"));
        // The session's size, not the one this client came with.
        let size = {
            let term = term.lock();
            (term.columns(), term.screen_lines())
        };
        assert_eq!(size, (20, 5));
        // The host echoes a key, and closes on `q`.
        attach.input(b"k");
        assert_eq!(tells.recv_timeout(WAIT).unwrap()["input"], "k");
        attach.input(b"q");
        ended.recv_timeout(WAIT).unwrap();
        assert_eq!(line(&term, 1), "k");
    }

    #[test]
    fn a_copy_watches_and_sends_no_keys() {
        let socket = scratch("copy").join("attach.sock");
        let tells = host(&socket, "the-token");
        let (term, attach, ended) = attached(&socket, false);
        assert_eq!(tells.recv_timeout(WAIT).unwrap()["mode"], "readonly");
        assert!(shows(&term, "held by a host"));
        attach.input(b"k");
        assert!(tells.recv_timeout(Duration::from_millis(300)).is_err());
        // Leaving ends the view; the session goes on.
        attach.close();
        ended.recv_timeout(WAIT).unwrap();
    }

    #[test]
    fn a_wrong_token_ends_the_view() {
        let socket = scratch("refused").join("attach.sock");
        let tells = host(&socket, "the-token");
        let file = address_file(&socket);
        std::fs::write(&file, std::fs::read_to_string(&file).unwrap().replace("the-token", "another")).unwrap();
        let (term, _attach, ended) = attached(&socket, true);
        assert_eq!(tells.recv_timeout(WAIT).unwrap()["token"], "another");
        ended.recv_timeout(WAIT).unwrap();
        assert_eq!(line(&term, 0), "");
    }
}
