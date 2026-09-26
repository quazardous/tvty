//! What the boxes one writes to aiball in share — a thread's reply, a new
//! ticket's body: @-mentions completed as they are typed, and an image
//! pasted from the clipboard, uploaded, its link put in the text.

use gpui_kit::*;

use crate::theme::p;

/// The `@name` being typed at the end of `text`, lowercase.
pub fn typed_mention(text: &str) -> Option<String> {
    let word = text.rsplit(char::is_whitespace).next()?;
    let name = word.strip_prefix('@')?;
    name.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        .then(|| name.to_lowercase())
}

/// `text` with the `@…` it ends with completed to `@name `.
pub fn complete_mention(text: &str, name: &str) -> Option<String> {
    let at = text.rfind('@')?;
    Some(format!("{}@{name} ", &text[..at]))
}

/// `text` with an uploaded image's link after it, on its own line.
pub fn with_image(text: &str, url: &str) -> String {
    let sep = if text.is_empty() || text.ends_with('\n') { "" } else { "\n" };
    format!("{text}{sep}![pasted]({url})\n")
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
            div()
                .id(SharedString::from(format!("{id}-{name}")))
                .px_1p5()
                .rounded_sm()
                .text_xs()
                .border_1()
                .border_color(p().border)
                .cursor_pointer()
                .hover(|d| d.bg(p().hover))
                .child(format!("@{name}"))
                .on_click(cx.listener(move |view, _, window, cx| pick(view, chosen.clone(), window, cx))),
        );
    }
    row
}

#[cfg(test)]
mod tests {
    use super::{complete_mention, typed_mention, with_image};

    #[test]
    fn a_mention_is_the_last_word() {
        assert_eq!(typed_mention("hello @Dav").as_deref(), Some("dav"));
        assert_eq!(typed_mention("hello @").as_deref(), Some(""));
        assert_eq!(typed_mention("hello @dav "), None);
        assert_eq!(typed_mention("mail a@b.c"), None);
        assert_eq!(complete_mention("hi @da", "david").as_deref(), Some("hi @david "));
    }

    #[test]
    fn an_image_goes_on_its_own_line() {
        assert_eq!(with_image("", "/u/a.png"), "![pasted](/u/a.png)\n");
        assert_eq!(with_image("look", "/u/a.png"), "look\n![pasted](/u/a.png)\n");
    }
}
