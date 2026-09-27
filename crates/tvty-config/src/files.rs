//! Where a program keeps its files (the XDG directories), and how: a set of
//! settings is a Rust type ([`Stored`]) that says once where it lives and
//! in which file, read with its defaults and written whole or not at all.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Where a set lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// What the user sets: `$XDG_CONFIG_HOME/<app>` (`~/.config/<app>`).
    Config,
    /// What the program remembers on its own: `$XDG_STATE_HOME/<app>`
    /// (`~/.local/state/<app>`).
    State,
    /// What the user adds: `$XDG_DATA_HOME/<app>` (`~/.local/share/<app>`).
    Data,
}

/// `app`'s directory in `place`: the XDG variable, else its usual default
/// under `$HOME`.
pub fn dir(app: &str, place: Place) -> Option<PathBuf> {
    let (variable, default) = match place {
        Place::Config => ("XDG_CONFIG_HOME", ".config"),
        Place::State => ("XDG_STATE_HOME", ".local/state"),
        Place::Data => ("XDG_DATA_HOME", ".local/share"),
    };
    let base = std::env::var_os(variable)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(default)))?;
    Some(base.join(app))
}

/// A set of settings, kept in a file.
pub trait Stored: Serialize + DeserializeOwned + Default + Clone + PartialEq + 'static {
    const PLACE: Place;
    /// Its file; `.toml` or `.json` says the format.
    const FILE: &'static str;
    /// The user edits it too: it is read again when it changes.
    const EDITED: bool = false;

    /// What serde cannot check (bounds, a keymap's commands): an error keeps
    /// the last good value.
    fn check(&self) -> Result<(), String> {
        Ok(())
    }

    /// The value an older version kept elsewhere, the first time there is
    /// no file.
    fn migrate() -> Option<Self> {
        None
    }
}

/// The set's file in `app`'s directory.
pub fn path<T: Stored>(app: &str) -> Option<PathBuf> {
    Some(dir(app, T::PLACE)?.join(T::FILE))
}

fn is_toml(file: &str) -> bool {
    file.ends_with(".toml")
}

/// A file's text as a value, checked.
pub fn parse<T: Stored>(text: &str) -> Result<T, String> {
    let value: T = if is_toml(T::FILE) {
        toml::from_str(text).map_err(|e| e.message().to_string())?
    } else {
        serde_json::from_str(text).map_err(|e| e.to_string())?
    };
    value.check()?;
    Ok(value)
}

/// A value as its file's text.
pub fn render<T: Stored>(value: &T) -> Result<String, String> {
    if is_toml(T::FILE) {
        toml::to_string_pretty(value).map_err(|e| e.to_string())
    } else {
        serde_json::to_string_pretty(value).map_err(|e| e.to_string())
    }
}

/// The file's value: none when there is no file. An error does not name the
/// file: whoever shows it does.
pub fn read<T: Stored>(path: &Path) -> Result<Option<T>, String> {
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

/// When the file was last changed: another time than the one seen means
/// someone edited it.
pub fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

// ── The log ──────────────────────────────────────────────────────────

/// The log's lines, to stderr and to a file (the previous run's set aside
/// with the extension `.1`).
pub struct LogTee(Option<std::fs::File>);

impl LogTee {
    /// A new log at `path` for this run. Without a file (no home, a disk
    /// full), stderr alone.
    pub fn open(path: Option<PathBuf>) -> Self {
        let file = path.and_then(|path| {
            std::fs::create_dir_all(path.parent()?).ok()?;
            let aside = path.with_extension(format!("{}.1", path.extension().and_then(|e| e.to_str()).unwrap_or("")));
            let _ = std::fs::rename(&path, aside);
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
