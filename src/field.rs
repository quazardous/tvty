//! Editing a text box the user is typing in: at the cursor, never by
//! rewriting the whole box — that sends the cursor back to the start and
//! breaks the undo. Setting a whole value (`set_value`) is for loading or
//! clearing a box, nothing else. Works on single lines and textareas alike.

use std::ops::Range;

use gpui_kit::component::input::{InputState, TextareaState};
use gpui_kit::*;

/// What these gestures need of a box: the kit's single line and textarea
/// share these methods, not a type the kit exports.
pub trait Editable: 'static {
    fn text(&self) -> String;
    /// The cursor, as a byte offset.
    fn at(&self) -> usize;
    fn select(&mut self, range: Range<usize>, cx: &mut Context<Self>)
    where
        Self: Sized;
    /// Replaces the selection with `text`, the cursor after it.
    fn put(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>)
    where
        Self: Sized;
}

macro_rules! editable {
    ($($state:ty),*) => {$(
        impl Editable for $state {
            fn text(&self) -> String {
                self.value().to_string()
            }
            fn at(&self) -> usize {
                self.cursor()
            }
            fn select(&mut self, range: Range<usize>, cx: &mut Context<Self>) {
                self.set_selected_range(range, cx);
            }
            fn put(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
                self.replace(text, window, cx);
            }
        }
    )*};
}
editable!(InputState, TextareaState);

/// The text before the cursor, and the text after it.
pub fn around_cursor<T: Editable>(input: &Entity<T>, cx: &App) -> (String, String) {
    let state = input.read(cx);
    let text = state.text();
    let at = state.at().min(text.len());
    (text[..at].to_string(), text[at..].to_string())
}

/// Puts `text` at the cursor, in place of the selection if any; the cursor
/// goes after it.
pub fn insert_at_cursor<T: Editable>(input: &Entity<T>, text: &str, window: &mut Window, cx: &mut App) {
    input.update(cx, |state, cx| state.put(text.to_string(), window, cx));
}

/// Replaces `range` (byte offsets) with `text`; the cursor goes after it.
pub fn replace_range<T: Editable>(input: &Entity<T>, range: Range<usize>, text: &str, window: &mut Window, cx: &mut App) {
    input.update(cx, |state, cx| {
        state.select(range, cx);
        state.put(text.to_string(), window, cx);
    });
}

/// Adds `text` at the end; the cursor goes after it.
pub fn append<T: Editable>(input: &Entity<T>, text: &str, window: &mut Window, cx: &mut App) {
    let end = input.read(cx).text().len();
    replace_range(input, end..end, text, window, cx);
}
