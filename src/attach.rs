//! Attaching to a session a host holds, without tmux: aiball's LOOP-HOST
//! protocol (version 1, aiball's `docs/LOOP-HOST.md`) over the session's
//! `attach.sock`. The daemon's session host serves it, and claude-loop's
//! proxy will too.
//!
//! Frames both ways: `[type: u8][length: u32 BE][payload]`. tvty says
//! `hello` (interactive, the whole stream), then feeds the `snapshot` and
//! the `output` to its terminal as a PTY's bytes would be; its keys go as
//! `input`, its size as `resize`. The session's end (`exited` for good, or
//! `closed`) ends the view, as a PTY's child exiting does.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
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

/// The history the snapshot brings above the screen.
const SCROLLBACK: u32 = 2000;

/// The way to a session: what the view writes to it.
pub struct Attach {
    stream: Mutex<UnixStream>,
}

impl Attach {
    /// Connects to `socket` as an interactive client of `size` (columns,
    /// lines), and feeds what comes to `term` on a thread of its own; the
    /// emulator's `listener` hears of each change, and of the end.
    pub fn connect<L>(socket: &Path, size: (u16, u16), term: Arc<FairMutex<Term<L>>>, listener: L) -> anyhow::Result<Arc<Self>>
    where
        L: EventListener + Clone + Send + 'static,
    {
        let stream = UnixStream::connect(socket)?;
        let reader = stream.try_clone()?;
        let attach = Arc::new(Self { stream: Mutex::new(stream) });
        attach.send(
            HELLO,
            json!({
                "version": 1,
                "client": "tvty",
                "mode": "interactive",
                "view": "stream",
                "scrollback": SCROLLBACK,
                "size": { "rows": size.1, "cols": size.0 },
            })
            .to_string()
            .as_bytes(),
        )?;
        std::thread::Builder::new()
            .name("attach".into())
            .spawn(move || read(reader, term, listener))?;
        Ok(attach)
    }

    /// Keys, as typed or pasted.
    pub fn input(&self, bytes: &[u8]) {
        let _ = self.send(INPUT, bytes);
    }

    /// The size this client would like (it takes it once it types).
    pub fn resize(&self, columns: u16, lines: u16) {
        let _ = self.send(RESIZE, json!({ "rows": lines, "cols": columns }).to_string().as_bytes());
    }

    /// This client took focus: its size is the one to use.
    pub fn focus(&self) {
        let _ = self.send(FOCUS, b"{}");
    }

    /// Leaves the session (it goes on without this client).
    pub fn close(&self) {
        if let Ok(stream) = self.stream.lock() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }

    fn send(&self, kind: u8, payload: &[u8]) -> anyhow::Result<()> {
        let mut frame = Vec::with_capacity(5 + payload.len());
        frame.push(kind);
        frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        frame.extend_from_slice(payload);
        let mut stream = self.stream.lock().map_err(|_| anyhow::anyhow!("attach: poisoned"))?;
        stream.write_all(&frame)?;
        Ok(())
    }
}

/// The session's side, until it ends: frames to the terminal.
fn read<L: EventListener + Clone>(mut stream: UnixStream, term: Arc<FairMutex<Term<L>>>, listener: L) {
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
                    term.lock().resize(TermSize::new(cols, rows));
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
fn frame(stream: &mut UnixStream) -> std::io::Result<(u8, Vec<u8>)> {
    let mut head = [0u8; 5];
    stream.read_exact(&mut head)?;
    let length = u32::from_be_bytes([head[1], head[2], head[3], head[4]]) as usize;
    let mut payload = vec![0u8; length];
    stream.read_exact(&mut payload)?;
    Ok((head[0], payload))
}
