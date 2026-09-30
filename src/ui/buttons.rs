//! Every clickable that is not a row of a list comes from here, in one of
//! five kinds — never made by hand, so that each one shows it is clickable,
//! and the same way.
//!
//! - [`link`]: an action said in words, in the accent colour, no border
//!   ("+ terminal", "← Tickets", "⇅ recent").
//! - [`icon`]: a glyph alone (‹ › ⤢ ⚙), in the muted colour; its tooltip,
//!   required, says the action and its shortcut ([`hint`]). [`remove`] is
//!   the one that takes something away (✕).
//! - [`chip`]: a bordered label, for an option or a state one can change
//!   ("+ New", the agent bar's chips, a choice among several).
//! - the answers of a dialog or a form — the kit's button, **all the same
//!   size**: [`primary`] the main one, [`secondary`] the others (Cancel,
//!   Not now…), [`answer`] with a colour of its own (`.danger()`,
//!   `.success()`, `.warning()`) for a decision. Never a chip nor a link
//!   beside them: side by side, they would not match.
//!
//! The same states for all: under the pointer, a light background and the
//! text brighter; pressed, a stronger background; [`Look::chosen`], the
//! background of a chosen row (and an accent border on a chip);
//! [`chip_if`] with nothing to do, greyed, no pointer, no hover. The hand pointer on
//! every one that acts.
//!
//! In a bar, groups of actions are set apart by a [`separator`]; inside a
//! group, a plain gap.

use crate::ui::Named as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::*;

use gpui_kit::prelude::FluentBuilder as _;

use crate::theme::p;
use crate::tip::Tip as _;

/// What the pointer does to a button: set once (GPUI takes one hover style).
#[derive(Clone, Copy, PartialEq)]
enum Hover {
    /// A light background.
    Plain,
    /// A light background, the text brighter.
    Brighter,
    /// A light background, the text in the danger colour.
    Danger,
    /// Nothing to do now: no pointer, no hover.
    Off,
}

fn base(id: impl Into<ElementId>, hover: Hover) -> Stateful<Div> {
    let id: ElementId = id.into();
    // Known by its id to the debug control, where it is painted.
    let d = div().named(id).flex().flex_none().items_center().gap_1().rounded_sm();
    match hover {
        Hover::Off => d.text_color(p().muted.opacity(0.5)),
        hover => d.cursor_pointer().active(|d| d.bg(p().active)).hover(move |d| match hover {
            Hover::Brighter => d.bg(p().hover).text_color(p().text),
            Hover::Danger => d.bg(p().hover).text_color(p().danger),
            _ => d.bg(p().hover),
        }),
    }
}

/// An action in words: accent text, a background under the pointer.
pub fn link(id: impl Into<ElementId>, label: impl IntoElement) -> Stateful<Div> {
    base(id, Hover::Plain).px_1().text_color(p().accent).child(label)
}

fn glyph_button(id: impl Into<ElementId>, glyph: impl IntoElement, tip: impl Into<SharedString>, hover: Hover) -> Stateful<Div> {
    base(id, hover).justify_center().min_w(px(20.)).px_1().text_color(p().muted).child(glyph).tip(tip)
}

/// A glyph alone, muted until the pointer comes; `tip` says what it does.
pub fn icon(id: impl Into<ElementId>, glyph: impl IntoElement, tip: impl Into<SharedString>) -> Stateful<Div> {
    glyph_button(id, glyph, tip, Hover::Brighter)
}

/// An icon that takes something away (✕, ×): the danger colour under the
/// pointer.
pub fn remove(id: impl Into<ElementId>, glyph: impl IntoElement, tip: impl Into<SharedString>) -> Stateful<Div> {
    glyph_button(id, glyph, tip, Hover::Danger)
}

/// A bordered label: an option, a state to change.
pub fn chip(id: impl Into<ElementId>, label: impl IntoElement) -> Stateful<Div> {
    chip_if(id, label, true)
}

/// A chip that may have nothing to do now: then greyed, no pointer, no
/// hover — its click is the caller's to leave out.
pub fn chip_if(id: impl Into<ElementId>, label: impl IntoElement, enabled: bool) -> Stateful<Div> {
    base(id, if enabled { Hover::Plain } else { Hover::Off })
        .px_1p5()
        .border_1()
        .border_color(p().border)
        .when(enabled, |d| d.text_color(p().text))
        .child(label)
}

/// An answer of a form or a dialog, the size they all share: a colour
/// to give (`.danger()`, `.success()`, `.warning()`).
pub fn answer(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    let id: ElementId = id.into();
    let mark = crate::inspect::enabled().then(|| crate::inspect::mark(id.to_string()));
    Button::new(id).small().label(label).children(mark)
}

/// The main answer of a form or a dialog.
pub fn primary(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    answer(id, label).primary()
}

/// Another answer beside it: Cancel, Not now, Back…
pub fn secondary(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    answer(id, label).outline()
}

/// Sets groups of actions apart, in a bar.
pub fn separator() -> Div {
    div().flex_none().w(px(1.)).h(px(12.)).mx_1().bg(p().border)
}

/// `text`, and the first key of `command` as the keymap in force says it:
/// an icon's tooltip.
pub fn hint(cx: &App, text: &str, command: &str) -> String {
    match crate::keymap::current(cx).keys_of(command).first() {
        Some(key) => format!("{text} · {}", key.pretty()),
        None => text.to_string(),
    }
}

/// The states a button of this module may take.
pub trait Look: Styled + StatefulInteractiveElement + Sized {
    /// Chosen among others: a chosen row's background, an accent border
    /// (seen on a chip only).
    fn chosen(self, on: bool) -> Self {
        if on { self.bg(p().active).border_color(p().accent) } else { self }
    }
}

impl<E: Styled + StatefulInteractiveElement + Sized> Look for E {}
