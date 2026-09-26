//! The internal bus: one global entity the views publish typed signals on
//! and listen to, so that a view need not know which other one cares — the
//! ticket list does not know the shell opens its tickets, a notification
//! does not know which lists light the ticket up.
//!
//! Publish with [`emit`]; listen with `cx.subscribe(&bus::bus(cx), …)`.

use gpui_kit::*;

use crate::notify::Notice;

#[derive(Clone, Debug)]
pub enum Signal {
    /// The notifications changed: shown, gone, hovered. The lists light
    /// their tickets up again ([`crate::notify::lit`]).
    Notices,
    /// A notification was clicked: go to what it is about.
    OpenNotice(Notice),
    /// Open a ticket of a project full screen (from the full list).
    OpenTicket { project: String, ticket: u64 },
    /// A new ticket: on this project (else the one shown), under this parent.
    AskNewTicket { project: Option<String>, parent: Option<u64> },
    /// Something changed on the board: read it again.
    BoardChanged,
    /// Show a thread's images in the viewer, from this one.
    OpenPictures { pictures: Vec<crate::images::Picture>, index: usize },
    /// Something happened: the activity service makes it a notification.
    Activity(crate::activity::Activity),
}

pub struct Bus;

impl EventEmitter<Signal> for Bus {}

struct GlobalBus(Entity<Bus>);

impl Global for GlobalBus {}

pub fn init(cx: &mut App) {
    let bus = cx.new(|_| Bus);
    cx.set_global(GlobalBus(bus));
}

pub fn bus(cx: &App) -> Entity<Bus> {
    cx.global::<GlobalBus>().0.clone()
}

/// Publishes `signal` to every listener.
pub fn emit(cx: &mut App, signal: Signal) {
    bus(cx).update(cx, |_, cx| cx.emit(signal));
}
