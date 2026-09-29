//! tvty's side of aiball's bus, kept in one place as it grows: the
//! subscriptions and where each resumes from, and the signals of the link
//! (connected, dropped, a subscription failed…) the views observe. What aiball pushes is kept by
//! [`crate::live`]; the transport is [`crate::wire`].

pub mod signals;
pub mod subscriptions;
