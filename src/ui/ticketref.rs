//! A ticket's reference — "#12", or a comment's "#C.abc" — wherever tvty
//! paints one: always the same link, followed the same way. A click emits
//! [`Signal::GoToTicket`](crate::bus::Signal::GoToTicket); the shell takes
//! it there (another project's ticket, compact, moves the scope first).
//!
//! - [`link`]: one reference, as a link.
//! - [`inline`]: a line of plain text whose references are links.
//! - [`linkify`] and [`on_link`]: a markdown text whose references are
//!   links, and the handler its view calls on a click.
//!
//! Under the pointer, a reference says its ticket: "#12 — its title", its
//! project and whether it is closed — from the board tvty holds when the
//! ticket is on it ([`learn`]), else asked of aiball at the first hover and
//! kept a while.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::aiball::TicketRow;
use crate::ui::buttons;

/// How long a title asked of aiball is trusted.
const KEPT: Duration = Duration::from_secs(600);

/// What tvty knows of a ticket, for its references' tooltips.
#[derive(Clone, Debug, PartialEq)]
struct Known {
    title: String,
    project: String,
    closed: bool,
    at: Instant,
}

/// The titles known, and the ones being asked.
#[derive(Default)]
struct Titles {
    known: HashMap<u64, Known>,
    asked: HashSet<u64>,
}

impl Global for Titles {}

impl Titles {
    /// A ticket's, when known and not too old.
    fn fresh(&self, ticket: u64, now: Instant) -> Option<&Known> {
        self.known.get(&ticket).filter(|k| now.duration_since(k.at) < KEPT)
    }
}

/// The board's tickets, as tvty holds them: their titles, known at once and
/// fresh — a title edited there shows at the next hover.
pub fn learn<'a>(cx: &mut App, rows: impl Iterator<Item = &'a TicketRow>) {
    let now = Instant::now();
    let titles = cx.default_global::<Titles>();
    for row in rows {
        titles.known.insert(row.id, Known { title: row.title.clone(), project: row.project.clone(), closed: row.closed, at: now });
    }
}

/// Asks aiball for a ticket not on the board, once at a time; the windows
/// draw again when it answers.
fn ask(ticket: u64, cx: &mut App) {
    if !cx.default_global::<Titles>().asked.insert(ticket) {
        return;
    }
    cx.spawn(async move |cx| {
        let brief = cx.background_executor().spawn(async move { crate::aiball::Aiball::from_env().ticket_brief(ticket) }).await;
        let _ = cx.update(|cx| {
            let titles = cx.default_global::<Titles>();
            titles.asked.remove(&ticket);
            match brief {
                Ok((title, project, closed)) => {
                    titles.known.insert(ticket, Known { title, project, closed, at: Instant::now() });
                }
                Err(error) => log::debug!("ticket #{ticket}'s title: {error:#}"),
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

/// The tooltip of a reference to `ticket`: its title (asked for when not
/// known), its project and state.
fn tip(ticket: u64, window: &mut Window, cx: &mut App) -> AnyView {
    Tooltip::element(move |_, cx: &mut App| {
        let known = cx.default_global::<Titles>().fresh(ticket, Instant::now()).cloned();
        if known.is_none() {
            ask(ticket, cx);
        }
        match known {
            Some(k) => div()
                .flex()
                .flex_col()
                .child(format!("#{ticket} — {}", k.title))
                .child(div().text_xs().opacity(0.7).child(format!("{} · {}", k.project, if k.closed { "closed" } else { "open" }))),
            None => div().child(format!("#{ticket} — …")),
        }
    })
    .build(window, cx)
}

/// The scheme of a reference made a link in markdown.
const SCHEME: &str = "tvty-ticket:";

/// Follows a reference: a ticket id, or a comment's hash; `project` when
/// known, else aiball says whose it is.
pub fn follow(cx: &mut App, reference: String, project: Option<String>) {
    crate::bus::emit(cx, crate::bus::Signal::GoToTicket { reference, project });
}

/// "#`ticket`", a link to it.
pub fn link(id: impl Into<ElementId>, ticket: u64, project: Option<String>) -> Stateful<Div> {
    buttons::link(id, format!("#{ticket}"))
        .px_0()
        .tooltip(move |window, cx| tip(ticket, window, cx))
        .on_click(move |_, _, cx| follow(cx, ticket.to_string(), project.clone()))
}

/// A line of text, its references links; `id` tells its links apart.
pub fn inline(id: impl Into<SharedString>, text: &str) -> Div {
    let id: SharedString = id.into();
    let mut line = div().flex().flex_wrap().items_center();
    for (i, piece) in pieces(text).into_iter().enumerate() {
        line = match piece {
            Piece::Text(text) => line.child(div().whitespace_nowrap().child(text.to_string())),
            Piece::Ref { said, reference } => {
                let ticket = reference.parse::<u64>().ok();
                line.child(
                    buttons::link(SharedString::from(format!("{id}-{i}")), said.to_string())
                        .px_0()
                        // A comment's reference ("#C.…") names no ticket to title.
                        .when_some(ticket, |d, ticket| d.tooltip(move |window, cx| tip(ticket, window, cx)))
                        .on_click(move |_, _, cx| follow(cx, reference.clone(), None)),
                )
            }
        };
    }
    line
}

/// A markdown text with its references made links, for a view whose
/// [`on_link`] follows them. Not in code, nor in a link or an address.
pub fn linkify(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    let mut fenced = false;
    for line in md.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            out.push_str(line);
            continue;
        }
        if fenced {
            out.push_str(line);
            continue;
        }
        let mut rest = line;
        while !rest.is_empty() {
            // What a reference may not be taken from: a code span, a link's
            // target, an address.
            let skip = [rest.find('`'), rest.find("]("), rest.find("http://"), rest.find("https://")]
                .into_iter()
                .flatten()
                .min();
            let (open, tail) = match skip {
                Some(at) => rest.split_at(at),
                None => (rest, ""),
            };
            for piece in pieces(open) {
                match piece {
                    Piece::Text(text) => out.push_str(text),
                    // Its text already a link's ("[#12]"): left as it is.
                    Piece::Ref { said, .. } if out.ends_with('[') => out.push_str(said),
                    Piece::Ref { said, reference } => out.push_str(&format!("[{said}]({SCHEME}{reference})")),
                }
            }
            if tail.is_empty() {
                break;
            }
            let end = if tail.starts_with('`') {
                let ticks = tail.len() - tail.trim_start_matches('`').len();
                tail[ticks..].find(&tail[..ticks]).map(|at| ticks + at + ticks)
            } else if tail.starts_with("](") {
                tail.find(')').map(|at| at + 1)
            } else {
                Some(tail.find(char::is_whitespace).unwrap_or(tail.len()))
            };
            let end = end.unwrap_or(tail.len());
            out.push_str(&tail[..end]);
            rest = &tail[end..];
        }
    }
    out
}

/// A markdown view's link handler: a reference is followed, any other
/// link opens in the browser.
pub fn on_link(url: &SharedString, _: &ClickEvent, _: &mut Window, cx: &mut App) {
    match url.strip_prefix(SCHEME) {
        Some(reference) => follow(cx, reference.to_string(), None),
        None => cx.open_url(url),
    }
}

#[derive(Debug, PartialEq)]
enum Piece<'a> {
    Text(&'a str),
    /// `said` as written ("#12", "#C.abc"); `reference` what aiball takes.
    Ref { said: &'a str, reference: String },
}

/// `text` cut at its references: "#" and digits, or "#C." and a hash, not
/// glued to a word before it nor after.
fn pieces(text: &str) -> Vec<Piece<'_>> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let glued = i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || matches!(bytes[i - 1], b'/' | b'&' | b'#' | b'_'));
        if bytes[i] != b'#' || glued {
            i += 1;
            continue;
        }
        let digits = bytes[i + 1..].iter().take_while(|b| b.is_ascii_digit()).count();
        let (end, reference) = if digits > 0 {
            (i + 1 + digits, text[i + 1..i + 1 + digits].to_string())
        } else if text[i + 1..].starts_with("C.") {
            let hash = bytes[i + 3..].iter().take_while(|b| b.is_ascii_alphanumeric()).count();
            if hash == 0 {
                i += 1;
                continue;
            }
            (i + 3 + hash, text[i + 3..i + 3 + hash].to_string())
        } else {
            i += 1;
            continue;
        };
        if bytes.get(end).is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_') {
            i = end;
            continue;
        }
        if start < i {
            out.push(Piece::Text(&text[start..i]));
        }
        out.push(Piece::Ref { said: &text[i..end], reference });
        start = end;
        i = end;
    }
    if start < text.len() {
        out.push(Piece::Text(&text[start..]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{KEPT, Known, Piece, Titles, linkify, pieces};
    use std::time::{Duration, Instant};

    #[test]
    fn a_title_is_trusted_a_while() {
        let now = Instant::now();
        let mut titles = Titles::default();
        let known = |at| Known { title: "t".into(), project: "p".into(), closed: false, at };
        titles.known.insert(1, known(now - Duration::from_secs(60)));
        titles.known.insert(2, known(now - KEPT - Duration::from_secs(1)));
        assert!(titles.fresh(1, now).is_some());
        assert!(titles.fresh(2, now).is_none(), "too old: asked again");
        assert!(titles.fresh(3, now).is_none());
    }

    #[test]
    fn a_reference_is_a_hash_and_digits_or_a_comment() {
        assert_eq!(
            pieces("see #12, and #C.ab3."),
            vec![
                Piece::Text("see "),
                Piece::Ref { said: "#12", reference: "12".into() },
                Piece::Text(", and "),
                Piece::Ref { said: "#C.ab3", reference: "ab3".into() },
                Piece::Text("."),
            ]
        );
        for plain in ["a#12", "#12a", "# 12", "#", "#C.", "##12", "x/#12", "&#12;"] {
            assert!(pieces(plain).iter().all(|p| matches!(p, Piece::Text(_))), "{plain}");
        }
    }

    #[test]
    fn markdown_references_become_links_outside_code_and_links() {
        assert_eq!(linkify("depends on #3."), "depends on [#3](tvty-ticket:3).");
        assert_eq!(linkify("`#3` and #4"), "`#3` and [#4](tvty-ticket:4)");
        assert_eq!(linkify("``a ` #3`` #4"), "``a ` #3`` [#4](tvty-ticket:4)");
        assert_eq!(linkify("```\n#3\n```\n#4"), "```\n#3\n```\n[#4](tvty-ticket:4)");
        assert_eq!(linkify("[#3](http://x/#3) #4"), "[#3](http://x/#3) [#4](tvty-ticket:4)");
        assert_eq!(linkify("https://x.org/a#12 #5"), "https://x.org/a#12 [#5](tvty-ticket:5)");
        assert_eq!(linkify("# Title\n## 2"), "# Title\n## 2");
        assert_eq!(linkify("![img](/uploads/a.png) #6"), "![img](/uploads/a.png) [#6](tvty-ticket:6)");
    }
}
