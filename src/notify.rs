//! Notifications: in the window's top left corner, above everything, a
//! little translucent; stacked top down, the newest first, at most N; each
//! gone after a few seconds, unless the pointer is on it. While one is up,
//! its ticket shines in every list shown ([`lit`]).
//!
//! Anything posts one ([`push`]); a click on it goes where it points,
//! through the bus ([`crate::bus::Signal::OpenNotice`]).

use std::time::{Duration, Instant};

use gpui_kit::*;

use crate::bus::{self, Signal};
use crate::theme::p;

pub const MAX_DEFAULT: usize = 5;
pub const SECONDS_DEFAULT: u64 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A decision waits on the user.
    Decision,
    /// Something new on a ticket.
    News,
    /// tvty says something.
    Info,
    /// Something failed.
    Error,
}

#[derive(Clone, Debug)]
pub struct Notice {
    pub id: u64,
    pub kind: Kind,
    /// Who or what: an agent, "tvty".
    pub from: String,
    pub text: String,
    /// The ticket it is about (its project, its id), if any.
    pub ticket: Option<(String, u64)>,
    /// The terminal it is about, if any.
    pub session: Option<String>,
    born: Instant,
    hovered: bool,
}

impl Notice {
    pub fn new(kind: Kind, from: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: 0,
            kind,
            from: from.into(),
            text: text.into(),
            ticket: None,
            session: None,
            born: Instant::now(),
            hovered: false,
        }
    }

    pub fn about(mut self, project: impl Into<String>, ticket: u64) -> Self {
        self.ticket = Some((project.into(), ticket));
        self
    }

    pub fn on(mut self, session: impl Into<String>) -> Self {
        self.session = Some(session.into());
        self
    }
}

/// What is shown, the newest first, and the limits.
struct Notices {
    shown: Vec<Notice>,
    next: u64,
    max: usize,
    ttl: Duration,
}

impl Global for Notices {}

impl Notices {
    /// Adds `notice` on top; one about the same ticket and kind is replaced.
    /// Answers whether the list changed.
    fn push(&mut self, mut notice: Notice) {
        self.next += 1;
        notice.id = self.next;
        notice.born = Instant::now();
        self.shown
            .retain(|n| !(n.ticket.is_some() && n.ticket == notice.ticket && n.kind == notice.kind));
        self.shown.insert(0, notice);
        self.shown.truncate(self.max);
    }

    /// Drops what lived its time (not while hovered). Answers whether any.
    fn expire(&mut self, now: Instant) -> bool {
        let ttl = self.ttl;
        let before = self.shown.len();
        self.shown.retain(|n| n.hovered || now.duration_since(n.born) < ttl);
        self.shown.len() != before
    }
}

pub fn init(cx: &mut App, max: usize, seconds: u64) {
    cx.set_global(Notices {
        shown: Vec::new(),
        next: 0,
        max: max.max(1),
        ttl: Duration::from_secs(seconds.max(1)),
    });
}

/// Shows `notice`; it goes by itself after its time.
pub fn push(cx: &mut App, notice: Notice) {
    let first = cx.global::<Notices>().shown.is_empty();
    cx.global_mut::<Notices>().push(notice);
    if first {
        tick(cx);
    }
    bus::emit(cx, Signal::Notices);
}

pub fn dismiss(cx: &mut App, id: u64) {
    cx.global_mut::<Notices>().shown.retain(|n| n.id != id);
    bus::emit(cx, Signal::Notices);
}

/// Drops what is about `session`: the user is there now.
pub fn dismiss_session(cx: &mut App, session: &str) {
    let notices = cx.global_mut::<Notices>();
    let before = notices.shown.len();
    notices.shown.retain(|n| n.session.as_deref() != Some(session));
    if notices.shown.len() != before {
        bus::emit(cx, Signal::Notices);
    }
}

/// The newest one shown, if any.
pub fn newest(cx: &App) -> Option<Notice> {
    cx.global::<Notices>().shown.first().cloned()
}

/// Whether a notification about `ticket` is up: the lists shine it.
pub fn lit(cx: &App, ticket: u64) -> bool {
    cx.global::<Notices>().shown.iter().any(|n| n.ticket.as_ref().is_some_and(|t| t.1 == ticket))
}

pub fn set_limits(cx: &mut App, max: usize, seconds: u64) {
    let notices = cx.global_mut::<Notices>();
    notices.max = max.max(1);
    notices.ttl = Duration::from_secs(seconds.max(1));
    notices.shown.truncate(notices.max);
    bus::emit(cx, Signal::Notices);
}

pub fn limits(cx: &App) -> (usize, u64) {
    let n = cx.global::<Notices>();
    (n.max, n.ttl.as_secs())
}

fn hover(cx: &mut App, id: u64, hovered: bool) {
    if let Some(n) = cx.global_mut::<Notices>().shown.iter_mut().find(|n| n.id == id) {
        n.hovered = hovered;
        // Its time starts again when the pointer leaves it.
        if !hovered {
            n.born = Instant::now();
        }
    }
}

/// While any is shown: drops the expired, four times a second.
fn tick(cx: &mut App) {
    cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(Duration::from_millis(250)).await;
            let alive = cx.update(|cx| {
                if cx.global_mut::<Notices>().expire(Instant::now()) {
                    bus::emit(cx, Signal::Notices);
                }
                !cx.global::<Notices>().shown.is_empty()
            });
            if !alive {
                break;
            }
        }
    })
    .detach();
}

fn colour(kind: Kind) -> Hsla {
    match kind {
        Kind::Decision => p().warning,
        Kind::News => p().accent,
        Kind::Info => p().info,
        Kind::Error => p().danger,
    }
}

/// The stack, for the window's top left corner, above everything.
pub fn stack(cx: &App) -> Option<AnyElement> {
    let shown = &cx.global::<Notices>().shown;
    if shown.is_empty() {
        return None;
    }
    let mut column = div()
        .id("notices")
        .absolute()
        .top(px(48.))
        .left(px(12.))
        .w(px(380.))
        .flex()
        .flex_col()
        .gap_2();
    for notice in shown {
        let (id, colour) = (notice.id, colour(notice.kind));
        let open = notice.clone();
        let what = match &notice.ticket {
            Some((_, ticket)) => format!("#{ticket} {}", notice.text),
            None => notice.text.clone(),
        };
        column = column.child(
            div()
                .id(SharedString::from(format!("notice-{id}")))
                .occlude()
                .flex()
                .items_start()
                .gap_2()
                .px_3()
                .py_2()
                .rounded_md()
                .bg(p().surface.opacity(0.88))
                .border_1()
                .border_color(colour.opacity(0.8))
                .shadow_lg()
                .text_sm()
                .text_color(p().text)
                .cursor_pointer()
                .on_hover(move |hovered, _, cx| hover(cx, id, *hovered))
                .on_click(move |_, _, cx| {
                    dismiss(cx, id);
                    bus::emit(cx, Signal::OpenNotice(open.clone()));
                })
                .child(div().mt(px(6.)).size(px(8.)).flex_none().rounded_full().bg(colour))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(div().text_xs().text_color(p().muted).child(notice.from.clone()))
                        .child(div().child(what)),
                )
                .child(
                    div()
                        .id(SharedString::from(format!("notice-close-{id}")))
                        .px_1()
                        .text_color(p().muted)
                        .hover(|d| d.text_color(p().text))
                        .child("×")
                        .on_click(move |_, _, cx| {
                            cx.stop_propagation();
                            dismiss(cx, id);
                        }),
                )
                .with_animation(
                    SharedString::from(format!("notice-in-{id}")),
                    Animation::new(Duration::from_millis(200)).with_easing(|t| 1. - (1. - t).powi(3)),
                    |d, t| d.opacity(t).ml(px(-24. * (1. - t))),
                ),
        );
    }
    Some(column.into_any_element())
}

/// The glow of a ticket a notification is about, on a list's row.
pub fn halo<E: Styled>(row: E, lit: bool) -> E {
    if !lit {
        return row;
    }
    // A thin outline and a soft glow inside it: the row stays readable.
    row.bg(p().accent.opacity(0.05)).shadow(vec![BoxShadow {
        color: p().accent.opacity(0.7),
        offset: point(px(0.), px(0.)),
        blur_radius: px(6.),
        spread_radius: px(-1.),
        inset: true,
    }])
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{Kind, Notice, Notices};

    fn notices(max: usize) -> Notices {
        Notices { shown: Vec::new(), next: 0, max, ttl: Duration::from_secs(6) }
    }

    #[test]
    fn the_newest_on_top_and_at_most_max() {
        let mut n = notices(3);
        for i in 1..=5 {
            n.push(Notice::new(Kind::News, "a", "x").about("p", i));
        }
        let tickets: Vec<u64> = n.shown.iter().map(|n| n.ticket.as_ref().unwrap().1).collect();
        assert_eq!(tickets, vec![5, 4, 3]);
    }

    #[test]
    fn the_same_ticket_and_kind_is_replaced() {
        let mut n = notices(5);
        n.push(Notice::new(Kind::News, "a", "x").about("p", 1));
        n.push(Notice::new(Kind::News, "a", "x").about("p", 2));
        n.push(Notice::new(Kind::News, "a", "y").about("p", 1));
        assert_eq!(n.shown.len(), 2);
        assert_eq!(n.shown[0].text, "y");
    }

    #[test]
    fn they_expire_unless_hovered() {
        let mut n = notices(5);
        n.push(Notice::new(Kind::News, "a", "x").about("p", 1));
        n.push(Notice::new(Kind::Info, "tvty", "y"));
        n.shown[1].hovered = true;
        assert!(n.expire(Instant::now() + Duration::from_secs(7)));
        assert_eq!(n.shown.len(), 1);
        assert_eq!(n.shown[0].text, "x");
    }
}
