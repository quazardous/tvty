//! The shortcuts: named commands bound to keys in a context. The context
//! follows the focus — `Terminal` (a terminal has it), under `Window` (the
//! rest of tvty) — and the deepest binding wins: a key bound in `Terminal`
//! is the terminal's, a key bound nowhere goes to the program.
//!
//! The defaults are below; `keymap.json`, in tvty's config directory,
//! overrides them — `{ "Terminal": { "ctrl-c": null }, "Window": {
//! "ctrl-alt-t": "theme.next" } }`: a key bound anew, or `null`, given back
//! to what has the focus. The file is read again when it changes.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use gpui_kit::*;

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

/// A command: what `keymap.json` names, where it is bound, its keys by
/// default, and what it does.
pub struct Command {
    pub name: &'static str,
    pub context: &'static str,
    pub keys: &'static [&'static str],
    pub what: &'static str,
    pub action: fn() -> Box<dyn Action>,
}

macro_rules! command {
    ($name:literal, $context:expr, [$($key:literal),*], $what:literal, $action:ident) => {
        Command { name: $name, context: $context, keys: &[$($key),*], what: $what, action: || Box::new($action) }
    };
}

/// Every command, with its default keys. With shift held, a key may come as
/// its shifted character: both are bound where it matters (`+`, `_`, `)`).
pub const COMMANDS: &[Command] = &[
    command!("notice.open", WINDOW, ["ctrl-enter"], "Go to the newest notification: the agent's terminal, its ticket", OpenNotice),
    command!("slider.next", WINDOW, ["ctrl-tab"], "Slider: the groups as stacks, most recent first; release Ctrl to open", SliderNext),
    command!("slider.back", WINDOW, ["ctrl-shift-tab"], "Slider, backwards", SliderBack),
    command!("tab.next", WINDOW, ["ctrl-pagedown"], "The tab after, in the group shown", TabNext),
    command!("tab.back", WINDOW, ["ctrl-pageup"], "The tab before, in the group shown", TabBack),
    command!("gallery.toggle", WINDOW, ["ctrl-shift-space"], "Gallery of every terminal; type to filter, arrows move, Enter opens", Gallery),
    command!("panel.toggle", WINDOW, ["ctrl-shift-t"], "Fold or unfold the ticket panel", TogglePanel),
    command!("sidebar.toggle", WINDOW, ["ctrl-shift-b"], "Fold or unfold the projects' list", ToggleSidebar),
    command!("options.toggle", WINDOW, ["ctrl-,"], "Options", ToggleOptions),
    command!("font.bigger", WINDOW, ["ctrl-+", "ctrl-shift-="], "Terminal font bigger", FontBigger),
    command!("font.smaller", WINDOW, ["ctrl-_", "ctrl-shift--"], "Terminal font smaller", FontSmaller),
    command!("font.reset", WINDOW, ["ctrl-)", "ctrl-shift-0"], "Terminal font back to its default", FontReset),
    command!("ticket.new", WINDOW, ["ctrl-shift-n"], "A new ticket (also + in the panel and the full list)", NewTicket),
    command!("list.full", WINDOW, ["ctrl-shift-l"], "The ticket list, full screen", FullList),
    command!("theme.next", WINDOW, ["ctrl-shift-k"], "Next colour theme", NextTheme),
    command!("terminal.copy", TERMINAL, ["ctrl-shift-c"], "Copy the selection", TerminalCopy),
    command!("terminal.paste", TERMINAL, ["ctrl-shift-v", "shift-insert"], "Paste the clipboard", TerminalPaste),
    command!("terminal.tab", TERMINAL, ["tab"], "Tab, to the program (not the focus to the next element)", SendTab),
    command!("terminal.back_tab", TERMINAL, ["shift-tab"], "Shift+Tab, to the program", SendBackTab),
];

// ── keymap.json ──────────────────────────────────────────────────────

/// The keymap in force: the kit's own bindings (kept to bind them again),
/// the overrides read from the file, and what was wrong with it.
pub struct Keymap {
    kit: Vec<KeyBinding>,
    pub overrides: Vec<Binding>,
    pub error: Option<String>,
    stamp: Option<SystemTime>,
}

impl Global for Keymap {}

/// `keymap.json`, beside tvty's state (`$XDG_CONFIG_HOME/tvty`).
pub fn path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("tvty").join("keymap.json"))
}

fn stamp() -> Option<SystemTime> {
    std::fs::metadata(path()?).and_then(|m| m.modified()).ok()
}

/// The overrides a `keymap.json` says, or what is wrong with it: an unknown
/// context or command, a key that does not parse.
pub fn parse(text: &str) -> Result<Vec<Binding>, String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
    let contexts = value.as_object().ok_or("a map of contexts is expected: { \"Terminal\": { … } }")?;
    let mut bindings = Vec::new();
    for (context, keys) in contexts {
        if context != WINDOW && context != TERMINAL {
            return Err(format!("unknown context {context:?} (Window or Terminal)"));
        }
        let keys = keys.as_object().ok_or(format!("{context}: a map of keys is expected"))?;
        for (key, command) in keys {
            if key.split_whitespace().any(|k| Keystroke::parse(k).is_err()) || key.trim().is_empty() {
                return Err(format!("{context}: {key:?} is not a key (ctrl-shift-b, alt-enter…)"));
            }
            let command = match command {
                serde_json::Value::Null => None,
                serde_json::Value::String(name) => Some(self::command(name).ok_or(format!("{context}: {key}: no command {name:?}"))?.name),
                _ => return Err(format!("{context}: {key}: a command name or null is expected")),
            };
            bindings.push(Binding { context: context.clone(), key: key.clone(), command, custom: true });
        }
    }
    Ok(bindings)
}

/// The file's overrides (none when there is no file), or its error.
fn read() -> Result<Vec<Binding>, String> {
    let Some(path) = path() else { return Ok(Vec::new()) };
    match std::fs::read_to_string(&path) {
        Ok(text) => parse(&text).map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Binds again: the kit's bindings, then tvty's in force.
fn apply(cx: &mut App, kit: &[KeyBinding], overrides: &[Binding]) {
    cx.clear_key_bindings();
    cx.bind_keys(kit.iter().cloned());
    cx.bind_keys(key_bindings(&effective(overrides)));
}

/// Binds the defaults and `keymap.json` over them — once the kit has bound
/// its own — and reads the file again whenever it changes.
pub fn init(cx: &mut App) {
    let kit: Vec<KeyBinding> = cx.key_bindings().borrow().bindings().cloned().collect();
    let (overrides, error) = match read() {
        Ok(overrides) => (overrides, None),
        Err(error) => (Vec::new(), Some(error)),
    };
    apply(cx, &kit, &overrides);
    cx.set_global(Keymap { kit, overrides, error, stamp: stamp() });
    cx.spawn(async move |cx| loop {
        cx.background_executor().timer(Duration::from_secs(2)).await;
        cx.update(reload_if_changed);
    })
    .detach();
}

/// The file changed: its bindings in force now, or, when it is wrong, the
/// ones before kept and the error said.
fn reload_if_changed(cx: &mut App) {
    let now = stamp();
    if cx.global::<Keymap>().stamp == now {
        return;
    }
    let kit = cx.global::<Keymap>().kit.clone();
    match read() {
        Ok(overrides) => {
            apply(cx, &kit, &overrides);
            let keymap = cx.global_mut::<Keymap>();
            keymap.overrides = overrides;
            keymap.error = None;
            keymap.stamp = now;
            crate::activity::publish(cx, crate::activity::Activity::done(None, "keymap.json: shortcuts updated"));
        }
        Err(error) => {
            let keymap = cx.global_mut::<Keymap>();
            keymap.error = Some(error.clone());
            keymap.stamp = now;
            crate::activity::publish(cx, crate::activity::Activity::failed(None, "keymap.json", error));
        }
    }
    cx.refresh_windows();
}

/// The bindings in force, and the file's error if any.
pub fn in_force(cx: &App) -> (Vec<Binding>, Option<String>) {
    let keymap = cx.global::<Keymap>();
    (effective(&keymap.overrides), keymap.error.clone())
}

/// A key as the options show it: `ctrl-shift-b` → `Ctrl+Shift+B`.
pub fn pretty(key: &str) -> String {
    key.split_whitespace()
        .map(|stroke| {
            // A last `-` alone is the minus key (`ctrl--`).
            let (mods, last) = match stroke.strip_suffix("--") {
                Some(mods) => (mods, "-"),
                None => stroke.rsplit_once('-').unwrap_or(("", stroke)),
            };
            let mut parts: Vec<String> = mods
                .split('-')
                .filter(|m| !m.is_empty())
                .map(|m| {
                    let mut c = m.chars();
                    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
                })
                .collect();
            let last = match last {
                "pageup" => "PgUp".to_string(),
                "pagedown" => "PgDn".to_string(),
                "enter" => "Enter".to_string(),
                "tab" => "Tab".to_string(),
                "space" => "Space".to_string(),
                "escape" => "Esc".to_string(),
                k if k.chars().count() == 1 => k.to_uppercase(),
                k => k.to_string(),
            };
            parts.push(last);
            parts.join("+")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A command by the name `keymap.json` gives it.
pub fn command(name: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.name == name)
}

/// One binding in force: a key, in a context, to a command — or to none
/// (`null` in `keymap.json`: the key goes to what has the focus).
#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub context: String,
    pub key: String,
    pub command: Option<&'static str>,
    /// From `keymap.json` rather than the defaults.
    pub custom: bool,
}

/// The bindings in force: the defaults, then `overrides` over them (a key
/// in a context bound anew, or to nothing).
pub fn effective(overrides: &[Binding]) -> Vec<Binding> {
    let mut bindings: Vec<Binding> = COMMANDS
        .iter()
        .flat_map(|c| {
            c.keys.iter().map(move |key| Binding { context: c.context.into(), key: (*key).into(), command: Some(c.name), custom: false })
        })
        .collect();
    for over in overrides {
        bindings.retain(|b| !(b.context == over.context && b.key == over.key));
        bindings.push(over.clone());
    }
    bindings
}

/// The bindings as GPUI takes them.
pub fn key_bindings(bindings: &[Binding]) -> Vec<KeyBinding> {
    bindings
        .iter()
        .filter_map(|b| {
            let action: Box<dyn Action> = match b.command {
                Some(name) => (command(name)?.action)(),
                None => Box::new(NoAction),
            };
            let context = KeyBindingContextPredicate::parse(&b.context).ok()?;
            KeyBinding::load(&b.key, action, Some(context.into()), false, None, &DummyKeyboardMapper).ok()
        })
        .collect()
}

/// A key bound to a command in `context`, masked where a deeper context
/// binds it too (a `Window` key a terminal takes).
pub fn masked(bindings: &[Binding]) -> Vec<(&Binding, &Binding)> {
    let deeper = |c: &str| c == TERMINAL;
    bindings
        .iter()
        .filter(|b| b.context == WINDOW && b.command.is_some())
        .filter_map(|b| bindings.iter().find(|d| deeper(&d.context) && d.key == b.key).map(|d| (b, d)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Binding, TERMINAL, WINDOW, effective, key_bindings, masked, parse, pretty};

    #[test]
    fn every_default_key_parses() {
        let bindings = effective(&[]);
        assert_eq!(key_bindings(&bindings).len(), bindings.len());
    }

    #[test]
    fn an_override_replaces_a_key_or_frees_it() {
        let overrides = [
            Binding { context: WINDOW.into(), key: "ctrl-shift-k".into(), command: None, custom: true },
            Binding { context: WINDOW.into(), key: "ctrl-alt-t".into(), command: Some("theme.next"), custom: true },
        ];
        let bindings = effective(&overrides);
        let at = |key: &str| bindings.iter().find(|b| b.context == WINDOW && b.key == key).map(|b| b.command);
        assert_eq!(at("ctrl-shift-k"), Some(None));
        assert_eq!(at("ctrl-alt-t"), Some(Some("theme.next")));
        // The others keep their default.
        assert_eq!(at("ctrl-shift-b"), Some(Some("sidebar.toggle")));
    }

    #[test]
    fn the_file_says_keys_anew_or_frees_them() {
        let bindings = parse(r#"{ "Terminal": { "ctrl-c": null }, "Window": { "ctrl-alt-t": "theme.next" } }"#).unwrap();
        assert_eq!(bindings.len(), 2);
        assert!(bindings.iter().any(|b| b.context == TERMINAL && b.key == "ctrl-c" && b.command.is_none()));
        assert!(bindings.iter().any(|b| b.context == WINDOW && b.command == Some("theme.next")));
    }

    #[test]
    fn a_wrong_file_says_what_is_wrong() {
        assert!(parse("{").unwrap_err().contains("not JSON"));
        assert!(parse(r#"{ "Panel": {} }"#).unwrap_err().contains("unknown context"));
        assert!(parse(r#"{ "Window": { "ctrl-shift-b": "no.such" } }"#).unwrap_err().contains("no command"));
    }

    #[test]
    fn keys_read_as_people_write_them() {
        assert_eq!(pretty("ctrl-shift-b"), "Ctrl+Shift+B");
        assert_eq!(pretty("ctrl-shift--"), "Ctrl+Shift+-");
        assert_eq!(pretty("ctrl-pagedown"), "Ctrl+PgDn");
        assert_eq!(pretty("shift-tab"), "Shift+Tab");
    }

    #[test]
    fn a_window_key_a_terminal_binds_is_masked_there() {
        let overrides = [Binding { context: TERMINAL.into(), key: "ctrl-shift-b".into(), command: None, custom: true }];
        let bindings = effective(&overrides);
        let masked = masked(&bindings);
        assert_eq!(masked.len(), 1);
        assert_eq!(masked[0].0.command, Some("sidebar.toggle"));
    }
}
