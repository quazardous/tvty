//! Commands bound to keys in contexts: the defaults a program declares, what
//! the user changed over them, and what is in force.
//!
//! Contexts are nested, outermost first (`Window`, then `Terminal` inside
//! it): a key bound in an inner context wins there, and masks the outer
//! binding of the same key.
//!
//! What the user changed is kept as the difference from the defaults only —
//! a key bound anew, or a default key given back to what has the focus —
//! and that difference is the file ([`KeymapFile`]).

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::key::Key;

/// A command a program offers: its name (what the file says), the context
/// it is bound in, its keys by default, and what it does.
#[derive(Clone, Copy, Debug)]
pub struct Command {
    pub name: &'static str,
    pub context: &'static str,
    pub keys: &'static [&'static str],
    pub what: &'static str,
}

/// One binding in force.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub context: String,
    pub key: Key,
    /// None: the key is given back to what has the focus.
    pub command: Option<&'static str>,
    /// Changed by the user rather than a default.
    pub custom: bool,
}

/// What is wrong with a keymap file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeymapError(pub String);

impl fmt::Display for KeymapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for KeymapError {}

/// The file: by context, a key bound to a command's name, or `false` to give
/// it back.
///
/// ```toml
/// [Terminal]
/// ctrl-c = false
///
/// [Window]
/// ctrl-alt-t = "theme.next"
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeymapFile(pub BTreeMap<String, BTreeMap<String, Target>>);

/// What a key is bound to in the file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Target {
    Command(String),
    /// `false`: given back. `true` means nothing and is refused.
    Bound(bool),
}

impl KeymapFile {
    pub fn from_toml(text: &str) -> Result<Self, KeymapError> {
        toml::from_str(text).map_err(|e| KeymapError(e.message().to_string()))
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }
}

/// The keymap: the commands and their defaults, and the user's changes.
#[derive(Clone, Debug)]
pub struct Keymap {
    commands: &'static [Command],
    /// Outermost first.
    contexts: &'static [&'static str],
    /// The defaults, parsed once: (context, key) → command.
    defaults: Vec<(String, Key, &'static str)>,
    /// The user's changes: (context, key) → a command, or given back.
    changes: BTreeMap<(String, Key), Option<&'static str>>,
}

impl Keymap {
    /// The defaults of `commands`, in `contexts` (outermost first). A default
    /// key that does not parse is the program's mistake: it panics.
    pub fn new(commands: &'static [Command], contexts: &'static [&'static str]) -> Self {
        let defaults = commands
            .iter()
            .flat_map(|c| {
                c.keys.iter().map(move |k| {
                    let key = Key::parse(k).unwrap_or_else(|e| panic!("command {}: {e}", c.name));
                    (c.context.to_string(), key, c.name)
                })
            })
            .collect();
        Keymap { commands, contexts, defaults, changes: BTreeMap::new() }
    }

    pub fn commands(&self) -> &'static [Command] {
        self.commands
    }

    pub fn contexts(&self) -> &'static [&'static str] {
        self.contexts
    }

    pub fn command(&self, name: &str) -> Option<&'static Command> {
        self.commands.iter().find(|c| c.name == name)
    }

    fn default_of(&self, context: &str, key: &Key) -> Option<&'static str> {
        self.defaults.iter().find(|(c, k, _)| c == context && k == key).map(|(_, _, name)| *name)
    }

    /// The command `key` runs in `context` (not counting inner contexts);
    /// None when nothing, or when it is given back.
    pub fn bound(&self, context: &str, key: &Key) -> Option<&'static str> {
        match self.changes.get(&(context.to_string(), key.clone())) {
            Some(change) => *change,
            None => self.default_of(context, key),
        }
    }

    /// Every binding in force: the defaults in the commands' order, then the
    /// user's own keys.
    pub fn bindings(&self) -> Vec<Binding> {
        let mut out: Vec<Binding> = Vec::new();
        for (context, key, name) in &self.defaults {
            match self.changes.get(&(context.clone(), key.clone())) {
                None => out.push(Binding { context: context.clone(), key: key.clone(), command: Some(name), custom: false }),
                Some(change) => out.push(Binding { context: context.clone(), key: key.clone(), command: *change, custom: true }),
            }
        }
        for ((context, key), change) in &self.changes {
            if self.default_of(context, key).is_none() {
                out.push(Binding { context: context.clone(), key: key.clone(), command: *change, custom: true });
            }
        }
        out
    }

    /// The keys that run `command` now.
    pub fn keys_of(&self, command: &str) -> Vec<Key> {
        self.bindings().into_iter().filter(|b| b.command == Some(command)).map(|b| b.key).collect()
    }

    /// The keys that run `command` by default.
    pub fn default_keys_of(&self, command: &str) -> Vec<Key> {
        self.defaults.iter().filter(|(_, _, name)| *name == command).map(|(_, key, _)| key.clone()).collect()
    }

    /// Whether the keys of `command` differ from its defaults.
    pub fn is_changed(&self, command: &str) -> bool {
        // A key of its own, or a default key taken from it (given back, or
        // bound to another command).
        self.changes.iter().any(|((context, key), change)| *change == Some(command) || self.default_of(context, key) == Some(command))
    }

    fn set(&mut self, context: &str, key: &Key, to: Option<&'static str>) {
        let slot = (context.to_string(), key.clone());
        if self.default_of(context, key) == to && to.is_some() {
            self.changes.remove(&slot);
        } else {
            self.changes.insert(slot, to);
        }
    }

    /// `key` runs `command` in the command's context — whatever it ran
    /// before there (see [`Keymap::conflict`]).
    pub fn bind(&mut self, command: &str, key: &Key) -> Result<(), KeymapError> {
        let command = self.command(command).ok_or_else(|| KeymapError(format!("no command {command:?}")))?;
        self.set(command.context, key, Some(command.name));
        Ok(())
    }

    /// `key` no longer runs anything in `context`: a default key is given
    /// back to what has the focus, a key of the user's is forgotten.
    pub fn unbind(&mut self, context: &str, key: &Key) {
        let slot = (context.to_string(), key.clone());
        if self.default_of(context, key).is_some() {
            self.changes.insert(slot, None);
        } else {
            self.changes.remove(&slot);
        }
    }

    /// `key` given back to what has the focus in `context`, even where it
    /// has no default there: it then masks an outer context's binding.
    pub fn give_back(&mut self, context: &str, key: &Key) {
        self.changes.insert((context.to_string(), key.clone()), None);
    }

    /// `old` replaced by `new` for `command`.
    pub fn rebind(&mut self, command: &str, old: &Key, new: &Key) -> Result<(), KeymapError> {
        let context = self.command(command).ok_or_else(|| KeymapError(format!("no command {command:?}")))?.context;
        self.unbind(context, old);
        self.bind(command, new)
    }

    /// `key` in `context` back to its default (or to nothing).
    pub fn restore(&mut self, context: &str, key: &Key) {
        self.changes.remove(&(context.to_string(), key.clone()));
    }

    /// `command`'s keys back to its defaults.
    pub fn reset(&mut self, command: &str) {
        let defaults = &self.defaults;
        self.changes.retain(|(context, key), change| {
            let default = defaults.iter().find(|(c, k, _)| c == context && k == key).map(|(_, _, n)| *n);
            !(*change == Some(command) || default == Some(command))
        });
    }

    pub fn reset_all(&mut self) {
        self.changes.clear();
    }

    /// What `key` would take from, bound to `command`: the command it runs
    /// now in the same context, if another.
    pub fn conflict(&self, command: &str, key: &Key) -> Option<&'static str> {
        let context = self.command(command)?.context;
        self.bound(context, key).filter(|other| *other != command)
    }

    /// A key bound in an outer context and bound (or given back) in an
    /// inner one too, where the inner binding wins: (outer, inner).
    pub fn masked(&self) -> Vec<(Binding, Binding)> {
        let bindings = self.bindings();
        let depth = |context: &str| self.contexts.iter().position(|c| *c == context).unwrap_or(0);
        bindings
            .iter()
            .filter(|outer| outer.command.is_some())
            .filter_map(|outer| {
                bindings
                    .iter()
                    .find(|inner| inner.key == outer.key && depth(&inner.context) > depth(&outer.context))
                    .map(|inner| (outer.clone(), inner.clone()))
            })
            .collect()
    }

    /// The user's changes read from the file; what is wrong with it named.
    pub fn load(&mut self, file: &KeymapFile) -> Result<(), KeymapError> {
        let mut changes = BTreeMap::new();
        for (context, keys) in &file.0 {
            if !self.contexts.contains(&context.as_str()) {
                return Err(KeymapError(format!("unknown context [{context}] ({})", self.contexts.iter().map(|c| format!("[{c}]")).collect::<Vec<_>>().join(" or "))));
            }
            for (written, target) in keys {
                let key = Key::parse(written).map_err(|e| KeymapError(format!("[{context}] {e}")))?;
                let to = match target {
                    Target::Bound(false) => None,
                    Target::Command(name) => {
                        let command = self.command(name).ok_or_else(|| KeymapError(format!("[{context}] {written}: no command {name:?}")))?;
                        if command.context != context {
                            return Err(KeymapError(format!("[{context}] {written}: {name} is bound in [{}]", command.context)));
                        }
                        Some(command.name)
                    }
                    Target::Bound(true) => return Err(KeymapError(format!("[{context}] {written}: a command's name or false is expected"))),
                };
                changes.insert((context.clone(), key), to);
            }
        }
        self.changes = changes;
        // What merely repeats a default is no change (a key given back where
        // it has no default is one: it masks an outer binding).
        let defaults = self.defaults.clone();
        self.changes.retain(|(context, key), to| {
            let default = defaults.iter().find(|(c, k, _)| c == context && k == key).map(|(_, _, n)| *n);
            default.is_none() || default != *to
        });
        Ok(())
    }

    /// The user's changes as the file keeps them: only the difference from
    /// the defaults, keys in their canonical form.
    pub fn file(&self) -> KeymapFile {
        let mut file = KeymapFile::default();
        for ((context, key), to) in &self.changes {
            let target = match to {
                Some(name) => Target::Command(name.to_string()),
                None => Target::Bound(false),
            };
            file.0.entry(context.clone()).or_default().insert(key.canonical(), target);
        }
        file
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, Keymap, KeymapFile};
    use crate::key::Key;

    const COMMANDS: &[Command] = &[
        Command { name: "sidebar.toggle", context: "Window", keys: &["ctrl-shift-b"], what: "" },
        Command { name: "theme.next", context: "Window", keys: &["ctrl-shift-k"], what: "" },
        Command { name: "font.bigger", context: "Window", keys: &["ctrl-+", "ctrl-shift-="], what: "" },
        Command { name: "terminal.copy", context: "Terminal", keys: &["ctrl-shift-c"], what: "" },
    ];
    const CONTEXTS: &[&str] = &["Window", "Terminal"];

    fn key(k: &str) -> Key {
        Key::parse(k).unwrap()
    }

    fn keymap() -> Keymap {
        Keymap::new(COMMANDS, CONTEXTS)
    }

    #[test]
    fn the_defaults_are_in_force_and_the_file_empty() {
        let map = keymap();
        assert_eq!(map.bound("Window", &key("Ctrl+Shift+B")), Some("sidebar.toggle"));
        assert_eq!(map.keys_of("font.bigger").len(), 2);
        assert_eq!(map.file(), KeymapFile::default());
    }

    #[test]
    fn a_key_bound_anew_and_one_given_back() {
        let mut map = keymap();
        map.bind("theme.next", &key("ctrl-alt-t")).unwrap();
        map.unbind("Window", &key("ctrl-shift-b"));
        assert_eq!(map.bound("Window", &key("ctrl-alt-t")), Some("theme.next"));
        assert_eq!(map.bound("Window", &key("ctrl-shift-b")), None);
        assert!(map.is_changed("sidebar.toggle") && map.is_changed("theme.next"));
        assert_eq!(map.file().to_toml(), "[Window]\nctrl-alt-t = \"theme.next\"\nctrl-shift-b = false\n");
    }

    #[test]
    fn the_defaults_stay_known_once_changed() {
        let mut map = keymap();
        map.unbind("Window", &key("ctrl-shift-b"));
        map.bind("sidebar.toggle", &key("ctrl-alt-b")).unwrap();
        assert_eq!(map.default_keys_of("sidebar.toggle"), vec![key("ctrl-shift-b")]);
        assert_eq!(map.default_keys_of("font.bigger").len(), 2);
        assert!(map.default_keys_of("no.such").is_empty());
    }

    #[test]
    fn a_key_rebound_to_its_default_leaves_the_file() {
        let mut map = keymap();
        map.rebind("sidebar.toggle", &key("ctrl-shift-b"), &key("ctrl-alt-b")).unwrap();
        assert_eq!(map.keys_of("sidebar.toggle"), vec![key("ctrl-alt-b")]);
        map.rebind("sidebar.toggle", &key("ctrl-alt-b"), &key("ctrl-shift-b")).unwrap();
        assert_eq!(map.file(), KeymapFile::default());
    }

    #[test]
    fn a_command_reset_drops_its_changes_only() {
        let mut map = keymap();
        map.unbind("Window", &key("ctrl-+"));
        map.bind("font.bigger", &key("ctrl-up")).unwrap();
        map.bind("theme.next", &key("ctrl-alt-t")).unwrap();
        map.reset("font.bigger");
        assert!(!map.is_changed("font.bigger"));
        assert_eq!(map.keys_of("font.bigger").len(), 2);
        assert!(map.is_changed("theme.next"));
        map.reset_all();
        assert_eq!(map.file(), KeymapFile::default());
    }

    #[test]
    fn a_key_taken_by_another_command_changes_both() {
        let mut map = keymap();
        map.bind("theme.next", &key("ctrl-shift-b")).unwrap();
        assert!(map.is_changed("theme.next") && map.is_changed("sidebar.toggle"));
        assert!(map.keys_of("sidebar.toggle").is_empty());
        // Its reset takes its key back.
        map.reset("sidebar.toggle");
        assert_eq!(map.keys_of("sidebar.toggle"), vec![key("ctrl-shift-b")]);
        assert_eq!(map.file(), KeymapFile::default());
    }

    #[test]
    fn a_conflict_names_the_command_it_takes_from() {
        let map = keymap();
        assert_eq!(map.conflict("theme.next", &key("ctrl-shift-b")), Some("sidebar.toggle"));
        assert_eq!(map.conflict("theme.next", &key("ctrl-shift-k")), None);
        // Another context: no conflict.
        assert_eq!(map.conflict("terminal.copy", &key("ctrl-shift-b")), None);
    }

    #[test]
    fn an_inner_context_masks_an_outer_key() {
        let mut map = keymap();
        map.unbind("Terminal", &key("ctrl-shift-c"));
        assert!(map.masked().is_empty());
        map.load(&KeymapFile::from_toml("[Terminal]\nctrl-shift-b = false\n").unwrap()).unwrap();
        let masked = map.masked();
        assert_eq!(masked.len(), 1);
        assert_eq!(masked[0].0.command, Some("sidebar.toggle"));
    }

    #[test]
    fn the_file_reads_back_as_it_was_written() {
        let mut map = keymap();
        map.load(&KeymapFile::from_toml("[Window]\n\"Ctrl+Alt+T\" = \"theme.next\"\nctrl-shift-k = \"theme.next\"\n").unwrap()).unwrap();
        // A change that repeats the default is dropped; keys canonical.
        assert_eq!(map.file().to_toml(), "[Window]\nctrl-alt-t = \"theme.next\"\n");
    }

    #[test]
    fn a_wrong_file_says_where() {
        let load = |text: &str| keymap().load(&KeymapFile::from_toml(text).unwrap()).unwrap_err().0;
        assert!(load("[Panel]\n").contains("unknown context [Panel]"));
        assert!(load("[Window]\nctrl-shift-b = \"no.such\"\n").contains("[Window] ctrl-shift-b: no command"));
        assert!(load("[Window]\nhyper-b = false\n").contains("[Window]"));
        assert!(load("[Window]\nctrl-b = true\n").contains("or false"));
        assert!(load("[Window]\nctrl-b = \"terminal.copy\"\n").contains("bound in [Terminal]"));
        assert!(KeymapFile::from_toml("[Window").is_err());
    }
}
