//! The contract of `docs/IPC.md`, held against every transport this
//! platform has: TCP everywhere, a Unix socket on Unix. A transport tvty
//! uses passes all of it.

use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use tvty_ipc::{Conn, Credentials, Endpoint, Listener};

/// A fresh place to listen, per transport.
fn places() -> Vec<(&'static str, Endpoint)> {
    let tcp = ("tcp", Endpoint::parse("tcp://127.0.0.1:0").unwrap());
    #[cfg(unix)]
    {
        let dir = std::env::temp_dir().join(format!("tvty-ipc-{}-{}", std::process::id(), tvty_ipc::fresh_secret()));
        std::fs::create_dir_all(&dir).unwrap();
        vec![tcp, ("unix", Endpoint::Unix(dir.join("sock")))]
    }
    #[cfg(not(unix))]
    vec![tcp]
}

/// A listener and both ends of one connection to it.
fn pair(place: &Endpoint) -> (Listener, Conn, Conn) {
    let listener = Listener::bind(place).unwrap();
    let client = listener.local().connect().unwrap();
    let server = listener.accept().unwrap();
    (listener, client, server)
}

fn each(test: impl Fn(&str, &Endpoint)) {
    for (name, place) in places() {
        test(name, &place);
    }
}

#[test]
fn bytes_go_both_ways() {
    each(|name, place| {
        let (_l, mut client, mut server) = pair(place);
        client.write_all(b"ping").unwrap();
        let mut got = [0u8; 4];
        server.read_exact(&mut got).unwrap();
        assert_eq!(&got, b"ping", "{name}");
        server.write_all(b"pong").unwrap();
        client.read_exact(&mut got).unwrap();
        assert_eq!(&got, b"pong", "{name}");
    });
}

#[test]
fn a_read_that_waits_too_long_says_so() {
    each(|name, place| {
        let (_l, mut client, _server) = pair(place);
        client.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
        let started = Instant::now();
        let error = client.read(&mut [0u8; 1]).unwrap_err();
        assert!(Conn::is_timeout(&error), "{name}: {error:?}");
        assert!(started.elapsed() < Duration::from_secs(2), "{name}: waited {:?}", started.elapsed());
    });
}

#[test]
fn one_thread_reads_while_another_writes() {
    each(|name, place| {
        let (_l, client, mut server) = pair(place);
        let mut reader = client.try_clone().unwrap();
        let mut writer = client;
        // The reader waits first, blocked; the writer must still get through.
        let (tx, rx) = mpsc::channel();
        let waiting = thread::spawn(move || {
            let mut got = [0u8; 5];
            reader.read_exact(&mut got).unwrap();
            tx.send(got).unwrap();
        });
        thread::sleep(Duration::from_millis(50));
        writer.write_all(b"hello").unwrap();
        let mut echoed = [0u8; 5];
        server.read_exact(&mut echoed).unwrap();
        server.write_all(&echoed).unwrap();
        assert_eq!(&rx.recv_timeout(Duration::from_secs(5)).unwrap(), b"hello", "{name}");
        waiting.join().unwrap();
    });
}

#[test]
fn shutdown_wakes_a_blocked_reader_and_twice_is_harmless() {
    each(|name, place| {
        let (_l, client, _server) = pair(place);
        let mut reader = client.try_clone().unwrap();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            // A closed connection reads 0 bytes or fails: either wakes it.
            let _ = reader.read(&mut [0u8; 1]);
            tx.send(()).unwrap();
        });
        thread::sleep(Duration::from_millis(50));
        client.shutdown();
        client.shutdown();
        assert!(rx.recv_timeout(Duration::from_secs(5)).is_ok(), "{name}: the reader stayed blocked");
    });
}

#[test]
fn a_listener_says_where_it_listens_and_takes_several() {
    each(|name, place| {
        let listener = Listener::bind(place).unwrap();
        if let Endpoint::Tcp(addr) = listener.local() {
            assert_ne!(addr.port(), 0, "{name}: :0 must become a real port");
        }
        for _ in 0..3 {
            let _client = listener.local().connect().unwrap();
            listener.accept().unwrap();
        }
    });
}

#[test]
fn nothing_listening_is_an_error_not_a_hang() {
    // A port just freed: nothing listens there any more.
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let started = Instant::now();
    assert!(Endpoint::parse(&format!("tcp://127.0.0.1:{port}")).unwrap().connect().is_err());
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn tcp_is_refused_without_a_secret() {
    let (_l, ..) = pair(&Endpoint::parse("tcp://127.0.0.1:0").unwrap());
    let tcp = Endpoint::parse("tcp://127.0.0.1:1").unwrap();
    assert!(Credentials::Peer.check(&tcp).is_err());
    assert!(Credentials::Bearer(tvty_ipc::fresh_secret()).check(&tcp).is_ok());
}

#[cfg(not(unix))]
#[test]
fn a_unix_socket_elsewhere_is_unsupported_not_a_crash() {
    let error = Endpoint::Unix("C:\\x\\attach.sock".into()).connect().unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
}
