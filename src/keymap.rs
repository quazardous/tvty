//! The shortcuts: named commands bound to keys in a context. The context
//! follows the focus — `Terminal` (a terminal has it), under `Window` (the
//! rest of tvty) — and the deepest binding wins: a key bound in `Terminal`
//! is the terminal's, a key bound nowhere goes to the program.
//!
//! The defaults are below; `keymap.toml`, in tvty's config directory,
//! overrides them —
//!
//! ```toml
//! [Terminal]
//! ctrl-c = false            # given back to the terminal's program
//!
//! [Window]
//! ctrl-alt-t = "theme.next" # bound anew
//! ```
//!
//! kept by [`crate::config`]: read again when it changes, a wrong one said
//! and the bindings before kept. The keys, the keymap and the file are
//! `tvty-keys`' (no GPUI in it); here are tvty's commands and GPUI's side.

use gpui_kit::*;
use serde::{Deserialize, Serialize};
pub use tvty_keys::{Binding, Command, Key, Keymap, KeymapFile};

use crate::config::{self, Place, Stored};

actions!(
    tvty,
    [
        OpenNotice,
        SliderNext,
        SliderBack,
        TabNext,
        TabBack,
        Gallery,
        TogglePanel,
        ToggleSidebar,
        FilterSessions,
        ToggleOptions,
        FontBigger,
        FontSmaller,
        FontReset,
        NewTicket,
        FullList,
        NextTheme,
        TerminalCopy,
        TerminalPaste,
        SendTab,
        SendBackTab,
    ]
);

/// The key context of the window's content (the shell).
pub const WINDOW: &str = "Window";
/// The key context of a terminal, under the window's.
pub const TERMINAL: &str = "Terminal";

/// Declares the commands: the table `tvty-keys` reads, and the action each
/// one dispatches.
macro_rules! commands {
    ($($name:literal, $context:expr, [$($key:literal),*], $what:literal, $action:ident;)*) => {
        /// Every command, with its default keys. With shift held, a key may
        /// come as its shifted character: both are bound where it matters
        /// (`+`, `_`, `)`).
        pub const COMMANDS: &[Command] = &[
            $(Command { name: $name, context: $context, keys: &[$($key),*], what: $what },)*
        ];

        /// The action a command dispatches.
        fn action(name: &str) -> Option<Box<dyn Action>> {
            match name {
                $($name => Some(Box::new($action)),)*
                _ => None,
            }
        }
    };
}

commands! {
    "notice.open", WINDOW, ["ctrl-enter"], "Go to the newest notification: the agent's terminal, its ticket", OpenNotice;
    "slider.next", WINDOW, ["ctrl-tab"], "Slider: the groups as stacks, most recent first; release Ctrl to open", SliderNext;
    "slider.back", WINDOW, ["ctrl-shift-tab"], "Slider, backwards", SliderBack;
    "tab.next", WINDOW, ["ctrl-pagedown"], "The tab after, in the group shown", TabNext;
    "tab.back", WINDOW, ["ctrl-pageup"], "The tab before, in the group shown", TabBack;
    "gallery.toggle", WINDOW, ["ctrl-shift-space"], "Gallery of every terminal; type to filter, arrows move, Enter opens", Gallery;
    "panel.toggle", WINDOW, ["ctrl-shift-t"], "Fold or unfold the ticket panel", TogglePanel;
    "sidebar.toggle", WINDOW, ["ctrl-shift-b"], "Fold or unfold the projects' list", ToggleSidebar;
    "sessions.filter", WINDOW, ["ctrl-shift-f"], "Filter the sessions: type, Enter opens, arrows move, Esc clears", FilterSessions;
    "options.toggle", WINDOW, ["ctrl-,"], "Options", ToggleOptions;
    "font.bigger", WINDOW, ["ctrl-+", "ctrl-shift-="], "Terminal font bigger", FontBigger;
    "font.smaller", WINDOW, ["ctrl-_", "ctrl-shift--"], "Terminal font smaller", FontSmaller;
    "font.reset", WINDOW, ["ctrl-)", "ctrl-shift-0"], "Terminal font back to its default", FontReset;
    "ticket.new", WINDOW, ["ctrl-shift-n"], "A new ticket (also + in the panel and the full list)", NewTicket;
    "list.full", WINDOW, ["ctrl-shift-l"], "The ticket list, full screen", FullList;
    "theme.next", WINDOW, ["ctrl-shift-k"], "Next colour theme", NextTheme;
    "terminal.copy", TERMINAL, ["ctrl-shift-c"], "Copy the selection", TerminalCopy;
    "terminal.paste", TERMINAL, ["ctrl-shift-v", "shift-insert"], "Paste the clipboard", TerminalPaste;
    "terminal.tab", TERMINAL, ["tab"], "Tab, to the program (not the focus to the next element)", SendTab;
    "terminal.back_tab", TERMINAL, ["shift-tab"], "Shift+Tab, to the program", SendBackTab;
}

/// The contexts, outermost first.
pub const CONTEXTS: &[&str] = &[WINDOW, TERMINAL];

// ── keymap.toml ──────────────────────────────────────────────────────

/// `keymap.toml`: the user's changes, as `tvty-keys` reads them.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Shortcuts(pub KeymapFile);

impl Stored for Shortcuts {
    const PLACE: Place = Place::Config;
    const FILE: &'static str = "keymap.toml";
    const EDITED: bool = true;

    fn check(&self) -> Result<(), String> {
        Keymap::new(COMMANDS, CONTEXTS).load(&self.0).map_err(|e| e.to_string())
    }
}

/// The kit's own bindings, kept to bind them again under tvty's.
struct KitBindings(Vec<KeyBinding>);

impl Global for KitBindings {}

pub fn path() -> Option<std::path::PathBuf> {
    config::path::<Shortcuts>()
}

/// The keymap in force: the defaults, and the file's changes as last read
/// right.
pub fn current(cx: &App) -> Keymap {
    let mut keymap = Keymap::new(COMMANDS, CONTEXTS);
    // A file that does not read is never stored: this cannot fail.
    let _ = keymap.load(&config::get::<Shortcuts>(cx).0);
    keymap
}

/// Binds again: the kit's bindings, then tvty's in force.
fn apply(cx: &mut App) {
    let kit = cx.global::<KitBindings>().0.clone();
    let bindings = key_bindings(&current(cx).bindings());
    cx.clear_key_bindings();
    cx.bind_keys(kit);
    cx.bind_keys(bindings);
}

/// Binds the defaults and `keymap.toml` over them — once the kit has bound
/// its own — and again whenever the file changes.
pub fn init(cx: &mut App) {
    let kit: Vec<KeyBinding> = cx.key_bindings().borrow().bindings().cloned().collect();
    cx.set_global(KitBindings(kit));
    config::register::<Shortcuts>(cx);
    apply(cx);
    cx.observe_global::<config::Store<Shortcuts>>(|cx| {
        apply(cx);
        cx.refresh_windows();
    })
    .detach();
}

/// Every binding off while the options listen for a key: the key pressed
/// comes to them, whatever it runs ([`resume`] binds them again).
pub fn suspend(cx: &mut App) {
    cx.clear_key_bindings();
}

pub fn resume(cx: &mut App) {
    apply(cx);
}

/// Keeps `keymap`'s changes in keymap.toml, the difference from the
/// defaults only; the store binds them.
pub fn save(cx: &mut App, keymap: &Keymap) {
    config::update::<Shortcuts>(cx, |file| file.0 = keymap.file());
}

/// The key a keystroke is; none for a modifier pressed alone.
pub fn key_of(keystroke: &Keystroke) -> Option<Key> {
    if matches!(keystroke.key.as_str(), "shift" | "control" | "alt" | "platform" | "function" | "") {
        return None;
    }
    Key::parse(&keystroke.unparse()).ok()
}

/// The keymap in force, and the file's error if any.
pub fn in_force(cx: &App) -> (Keymap, Option<String>) {
    (current(cx), config::error::<Shortcuts>(cx))
}

/// The bindings as GPUI takes them.
pub fn key_bindings(bindings: &[Binding]) -> Vec<KeyBinding> {
    bindings
        .iter()
        .filter_map(|b| {
            let action: Box<dyn Action> = match b.command {
                Some(name) => action(name)?,
                None => Box::new(NoAction),
            };
            let context = KeyBindingContextPredicate::parse(&b.context).ok()?;
            KeyBinding::load(&b.key.canonical(), action, Some(context.into()), false, None, &DummyKeyboardMapper).ok()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{COMMANDS, CONTEXTS, Keymap, action, key_bindings};

    #[test]
    fn every_default_key_binds() {
        let bindings = Keymap::new(COMMANDS, CONTEXTS).bindings();
        assert_eq!(key_bindings(&bindings).len(), bindings.len());
    }

    #[test]
    fn every_command_has_its_action() {
        assert!(COMMANDS.iter().all(|c| action(c.name).is_some()));
    }
}
