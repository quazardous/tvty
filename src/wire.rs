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

use anyhow::anyhow;
use futures::channel::mpsc::UnboundedSender;
use serde_json::{Value, json};

/// How long a call waits for its answer.
const CALL_TIMEOUT: Duration = Duration::from_secs(10);

/// The method of the notice the wire gives itself when a connection that
/// ran is gone: `{"why": …}`.
pub const DROPPED: &str = "tvty.dropped";

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

/// What goes to the connection's thread.
enum Outgoing {
    /// A call: its frame, and where its answer goes.
    Call { frame: String, id: u64, answer: Sender<anyhow::Result<Value>> },
    /// Connect again, as the user now set.
    Reconnect,
}

/// The bus, as the rest of tvty holds it: cheap to clone.
#[derive(Clone)]
pub struct Wire {
    outgoing: Sender<Outgoing>,
    next_id: Arc<Mutex<u64>>,
    hello: Arc<Mutex<Option<Hello>>>,
    /// Who the connection runs as (the local socket trusts the header).
    user: Arc<Mutex<String>>,
}

impl Wire {
    /// Connects as `user` (the local socket trusts the header), and keeps
    /// the connection up. The daemon's notifications go to `notices`.
    pub fn start(user: String, notices: UnboundedSender<Notification>) -> Self {
        let (outgoing, queue) = mpsc::channel::<Outgoing>();
        let hello = Arc::new(Mutex::new(None));
        let user = Arc::new(Mutex::new(user));
        let wire = Self { outgoing, next_id: Arc::new(Mutex::new(1)), hello: hello.clone(), user: user.clone() };
        std::thread::Builder::new()
            .name("aiball-bus".into())
            .spawn(move || run(user, queue, notices, hello))
            .expect("failed to start the bus thread");
        wire
    }

    /// Runs as `user` from now on: connects again when it changed. The
    /// connection opens before tvty knows who the user is (it reads the
    /// consumers to find out), then runs as them — writes need it.
    pub fn set_user(&self, user: &str) {
        let Ok(mut current) = self.user.lock() else { return };
        if *current != user {
            *current = user.to_string();
            let _ = self.outgoing.send(Outgoing::Reconnect);
        }
    }

    /// Drops the connection and connects again, as a drop would: for a test
    /// (the debug control's `bus-reconnect`).
    pub fn reconnect(&self) {
        let _ = self.outgoing.send(Outgoing::Reconnect);
    }

    /// Who the connection runs as, once the daemon greeted it.
    pub fn hello(&self) -> Option<Hello> {
        self.hello.lock().ok().and_then(|h| h.clone())
    }

    /// Calls `method` and waits for its answer. Blocking: off the UI thread.
    pub fn call(&self, method: &str, params: Value) -> anyhow::Result<Value> {
        self.call_waiting(method, params, CALL_TIMEOUT)
    }

    /// A call on a connection of its own, as the same user, closed once
    /// answered: the bus answers a connection's calls in turn, and a method
    /// the daemon takes long to answer (it waits for a process to end) would
    /// hold up every other call behind it. Blocking: off the UI thread.
    pub fn call_alone(&self, method: &str, params: Value, timeout: Duration) -> anyhow::Result<Value> {
        let user = self.user.lock().map(|u| u.clone()).map_err(|_| anyhow!("the bus is gone"))?;
        let mut socket = connect(&user)?;
        let frame = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
        socket.send(tungstenite::Message::text(frame)).map_err(|e| anyhow!("{method}: {e}"))?;
        let until = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < until {
            match socket.read() {
                Ok(tungstenite::Message::Text(text)) => {
                    for message in messages(&text) {
                        if let Incoming::Answer { id: 1, result } = message {
                            let _ = socket.close(None);
                            return result.map_err(|e| anyhow!("{method}: {e:#}"));
                        }
                    }
                }
                Ok(tungstenite::Message::Close(_)) => return Err(anyhow!("{method}: aiball's bus closed")),
                Ok(_) => {}
                Err(tungstenite::Error::Io(e)) if tvty_ipc::Conn::is_timeout(&e) => {
                    // The daemon's pings get their pongs.
                    let _ = socket.flush();
                }
                Err(error) => return Err(anyhow!("{method}: {error}")),
            }
        }
        let _ = socket.close(None);
        Err(anyhow!("{method}: no answer from aiball's bus"))
    }

    /// [`Self::call`], for a method the daemon may take longer to answer
    /// (it waits for a process to end): `timeout` at most.
    pub fn call_waiting(&self, method: &str, params: Value, timeout: Duration) -> anyhow::Result<Value> {
        let id = {
            let mut next = self.next_id.lock().map_err(|_| anyhow!("the bus is gone"))?;
            *next += 1;
            *next
        };
        let frame = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }).to_string();
        let (answer, answered) = mpsc::channel();
        self.outgoing.send(Outgoing::Call { frame, id, answer }).map_err(|_| anyhow!("the bus is gone"))?;
        match answered.recv_timeout(timeout) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => Err(anyhow!("{method}: no answer from aiball's bus")),
            Err(RecvTimeoutError::Disconnected) => Err(anyhow!("{method}: aiball's bus dropped")),
        }
    }
}

/// The connection's thread: connect, serve, reconnect, until tvty is gone.
fn run(user: Arc<Mutex<String>>, queue: Receiver<Outgoing>, notices: UnboundedSender<Notification>, hello: Arc<Mutex<Option<Hello>>>) {
    loop {
        let as_user = user.lock().map(|u| u.clone()).unwrap_or_default();
        let asked_again = match connect(&as_user) {
            Ok(socket) => {
                let (asked_again, why) = serve(socket, &queue, &notices, &hello);
                // A connection that ran, gone: said once (a daemon that stays
                // down is not said again at each try).
                let _ = notices.unbounded_send(Notification { method: DROPPED.into(), params: serde_json::json!({ "why": why }) });
                asked_again
            }
            Err(error) => {
                log::debug!("aiball bus: {error:#}");
                false
            }
        };
        if let Ok(mut h) = hello.lock() {
            *h = None;
        }
        if notices.is_closed() {
            return;
        }
        // A new user: connect again at once.
        if asked_again {
            continue;
        }
        // While down, calls fail at once rather than wait.
        let until = std::time::Instant::now() + Duration::from_secs(5);
        while let Some(left) = until.checked_duration_since(std::time::Instant::now()) {
            match queue.recv_timeout(left) {
                Ok(Outgoing::Call { answer, .. }) => {
                    let _ = answer.send(Err(anyhow!("aiball's bus is not connected")));
                }
                Ok(Outgoing::Reconnect) => break,
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }
}

type Socket = tungstenite::WebSocket<tvty_ipc::Conn>;

/// The bus, where aiball is (docs/IPC.md), as `user`: over its socket the
/// header says who; over TCP the human's token does, and aiball honours
/// the header for it.
fn connect(user: &str) -> anyhow::Result<Socket> {
    use anyhow::Context as _;
    use tungstenite::client::IntoClientRequest as _;
    let at = crate::aiball::location().map_err(|e| anyhow!("{e}"))?;
    at.credentials.check(&at.endpoint)?;
    let stream = at.endpoint.connect().with_context(|| format!("aiball at {}", at.endpoint))?;
    let mut request = format!("ws://{}/bus", at.endpoint.http_host()).into_client_request()?;
    let headers = request.headers_mut();
    if let Some((name, value)) = at.credentials.header() {
        headers.insert(name, value.parse()?);
    }
    headers.insert("x-aiball-consumer", user.parse()?);
    // What the client runs on: aiball tags the tickets filed with it.
    headers.insert("x-aiball-platform", std::env::consts::OS.parse()?);
    let (socket, _) = tungstenite::client(request, stream).map_err(|e| anyhow!("/bus: {e}"))?;
    // Short reads, so that calls waiting to go out are not held up.
    socket.get_ref().set_read_timeout(Some(Duration::from_millis(20)))?;
    Ok(socket)
}

/// Sends the calls queued and reads what comes, until the connection drops
/// (`false`) or another user is asked for (`true`).
fn serve(mut socket: Socket, queue: &Receiver<Outgoing>, notices: &UnboundedSender<Notification>, hello: &Mutex<Option<Hello>>) -> (bool, String) {
    log::info!("aiball bus: connected");
    let mut waiting: HashMap<u64, Sender<anyhow::Result<Value>>> = HashMap::new();
    let mut asked_again = false;
    let mut why = String::from("tvty is closing");
    'connected: loop {
        while let Ok(outgoing) = queue.try_recv() {
            match outgoing {
                Outgoing::Call { frame, id, answer } => {
                    if let Err(error) = socket.send(tungstenite::Message::text(frame)) {
                        let _ = answer.send(Err(anyhow!("aiball's bus: {error}")));
                        why = format!("{error}");
                        break 'connected;
                    }
                    waiting.insert(id, answer);
                }
                Outgoing::Reconnect => {
                    asked_again = true;
                    why = "connected again on purpose".into();
                    let _ = socket.close(None);
                    break 'connected;
                }
            }
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
                why = "closed by the daemon".into();
                break;
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(e)) if tvty_ipc::Conn::is_timeout(&e) => {}
            Err(error) => {
                log::info!("aiball bus: dropped: {error}");
                why = format!("{error}");
                break;
            }
        }
    }
    for (_, answer) in waiting {
        let _ = answer.send(Err(anyhow!("aiball's bus dropped before answering")));
    }
    (asked_again, why)
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
