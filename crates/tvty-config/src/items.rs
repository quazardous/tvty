//! One shape for every setting an options page lists, whoever provides it —
//! the program's own preferences, its shortcuts, another program's config
//! (aiball's) — so that one tree, one search and one "modified" mark serve
//! them all. A provider turns what it holds into [`Item`]s; writing a value
//! back stays the provider's business.
//!
//! Also here: durations as people write them (`1h30m`), the grammar aiball
//! uses for its config, reproduced as is.

use std::fmt;

/// Who provides a setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Provider {
    /// The program's own preferences (its schema, its settings file).
    Program,
    /// Its keyboard shortcuts.
    Shortcuts,
    /// A config another program serves (aiball's), per layer.
    Remote,
}

/// A setting as a page lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub provider: Provider,
    /// Unique within its provider: a settings key, a command's name.
    pub key: String,
    /// Where it sits in the tree: a page, and a group on it (empty: none).
    pub page: String,
    pub group: String,
    pub label: String,
    pub about: String,
    /// Its value as shown (`14 px`, `1h30m`, `Ctrl+Shift+B`).
    pub value: String,
    /// Not at its default (or, in a layer, set there).
    pub modified: bool,
    /// Only a human may change it.
    pub protected: bool,
    /// Its value comes from a layer below (a project showing the global).
    pub inherited: bool,
    /// More words a search finds it by (a shortcut's keys, a list's choices).
    pub words: Vec<String>,
}

/// What the search box asks: words that must all be found (in the label, the
/// text, the key, the value, the page or group, the extra words), and
/// `@modified` for the modified ones only.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    pub words: Vec<String>,
    pub modified_only: bool,
}

impl Query {
    pub fn parse(text: &str) -> Self {
        let mut query = Query::default();
        for word in text.split_whitespace() {
            match word.to_lowercase().as_str() {
                "@modified" => query.modified_only = true,
                word => query.words.push(word.to_string()),
            }
        }
        query
    }

    /// Nothing asked: every item, as its page shows it.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty() && !self.modified_only
    }

    pub fn matches(&self, item: &Item) -> bool {
        if self.modified_only && !item.modified {
            return false;
        }
        let haystack = [&item.label, &item.about, &item.key, &item.value, &item.page, &item.group]
            .into_iter()
            .chain(item.words.iter())
            .map(|s| s.to_lowercase())
            .collect::<Vec<_>>()
            .join("\n");
        // `_` and `.` read as spaces: `font size` finds `terminal_font_size`.
        let haystack = haystack.replace(['_', '.'], " ") + "\n" + &haystack;
        self.words.iter().all(|w| haystack.contains(w.as_str()))
    }
}

/// The items `query` finds, in their order.
pub fn search<'a>(items: &'a [Item], query: &Query) -> Vec<&'a Item> {
    items.iter().filter(|i| query.matches(i)).collect()
}

// ── Durations ────────────────────────────────────────────────────────

/// What is not a duration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurationError(pub String);

impl fmt::Display for DurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} is not a duration (90s, 15m, 1h30m, 2d)", self.0)
    }
}

/// Seconds from `d h m s` — in that order, each unit at most once, spaces
/// allowed between parts — or from a bare number of seconds.
pub fn parse_duration(input: &str) -> Result<u64, DurationError> {
    let error = || DurationError(input.to_string());
    let text = input.trim().to_lowercase();
    if text.is_empty() {
        return Err(error());
    }
    if text.chars().all(|c| c.is_ascii_digit()) {
        return text.parse().map_err(|_| error());
    }
    let mut seconds = 0u64;
    let mut rest = text.as_str();
    let mut any = false;
    for (unit, size) in [('d', 86_400u64), ('h', 3_600), ('m', 60), ('s', 1)] {
        rest = rest.trim_start();
        let digits = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits == 0 || rest[digits..].chars().next() != Some(unit) {
            continue;
        }
        let n: u64 = rest[..digits].parse().map_err(|_| error())?;
        seconds += n * size;
        rest = &rest[digits + 1..];
        any = true;
    }
    if !any || !rest.trim().is_empty() {
        return Err(error());
    }
    Ok(seconds)
}

/// The notation for a number of seconds, largest units first: `1h30m`,
/// `2d`, `0`.
pub fn format_duration(seconds: u64) -> String {
    if seconds == 0 {
        return "0".into();
    }
    let mut rest = seconds;
    let mut out = String::new();
    for (unit, size) in [('d', 86_400u64), ('h', 3_600), ('m', 60), ('s', 1)] {
        let n = rest / size;
        rest -= n * size;
        if n > 0 {
            out.push_str(&format!("{n}{unit}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Item, Provider, Query, format_duration, parse_duration, search};

    fn item(key: &str, label: &str, modified: bool) -> Item {
        Item {
            provider: Provider::Program,
            key: key.into(),
            page: "Appearance".into(),
            group: "Sizes".into(),
            label: label.into(),
            about: "The terminals' text.".into(),
            value: "14 px".into(),
            modified,
            protected: false,
            inherited: false,
            words: vec![],
        }
    }

    #[test]
    fn a_search_finds_by_label_key_text_or_value() {
        let items = [item("appearance.terminal_font_size", "Terminal font", true), item("scroll.speed", "Wheel speed", false)];
        let found = |q: &str| search(&items, &Query::parse(q)).into_iter().map(|i| i.key.clone()).collect::<Vec<_>>();
        assert_eq!(found("font"), ["appearance.terminal_font_size"]);
        assert_eq!(found("FONT size"), ["appearance.terminal_font_size"]);
        assert_eq!(found("wheel"), ["scroll.speed"]);
        assert_eq!(found("14 px").len(), 2);
        assert!(found("nothing").is_empty());
    }

    #[test]
    fn modified_only_with_the_at_word() {
        let items = [item("a", "One", true), item("b", "Two", false)];
        let query = Query::parse("@modified");
        assert!(query.modified_only && !query.is_empty());
        assert_eq!(search(&items, &query).len(), 1);
        assert!(Query::parse("  ").is_empty());
    }

    #[test]
    fn durations_as_aiball_writes_them() {
        for (text, seconds) in [("90s", 90), ("15m", 900), ("1h30m", 5400), ("1h 30m", 5400), ("2d", 172_800), ("0", 0), ("300", 300), ("1D2H", 93_600)] {
            assert_eq!(parse_duration(text), Ok(seconds), "{text}");
        }
        for bad in ["", "1x", "30m1h", "1h1h", "h", "1.5h"] {
            assert!(parse_duration(bad).is_err(), "{bad}");
        }
        assert_eq!(format_duration(5400), "1h30m");
        assert_eq!(format_duration(172_800), "2d");
        assert_eq!(format_duration(0), "0");
        assert_eq!(format_duration(3661), "1h1m1s");
    }
}
