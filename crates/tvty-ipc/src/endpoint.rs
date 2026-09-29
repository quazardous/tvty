//! An address, and who we are when we get there.

use std::fmt;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;

use crate::Conn;

/// Where to connect, or listen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Endpoint {
    /// A Unix socket (Linux, macOS). Representable anywhere — a session's
    /// `attach.socket` is a path whatever the platform — but connecting to
    /// one elsewhere is `Unsupported`.
    Unix(PathBuf),
    /// TCP; on the loopback unless asked otherwise ([`Endpoint::parse_remote`]).
    Tcp(SocketAddr),
}

/// Why a string is not an [`Endpoint`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndpointError {
    /// Neither a path nor `tcp://`/`http://host:port`.
    Unrecognised(String),
    /// A host that does not resolve, or a port that is not one.
    BadAddress(String),
    /// TCP to another host, where only this machine's loopback is expected.
    NotLoopback(String),
}

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unrecognised(s) => write!(f, "{s:?} is neither a socket path nor tcp://host:port"),
            Self::BadAddress(s) => write!(f, "{s:?}: no such host or port"),
            Self::NotLoopback(s) => write!(f, "{s:?} is not this machine (only the loopback is expected)"),
        }
    }
}

impl std::error::Error for EndpointError {}

impl Endpoint {
    /// `/path`, `unix:/path`, `tcp://host:port` or `http://host:port[/]`,
    /// TCP on the loopback only.
    pub fn parse(s: &str) -> Result<Self, EndpointError> {
        let endpoint = Self::parse_remote(s)?;
        match &endpoint {
            Self::Tcp(addr) if !addr.ip().is_loopback() => Err(EndpointError::NotLoopback(s.to_string())),
            _ => Ok(endpoint),
        }
    }

    /// As [`Endpoint::parse`], TCP to any host.
    pub fn parse_remote(s: &str) -> Result<Self, EndpointError> {
        let s = s.trim();
        if let Some(rest) = s.strip_prefix("tcp://").or_else(|| s.strip_prefix("http://")) {
            let host_port = rest.trim_end_matches('/');
            let addr = host_port
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next())
                .ok_or_else(|| EndpointError::BadAddress(s.to_string()))?;
            return Ok(Self::Tcp(addr));
        }
        if let Some(path) = s.strip_prefix("unix:") {
            return Ok(Self::Unix(PathBuf::from(path)));
        }
        let path = PathBuf::from(s);
        if !s.is_empty() && (path.is_absolute() || s.starts_with('/')) {
            return Ok(Self::Unix(path));
        }
        Err(EndpointError::Unrecognised(s.to_string()))
    }

    /// A connection to it.
    pub fn connect(&self) -> io::Result<Conn> {
        Conn::connect(self)
    }

    /// Whether a connection there needs a secret: anyone on the machine can
    /// reach a TCP port.
    pub fn needs_secret(&self) -> bool {
        matches!(self, Self::Tcp(_))
    }

    /// The `Host` an HTTP request (or a WebSocket upgrade) carries.
    pub fn http_host(&self) -> String {
        match self {
            Self::Tcp(addr) => addr.to_string(),
            // A Unix socket has no host; aiball's clients say "aiball".
            Self::Unix(_) => "aiball".to_string(),
        }
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unix(path) => write!(f, "{}", path.display()),
            Self::Tcp(addr) => write!(f, "tcp://{addr}"),
        }
    }
}

/// Who we are, apart from where we go.
#[derive(Clone, PartialEq, Eq)]
pub enum Credentials {
    /// Nothing to send: the operating system vouches (a Unix socket).
    Peer,
    /// A secret, sent as `Authorization: Bearer <secret>`.
    Bearer(String),
}

impl Credentials {
    /// The header an HTTP request carries, if any.
    pub fn header(&self) -> Option<(&'static str, String)> {
        match self {
            Self::Peer => None,
            Self::Bearer(secret) => Some(("authorization", format!("Bearer {secret}"))),
        }
    }

    /// Whether these are enough for `endpoint`: TCP without a secret is
    /// refused before anything is sent.
    pub fn check(&self, endpoint: &Endpoint) -> io::Result<()> {
        if endpoint.needs_secret() && *self == Self::Peer {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("{endpoint} is TCP: it needs a secret, and there is none"),
            ));
        }
        Ok(())
    }
}

impl fmt::Debug for Credentials {
    /// Never the secret itself: it would end up in a log.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Peer => write!(f, "Peer"),
            Self::Bearer(_) => write!(f, "Bearer(…)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_forms_of_an_address() {
        assert_eq!(Endpoint::parse("/run/a/sock"), Ok(Endpoint::Unix("/run/a/sock".into())));
        assert_eq!(Endpoint::parse("unix:/run/a/sock"), Ok(Endpoint::Unix("/run/a/sock".into())));
        let tcp = Endpoint::Tcp("127.0.0.1:7777".parse().unwrap());
        assert_eq!(Endpoint::parse("tcp://127.0.0.1:7777"), Ok(tcp.clone()));
        assert_eq!(Endpoint::parse("http://127.0.0.1:7777/"), Ok(tcp));
        assert!(matches!(Endpoint::parse("sock"), Err(EndpointError::Unrecognised(_))));
        assert!(matches!(Endpoint::parse("tcp://127.0.0.1:notaport"), Err(EndpointError::BadAddress(_))));
    }

    #[test]
    fn tcp_is_the_loopback_unless_asked() {
        assert!(matches!(Endpoint::parse("tcp://192.0.2.1:7777"), Err(EndpointError::NotLoopback(_))));
        assert!(Endpoint::parse_remote("tcp://192.0.2.1:7777").is_ok());
    }

    #[test]
    fn tcp_without_a_secret_is_refused_before_sending() {
        let tcp = Endpoint::parse("tcp://127.0.0.1:7777").unwrap();
        assert_eq!(Credentials::Peer.check(&tcp).unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        assert!(Credentials::Bearer("s".into()).check(&tcp).is_ok());
        assert!(Credentials::Peer.check(&Endpoint::Unix("/s".into())).is_ok());
    }

    #[test]
    fn a_secret_is_never_printed() {
        let secret = Credentials::Bearer("aiball-0123".into());
        assert!(!format!("{secret:?}").contains("0123"));
        assert_eq!(secret.header(), Some(("authorization", "Bearer aiball-0123".to_string())));
    }
}
