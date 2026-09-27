//! The options page: its sections, the keys that are not commands, and what
//! the pages list as items — tvty's preferences and its shortcuts, for the
//! tree, the search and the "modified" mark.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Appearance,
    Layout,
    TicketList,
    Shortcuts,
    About,
}

impl Section {
    pub const ALL: [Section; 5] = [
        Section::Appearance,
        Section::Layout,
        Section::TicketList,
        Section::Shortcuts,
        Section::About,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Section::Appearance => "Appearance",
            Section::Layout => "Layout",
            Section::TicketList => "Ticket list",
            Section::Shortcuts => "Keyboard shortcuts",
            Section::About => "About",
        }
    }
}

/// The keys that are not commands (see [`crate::keymap`]): those of what
/// is up, and the mouse's. `(keys, what they do)`.
pub const FIXED_KEYS: &[(&str, &str)] = &[
    ("Esc", "Close the gallery, the slider, the theme list, the options, the full list"),
    ("Arrows", "In the slider and the gallery: move to the card seen there"),
    ("Drag · double click · triple click", "In a terminal: select text · a word · a line — copied to the primary selection"),
    ("Middle click", "In a terminal: paste the primary selection"),
    ("Right click", "In a terminal: Copy, Paste"),
    ("Mouse wheel", "Scroll the history"),
];

// ── What the pages list, as items (see `tvty_config::items`) ─────────

use tvty_config::schema::shown;
use tvty_config::{Item, Kind, Provider, Value};

use crate::keymap::{self, Keymap};
use crate::settings::{Preferences, SCHEMA};

/// tvty's preferences, from their schema: one item each.
pub fn program_items(prefs: &Preferences) -> Vec<Item> {
    SCHEMA
        .0
        .iter()
        .filter(|s| !matches!(s.kind, Kind::Action { .. }))
        .map(|s| {
            let value = match (s.kind, SCHEMA.value(prefs, s.key)) {
                (Kind::Number { unit, .. }, Some(Value::Number(n))) => shown(n, unit),
                (Kind::Toggle { on, off, .. }, Some(Value::Toggle(checked))) => (if checked { on } else { off }).to_string(),
                (Kind::Choice, Some(Value::Choice(choice))) => choice.unwrap_or_else(|| "default".into()),
                _ => String::new(),
            };
            Item {
                provider: Provider::Program,
                key: s.key.to_string(),
                page: s.page.to_string(),
                group: s.group.to_string(),
                label: s.label.to_string(),
                about: s.about.to_string(),
                value,
                modified: SCHEMA.is_modified(prefs, s.key),
                protected: false,
                inherited: false,
                words: Vec::new(),
            }
        })
        .collect()
}

/// The page and groups the shortcuts sit in.
pub const SHORTCUTS_PAGE: &str = "Keyboard shortcuts";

pub fn context_group(context: &str) -> &'static str {
    if context == keymap::TERMINAL { "Terminal" } else { "Window" }
}

/// The shortcuts: one item per command, found by its name, what it does and
/// its keys (as shown and as written in keymap.toml).
pub fn shortcut_items(map: &Keymap) -> Vec<Item> {
    map.commands()
        .iter()
        .map(|c| {
            let keys = map.keys_of(c.name);
            Item {
                provider: Provider::Shortcuts,
                key: c.name.to_string(),
                page: SHORTCUTS_PAGE.to_string(),
                group: context_group(c.context).to_string(),
                label: c.what.to_string(),
                about: String::new(),
                value: keys.iter().map(|k| k.pretty()).collect::<Vec<_>>().join(", "),
                modified: map.is_changed(c.name),
                protected: false,
                inherited: false,
                words: std::iter::once(c.name.to_string()).chain(keys.iter().map(|k| k.canonical())).collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{program_items, shortcut_items};
    use crate::keymap::{COMMANDS, CONTEXTS, Key, Keymap};
    use crate::settings::{Preferences, SCHEMA};
    use tvty_config::{Query, search};

    #[test]
    fn every_preference_is_an_item_and_says_when_it_is_modified() {
        let mut prefs = Preferences::default();
        let items = program_items(&prefs);
        assert_eq!(items.len(), SCHEMA.0.len());
        assert!(items.iter().all(|i| !i.modified));
        prefs.appearance.terminal_font_size = Some(20.);
        let items = program_items(&prefs);
        let font = items.iter().find(|i| i.key == "appearance.terminal_font_size").unwrap();
        assert!(font.modified);
        assert_eq!(font.value, "20 px");
    }

    #[test]
    fn a_shortcut_is_found_by_its_key_or_its_name() {
        let mut map = Keymap::new(COMMANDS, CONTEXTS);
        let items = shortcut_items(&map);
        let found = |items: &[tvty_config::Item], q: &str| search(items, &Query::parse(q)).into_iter().map(|i| i.key.clone()).collect::<Vec<_>>();
        assert_eq!(found(&items, "ctrl-shift-b"), ["sidebar.toggle"]);
        assert_eq!(found(&items, "Ctrl+Shift+B"), ["sidebar.toggle"]);
        assert!(found(&items, "sidebar").contains(&"sidebar.toggle".to_string()));
        map.bind("theme.next", &Key::parse("ctrl-alt-t").unwrap()).unwrap();
        let items = shortcut_items(&map);
        assert_eq!(found(&items, "@modified"), ["theme.next"]);
    }
}
