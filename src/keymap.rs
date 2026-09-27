//! The shortcuts: named commands bound to keys in a context. The context
//! follows the focus — `Terminal` (a terminal has it), under `Window` (the
//! rest of tvty) — and the deepest binding wins: a key bound in `Terminal`
//! is the terminal's, a key bound nowhere goes to the program.
//!
//! The defaults are below; `keymap.json`, in tvty's config directory,
//! overrides them (see [`crate::keyfile`]).

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
    #[allow(dead_code)] // read by Options > Shortcuts, next
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
    command!("terminal.paste", TERMINAL, ["ctrl-shift-v"], "Paste", TerminalPaste),
    command!("terminal.tab", TERMINAL, ["tab"], "Tab, to the program (not the focus to the next element)", SendTab),
    command!("terminal.back_tab", TERMINAL, ["shift-tab"], "Shift+Tab, to the program", SendBackTab),
];

/// Binds the defaults.
pub fn init(cx: &mut App) {
    cx.bind_keys(key_bindings(&effective(&[])));
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
#[allow(dead_code)] // read by Options > Shortcuts, next
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
    use super::{Binding, TERMINAL, WINDOW, effective, key_bindings, masked};

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
    fn a_window_key_a_terminal_binds_is_masked_there() {
        let overrides = [Binding { context: TERMINAL.into(), key: "ctrl-shift-b".into(), command: None, custom: true }];
        let bindings = effective(&overrides);
        let masked = masked(&bindings);
        assert_eq!(masked.len(), 1);
        assert_eq!(masked[0].0.command, Some("sidebar.toggle"));
    }
}
