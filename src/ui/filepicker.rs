//! An in-app picker of folders or files: a folder's entries, a field that
//! filters them or takes a typed path (its completions offered), the way up. Drawn
//! by its caller; this is its model, ported from egui-pop's `FilePicker`
//! (`egui-pop-api/src/files.rs`), whose behaviour it keeps.
//!
//! It never opens the system's chooser: that one goes through the desktop
//! portal, which a test instance must not reach.

use std::path::{Path, PathBuf};

/// What is chosen: a folder, or a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Folder,
    File,
}

/// One entry of a folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

/// The picker: the folder shown, the field, what was wrong last.
#[derive(Clone, Debug)]
pub struct FilePicker {
    pub kind: Kind,
    /// The folder shown.
    pub dir: PathBuf,
    /// The field: a filter, or a path when it holds a separator or starts
    /// with `~`.
    pub text: String,
    pub show_hidden: bool,
    /// Why the last Enter did nothing.
    pub error: Option<String>,
    listing: Option<(PathBuf, bool, Vec<Entry>)>,
}

/// The home folder.
pub fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).filter(|d| d.is_dir())
}

impl FilePicker {
    /// A picker in `dir` (else home, else the root).
    pub fn new(kind: Kind, dir: Option<PathBuf>) -> Self {
        let dir = dir.filter(|d| d.is_dir()).or_else(home).unwrap_or_else(|| PathBuf::from("/"));
        Self { kind, dir, text: String::new(), show_hidden: false, error: None, listing: None }
    }

    /// Shows another folder.
    pub fn go(&mut self, dir: PathBuf) {
        self.dir = dir;
        self.error = None;
        self.text.clear();
    }

    /// Shows the parent folder.
    pub fn up(&mut self) {
        if let Some(p) = self.dir.parent() {
            self.go(p.to_path_buf());
        }
    }

    /// The folder's entries: folders, then files (none when choosing a
    /// folder), by name; hidden ones only when asked. Read once per folder.
    pub fn entries(&mut self) -> Vec<Entry> {
        if let Some((d, h, e)) = &self.listing
            && (d, *h) == (&self.dir, self.show_hidden)
        {
            return e.clone();
        }
        let mut all: Vec<Entry> = std::fs::read_dir(&self.dir)
            .map(|r| {
                r.filter_map(Result::ok)
                    .map(|e| {
                        let path = e.path();
                        Entry { name: e.file_name().to_string_lossy().into_owned(), is_dir: path.is_dir(), path }
                    })
                    .filter(|e| self.show_hidden || !e.name.starts_with('.'))
                    .filter(|e| e.is_dir || self.kind != Kind::Folder)
                    .collect()
            })
            .unwrap_or_default();
        all.sort_by(|a, b| (!a.is_dir, a.name.to_lowercase()).cmp(&(!b.is_dir, b.name.to_lowercase())));
        self.listing = Some((self.dir.clone(), self.show_hidden, all.clone()));
        all
    }

    /// Whether the field holds a path rather than a filter.
    pub fn is_path(&self) -> bool {
        self.text.contains('/') || self.text.contains(std::path::MAIN_SEPARATOR) || self.text.starts_with('~')
    }

    /// The field as a path: `~` is home, a relative path is in the folder.
    pub fn typed_path(&self) -> PathBuf {
        let t = self.text.trim();
        let expanded = match t.strip_prefix('~') {
            Some(rest) => home().unwrap_or_default().join(rest.trim_start_matches(['/', std::path::MAIN_SEPARATOR])),
            None => PathBuf::from(t),
        };
        if expanded.is_absolute() { expanded } else { self.dir.join(expanded) }
    }

    /// The entries shown: all of them, or those the filter matches (a name
    /// starting with it first, then one holding it).
    pub fn visible(&mut self) -> Vec<Entry> {
        let entries = self.entries();
        let filter = self.text.trim().to_lowercase();
        if self.is_path() || filter.is_empty() {
            return entries;
        }
        let (mut starts, mut holds): (Vec<Entry>, Vec<Entry>) = (Vec::new(), Vec::new());
        for e in entries {
            let name = e.name.to_lowercase();
            if name.starts_with(&filter) {
                starts.push(e);
            } else if name.contains(&filter) {
                holds.push(e);
            }
        }
        starts.extend(holds);
        starts
    }

    /// What a typed path may complete to: the names in its folder its last
    /// part starts (folders end with a separator).
    pub fn completions(&self) -> Vec<String> {
        if !self.is_path() {
            return Vec::new();
        }
        let text = self.text.as_str();
        let cut = text.rfind(['/', std::path::MAIN_SEPARATOR]).map_or(0, |i| i + 1);
        let (head, part) = text.split_at(cut);
        let head_path = if head.is_empty() {
            self.dir.clone()
        } else {
            let mut p = self.clone();
            p.text = head.to_string();
            p.typed_path()
        };
        let Ok(read) = std::fs::read_dir(&head_path) else { return Vec::new() };
        let lower = part.to_lowercase();
        let mut out: Vec<String> = read
            .filter_map(Result::ok)
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let is_dir = e.path().is_dir();
                let shown = (self.show_hidden || !name.starts_with('.') || part.starts_with('.'))
                    && name.to_lowercase().starts_with(&lower)
                    && (is_dir || self.kind != Kind::Folder);
                shown.then(|| format!("{head}{name}{}", if is_dir { "/" } else { "" }))
            })
            .collect();
        out.sort();
        out
    }

    /// Enter: into a folder typed or the first one shown, or a file chosen.
    pub fn activate(&mut self) -> Option<PathBuf> {
        self.error = None;
        if self.is_path() {
            let p = self.typed_path();
            if p.is_dir() {
                self.go(p);
                return None;
            }
            if self.kind == Kind::File && p.is_file() {
                return Some(p);
            }
            self.error = Some(p.display().to_string());
            return None;
        }
        let e = self.visible().into_iter().next()?;
        if e.is_dir {
            self.go(e.path);
            return None;
        }
        Some(e.path)
    }

    /// The choice: the folder shown (or a folder typed), choosing a folder;
    /// what Enter would choose otherwise.
    pub fn accept(&mut self) -> Option<PathBuf> {
        match self.kind {
            Kind::Folder if self.is_path() && self.typed_path().is_dir() => Some(self.typed_path()),
            Kind::Folder => Some(self.dir.clone()),
            Kind::File => self.activate(),
        }
    }
}

/// `path` with the home folder as `~`, as the picker shows its folder.
pub fn shown(path: &Path) -> String {
    crate::shell::home_short(&path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::{FilePicker, Kind};

    /// A folder of the tests' own, emptied first: (alpha/inner, beta, .hidden, notes.txt).
    struct Tree(std::path::PathBuf);

    impl Tree {
        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for Tree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn tree(name: &str) -> Tree {
        let t = Tree(std::env::temp_dir().join(format!("tvty-filepicker-{name}-{}", std::process::id())));
        let _ = std::fs::remove_dir_all(t.path());
        for d in ["alpha", "beta", ".hidden", "alpha/inner"] {
            std::fs::create_dir_all(t.path().join(d)).unwrap();
        }
        std::fs::write(t.path().join("notes.txt"), "x").unwrap();
        t
    }

    #[test]
    fn a_folder_picker_lists_folders_only_and_hides_dot_ones_unless_asked() {
        let t = tree("list");
        let mut p = FilePicker::new(Kind::Folder, Some(t.path().to_path_buf()));
        let names: Vec<String> = p.entries().into_iter().map(|e| e.name).collect();
        assert_eq!(names, ["alpha", "beta"]);
        p.show_hidden = true;
        let names: Vec<String> = p.entries().into_iter().map(|e| e.name).collect();
        assert_eq!(names, [".hidden", "alpha", "beta"]);
        let mut f = FilePicker::new(Kind::File, Some(t.path().to_path_buf()));
        assert!(f.entries().iter().any(|e| e.name == "notes.txt"));
    }

    #[test]
    fn the_field_filters_then_enter_goes_in_and_accept_takes_the_folder() {
        let t = tree("filter");
        let mut p = FilePicker::new(Kind::Folder, Some(t.path().to_path_buf()));
        p.text = "et".into();
        assert_eq!(p.visible().into_iter().map(|e| e.name).collect::<Vec<_>>(), ["beta"]);
        p.text = "al".into();
        assert_eq!(p.activate(), None);
        assert_eq!(p.dir, t.path().join("alpha"));
        assert_eq!(p.accept(), Some(t.path().join("alpha")));
        p.up();
        assert_eq!(p.dir, t.path());
    }

    #[test]
    fn a_typed_path_completes_and_a_missing_one_is_said() {
        let t = tree("complete");
        let mut p = FilePicker::new(Kind::Folder, Some(t.path().to_path_buf()));
        p.text = format!("{}/al", t.path().display());
        assert_eq!(p.completions(), [format!("{}/alpha/", t.path().display())]);
        p.text = format!("{}/nowhere", t.path().display());
        assert_eq!(p.activate(), None);
        assert!(p.error.is_some());
    }
}
