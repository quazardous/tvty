//! A connection: the same guarantees whatever carries it.

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use crate::Endpoint;

/// How long a TCP connection to a local port may take before it is given
/// up: the loopback answers at once or not at all.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// A connection to an [`Endpoint`] (see `docs/IPC.md`, "A connection").
#[derive(Debug)]
pub struct Conn(Inner);

#[derive(Debug)]
enum Inner {
    /// Shared, not duplicated, between the handles of one connection: on
    /// Windows a read blocked on one handle is woken only through that very
    /// handle (see `shutdown`).
    Tcp(Arc<TcpStream>),
    #[cfg(unix)]
    Unix(std::os::unix::net::UnixStream),
}

#[cfg(windows)]
unsafe extern "system" {
    /// kernel32: cancels the I/O pending on a handle, from any thread.
    fn CancelIoEx(handle: *mut std::ffi::c_void, overlapped: *mut std::ffi::c_void) -> i32;
}

impl Conn {
    pub(crate) fn connect(endpoint: &Endpoint) -> io::Result<Self> {
        match endpoint {
            Endpoint::Tcp(addr) => {
                let stream = TcpStream::connect_timeout(addr, CONNECT_TIMEOUT)?;
                // Small frames, answered at once: no Nagle delay.
                stream.set_nodelay(true)?;
                Ok(Self(Inner::Tcp(Arc::new(stream))))
            }
            #[cfg(unix)]
            Endpoint::Unix(path) => Ok(Self(Inner::Unix(std::os::unix::net::UnixStream::connect(path)?))),
            #[cfg(not(unix))]
            Endpoint::Unix(path) => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("{}: no Unix sockets on this system", path.display()),
            )),
        }
    }

    pub(crate) fn from_tcp(stream: TcpStream) -> io::Result<Self> {
        stream.set_nodelay(true)?;
        Ok(Self(Inner::Tcp(Arc::new(stream))))
    }

    #[cfg(unix)]
    pub(crate) fn from_unix(stream: std::os::unix::net::UnixStream) -> Self {
        Self(Inner::Unix(stream))
    }

    /// A read that waits longer fails with `WouldBlock` or `TimedOut` (the
    /// transports differ: treat both alike); `None` waits for ever.
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        match &self.0 {
            Inner::Tcp(s) => s.set_read_timeout(timeout),
            #[cfg(unix)]
            Inner::Unix(s) => s.set_read_timeout(timeout),
        }
    }

    /// A write that waits longer fails the same way.
    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        match &self.0 {
            Inner::Tcp(s) => s.set_write_timeout(timeout),
            #[cfg(unix)]
            Inner::Unix(s) => s.set_write_timeout(timeout),
        }
    }

    /// A second handle on the same connection: one thread reads while
    /// another writes.
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self(match &self.0 {
            Inner::Tcp(s) => Inner::Tcp(Arc::clone(s)),
            #[cfg(unix)]
            Inner::Unix(s) => Inner::Unix(s.try_clone()?),
        }))
    }

    /// Ends the connection both ways, waking a thread blocked in a read on
    /// any handle of it. Calling it again does nothing.
    pub fn shutdown(&self) {
        let _ = match &self.0 {
            Inner::Tcp(s) => {
                let done = s.shutdown(Shutdown::Both);
                // Windows does not wake a read blocked on the socket when it
                // is shut down (Linux does): cancel it.
                #[cfg(windows)]
                {
                    use std::os::windows::io::AsRawSocket;
                    // SAFETY: a live socket handle, and no OVERLAPPED:
                    // every pending operation on it is cancelled.
                    unsafe { CancelIoEx(s.as_raw_socket() as *mut std::ffi::c_void, std::ptr::null_mut()) };
                }
                done
            }
            #[cfg(unix)]
            Inner::Unix(s) => s.shutdown(Shutdown::Both),
        };
    }

    /// Whether a read error only means the timeout passed.
    pub fn is_timeout(error: &io::Error) -> bool {
        matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut)
    }
}

impl Read for Conn {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.0 {
            Inner::Tcp(s) => (&**s).read(buf),
            #[cfg(unix)]
            Inner::Unix(s) => s.read(buf),
        }
    }
}

impl Write for Conn {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.0 {
            Inner::Tcp(s) => (&**s).write(buf),
            #[cfg(unix)]
            Inner::Unix(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.0 {
            Inner::Tcp(s) => (&**s).flush(),
            #[cfg(unix)]
            Inner::Unix(s) => s.flush(),
        }
    }
}
