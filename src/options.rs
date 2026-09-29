//! The options page: its sections, the keys that are not commands, and what
//! the pages list as items — tvty's preferences and its shortcuts, for the
//! tree, the search and the "modified" mark.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    /// A project's own settings (its folders' `.aiball.yaml`, the board
    /// keys it overrides): shown when the options are scoped to a project.
    Project,
    Appearance,
    Layout,
    TicketList,
    Shortcuts,
    /// aiball's own config, as the daemon serves it.
    Aiball,
    About,
}

impl Section {
    pub const ALL: [Section; 7] = [
        Section::Project,
        Section::Appearance,
        Section::Layout,
        Section::TicketList,
        Section::Shortcuts,
        Section::Aiball,
        Section::About,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Section::Project => "Project",
            Section::Appearance => "Appearance",
            Section::Layout => "Layout",
            Section::TicketList => "Ticket list",
            Section::Shortcuts => "Keyboard shortcuts",
            Section::Aiball => "aiball",
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
    ("Right click", "In a terminal: Copy, Paste — on a link, Open link, Copy link"),
    ("Ctrl+click", "In a terminal, on a link: open it"),
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

/// A dotted section as a heading: `tickets.wait_credit.earn` →
/// `Tickets › Wait credit › Earn`.
pub fn group_title(group: &str) -> String {
    group
        .split('.')
        .map(|part| {
            let words = part.replace('_', " ");
            let mut chars = words.chars();
            chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" › ")
}

/// A value of aiball's config as shown: `1h30m`, `500 characters`, `on`.
pub fn remote_value(entry: &crate::aiball::ConfigEntry, value: &serde_json::Value) -> String {
    match (entry.kind.as_str(), value) {
        (_, serde_json::Value::Null) => "—".into(),
        ("duration", v) => v.as_u64().map(tvty_config::format_duration).unwrap_or_else(|| v.to_string()),
        ("boolean", v) => if v.as_bool() == Some(true) { "on".into() } else { "off".into() },
        ("number", v) => {
            let n = v.as_f64().unwrap_or_default();
            shown(n, entry.unit.as_deref().unwrap_or(""))
        }
        (_, serde_json::Value::String(text)) => text.clone(),
        (_, v) => v.to_string(),
    }
}

/// aiball's config as items, in the layer it was read in (the board's, or a
/// project's): set in that layer is "modified"; a project's value that
/// comes from the board is "inherited".
pub fn remote_items(config: &crate::aiball::ManagedConfig) -> Vec<Item> {
    let in_project = config.project.is_some();
    config
        .config
        .iter()
        .map(|entry| {
            let own = if in_project { &entry.project } else { &entry.global };
            Item {
                provider: Provider::Remote,
                key: entry.key.clone(),
                page: Section::Aiball.title().to_string(),
                group: group_title(entry.group.as_deref().unwrap_or("")),
                label: entry.label.clone(),
                about: entry.description.clone(),
                value: remote_value(entry, &entry.value),
                modified: !own.is_null(),
                protected: entry.protected,
                inherited: in_project && own.is_null(),
                words: entry.options.clone().unwrap_or_default(),
            }
        })
        .collect()
}

/// The page and groups the shortcuts sit in.
pub const SHORTCUTS_PAGE: &str = "Keyboard shortcuts";

pub fn context_group(context: &str) -> &'static str {
    match context {
        keymap::TERMINAL => "Terminal",
        keymap::WORKSPACE => "Workspace",
        _ => "Window",
    }
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
    fn aiballs_config_reads_as_items() {
        let config: crate::aiball::ManagedConfig = serde_json::from_value(serde_json::json!({
            "project": "demo",
            "config": [
                { "key": "tickets.steps.stale", "scope": "global+project", "type": "duration", "options": null, "protected": false,
                  "label": "Stale step", "description": "When a step goes quiet.", "group": "tickets.steps",
                  "min": 3600, "max": 2592000, "step": 3600, "unit": null,
                  "default": 86400, "global": 172800, "project": null, "value": 172800 },
                { "key": "tickets.rules.summary_max", "scope": "global+project", "type": "number", "options": null, "protected": true,
                  "label": "Summary budget", "description": "", "group": "tickets.rules",
                  "min": 0, "max": 5000, "step": 50, "unit": "characters",
                  "default": 500, "global": null, "project": 300, "value": 300 },
                { "key": "autopoll.tone", "scope": "project", "type": "enum", "options": ["hint", "directive"], "protected": false,
                  "label": "Tone", "description": "", "group": "autopoll", "min": null, "max": null, "step": null, "unit": null,
                  "default": "directive", "global": null, "project": null, "value": "directive", "sources": ["file"] }
            ]
        }))
        .unwrap();
        let items = super::remote_items(&config);
        assert_eq!(items[0].group, "Tickets › Steps");
        assert_eq!(items[0].value, "2d");
        assert!(items[0].inherited && !items[0].modified);
        assert_eq!(items[1].value, "300 characters");
        assert!(items[1].modified && items[1].protected && !items[1].inherited);
        // What config.set can write: the store's keys, not a file's.
        assert!(config.config[0].writable() && config.config[0].settable_in(true));
        assert!(!config.config[2].writable() && !config.config[2].settable_in(true));
        assert_eq!(super::group_title("tickets.wait_credit.earn"), "Tickets › Wait credit › Earn");
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
