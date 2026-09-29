//! How tvty talks to other processes — the contract is `docs/IPC.md`.
//!
//! - [`Endpoint`]: an address, a Unix socket or TCP on the loopback;
//! - [`Credentials`]: who we are, apart from where we go — the operating
//!   system's word, or a secret;
//! - [`Conn`]: a connection with the same guarantees on every transport
//!   (read timeout, a second handle for a writer thread, a shutdown that
//!   wakes a blocked reader);
//! - [`Listener`]: the other side;
//! - [`aiball`]: where aiball is, found as aiball's own clients find it.
//!
//! Nothing else in tvty knows which transport it is on.

pub mod aiball;
mod conn;
mod endpoint;
mod listener;

pub use conn::Conn;
pub use endpoint::{Credentials, Endpoint, EndpointError};
pub use listener::{Listener, fresh_secret};
