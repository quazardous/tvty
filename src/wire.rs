//! aiball's bus: one permanent connection to the daemon, JSON-RPC 2.0 over a
//! WebSocket on `/bus` (aiball's `docs/API-BUS.md`). Calls go out and their
//! answers come back on it, matched by id; what the daemon sends on its own
//! (its greeting, later the subscriptions' data) comes out as notifications.
//!
//! The connection lives on a thread of its own, which reconnects after a
//! drop. A call made while it is down fails at once; the caller reads again
//! once it is back.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context as _, anyhow};
use futures::channel::mpsc::UnboundedSender;
use serde_json::{Value, json};

/// How long a call waits for its answer.
const CALL_TIMEOUT: Duration = Duration::from_secs(10);

/// Something the daemon sent on its own.
#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

/// Who the connection runs as, from the daemon's greeting.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hello {
    pub version: u64,
    pub consumer: String,
    pub kind: String,
}

impl Hello {
    fn of(params: &Value) -> Self {
        let text = |key: &str| params.get(key).and_then(Value::as_str).unwrap_or_default().to_string();
        Self { version: params.get("version").and_then(Value::as_u64).unwrap_or(0), consumer: text("consumer"), kind: text("kind") }
    }
}

/// A call on its way: its frame, and where its answer goes.
struct Outgoing {
    frame: String,
    id: u64,
    answer: Sender<anyhow::Result<Value>>,
}

/// The bus, as the rest of tvty holds it: cheap to clone.
#[derive(Clone)]
pub struct Wire {
    outgoing: Sender<Outgoing>,
    next_id: Arc<Mutex<u64>>,
    hello: Arc<Mutex<Option<Hello>>>,
}

impl Wire {
    /// Connects as `user` (the local socket trusts the header), and keeps
    /// the connection up. The daemon's notifications go to `notices`.
    pub fn start(user: String, notices: UnboundedSender<Notification>) -> Self {
        let (outgoing, queue) = mpsc::channel::<Outgoing>();
        let hello = Arc::new(Mutex::new(None));
        let wire = Self { outgoing, next_id: Arc::new(Mutex::new(1)), hello: hello.clone() };
        std::thread::Builder::new()
            .name("aiball-bus".into())
            .spawn(move || run(user, queue, notices, hello))
            .expect("failed to start the bus thread");
        wire
    }

    /// Who the connection runs as, once the daemon greeted it.
    pub fn hello(&self) -> Option<Hello> {
        self.hello.lock().ok().and_then(|h| h.clone())
    }

    /// Calls `method` and waits for its answer. Blocking: off the UI thread.
    pub fn call(&self, method: &str, params: Value) -> anyhow::Result<Value> {
        let id = {
            let mut next = self.next_id.lock().map_err(|_| anyhow!("the bus is gone"))?;
            *next += 1;
            *next
        };
        let frame = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }).to_string();
        let (answer, answered) = mpsc::channel();
        self.outgoing.send(Outgoing { frame, id, answer }).map_err(|_| anyhow!("the bus is gone"))?;
        match answered.recv_timeout(CALL_TIMEOUT) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => Err(anyhow!("{method}: no answer from aiball's bus")),
            Err(RecvTimeoutError::Disconnected) => Err(anyhow!("{method}: aiball's bus dropped")),
        }
    }
}

/// The connection's thread: connect, serve, reconnect, until tvty is gone.
fn run(user: String, queue: Receiver<Outgoing>, notices: UnboundedSender<Notification>, hello: Arc<Mutex<Option<Hello>>>) {
    loop {
        match connect(&user) {
            Ok(socket) => serve(socket, &queue, &notices, &hello),
            Err(error) => log::debug!("aiball bus: {error:#}"),
        }
        if let Ok(mut h) = hello.lock() {
            *h = None;
        }
        if notices.is_closed() {
            return;
        }
        // While down, calls fail at once rather than wait.
        let until = std::time::Instant::now() + Duration::from_secs(5);
        while let Some(left) = until.checked_duration_since(std::time::Instant::now()) {
            match queue.recv_timeout(left) {
                Ok(call) => {
                    let _ = call.answer.send(Err(anyhow!("aiball's bus is not connected")));
                }
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }
}

type Socket = tungstenite::WebSocket<std::os::unix::net::UnixStream>;

#[cfg(unix)]
fn connect(user: &str) -> anyhow::Result<Socket> {
    use tungstenite::client::IntoClientRequest as _;
    let stream = std::os::unix::net::UnixStream::connect(crate::aiball::socket_path()).context("aiball's socket")?;
    let mut request = "ws://aiball/bus".into_client_request()?;
    request.headers_mut().insert("x-aiball-consumer", user.parse()?);
    let (socket, _) = tungstenite::client(request, stream).map_err(|e| anyhow!("/bus: {e}"))?;
    // Short reads, so that calls waiting to go out are not held up.
    socket.get_ref().set_read_timeout(Some(Duration::from_millis(20)))?;
    Ok(socket)
}

/// Sends the calls queued and reads what comes, until the connection drops.
fn serve(mut socket: Socket, queue: &Receiver<Outgoing>, notices: &UnboundedSender<Notification>, hello: &Mutex<Option<Hello>>) {
    log::info!("aiball bus: connected");
    let mut waiting: HashMap<u64, Sender<anyhow::Result<Value>>> = HashMap::new();
    'connected: loop {
        while let Ok(call) = queue.try_recv() {
            if let Err(error) = socket.send(tungstenite::Message::text(call.frame)) {
                let _ = call.answer.send(Err(anyhow!("aiball's bus: {error}")));
                break 'connected;
            }
            waiting.insert(call.id, call.answer);
        }
        match socket.read() {
            Ok(tungstenite::Message::Text(text)) => {
                for message in messages(&text) {
                    match message {
                        Incoming::Answer { id, result } => {
                            if let Some(answer) = waiting.remove(&id) {
                                let _ = answer.send(result);
                            }
                        }
                        Incoming::Notification(notice) => {
                            if notice.method == "bus.hello" {
                                if let Ok(mut h) = hello.lock() {
                                    *h = Some(Hello::of(&notice.params));
                                }
                            }
                            if notices.unbounded_send(notice).is_err() {
                                break 'connected;
                            }
                        }
                    }
                }
            }
            Ok(tungstenite::Message::Close(frame)) => {
                log::info!("aiball bus: closed by the daemon ({frame:?})");
                break;
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(error) => {
                log::info!("aiball bus: dropped: {error}");
                break;
            }
        }
    }
    for (_, answer) in waiting {
        let _ = answer.send(Err(anyhow!("aiball's bus dropped before answering")));
    }
}

/// What a frame from the daemon holds.
#[derive(Debug)]
enum Incoming {
    Answer { id: u64, result: anyhow::Result<Value> },
    Notification(Notification),
}

/// The messages of one frame: one, or a batch's.
fn messages(text: &str) -> Vec<Incoming> {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        log::warn!("aiball bus: a frame that is not JSON");
        return Vec::new();
    };
    let each = |v: &Value| -> Option<Incoming> {
        if let Some(id) = v.get("id").and_then(Value::as_u64) {
            let result = match v.get("error") {
                Some(error) => {
                    let message = error.get("message").and_then(Value::as_str).unwrap_or("refused");
                    let code = error.pointer("/data/code").and_then(Value::as_str).unwrap_or("");
                    Err(anyhow!("{message} ({code})"))
                }
                None => Ok(v.get("result").cloned().unwrap_or(Value::Null)),
            };
            return Some(Incoming::Answer { id, result });
        }
        let method = v.get("method")?.as_str()?.to_string();
        Some(Incoming::Notification(Notification { method, params: v.get("params").cloned().unwrap_or(Value::Null) }))
    };
    match &value {
        Value::Array(batch) => batch.iter().filter_map(each).collect(),
        single => each(single).into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Hello, Incoming, messages};

    #[test]
    fn answers_errors_and_notifications_are_told_apart() {
        let got = messages(r#"[{"jsonrpc":"2.0","id":2,"result":{"consumer":"david"}},{"jsonrpc":"2.0","id":3,"error":{"code":404,"message":"no such method","data":{"code":"NOT_FOUND"}}}]"#);
        assert!(matches!(&got[0], Incoming::Answer { id: 2, result: Ok(v) } if v["consumer"] == "david"));
        assert!(matches!(&got[1], Incoming::Answer { id: 3, result: Err(e) } if e.to_string().contains("NOT_FOUND")));
        let hello = messages(r#"{"jsonrpc":"2.0","method":"bus.hello","params":{"version":1,"consumer":"david","kind":"human"}}"#);
        match &hello[0] {
            Incoming::Notification(n) => assert_eq!(
                Hello::of(&n.params),
                Hello { version: 1, consumer: "david".into(), kind: "human".into() }
            ),
            other => panic!("{other:?}"),
        }
    }
}
