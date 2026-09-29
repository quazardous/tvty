//! Whether the pointer gives the keyboard: over the terminal, over a box to
//! write in. As the window manager does between windows (`system`), or as
//! the user chose (`click`, `hover`: Options > Appearance > Mouse). The
//! window manager's is read once and followed: GNOME's `focus-mode`, KDE's
//! `FocusPolicy`; elsewhere, a click.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use gpui_kit::*;

use crate::settings::Preferences;

/// What the window manager does, as read, and where it was read.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SystemFocus {
    pub hover: bool,
    /// `GNOME`, `KDE`, `test`; none: not read (a click).
    pub from: Option<&'static str>,
}

impl Global for SystemFocus {}

/// GNOME's `focus-mode`, as `gsettings` prints it: `'click'`, `'sloppy'`,
/// `'mouse'`.
fn gnome(said: &str) -> Option<bool> {
    match said.trim().trim_matches('\'') {
        "click" => Some(false),
        "sloppy" | "mouse" => Some(true),
        _ => None,
    }
}

/// KDE's `FocusPolicy` (kwinrc, Windows): none written is a click.
fn kde(said: &str) -> Option<bool> {
    match said.trim() {
        "" | "ClickToFocus" => Some(false),
        "FocusFollowsMouse" | "FocusUnderMouse" | "FocusStrictlyUnderMouse" => Some(true),
        _ => None,
    }
}

fn output(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).stderr(Stdio::null()).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

const GNOME_SCHEMA: &str = "org.gnome.desktop.wm.preferences";

/// The window manager's way, read now. `TVTY_SYSTEM_FOCUS` (`click`,
/// `sloppy`) stands for it in tests.
pub fn read() -> SystemFocus {
    if let Ok(said) = std::env::var("TVTY_SYSTEM_FOCUS") {
        return SystemFocus { hover: gnome(&said).unwrap_or(false), from: Some("test") };
    }
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_uppercase();
    if desktop.contains("KDE") {
        for tool in ["kreadconfig6", "kreadconfig5"] {
            if let Some(hover) = output(tool, &["--file", "kwinrc", "--group", "Windows", "--key", "FocusPolicy"]).as_deref().and_then(kde) {
                return SystemFocus { hover, from: Some("KDE") };
            }
        }
    }
    if let Some(hover) = output("gsettings", &["get", GNOME_SCHEMA, "focus-mode"]).as_deref().and_then(gnome) {
        return SystemFocus { hover, from: Some("GNOME") };
    }
    SystemFocus::default()
}

/// Reads the window manager's way, then follows GNOME's changes.
pub fn start(cx: &mut App) {
    let first = read();
    cx.set_global(first);
    if first.from != Some("GNOME") {
        return;
    }
    let (tx, rx) = futures::channel::mpsc::unbounded::<bool>();
    std::thread::spawn(move || {
        let Ok(mut child) = Command::new("gsettings")
            .args(["monitor", GNOME_SCHEMA, "focus-mode"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        else {
            return;
        };
        let Some(stdout) = child.stdout.take() else { return };
        // `focus-mode: 'sloppy'`, a line a change.
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Some(hover) = line.split_once(':').and_then(|(_, v)| gnome(v))
                && tx.unbounded_send(hover).is_err()
            {
                break;
            }
        }
        let _ = child.kill();
    });
    cx.spawn(async move |cx| {
        use futures::StreamExt as _;
        let mut rx = rx;
        while let Some(hover) = rx.next().await {
            log::info!("focus: GNOME now {}", if hover { "hover" } else { "click" });
            let _ = cx.update(|cx| cx.set_global(SystemFocus { hover, from: Some("GNOME") }));
        }
    })
    .detach();
}

/// Whether the pointer gives the keyboard now: the user's choice, else the
/// window manager's.
pub fn hover(cx: &App) -> bool {
    match crate::config::get::<Preferences>(cx).mouse.focus.as_deref() {
        Some("hover") => true,
        Some("click") => false,
        _ => cx.try_global::<SystemFocus>().is_some_and(|s| s.hover),
    }
}

/// What `system` means here, for the options: `GNOME: hover`.
pub fn system_said(cx: &App) -> String {
    let system = cx.try_global::<SystemFocus>().copied().unwrap_or_default();
    let way = if system.hover { "hover" } else { "click" };
    match system.from {
        Some(from) => format!("As the system ({from}: {way})"),
        None => "As the system (not read: click)".to_string(),
    }
}

/// A box to write in takes the keyboard when the pointer moves over it, if
/// the pointer gives it (no button held: not in a selection or a drag).
pub fn on_hover<E: InteractiveElement>(element: E, focus: FocusHandle) -> E {
    element.on_mouse_move(move |event: &MouseMoveEvent, window, cx| {
        if event.pressed_button.is_none() && !focus.contains_focused(window, cx) && hover(cx) {
            window.focus(&focus, cx);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{gnome, kde};

    #[test]
    fn the_window_managers_ways_read() {
        assert_eq!(gnome("'click'\n"), Some(false));
        assert_eq!(gnome("'sloppy'"), Some(true));
        assert_eq!(gnome("'mouse'"), Some(true));
        assert_eq!(gnome("nothing"), None);
        assert_eq!(kde(""), Some(false));
        assert_eq!(kde("FocusFollowsMouse\n"), Some(true));
        assert_eq!(kde("ClickToFocus"), Some(false));
    }
}
