//! The other side of a [`Conn`].

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io;
use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Conn, Endpoint};

/// A listening endpoint (see `docs/IPC.md`, "Listening").
#[derive(Debug)]
pub struct Listener {
    inner: Inner,
    local: Endpoint,
}

#[derive(Debug)]
enum Inner {
    Tcp(TcpListener),
    #[cfg(unix)]
    Unix(std::os::unix::net::UnixListener),
}

impl Listener {
    /// Listens on `endpoint`: a Unix socket made `0600`, or a TCP port —
    /// `tcp://127.0.0.1:0` takes a free one, which [`Listener::local`] says.
    /// A socket file already there is the caller's to remove.
    pub fn bind(endpoint: &Endpoint) -> io::Result<Self> {
        match endpoint {
            Endpoint::Tcp(addr) => {
                let listener = TcpListener::bind(addr)?;
                let local = Endpoint::Tcp(listener.local_addr()?);
                Ok(Self { inner: Inner::Tcp(listener), local })
            }
            #[cfg(unix)]
            Endpoint::Unix(path) => {
                use std::os::unix::fs::PermissionsExt;
                let listener = std::os::unix::net::UnixListener::bind(path)?;
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
                Ok(Self { inner: Inner::Unix(listener), local: endpoint.clone() })
            }
            #[cfg(not(unix))]
            Endpoint::Unix(path) => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("{}: no Unix sockets on this system", path.display()),
            )),
        }
    }

    /// Where it listens (the port taken, for `:0`).
    pub fn local(&self) -> &Endpoint {
        &self.local
    }

    /// The next connection.
    pub fn accept(&self) -> io::Result<Conn> {
        match &self.inner {
            Inner::Tcp(l) => Conn::from_tcp(l.accept()?.0),
            #[cfg(unix)]
            Inner::Unix(l) => Ok(Conn::from_unix(l.accept()?.0)),
        }
    }
}

/// A secret to hand to a client out of band (a file only the user reads):
/// 128 bits from the operating system's randomness, as hex. std seeds each
/// `RandomState` from it, so two of them make a secret without a dependency.
pub fn fresh_secret() -> String {
    let mut words = [0u64; 2];
    for (i, word) in words.iter_mut().enumerate() {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_usize(i);
        hasher.write_u128(SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
        *word = hasher.finish();
    }
    format!("{:016x}{:016x}", words[0], words[1])
}

#[cfg(test)]
mod tests {
    use super::fresh_secret;

    #[test]
    fn secrets_are_long_and_never_twice() {
        let (a, b) = (fresh_secret(), fresh_secret());
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
    }
}
