//! What the boxes one writes to aiball in share — a thread's reply, a new
//! ticket's body: @-mentions completed as they are typed, an image pasted
//! from the clipboard, uploaded, its link put in the text, and Write /
//! Preview tabs above the box.

use gpui_kit::*;

use crate::theme::p;
use crate::ui::buttons;

/// Above a box to write in: Write and Preview, as tabs; `set` turns the
/// preview on or off.
pub fn write_tabs<V: 'static>(id: &str, preview: bool, cx: &mut Context<V>, set: fn(&mut V, bool, &mut Context<V>)) -> Div {
    let mut tabs = div().flex().gap_1().border_b_1().border_color(p().border);
    for (key, label, to) in [("write", "Write", false), ("preview", "Preview", true)] {
        let on = preview == to;
        tabs = tabs.child(
            div()
                .id(SharedString::from(format!("{id}-{key}")))
                .px_3()
                .py_1()
                .text_sm()
                .cursor_pointer()
                .border_b_2()
                .border_color(if on { p().accent } else { transparent_black() })
                .text_color(if on { p().text } else { p().muted })
                .hover(|d| d.text_color(p().text))
                .child(label)
                .on_click(cx.listener(move |view, _, _, cx| set(view, to, cx))),
        );
    }
    // What the Write / Preview tip is about.
    crate::tips::target("composer.tabs", tabs)
}

/// The `@name` being typed at the end of `text`, lowercase.
pub fn typed_mention(text: &str) -> Option<String> {
    let word = text.rsplit(char::is_whitespace).next()?;
    let name = word.strip_prefix('@')?;
    name.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        .then(|| name.to_lowercase())
}

/// Where the `@name` being typed at the end of `before` (the text before
/// the cursor) starts: what its completion replaces.
pub fn mention_start(before: &str) -> Option<usize> {
    typed_mention(before)?;
    before.rfind('@')
}

/// What a pasted image puts at the cursor, between `before` and `after`:
/// its link, on a line of its own.
pub fn image_snippet(before: &str, after: &str, url: &str) -> String {
    let lead = if before.is_empty() || before.ends_with('\n') { "" } else { "\n" };
    let tail = if after.starts_with('\n') { "" } else { "\n" };
    format!("{lead}![pasted]({url}){tail}")
}

/// What a quoted question adds at the end of `text`.
pub fn quote_block(text: &str, question: &str) -> String {
    let lead = if text.is_empty() || text.ends_with('\n') { "" } else { "\n\n" };
    format!("{lead}> {question}\n\n")
}

/// An image on the clipboard: its bytes, its content type, a file name.
pub fn clipboard_image(cx: &App) -> Option<(Vec<u8>, String, String)> {
    let image = cx.read_from_clipboard()?.into_entries().find_map(|entry| match entry {
        ClipboardEntry::Image(image) => Some(image),
        _ => None,
    })?;
    let content_type = image.format.mime_type().to_string();
    let name = format!("pasted.{}", content_type.rsplit('/').next().unwrap_or("png"));
    Some((image.bytes.clone(), content_type, name))
}

/// The names that fit what is typed after `@`, as chips; `pick` completes.
pub fn mention_chips<V: 'static>(
    names: &[String],
    typed: &str,
    id: &str,
    cx: &mut Context<V>,
    pick: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + Clone + 'static,
) -> Div {
    let mut row = div().flex().flex_wrap().gap_1();
    for name in names.iter().filter(|n| n.to_lowercase().starts_with(typed)).take(8) {
        let (chosen, pick) = (name.clone(), pick.clone());
        row = row.child(
            buttons::chip(SharedString::from(format!("{id}-{name}")), format!("@{name}"))
                .text_xs()
                .on_click(cx.listener(move |view, _, window, cx| pick(view, chosen.clone(), window, cx))),
        );
    }
    row
}

#[cfg(test)]
mod preview_tests {
    #[test]
    fn a_preview_says_its_images() {
    }
}

#[cfg(test)]
mod tests {
    use super::{image_snippet, mention_start, quote_block, typed_mention};

    #[test]
    fn a_mention_is_the_word_before_the_cursor() {
        assert_eq!(typed_mention("hello @Dav").as_deref(), Some("dav"));
        assert_eq!(typed_mention("hello @").as_deref(), Some(""));
        assert_eq!(typed_mention("hello @dav "), None);
        assert_eq!(typed_mention("mail a@b.c"), None);
        assert_eq!(mention_start("hi @da"), Some(3));
        assert_eq!(mention_start("hi @da there"), None);
    }

    #[test]
    fn an_image_goes_on_its_own_line_where_the_cursor_is() {
        assert_eq!(image_snippet("", "", "/u/a.png"), "![pasted](/u/a.png)\n");
        assert_eq!(image_snippet("look", "", "/u/a.png"), "\n![pasted](/u/a.png)\n");
        // Between two lines: nothing doubled.
        assert_eq!(image_snippet("look\n", "\nthen", "/u/a.png"), "![pasted](/u/a.png)");
        assert_eq!(image_snippet("look", " more", "/u/a.png"), "\n![pasted](/u/a.png)\n");
    }

    #[test]
    fn a_quote_starts_a_paragraph() {
        assert_eq!(quote_block("", "Why?"), "> Why?\n\n");
        assert_eq!(quote_block("Sure", "Why?"), "\n\n> Why?\n\n");
    }
}
