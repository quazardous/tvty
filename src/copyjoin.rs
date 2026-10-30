//! A terminal's copy, the lines the screen cut put back together. A program
//! that draws its own screen (Claude Code) cuts its long lines itself, with a
//! real line break and its indentation: the terminal cannot tell them from
//! lines of their own, so this tells by their shape.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// `text` as selected in a terminal `columns` wide, its first line starting
/// at `first_column`: a line goes on the next when the next one's first word
/// would not have fit at its end, with the same indentation, and nothing in
/// it that starts a line of its own (a bullet, a frame).
pub fn join_screen_lines(text: &str, columns: usize, first_column: usize) -> String {
    let lines: Vec<&str> = text.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).collect();
    let mut joined = String::with_capacity(text.len());
    let mut indent = None;
    for (at, line) in lines.iter().enumerate() {
        let Some(next) = lines.get(at + 1) else {
            joined.push_str(if indent.is_some() { line.trim_start() } else { line });
            break;
        };
        joined.push_str(if indent.is_some() { line.trim_start() } else { line });
        let start = if at == 0 { first_column } else { 0 };
        // On a cut, the paragraph's indentation is the one its first line
        // set, or what follows its bullet.
        let expected = indent.unwrap_or_else(|| continuation_indent(line));
        match cut(line, next, start, columns, expected) {
            Some(glue) => {
                joined.push_str(glue);
                indent = Some(expected);
            }
            None => {
                joined.push('\n');
                indent = None;
            }
        }
    }
    joined
}

/// How `line` goes on with `next`, if the screen cut it there.
fn cut(line: &str, next: &str, start: usize, columns: usize, indent: usize) -> Option<&'static str> {
    let (line_text, next_text) = (line.trim(), next.trim());
    if line_text.is_empty() || next_text.is_empty() || framed(line_text) || framed(next_text) || bullet_width(next_text).is_some() {
        return None;
    }
    if leading(next) != indent {
        return None;
    }
    let width = start + line.trim_end().width();
    // Programs keep off the last column: what fills the line up to it.
    let room = columns.saturating_sub(1);
    // One word longer than the line, cut in its middle: nothing was there.
    let words = &line_text[bullet_width(line_text).map_or(0, |bullet| bullet_bytes(line_text, bullet))..];
    if width >= room && !words.contains(' ') {
        return Some("");
    }
    let word = next_text.split(' ').next().unwrap_or_default().width();
    (width + 1 + word > room).then_some(" ")
}

fn leading(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

/// Where a line's text goes on when it wraps: under its text, past a bullet.
fn continuation_indent(line: &str) -> usize {
    let text = line.trim_start();
    leading(line) + bullet_width(text).unwrap_or(0)
}

/// How wide the bullet that starts `text` is, with the space after it.
fn bullet_width(text: &str) -> Option<usize> {
    let first = text.chars().next()?;
    if matches!(first, '-' | '*' | '+' | '•' | '●' | '○' | '◦' | '⎿' | '❯' | '>' | '✓' | '✗' | '⏺') {
        return text[first.len_utf8()..].starts_with(' ').then(|| first.width().unwrap_or(1) + 1);
    }
    let digits = text.chars().take_while(char::is_ascii_digit).count();
    (digits > 0 && text[digits..].starts_with(". ")).then_some(digits + 2)
}

/// How many bytes a bullet `width` wide takes at the start of `text`.
fn bullet_bytes(text: &str, width: usize) -> usize {
    text.char_indices().scan(0, |seen, (at, c)| {
        *seen += c.width().unwrap_or(0);
        Some((at + c.len_utf8(), *seen))
    }).find(|(_, seen)| *seen >= width).map_or(0, |(at, _)| at)
}

/// A line of a box or a table: what its borders hold is not a paragraph.
fn framed(text: &str) -> bool {
    text.chars().next().is_some_and(|c| ('\u{2500}'..='\u{259F}').contains(&c))
}

#[cfg(test)]
mod tests {
    use super::join_screen_lines as join;

    #[test]
    fn a_line_cut_before_a_word_that_would_not_fit_is_joined() {
        // 20 wide: "two" (3) would not fit after 18 + a space.
        assert_eq!(join("  sudo sh -c 'one |\n  two'", 20, 0), "  sudo sh -c 'one | two'");
        // It would have fit: a line of its own.
        assert_eq!(join("  short line\n  two", 20, 0), "  short line\n  two");
    }

    #[test]
    fn a_word_cut_at_the_edge_is_joined_without_a_space() {
        assert_eq!(join(" /tmp/claude/Private-dev-p\n rojects/tvty", 26, 0), " /tmp/claude/Private-dev-projects/tvty");
    }

    #[test]
    fn a_line_up_to_the_edge_is_cut_between_words_unless_it_is_one_word() {
        // As Claude Code does: one column kept free.
        assert_eq!(join(" /tmp/x/Private-dev-p\n rojects/tvty", 22, 0), " /tmp/x/Private-dev-projects/tvty");
        assert_eq!(join(" one two three four f\n five", 22, 0), " one two three four f five");
        // The same past a bullet.
        assert_eq!(join("● /tmp/x/Private-dev-p\n  rojects", 23, 0), "● /tmp/x/Private-dev-projects");
    }

    #[test]
    fn a_paragraph_keeps_its_first_indent_and_goes_on_past_a_bullet() {
        assert_eq!(join("● one two three four\n  five six seven ei\n  nine", 21, 0), "● one two three four five six seven ei nine");
        // A bullet starts a line of its own.
        assert_eq!(join("- one two three four\n- five", 20, 0), "- one two three four\n- five");
        assert_eq!(join("1. one two three fou\n2. five", 20, 0), "1. one two three fou\n2. five");
    }

    #[test]
    fn what_is_not_a_cut_stays() {
        // Another indentation: a code block, a new level.
        assert_eq!(join("  one two three fourt\n    five", 21, 0), "  one two three fourt\n    five");
        // An empty line, a frame.
        assert_eq!(join("one two three four f\n\nsix", 20, 0), "one two three four f\n\nsix");
        assert_eq!(join("│ one two three four\n│ five", 20, 0), "│ one two three four\n│ five");
        assert_eq!(join("────────────────────\nfive", 20, 0), "────────────────────\nfive");
        assert_eq!(join("single line", 20, 0), "single line");
    }

    #[test]
    fn a_selection_that_starts_inside_a_line_counts_from_there() {
        // "three four" alone would leave room; from column 10 it fills the line.
        assert_eq!(join("three four\nfive", 20, 10), "three four five");
        assert_eq!(join("three four\nfive", 20, 0), "three four\nfive");
    }

    #[test]
    fn wide_characters_count_for_two() {
        // 8 wide characters fill 16 columns: "word" would not fit in 20.
        assert_eq!(join("日本語日本語日本\nword", 20, 0), "日本語日本語日本 word");
    }
}
