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
/// This client closes the session's other clients; the host answers how
/// many with `detached_others` (hosts whose welcome says the feature).
const DETACH_OTHERS: u8 = 0x0f;
const DETACHED_OTHERS: u8 = 0x10;

/// The connection to a session (docs/IPC.md): its `attach.sock`, a Unix
/// socket, or the loopback port its address file names.
type Stream = tvty_ipc::Conn;

/// The history the snapshot brings above the screen.
const SCROLLBACK: u32 = 2000;

/// How long a session just started may take to listen on its socket.
const READY_TRIES: u32 = 30;
const READY_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);
/// How long a hello waits for the view's own size (in fifths of a pause).
const SIZE_TRIES: u32 = 50;

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
    /// What the host said it does beyond version 1 (its welcome's
    /// `features`): `detach_others`.
    features: Mutex<Vec<String>>,
    /// Another client closed this one (`closed` with `detached_by_other`).
    detached_by_other: std::sync::atomic::AtomicBool,
    /// How many clients this one's last `detach_others` closed, once said.
    others_closed: Mutex<Option<u64>>,
    /// The view said its real size (`resize`): until then the size held is
    /// a placeholder, which must not become the session's.
    sized: std::sync::atomic::AtomicBool,
    /// Connected before its real size: the focus waits for it.
    focus_waits: std::sync::atomic::AtomicBool,
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
            features: Mutex::new(Vec::new()),
            detached_by_other: std::sync::atomic::AtomicBool::new(false),
            others_closed: Mutex::new(None),
            sized: std::sync::atomic::AtomicBool::new(false),
            focus_waits: std::sync::atomic::AtomicBool::new(false),
        });
        let (socket, this) = (socket.to_path_buf(), attach.clone());
        std::thread::Builder::new().name("attach".into()).spawn(move || {
            match this.open(&socket, interactive) {
                Ok(reader) => read(reader, &this, term, listener),
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
        // The size the hello asks for becomes the session's when nobody owns
        // it: the view's own, once laid out — within a frame or two. Not
        // known by then, none is asked (the session keeps its size until
        // this client's first resize and focus).
        if interactive {
            for _ in 0..SIZE_TRIES {
                if self.sized.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(READY_PAUSE / 5);
            }
        }
        let asked = self.sized.load(std::sync::atomic::Ordering::Relaxed).then(|| *self.size.lock().unwrap_or_else(|e| e.into_inner()));
        let mut hello = json!({
            "version": 1,
            "client": "tvty",
            "mode": if interactive { "interactive" } else { "readonly" },
            "view": "stream",
            "scrollback": SCROLLBACK,
        });
        if let Some((columns, lines)) = asked {
            hello["size"] = json!({ "rows": lines, "cols": columns });
        }
        if let Some(token) = token {
            hello["token"] = Value::String(token);
        }
        // The hello first, before the stream is this client's: what the view
        // writes meanwhile (a resize) waits for it.
        let mut stream = stream;
        stream.write_all(&frame_of(HELLO, hello.to_string().as_bytes()))?;
        let reader = stream.try_clone()?;
        *self.stream.lock().map_err(|_| anyhow::anyhow!("attach: poisoned"))? = Some(stream);
        if self.closed.load(std::sync::atomic::Ordering::Relaxed) {
            self.close();
        }
        // Laid out between the hello and now: the size it did not carry.
        let now = self.sized.load(std::sync::atomic::Ordering::Relaxed).then(|| *self.size.lock().unwrap_or_else(|e| e.into_inner()));
        if interactive && now.is_some() && now != asked {
            let (columns, lines) = now.unwrap_or_default();
            self.send(RESIZE, json!({ "rows": lines, "cols": columns }).to_string().as_bytes())?;
        }
        // Opened to be shown: this client's size is the one to use — once
        // it is known. The placeholder taken as the session's size would
        // make its program redraw at it, then again at the real one (Claude
        // Code leaves both in the history).
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
        self.sized.store(true, std::sync::atomic::Ordering::Relaxed);
        if self.interactive {
            let _ = self.send(RESIZE, json!({ "rows": lines, "cols": columns }).to_string().as_bytes());
            // Connected before its size was known: it takes the size now.
            if self.focus_waits.swap(false, std::sync::atomic::Ordering::Relaxed) {
                self.focus();
            }
        }
    }

    /// This client took focus: its size is the one to use.
    /// Before its real size is known, the focus waits for it (`resize`).
    pub fn focus(&self) {
        if !self.interactive {
            return;
        }
        if self.sized.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = self.send(FOCUS, b"{}");
        } else {
            self.focus_waits.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// The host closes the session's other clients (claude-loop's terminal,
    /// another tvty): this one and the session go on. Only an interactive
    /// client, on a host that says it can.
    pub fn detach_others(&self) {
        if self.can_detach_others() {
            if let Ok(mut closed) = self.others_closed.lock() {
                *closed = None;
            }
            let _ = self.send(DETACH_OTHERS, b"{}");
        }
    }

    /// Whether this client may close the others: interactive, on a host
    /// whose welcome says `detach_others`.
    pub fn can_detach_others(&self) -> bool {
        self.interactive && self.features.lock().is_ok_and(|f| f.iter().any(|f| f == "detach_others"))
    }

    /// How many clients the last `detach_others` closed, once the host said
    /// it (taken: said once).
    pub fn take_others_closed(&self) -> Option<u64> {
        self.others_closed.lock().ok()?.take()
    }

    /// Another client closed this one: not an end of the session.
    pub fn detached_by_other(&self) -> bool {
        self.detached_by_other.load(std::sync::atomic::Ordering::Relaxed)
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
        let frame = frame_of(kind, payload);
        let mut stream = self.stream.lock().map_err(|_| anyhow::anyhow!("attach: poisoned"))?;
        // Not connected yet: nothing to write to.
        let Some(stream) = stream.as_mut() else { return Ok(()) };
        stream.write_all(&frame)?;
        Ok(())
    }
}

/// A frame: `[type][length, big-endian][payload]`.
fn frame_of(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(5 + payload.len());
    frame.push(kind);
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(payload);
    frame
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
fn read<L: EventListener + Clone>(mut stream: Stream, attach: &Attach, term: Arc<FairMutex<Term<L>>>, listener: L) {
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
                if kind == WELCOME {
                    let features = value.get("features").and_then(Value::as_array).into_iter().flatten();
                    if let Ok(mut said) = attach.features.lock() {
                        *said = features.filter_map(Value::as_str).map(str::to_string).collect();
                    }
                }
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
                let value: Value = serde_json::from_slice(&payload).unwrap_or_default();
                if value.get("reason").and_then(Value::as_str) == Some("detached_by_other") {
                    log::info!("attach: another client closed this one; the session goes on");
                    attach.detached_by_other.store(true, std::sync::atomic::Ordering::Relaxed);
                } else {
                    log::info!("attach: the session closed ({value})");
                }
                break;
            }
            DETACHED_OTHERS => {
                let value: Value = serde_json::from_slice(&payload).unwrap_or_default();
                let count = value.get("count").and_then(Value::as_u64).unwrap_or(0);
                log::info!("attach: {count} other client(s) closed");
                if let Ok(mut closed) = attach.others_closed.lock() {
                    *closed = Some(count);
                }
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

    /// A real host's session, watched as a copy (it neither types nor
    /// resizes): `TVTY_ATTACH_PROBE` names its `attach.sock`.
    ///
    ///     TVTY_ATTACH_PROBE=…/hosts/NAME/attach.sock cargo test --bin tvty real_host -- --ignored --nocapture
    #[test]
    #[ignore = "needs a host that runs: TVTY_ATTACH_PROBE"]
    fn a_real_host_is_watched() {
        let socket = PathBuf::from(std::env::var("TVTY_ATTACH_PROBE").expect("TVTY_ATTACH_PROBE: a session's attach.sock"));
        let (term, attach, ended) = attached(&socket, false);
        let started = std::time::Instant::now();
        // Its screen, once it came.
        let blank = || {
            let rows = term.lock().screen_lines() as i32;
            (0..rows).all(|row| line(&term, row).is_empty())
        };
        while started.elapsed() < WAIT && blank() {
            assert!(ended.try_recv().is_err(), "the view ended before any screen");
            std::thread::sleep(Duration::from_millis(50));
        }
        let first = started.elapsed();
        let (columns, rows) = {
            let term = term.lock();
            (term.columns(), term.screen_lines())
        };
        println!("attached in {first:?}; the session is {columns}x{rows}:");
        for row in 0..rows as i32 {
            let text = line(&term, row);
            if !text.is_empty() {
                println!("  {text}");
            }
        }
        attach.close();
        ended.recv_timeout(WAIT).unwrap();
    }

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

    /// A view attached before it is laid out says no size of its own: the
    /// placeholder it holds is not taken as the session's (no `focus`) until
    /// its real size comes, and then it takes it.
    #[test]
    fn the_session_takes_no_size_before_the_view_knows_its_own() {
        // Laid out before it connects: its hello asks its own size.
        let early = scratch("sized-early").join("attach.sock");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::fs::write(address_file(&early), json!({ "port": port, "token": "t" }).to_string()).unwrap();
        let (told, hellos) = mpsc::channel();
        std::thread::spawn(move || {
            let mut stream = listener.incoming().next().unwrap().unwrap();
            if let Ok((HELLO, payload)) = read_frame(&mut stream) {
                let _ = told.send(serde_json::from_slice::<Value>(&payload).unwrap());
            }
        });
        let (_, attach, _) = attached(&early, true);
        attach.resize(150, 40);
        assert_eq!(hellos.recv_timeout(WAIT).unwrap()["size"], json!({ "rows": 40, "cols": 150 }));
        attach.close();

        let socket = scratch("sized").join("attach.sock");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::fs::write(address_file(&socket), json!({ "port": port, "token": "t" }).to_string()).unwrap();
        let (told, tells) = mpsc::channel();
        let (told_size, sizes) = mpsc::channel();
        std::thread::spawn(move || {
            let mut stream = listener.incoming().next().unwrap().unwrap();
            while let Ok((kind, payload)) = read_frame(&mut stream) {
                if kind == HELLO {
                    // What size it asked for, if any.
                    let hello: Value = serde_json::from_slice(&payload).unwrap();
                    let _ = told_size.send(hello.get("size").cloned());
                }
                let _ = told.send(kind);
                if kind == HELLO {
                    write(&mut stream, WELCOME, json!({ "version": 1, "size": { "rows": 40, "cols": 150 } }).to_string().as_bytes());
                }
            }
        });
        let (_, attach, _) = attached(&socket, true);
        assert_eq!(tells.recv_timeout(WAIT).unwrap(), HELLO);
        // Not laid out in time: the hello asks no size (the placeholder
        // would have become the session's).
        assert_eq!(sizes.recv_timeout(WAIT).unwrap(), None);
        // Connected, not laid out yet: nothing more.
        assert!(tells.recv_timeout(Duration::from_millis(400)).is_err());
        attach.resize(150, 40);
        assert_eq!(tells.recv_timeout(WAIT).unwrap(), RESIZE);
        assert_eq!(tells.recv_timeout(WAIT).unwrap(), FOCUS);
        // Shown again later: the focus goes at once.
        attach.focus();
        assert_eq!(tells.recv_timeout(WAIT).unwrap(), FOCUS);
        attach.close();
    }

    /// A host that says `detach_others`: two clients, the first closes
    /// the other, which hears it was detached by another client.
    #[test]
    fn a_client_closes_the_others_and_the_other_hears_why() {
        let socket = scratch("others").join("attach.sock");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::fs::write(address_file(&socket), json!({ "port": port, "token": "t" }).to_string()).unwrap();
        std::thread::spawn(move || {
            let mut clients = Vec::new();
            for stream in listener.incoming().take(2) {
                let mut stream = stream.unwrap();
                read_frame(&mut stream).unwrap();
                write(&mut stream, WELCOME, json!({ "version": 1, "size": { "rows": 5, "cols": 20 }, "features": ["detach_others"] }).to_string().as_bytes());
                clients.push(stream);
            }
            let (mut first, mut second) = (clients.remove(0), clients.remove(0));
            while let Ok((kind, _)) = read_frame(&mut first) {
                if kind == DETACH_OTHERS {
                    write(&mut second, CLOSED, br#"{"reason":"detached_by_other"}"#);
                    write(&mut first, DETACHED_OTHERS, br#"{"count":1}"#);
                }
            }
        });
        let (_, first, _) = attached(&socket, true);
        for _ in 0..100 {
            if first.can_detach_others() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let (_, second, second_ended) = attached(&socket, true);
        for _ in 0..100 {
            if second.can_detach_others() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(first.can_detach_others());
        first.detach_others();
        second_ended.recv_timeout(WAIT).unwrap();
        assert!(second.detached_by_other());
        assert!(!first.detached_by_other());
        let mut count = None;
        for _ in 0..100 {
            count = first.take_others_closed();
            if count.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(count, Some(1));
        first.close();
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
