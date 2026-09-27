//! A key as people write it and as a program binds it.
//!
//! `Ctrl+Shift+B`, `ctrl-shift-b` and `shift-ctrl-b` are one key: parsed
//! into a [`Stroke`] (its modifiers and the key itself), written back in one
//! canonical form (`ctrl-shift-b`, GPUI's syntax) and shown as people read
//! it (`Ctrl+Shift+B`). A [`Key`] is one stroke or a sequence of them
//! (`ctrl-k ctrl-s`).

use std::fmt;

/// What does not parse as a key, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyError {
    pub input: String,
    pub why: String,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} is not a key: {}", self.input, self.why)
    }
}

impl std::error::Error for KeyError {}

/// One stroke: modifiers held, and a key.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Stroke {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// The Super / Windows / Command key.
    pub super_key: bool,
    pub function: bool,
    /// Lowercase: a character (`b`, `=`, `-`) or a named key (`pagedown`).
    pub key: String,
}

/// The keys that have a name, as the canonical form writes them.
const NAMED: &[&str] = &[
    "enter", "tab", "space", "backspace", "delete", "insert", "escape", "home", "end", "pageup", "pagedown", "up",
    "down", "left", "right", "menu", "capslock", "printscreen", "scrolllock", "pause",
];

/// Other names people write for them.
const ALIASES: &[(&str, &str)] = &[
    ("return", "enter"),
    ("esc", "escape"),
    ("del", "delete"),
    ("ins", "insert"),
    ("pgup", "pageup"),
    ("pgdn", "pagedown"),
    ("pgdown", "pagedown"),
    ("bksp", "backspace"),
    ("arrowup", "up"),
    ("arrowdown", "down"),
    ("arrowleft", "left"),
    ("arrowright", "right"),
    ("plus", "+"),
    ("minus", "-"),
];

fn is_function_key(key: &str) -> bool {
    key.strip_prefix('f').and_then(|n| n.parse::<u8>().ok()).is_some_and(|n| (1..=35).contains(&n))
}

impl Stroke {
    /// A stroke as people write it: modifiers and key joined by `-` or `+`,
    /// in any order and case.
    pub fn parse(input: &str) -> Result<Self, KeyError> {
        let error = |why: &str| KeyError { input: input.to_string(), why: why.to_string() };
        let text = input.trim();
        if text.is_empty() {
            return Err(error("empty"));
        }
        // The key is what follows the last separator; a separator at the
        // end is the key itself (`ctrl--`, `ctrl++`, `ctrl-+`).
        let (mods, key) = if text.chars().count() == 1 {
            ("", text)
        } else if let Some(mods) = text.strip_suffix("--").or_else(|| text.strip_suffix("++")).or_else(|| text.strip_suffix("-+")).or_else(|| text.strip_suffix("+-")) {
            (mods, &text[mods.len() + 1..])
        } else {
            match text.rfind(['-', '+']) {
                Some(at) => (&text[..at], &text[at + 1..]),
                None => ("", text),
            }
        };
        let mut stroke = Stroke { ctrl: false, alt: false, shift: false, super_key: false, function: false, key: String::new() };
        for modifier in mods.split(['-', '+']).filter(|m| !m.is_empty()) {
            match modifier.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => stroke.ctrl = true,
                "alt" | "option" | "opt" => stroke.alt = true,
                "shift" => stroke.shift = true,
                "super" | "cmd" | "command" | "win" | "meta" | "platform" => stroke.super_key = true,
                "fn" => stroke.function = true,
                other => return Err(error(&format!("{other:?} is not a modifier (ctrl, alt, shift, super)"))),
            }
        }
        let lower = key.to_lowercase();
        let lower = ALIASES.iter().find(|(alias, _)| *alias == lower).map_or(lower, |(_, name)| (*name).to_string());
        let single = lower.chars().count() == 1 && !lower.chars().all(char::is_whitespace);
        if !(single || NAMED.contains(&lower.as_str()) || is_function_key(&lower)) {
            return Err(error(&format!("no key named {key:?}")));
        }
        // The case of a letter does not matter: `Ctrl+Shift+B` is written
        // as it is shown; shift is said, never implied by a capital.
        stroke.key = lower;
        Ok(stroke)
    }

    /// The canonical form: `ctrl-alt-shift-super-fn-key`.
    pub fn canonical(&self) -> String {
        let mut out = String::new();
        for (on, name) in [(self.ctrl, "ctrl"), (self.alt, "alt"), (self.shift, "shift"), (self.super_key, "super"), (self.function, "fn")] {
            if on {
                out.push_str(name);
                out.push('-');
            }
        }
        out.push_str(&self.key);
        out
    }

    /// As people read it: `Ctrl+Shift+B`, `Ctrl+PgDn`.
    pub fn pretty(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for (on, name) in [(self.ctrl, "Ctrl"), (self.alt, "Alt"), (self.shift, "Shift"), (self.super_key, "Super"), (self.function, "Fn")] {
            if on {
                parts.push(name.to_string());
            }
        }
        parts.push(match self.key.as_str() {
            "pageup" => "PgUp".to_string(),
            "pagedown" => "PgDn".to_string(),
            "escape" => "Esc".to_string(),
            "backspace" => "Backspace".to_string(),
            k if k.chars().count() == 1 => k.to_uppercase(),
            k if is_function_key(k) => k.to_uppercase(),
            k => {
                let mut chars = k.chars();
                chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
            }
        });
        parts.join("+")
    }

    /// Whether the stroke is typing — a character, space, enter, backspace
    /// without Ctrl, Alt or Super: what a terminal's program must get.
    pub fn is_typing(&self) -> bool {
        let character = self.key.chars().count() == 1 || matches!(self.key.as_str(), "space" | "enter" | "backspace");
        character && !(self.ctrl || self.alt || self.super_key)
    }
}

/// A key: one stroke, or a sequence of them.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key(pub Vec<Stroke>);

impl Key {
    /// Strokes separated by spaces: `ctrl-k ctrl-s`.
    pub fn parse(input: &str) -> Result<Self, KeyError> {
        let strokes = input.split_whitespace().map(Stroke::parse).collect::<Result<Vec<_>, _>>()?;
        if strokes.is_empty() {
            return Err(KeyError { input: input.to_string(), why: "empty".to_string() });
        }
        Ok(Key(strokes))
    }

    pub fn canonical(&self) -> String {
        self.0.iter().map(Stroke::canonical).collect::<Vec<_>>().join(" ")
    }

    pub fn pretty(&self) -> String {
        self.0.iter().map(Stroke::pretty).collect::<Vec<_>>().join(" ")
    }

    /// A key whose first stroke is typing (see [`Stroke::is_typing`]).
    pub fn is_typing(&self) -> bool {
        self.0.first().is_some_and(Stroke::is_typing)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical())
    }
}

impl std::str::FromStr for Key {
    type Err = KeyError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Key::parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, Stroke};

    fn canonical(input: &str) -> String {
        Key::parse(input).unwrap().canonical()
    }

    #[test]
    fn one_key_however_it_is_written() {
        for written in ["ctrl-shift-b", "Ctrl+Shift+B", "shift-ctrl-b", "CTRL+shift+b", "control-shift-b"] {
            assert_eq!(canonical(written), "ctrl-shift-b", "{written}");
        }
        assert_eq!(canonical("cmd-alt-x"), "alt-super-x");
    }

    #[test]
    fn a_separator_at_the_end_is_the_key() {
        assert_eq!(canonical("ctrl--"), "ctrl--");
        assert_eq!(canonical("ctrl-shift--"), "ctrl-shift--");
        assert_eq!(canonical("Ctrl++"), "ctrl-+");
        assert_eq!(canonical("ctrl-+"), "ctrl-+");
        assert_eq!(canonical("ctrl-plus"), "ctrl-+");
        assert_eq!(canonical("-"), "-");
        assert_eq!(canonical("ctrl-,"), "ctrl-,");
    }

    #[test]
    fn named_keys_and_their_aliases() {
        assert_eq!(canonical("ctrl-PgDn"), "ctrl-pagedown");
        assert_eq!(canonical("Esc"), "escape");
        assert_eq!(canonical("shift-Return"), "shift-enter");
        assert_eq!(canonical("F12"), "f12");
        assert_eq!(canonical("ctrl-k ctrl-s"), "ctrl-k ctrl-s");
    }

    #[test]
    fn a_capital_letter_is_the_letter() {
        assert_eq!(canonical("ctrl-B"), "ctrl-b");
        assert_eq!(canonical(&Key::parse("ctrl-shift-b").unwrap().pretty()), "ctrl-shift-b");
    }

    #[test]
    fn what_is_not_a_key_says_why() {
        assert!(Key::parse("").is_err());
        assert!(Key::parse("ctrl-").unwrap_err().why.contains("no key"));
        assert!(Key::parse("hyper-b").unwrap_err().why.contains("not a modifier"));
        assert!(Key::parse("ctrl-pagedwn").unwrap_err().why.contains("no key named"));
        assert!(Key::parse("f99").is_err());
    }

    #[test]
    fn keys_read_as_people_write_them() {
        let pretty = |k: &str| Key::parse(k).unwrap().pretty();
        assert_eq!(pretty("ctrl-shift-b"), "Ctrl+Shift+B");
        assert_eq!(pretty("ctrl-shift--"), "Ctrl+Shift+-");
        assert_eq!(pretty("ctrl-pagedown"), "Ctrl+PgDn");
        assert_eq!(pretty("shift-tab"), "Shift+Tab");
        assert_eq!(pretty("ctrl-k ctrl-s"), "Ctrl+K Ctrl+S");
        assert_eq!(pretty("f5"), "F5");
    }

    #[test]
    fn typing_belongs_to_the_program() {
        let typing = |k: &str| Stroke::parse(k).unwrap().is_typing();
        assert!(typing("a") && typing("shift-a") && typing("space") && typing("enter"));
        assert!(!typing("ctrl-a") && !typing("alt-x") && !typing("tab") && !typing("f5") && !typing("ctrl-enter"));
    }
}
