//! Links in a terminal's text: where an address starts and ends on a line
//! (a line the program wrapped already joined up), without the punctuation
//! that closes the sentence around it.

/// The schemes a link starts with.
const SCHEMES: &[&str] = &["https://", "http://", "file://", "mailto:"];

/// Every link in `text`, as char ranges (start, end exclusive).
pub fn find(text: &str) -> Vec<(usize, usize)> {
    let chars: Vec<char> = text.chars().collect();
    let lower: String = text.to_lowercase();
    let lower: Vec<char> = lower.chars().collect();
    let mut found = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        let scheme = SCHEMES.iter().find(|s| lower.len() >= at + s.len() && lower[at..at + s.len()].iter().copied().eq(s.chars()));
        // A scheme glued to a word before it (`xhttps://`) is not a link's start.
        let starts = at == 0 || !chars[at - 1].is_alphanumeric();
        let Some(scheme) = scheme.filter(|_| starts) else {
            at += 1;
            continue;
        };
        let mut end = at + scheme.len();
        while end < chars.len() && is_link_char(chars[end]) {
            end += 1;
        }
        let end = trim_end(&chars, at, end);
        if end > at + scheme.len() {
            found.push((at, end));
            at = end;
        } else {
            at += scheme.len();
        }
    }
    found
}

fn is_link_char(c: char) -> bool {
    !c.is_whitespace() && !matches!(c, '<' | '>' | '"' | '`' | '{' | '}' | '|' | '\\' | '^') && !c.is_control()
}

/// Where a link ends once the sentence's punctuation is left out: a final
/// `.` `,` `;` `:` `!` `?` `'`, a `)` or `]` it did not open.
fn trim_end(chars: &[char], start: usize, mut end: usize) -> usize {
    loop {
        let Some(&last) = chars.get(end.wrapping_sub(1)).filter(|_| end > start) else { return end };
        let unbalanced = |open: char, close: char| {
            let link = &chars[start..end];
            link.iter().filter(|&&c| c == close).count() > link.iter().filter(|&&c| c == open).count()
        };
        let drop = match last {
            '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '*' => true,
            ')' => unbalanced('(', ')'),
            ']' => unbalanced('[', ']'),
            _ => false,
        };
        if !drop {
            return end;
        }
        end -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::find;

    fn links(text: &str) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        find(text).into_iter().map(|(a, b)| chars[a..b].iter().collect()).collect()
    }

    #[test]
    fn links_are_found_without_the_sentence_around_them() {
        assert_eq!(links("see https://example.test/a?b=1, then"), vec!["https://example.test/a?b=1"]);
        assert_eq!(links("(https://example.test/x)."), vec!["https://example.test/x"]);
        assert_eq!(links("https://en.wikipedia.org/wiki/Rust_(language) ok"), vec!["https://en.wikipedia.org/wiki/Rust_(language)"]);
        assert_eq!(links("two: http://a.test and file:///tmp/x.txt!"), vec!["http://a.test", "file:///tmp/x.txt"]);
        assert_eq!(links("\"https://example.test/q\""), vec!["https://example.test/q"]);
        assert_eq!(links("mail mailto:someone@example.test."), vec!["mailto:someone@example.test"]);
        // A scheme alone, or glued to a word, is not a link.
        assert!(links("https:// and xhttps://example.test").is_empty());
        assert!(links("nothing here").is_empty());
        // Wide characters count as one each.
        assert_eq!(links("→ https://example.test/é"), vec!["https://example.test/é"]);
    }
}
