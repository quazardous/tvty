//! Where tvty keeps things, and how: one way for every set of settings.
//!
//! A set is a Rust type (`Stored`, from `tvty-config`, with no GPUI in it),
//! with its defaults, that says once where it lives — `Config`
//! (`$XDG_CONFIG_HOME/tvty`: what the user sets, and may edit by hand),
//! `State` (`$XDG_STATE_HOME/tvty`: what tvty remembers on its own) or
//! `Data` — and in which file (`.toml` or `.json`, by its name). Registered,
//! it becomes a GPUI global, `Store<T>`, which every set keeps the same way:
//!
//! - read at start, the defaults filling what the file leaves out;
//! - a file that does not read is said, and the last good value is kept
//!   (the defaults, at start); it is never written over;
//! - written atomically (a temporary file, then renamed), once changes rest
//!   (300 ms), and at quit;
//! - read again when the user edits it (`EDITED`), and its observers told;
//! - migrated, the first time, from where an older tvty kept it.
//!
//! The log lives here too: `$XDG_STATE_HOME/tvty/tvty.log`.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use gpui_kit::*;
pub use tvty_config::{Place, Stored, render, write_atomic};
use tvty_config::{modified as stamp, read};

/// tvty's directory in `place`.
pub fn dir(place: Place) -> Option<PathBuf> {
    tvty_config::dir("tvty", place)
}

pub fn path<T: Stored>() -> Option<PathBuf> {
    tvty_config::path::<T>("tvty")
}

/// A registered set: its value, and what the file said last.
pub struct Store<T: Stored> {
    value: T,
    /// The file's time when last read or written: another means an edit.
    stamp: Option<SystemTime>,
    error: Option<String>,
    /// Changes waiting to be written.
    pending: u64,
}

impl<T: Stored> Global for Store<T> {}

/// Reads the set (or migrates it, or takes its defaults), makes it a global,
/// writes it at quit, and reads it again when the user edits it.
pub fn register<T: Stored>(cx: &mut App) {
    let path = path::<T>();
    let found = path.as_deref().map(read::<T>).unwrap_or(Ok(None));
    let (value, error, migrated) = match found {
        Ok(Some(value)) => (value, None, false),
        Ok(None) => match T::migrate() {
            Some(value) => (value, None, true),
            None => (T::default(), None, false),
        },
        Err(error) => {
            log::warn!("{}: {error}", path.as_deref().unwrap_or(Path::new(T::FILE)).display());
            (T::default(), Some(error), false)
        }
    };
    let stamp = path.as_deref().and_then(stamp);
    cx.set_global(Store { value, stamp, error, pending: 0 });
    if migrated {
        save::<T>(cx);
    }
    // At quit, what waits is written.
    cx.on_app_quit(|cx| {
        if cx.global::<Store<T>>().pending > 0 {
            save::<T>(cx);
        }
        async {}
    })
    .detach();
    if T::EDITED {
        cx.spawn(async move |cx| loop {
            cx.background_executor().timer(Duration::from_secs(2)).await;
            cx.update(reload_if_changed::<T>);
        })
        .detach();
    }
}

/// The set's value.
pub fn get<T: Stored>(cx: &App) -> &T {
    &cx.global::<Store<T>>().value
}

/// What was wrong with the file last read, if anything (without its name).
pub fn error<T: Stored>(cx: &App) -> Option<String> {
    cx.global::<Store<T>>().error.clone()
}

/// Says, as a notification, what was wrong with the file read at start —
/// once the window listens.
pub fn report<T: Stored>(cx: &mut App) {
    if let Some(error) = error::<T>(cx) {
        crate::activity::publish(cx, crate::activity::Activity::failed(None, T::FILE, format!("{error} — the defaults are used, the file is left as it is")));
    }
}

/// Changes the set: its observers are told at once, the file is written
/// once the changes rest.
pub fn update<T: Stored>(cx: &mut App, change: impl FnOnce(&mut T)) {
    let mut value = get::<T>(cx).clone();
    change(&mut value);
    if value == *get::<T>(cx) {
        return;
    }
    let store = cx.global_mut::<Store<T>>();
    store.value = value;
    store.pending += 1;
    let waiting = store.pending;
    cx.spawn(async move |cx| {
        cx.background_executor().timer(Duration::from_millis(300)).await;
        cx.update(|cx| {
            if cx.global::<Store<T>>().pending == waiting {
                save::<T>(cx);
            }
        });
    })
    .detach();
}

/// Writes the set now — unless its file does not read: that one is the
/// user's to mend, never written over.
fn save<T: Stored>(cx: &mut App) {
    let Some(path) = path::<T>() else { return };
    if let Some(error) = error::<T>(cx) {
        log::warn!("{}: not written, it does not read ({error})", path.display());
        cx.global_mut::<Store<T>>().pending = 0;
        return;
    }
    let written = render(get::<T>(cx)).and_then(|text| write_atomic(&path, &text).map_err(|e| e.to_string()));
    if let Err(error) = &written {
        log::warn!("{}: {error}", path.display());
    }
    let store = cx.global_mut::<Store<T>>();
    store.pending = 0;
    // tvty's own write is not an edit.
    store.stamp = stamp(&path);
}

/// The user edited the file: its value in force, or, when it does not
/// read, the last good one kept and the error said.
fn reload_if_changed<T: Stored>(cx: &mut App) {
    let Some(path) = path::<T>() else { return };
    let now = stamp(&path);
    if cx.global::<Store<T>>().stamp == now {
        return;
    }
    let name = T::FILE;
    match read::<T>(&path) {
        Ok(found) => {
            let value = found.unwrap_or_default();
            let store = cx.global_mut::<Store<T>>();
            store.stamp = now;
            store.error = None;
            if store.value != value {
                store.value = value;
                crate::activity::publish(cx, crate::activity::Activity::done(None, format!("{name}: read again")));
            }
        }
        Err(error) => {
            let store = cx.global_mut::<Store<T>>();
            store.stamp = now;
            log::warn!("{}: {error}", path.display());
            store.error = Some(error.clone());
            crate::activity::publish(cx, crate::activity::Activity::failed(None, name, error));
        }
    }
}

// ── The log ──────────────────────────────────────────────────────────

/// The log, to stderr and to `tvty.log` in the state place — the previous
/// run's kept as `tvty.log.1`.
pub fn log() -> tvty_config::LogTee {
    tvty_config::LogTee::open(dir(Place::State).map(|d| d.join("tvty.log")))
}
