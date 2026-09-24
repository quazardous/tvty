//! claude-loop's tmux bar, redrawn small: what the loop's pane analyser
//! sees of Claude — its state as the bar's colour, and the words and glyphs
//! on it (🧠 working, 💤 idle, ▶ autonomous, ⏸ held, 웃 a human, the
//! counters). Read from tmux, which expands the bar for us.

use gpui_kit::*;

use crate::terminal::{FONT_FALLBACKS, FONT_FAMILY, xterm_rgb};

#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub text: String,
    pub fg: Option<u32>,
    pub bg: Option<u32>,
}

/// Splits an expanded tmux status line (`#[fg=colour40,bg=colour16]▶ …`)
/// into styled runs, starting from the bar's own colours.
pub fn parse(line: &str, fg: Option<u32>, bg: Option<u32>) -> Vec<Segment> {
    let (base_fg, base_bg) = (fg, bg);
    let (mut fg, mut bg) = (fg, bg);
    let mut segments: Vec<Segment> = Vec::new();
    let mut text = String::new();
    let mut rest = line;
    let flush = |text: &mut String, segments: &mut Vec<Segment>, fg, bg| {
        if !text.is_empty() {
            segments.push(Segment { text: std::mem::take(text), fg, bg });
        }
    };
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("##") {
            text.push('#');
            rest = after;
        } else if let Some(after) = rest.strip_prefix("#[") {
            let Some(end) = after.find(']') else { break };
            flush(&mut text, &mut segments, fg, bg);
            for attr in after[..end].split([',', ' ']).filter(|a| !a.is_empty()) {
                match attr.split_once('=') {
                    Some(("fg", value)) => fg = colour(value).or(base_fg),
                    Some(("bg", value)) => bg = colour(value).or(base_bg),
                    _ if attr == "default" => (fg, bg) = (base_fg, base_bg),
                    _ => {}
                }
            }
            rest = &after[end + 1..];
        } else {
            let next = rest.find('#').map(|i| i.max(1)).unwrap_or(rest.len());
            text.push_str(&rest[..next]);
            rest = &rest[next..];
        }
    }
    flush(&mut text, &mut segments, fg, bg);
    segments
}

/// A tmux colour: `colourN`, `#rrggbb`, a name; `None` for `default`.
pub fn colour(value: &str) -> Option<u32> {
    const NAMES: [&str; 8] = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"];
    let value = value.trim();
    if let Some(n) = value.strip_prefix("colour").or_else(|| value.strip_prefix("color")) {
        return n.parse::<usize>().ok().filter(|&n| n < 256).map(xterm_rgb);
    }
    if let Some(hex) = value.strip_prefix('#') {
        return u32::from_str_radix(hex, 16).ok();
    }
    let (bright, name) = match value.strip_prefix("bright") {
        Some(name) => (8, name),
        None => (0, value),
    };
    NAMES.iter().position(|n| *n == name).map(|i| xterm_rgb(i + bright))
}

/// The bar's background colour: the loop's state at a glance.
pub fn state_colour(segments: &[Segment]) -> Option<u32> {
    segments.first().and_then(|s| s.bg)
}

/// The bar, one line high, clipped to its width.
pub fn view(segments: &[Segment]) -> impl IntoElement + use<> {
    let mut line = div()
        .flex()
        .flex_none()
        .h(px(15.))
        .overflow_hidden()
        .whitespace_nowrap()
        .rounded_sm()
        .font(Font {
            fallbacks: Some(FontFallbacks::from_fonts(
                FONT_FALLBACKS.iter().map(|f| f.to_string()).collect(),
            )),
            ..font(FONT_FAMILY)
        })
        .text_size(px(10.))
        .line_height(px(15.));
    for segment in segments {
        // The bar's shading ramps are decoration: small, they only take room.
        let text: String = segment.text.chars().filter(|c| !matches!(c, '▓' | '▒' | '░')).collect();
        if text.is_empty() {
            continue;
        }
        line = line.child(
            div()
                .flex_none()
                .when_some(segment.bg, |d, bg| d.bg(rgb(bg)))
                .when_some(segment.fg, |d, fg| d.text_color(rgb(fg)))
                .child(text),
        );
    }
    line
}

use gpui_kit::prelude::FluentBuilder as _;

#[cfg(test)]
mod tests {
    use super::{colour, parse, state_colour};

    #[test]
    fn parses_a_claude_loop_bar() {
        let line = "#[bg=colour33] #[fg=colour33,bg=colour16]▓▒░ #[fg=colour40,bg=colour16]▶#[fg=colour15] claude 🧠";
        let segments = parse(line, Some(0), None);
        assert_eq!(segments[0].text, " ");
        assert_eq!(segments[0].bg, colour("colour33"));
        assert_eq!(segments[1].text, "▓▒░ ");
        assert_eq!(segments[2].text, "▶");
        assert_eq!(segments[2].fg, colour("colour40"));
        assert_eq!(segments[3].text, " claude 🧠");
        assert_eq!(segments[3].bg, colour("colour16"));
        assert_eq!(state_colour(&segments), colour("colour33"));
    }

    #[test]
    fn keeps_escaped_hashes() {
        let segments = parse("a ## b", None, None);
        assert_eq!(segments[0].text, "a # b");
    }
}
