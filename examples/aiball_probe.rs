//! Where tvty finds aiball, and whether aiball knows who it is: read only.
//!
//!     cargo run --example aiball_probe
//!
//! Locates aiball as tvty does (docs/IPC.md), asks its version (a public
//! route: the address is right), then `bus.whoami` on its bus (the
//! credentials are right, and say whom tvty acts as). Changes nothing.

use std::io::{Read, Write};
use std::time::Duration;

use tungstenite::client::IntoClientRequest as _;

fn main() -> anyhow::Result<()> {
    let at = tvty_ipc::aiball::locate().map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("aiball at {} with {:?}", at.endpoint, at.credentials);

    let mut conn = at.endpoint.connect()?;
    conn.set_read_timeout(Some(Duration::from_secs(5)))?;
    write!(conn, "GET /api/version HTTP/1.0\r\nHost: {}\r\n\r\n", at.endpoint.http_host())?;
    let mut answer = String::new();
    conn.read_to_string(&mut answer)?;
    println!("version: {}", answer.split("\r\n\r\n").nth(1).unwrap_or(&answer).trim());

    at.credentials.check(&at.endpoint)?;
    let stream = at.endpoint.connect()?;
    let mut request = format!("ws://{}/bus", at.endpoint.http_host()).into_client_request()?;
    if let Some((name, value)) = at.credentials.header() {
        request.headers_mut().insert(name, value.parse()?);
    }
    let (mut bus, _) = tungstenite::client(request, stream).map_err(|e| anyhow::anyhow!("/bus: {e}"))?;
    bus.get_ref().set_read_timeout(Some(Duration::from_secs(5)))?;
    bus.send(tungstenite::Message::text(r#"{"jsonrpc":"2.0","id":1,"method":"bus.whoami","params":{}}"#))?;
    loop {
        let message = bus.read()?;
        let text = message.to_text()?;
        if text.contains("\"id\":1") {
            println!("whoami: {text}");
            break;
        }
    }
    let _ = bus.close(None);
    Ok(())
}
