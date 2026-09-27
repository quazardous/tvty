//! Where tvty keeps things, and how: one way for every set of settings.
//!
//! A set is a Rust type (`Stored`), with its defaults, that says once where
//! it lives — `Config` (`$XDG_CONFIG_HOME/tvty`: what the user sets, and may
//! edit by hand), `State` (`$XDG_STATE_HOME/tvty`: what tvty remembers on its
//! own) or `Data` — and in which file (`.toml` or `.json`, by its name).
//! Registered, it becomes a GPUI global, `Store<T>`, which every set keeps
//! the same way:
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
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Where a set lives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Place {
    /// What the user sets: `$XDG_CONFIG_HOME/tvty` (`~/.config/tvty`).
    Config,
    /// What tvty remembers on its own: `$XDG_STATE_HOME/tvty` (`~/.local/state/tvty`).
    State,
    /// What the user adds (fonts): `$XDG_DATA_HOME/tvty` (`~/.local/share/tvty`).
    Data,
}

/// tvty's directory in `place`.
pub fn dir(place: Place) -> Option<PathBuf> {
    let (variable, default) = match place {
        Place::Config => ("XDG_CONFIG_HOME", ".config"),
        Place::State => ("XDG_STATE_HOME", ".local/state"),
        Place::Data => ("XDG_DATA_HOME", ".local/share"),
    };
    let base = std::env::var_os(variable)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(default)))?;
    Some(base.join("tvty"))
}

/// A set of settings, kept in a file.
pub trait Stored: Serialize + DeserializeOwned + Default + Clone + PartialEq + 'static {
    const PLACE: Place;
    /// Its file; `.toml` or `.json` says the format.
    const FILE: &'static str;
    /// The user edits it too: it is read again when it changes.
    const EDITED: bool = false;

    /// What serde cannot check (a keymap's commands): an error keeps the
    /// last good value.
    fn check(&self) -> Result<(), String> {
        Ok(())
    }

    /// The value an older tvty kept elsewhere, the first time there is no
    /// file.
    fn migrate() -> Option<Self> {
        None
    }
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

pub fn path<T: Stored>() -> Option<PathBuf> {
    Some(dir(T::PLACE)?.join(T::FILE))
}

fn toml(file: &str) -> bool {
    file.ends_with(".toml")
}

/// A file's text as a value.
pub fn parse<T: Stored>(text: &str) -> Result<T, String> {
    let value: T = if toml(T::FILE) {
        ::toml::from_str(text).map_err(|e| e.message().to_string())?
    } else {
        serde_json::from_str(text).map_err(|e| e.to_string())?
    };
    value.check()?;
    Ok(value)
}

/// A value as its file's text.
pub fn render<T: Stored>(value: &T) -> Result<String, String> {
    if toml(T::FILE) {
        ::toml::to_string_pretty(value).map_err(|e| e.to_string())
    } else {
        serde_json::to_string_pretty(value).map_err(|e| e.to_string())
    }
}

/// The file's value: none when there is no file. An error does not say
/// the file: whoever shows it does.
fn read<T: Stored>(path: &Path) -> Result<Option<T>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Writes `text` so that the file is never half written: a temporary file
/// beside it, then renamed over it.
pub fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let temporary = path.with_extension(format!("{}.tmp", path.extension().and_then(|e| e.to_str()).unwrap_or("")));
    std::fs::write(&temporary, text)?;
    std::fs::rename(&temporary, path)
}

fn stamp(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

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

/// Where the log goes: `tvty.log` in the state place, the previous run's
/// kept as `tvty.log.1`.
pub fn log_path() -> Option<PathBuf> {
    Some(dir(Place::State)?.join("tvty.log"))
}

/// The log's lines, to stderr and to [`log_path`].
pub struct LogTee(Option<std::fs::File>);

impl LogTee {
    /// A new log for this run: the previous one set aside first. Without a
    /// file (no home, a disk full), stderr alone.
    pub fn open() -> Self {
        let file = log_path().and_then(|path| {
            std::fs::create_dir_all(path.parent()?).ok()?;
            let _ = std::fs::rename(&path, path.with_extension("log.1"));
            std::fs::File::create(&path).ok()
        });
        Self(file)
    }
}

impl std::io::Write for LogTee {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let _ = std::io::stderr().write_all(bytes);
        if let Some(file) = &mut self.0 {
            let _ = file.write_all(bytes);
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(file) = &mut self.0 {
            file.flush()?;
        }
        std::io::stderr().flush()
    }
}

#[cfg(test)]
mod tests {
    use super::{Place, Stored, parse, render, write_atomic};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Sample {
        name: String,
        size: u32,
        #[serde(default)]
        inner: Inner,
    }

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Inner {
        on: bool,
    }

    impl Stored for Sample {
        const PLACE: Place = Place::Config;
        const FILE: &'static str = "sample.toml";

        fn check(&self) -> Result<(), String> {
            if self.size > 100 { Err("size: 100 at most".into()) } else { Ok(()) }
        }
    }

    #[test]
    fn what_a_file_leaves_out_takes_the_defaults() {
        let sample: Sample = parse("name = \"a\"\n").unwrap();
        assert_eq!(sample, Sample { name: "a".into(), ..Default::default() });
    }

    #[test]
    fn a_file_that_does_not_read_says_why() {
        assert!(parse::<Sample>("name = ").is_err());
        assert_eq!(parse::<Sample>("size = 500").unwrap_err(), "size: 100 at most");
    }

    #[test]
    fn a_value_goes_to_its_file_and_back() {
        let sample = Sample { name: "b".into(), size: 7, inner: Inner { on: true } };
        let text = render(&sample).unwrap();
        assert!(text.contains("[inner]"), "sections: {text}");
        assert_eq!(parse::<Sample>(&text).unwrap(), sample);
    }

    #[test]
    fn a_file_is_written_whole_or_not_at_all() {
        let dir = std::env::temp_dir().join(format!("tvty-config-test-{}", std::process::id()));
        let path = dir.join("sub").join("sample.toml");
        write_atomic(&path, "name = \"c\"\n").unwrap();
        write_atomic(&path, "name = \"d\"\n").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "name = \"d\"\n");
        // No temporary file left beside it.
        assert_eq!(std::fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
