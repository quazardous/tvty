//! The activity service: what happens — a gesture of the user in any view,
//! a ping from aiball — is published on the bus ([`Signal::Activity`]), and
//! this one place turns it into a notification of the right colour, whose
//! ticket then shines in the lists ([`crate::notify::halo`]). Views never
//! push notifications about activity themselves.

use gpui_kit::*;

use crate::bus::{self, Signal};
use crate::notify::{self, Kind, Notice};

/// Something that happened.
#[derive(Clone, Debug)]
pub struct Activity {
    /// Who: "you" for the user's own gestures, else an agent, or tvty.
    pub by: String,
    /// One of the user's own gestures: shown only while [`Prefs::own`].
    pub own: bool,
    pub kind: Kind,
    /// The ticket it is about, and its project.
    pub about: Option<(String, u64)>,
    /// The terminal it concerns, when one runs here.
    pub session: Option<String>,
    pub text: String,
    /// What was written, quoted under the text.
    pub detail: Option<String>,
}

impl Activity {
    /// A gesture of the user that went through: "closed", "plan accepted"…
    pub fn done(about: Option<(String, u64)>, what: impl Into<String>) -> Self {
        Self { by: "you".into(), own: true, kind: Kind::Done, about, session: None, text: what.into(), detail: None }
    }

    /// A gesture of the user that aiball refused: always shown.
    pub fn failed(about: Option<(String, u64)>, what: &str, error: impl std::fmt::Display) -> Self {
        let text = if what.is_empty() { error.to_string() } else { format!("{what}: {error}") };
        Self { by: "you".into(), own: false, kind: Kind::Error, about, session: None, text, detail: None }
    }

    /// Something an agent or aiball did.
    pub fn news(by: impl Into<String>, kind: Kind, about: Option<(String, u64)>, text: impl Into<String>) -> Self {
        Self { by: by.into(), own: false, kind, about, session: None, text: text.into(), detail: None }
    }

    /// Quoting what was written.
    pub fn quoting(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into()).filter(|d| !d.is_empty());
        self
    }

    /// On this terminal.
    pub fn on(mut self, session: Option<String>) -> Self {
        self.session = session;
        self
    }
}

/// What the user wants to hear of.
struct Prefs {
    /// Their own gestures, once through.
    own: bool,
}

impl Global for Prefs {}

/// Listens to the bus for activity, from start to end.
pub fn init(cx: &mut App, own: bool) {
    cx.set_global(Prefs { own });
    cx.subscribe(&bus::bus(cx), |_, signal: &Signal, cx| {
        if let Signal::Activity(activity) = signal {
            report(cx, activity.clone());
        }
    })
    .detach();
}

/// Whether the user's own gestures are notified.
pub fn set_own(cx: &mut App, own: bool) {
    cx.global_mut::<Prefs>().own = own;
}

/// Publishes `activity` on the bus.
pub fn publish(cx: &mut App, activity: Activity) {
    bus::emit(cx, Signal::Activity(activity));
}

fn report(cx: &mut App, activity: Activity) {
    if activity.own && !cx.global::<Prefs>().own {
        return;
    }
    let mut notice = Notice::new(activity.kind, activity.by, activity.text);
    if let Some((project, ticket)) = activity.about {
        notice = notice.about(project, ticket);
    }
    if let Some(session) = activity.session {
        notice = notice.on(session);
    }
    notice.detail = activity.detail;
    notify::push(cx, notice);
}
