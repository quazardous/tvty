//! What a ticket is, as a column of fields — a new ticket's and a ticket's
//! in full screen, alike: a group's title, a field (its name, then its
//! widget), the dropdown each field is chosen in, a tag with its ✕, what
//! the scope means. The views keep their own state and gestures; the look
//! is one.

use gpui_kit::*;

use crate::theme::p;
use crate::ui::buttons;
use crate::ui::combo::{self, ComboState};

/// The names' column width.
pub const LABEL_WIDTH: f32 = 80.;

/// A group's title: WHERE, FIELDS, PEOPLE…
pub fn group(title: &'static str) -> Div {
    div().pt_3().pb_1().text_xs().font_weight(FontWeight::BOLD).text_color(p().muted).child(title.to_uppercase())
}

/// A field: its name, and its widget beside it, wrapping.
pub fn field(name: &'static str, widget: impl IntoElement) -> Div {
    div()
        .flex()
        .gap_2()
        .py_1()
        .child(div().w(px(LABEL_WIDTH)).flex_none().pt_0p5().text_color(p().muted).child(name))
        .child(div().flex_1().min_w_0().child(widget))
}

/// A field that says, not chosen here: its name, and its value.
pub fn said(name: &'static str, value: impl IntoElement) -> Div {
    div()
        .flex()
        .gap_2()
        .py_1()
        .child(div().w(px(LABEL_WIDTH)).flex_none().text_color(p().muted).child(name))
        .child(div().flex_1().min_w_0().child(value))
}

/// The dropdown a field is chosen in, searched as it is typed: `id` names
/// it (for the debug control too), `placeholder` shows while nothing is
/// chosen, `search` in its search box; `clearable`: its ✕ empties it.
pub fn picker(state: &Entity<ComboState>, id: impl Into<SharedString>, placeholder: &str, search: &str, clearable: bool) -> Div {
    let id = id.into();
    // As wide as the column lets it, up to 220 px.
    div()
        .w_full()
        .max_w(px(220.))
        .min_w(px(0.))
        .child(combo::view(state, id.clone(), placeholder, search).cleanable(clearable))
        .children(crate::inspect::mark_if(id.to_string()))
}

/// A tag the ticket carries, and the ✕ that takes it out (`drop`, its
/// click the caller's).
pub fn tag(id: impl Into<SharedString>, name: &str, drop: Stateful<Div>) -> Stateful<Div> {
    div()
        .id(id.into())
        .flex()
        .items_center()
        .gap_1()
        .px_1p5()
        .py_0p5()
        .rounded_sm()
        .text_xs()
        .border_1()
        .border_color(p().accent)
        .bg(p().accent.opacity(0.15))
        .child(name.to_string())
        .child(drop.min_w(px(0.)))
}

/// The ✕ of a [`tag`].
pub fn tag_drop(id: impl Into<SharedString>) -> Stateful<Div> {
    buttons::remove(id.into(), "✕", "take it out")
}

/// The tags and the dropdown that adds one, wrapping together.
pub fn tags(chips: Vec<Stateful<Div>>, picker: Div) -> Div {
    div().flex().flex_wrap().items_center().gap_1().children(chips).child(picker)
}

/// What a scope means, under its field: two lines kept whatever it says, so
/// the fields below do not move as it changes.
pub fn scope_note(scope: &str) -> Div {
    div().pl(px(LABEL_WIDTH + 8.)).pb_1().min_h(px(36.)).text_xs().text_color(p().muted).child(match scope {
        "internal" => "notifies nobody but who is mentioned",
        "broadcast" => "notifies the project's followers too",
        _ => "notifies the ticket's subscribers and the project's owners",
    })
}

/// The values of the short fields, as aiball takes them.
pub const INTENTS: &[&str] = &["request", "question", "fyi", "feature", "panic"];
pub const PRIORITIES: &[&str] = &["urgent", "high", "normal", "low"];
pub const LEVELS: &[&str] = &["task", "milestone", "roadmap"];
pub const SCOPES: &[&str] = &["internal", "default", "broadcast"];
