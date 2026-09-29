//! "Did you know?": short tips, one idea each, shown one at a time in a
//! small card that never blocks — once after start, and the first time a
//! surface is entered (the full list, a ticket full screen, the gallery…).
//!
//! The tips are files, `tips/<id>.md`, built into the binary:
//!
//! ```text
//! ---
//! id: slider
//! surface: Workspace          # where it makes sense: a key context
//! command: slider.next        # its key; once used, the tip is not shown
//! ---
//! **{key}** shows your projects as stacks…
//! ```
//!
//! `{key}` is the command's key as the keymap in force says it (a key
//! changed in keymap.toml shows as changed), `{key:other.command}` another
//! one's. `**bold**` and `` `code` `` are the only marks.
//!
//! Which one: not marked "got it", not shown in the last day, its command
//! not used yet (tvty counts the commands run, locally), the one shown
//! longest ago first. What was shown and understood is kept in
//! `~/.local/state/tvty/tips.json`.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use gpui_kit::*;
use serde::{Deserialize, Serialize};

use crate::config::{self, Place, Stored};

/// A tip, as its file says it.
#[derive(Clone, Debug, PartialEq)]
pub struct Tip {
    pub id: String,
    /// The key context it makes sense in (`Workspace`, `FullList`, …).
    pub surface: String,
    /// Its command: its key, and once run, the tip is not shown.
    pub command: Option<String>,
    /// The text, with its marks and `{key}`s.
    pub text: String,
    /// The element it is about, as [`target`] names it: the card sits beside
    /// it, the element haloed. None: the card keeps its corner.
    pub target: Option<String>,
}

macro_rules! tip_files {
    ($($id:literal),* $(,)?) => {
        /// Every tip's file, by id.
        const FILES: &[(&str, &str)] = &[$(($id, include_str!(concat!("../tips/", $id, ".md")))),*];
    };
}

tip_files![
    "slider",
    "gallery",
    "gallery-filter",
    "notice",
    "goto",
    "full-list",
    "bulk",
    "afk",
    "remote-control",
    "move-loop",
    "critical",
    "references",
    "comment-link",
    "reply",
    "preview",
    "escape",
    "sections",
    "fullscreen",
    "filter",
    "new-project",
    "restart",
    "shortcuts",
    "opacity",
    "aiball-board",
    "font",
    "new-ticket",
    "room",
    "themes",
    "wizard",
];

/// A tip's file read: its header, then its text.
fn parse(file: &str) -> Result<Tip, String> {
    let rest = file.strip_prefix("---\n").ok_or("no header")?;
    let (header, text) = rest.split_once("\n---\n").ok_or("the header does not end")?;
    let mut fields: HashMap<&str, &str> = HashMap::new();
    for line in header.lines() {
        let line = line.split(" #").next().unwrap_or(line);
        let (key, value) = line.split_once(':').ok_or_else(|| format!("not a field: {line}"))?;
        fields.insert(key.trim(), value.trim());
    }
    let field = |name: &str| fields.get(name).filter(|v| !v.is_empty()).map(|v| v.to_string());
    Ok(Tip {
        id: field("id").ok_or("no id")?,
        surface: field("surface").ok_or("no surface")?,
        command: field("command"),
        target: field("target"),
        text: text.trim().to_string(),
    })
}

/// Every tip, in the order of their files.
pub fn all() -> &'static [Tip] {
    static TIPS: std::sync::OnceLock<Vec<Tip>> = std::sync::OnceLock::new();
    TIPS.get_or_init(|| FILES.iter().filter_map(|(id, file)| parse(file).map_err(|e| log::warn!("tip {id}: {e}")).ok()).collect())
}

/// The text as shown: its keys as the keymap in force has them, its marks
/// taken out and said as ranges — bold, and code.
pub fn shown(tip: &Tip, key_of: impl Fn(&str) -> Option<String>) -> (String, Vec<(Range<usize>, Mark)>) {
    let mut text = String::new();
    let mut rest = tip.text.as_str();
    while let Some(start) = rest.find("{key") {
        text.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('}') else { break };
        let inner = &rest[start + 1..start + end];
        let command = inner.strip_prefix("key:").or(tip.command.as_deref().filter(|_| inner == "key"));
        text.push_str(&command.and_then(&key_of).unwrap_or_else(|| "?".into()));
        rest = &rest[start + end + 1..];
    }
    text.push_str(rest);
    marks(&text)
}

/// How a part of a tip is set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mark {
    Bold,
    Code,
}

fn marks(marked: &str) -> (String, Vec<(Range<usize>, Mark)>) {
    let mut text = String::new();
    let mut ranges = Vec::new();
    let mut open: Option<(usize, Mark)> = None;
    let mut rest = marked;
    while !rest.is_empty() {
        let (token, mark) = if rest.starts_with("**") {
            ("**", Mark::Bold)
        } else if rest.starts_with('`') {
            ("`", Mark::Code)
        } else {
            let c = rest.chars().next().unwrap();
            text.push(c);
            rest = &rest[c.len_utf8()..];
            continue;
        };
        match open {
            Some((start, was)) if was == mark => {
                ranges.push((start..text.len(), mark));
                open = None;
            }
            None => open = Some((text.len(), mark)),
            // Another mark inside: taken as text.
            Some(_) => text.push_str(token),
        }
        rest = &rest[token.len()..];
    }
    (text, ranges)
}

// ── The elements tips are about ──────────────────────────────────────

/// Where each target was last painted, and in which frame.
static TARGETS: Mutex<Vec<(&'static str, Bounds<Pixels>, u64)>> = Mutex::new(Vec::new());
/// The window's frames, counted as the shell draws: a target painted in the
/// last one is on screen.
static FRAME: AtomicU64 = AtomicU64::new(0);
/// The target of the tip on screen: haloed.
static ACTIVE: Mutex<Option<String>> = Mutex::new(None);

/// A new frame of the window (the shell's render).
pub fn next_frame() {
    FRAME.fetch_add(1, Ordering::Relaxed);
}

/// The target of the tip shown, if any: its element draws a halo.
pub fn set_active(target: Option<String>) {
    if let Ok(mut active) = ACTIVE.lock() {
        *active = target;
    }
}

fn is_active(id: &str) -> bool {
    ACTIVE.lock().is_ok_and(|a| a.as_deref() == Some(id))
}

/// Where `id` is on screen: painted in this frame or the last.
pub fn bounds_of(id: &str) -> Option<Bounds<Pixels>> {
    let frame = FRAME.load(Ordering::Relaxed);
    TARGETS.lock().ok()?.iter().find(|(t, _, at)| *t == id && at + 1 >= frame).map(|(_, b, _)| *b)
}

/// Where the shell's root is painted, measured as the targets are: the card
/// is placed in the root, from positions measured the same way.
pub fn root_mark() -> impl IntoElement {
    canvas(
        |bounds, _, _| {
            let frame = FRAME.load(Ordering::Relaxed);
            if let Ok(mut targets) = TARGETS.lock() {
                targets.retain(|(t, _, _)| *t != ROOT);
                targets.push((ROOT, bounds, frame));
            }
        },
        |_, _, _, _| {},
    )
    // Pinned to its parent's corner: an absolute element without offsets
    // sits where it would in the flow, below what comes before it.
    .absolute()
    .inset_0()
}

/// The shell's root, as [`root_mark`] measures it.
pub const ROOT: &str = "shell.root";

/// `element`, as the target `id` of a tip: where it is painted is kept, and
/// while its tip is up it wears a halo in the tips' colour.
pub fn target(id: &'static str, element: impl IntoElement) -> Div {
    let halo = is_active(id).then(|| {
        let colour = crate::theme::tip();
        div()
            .absolute()
            .top(px(-3.))
            .left(px(-3.))
            .right(px(-3.))
            .bottom(px(-3.))
            .rounded_md()
            .border_2()
            .border_color(colour)
            .shadow(vec![BoxShadow { color: colour.opacity(0.45), offset: point(px(0.), px(0.)), blur_radius: px(10.), spread_radius: px(1.), inset: false }])
    });
    div()
        .relative()
        .child(element)
        .child(
            canvas(
                move |bounds, _, _| {
                    let frame = FRAME.load(Ordering::Relaxed);
                    if let Ok(mut targets) = TARGETS.lock() {
                        targets.retain(|(t, _, _)| *t != id);
                        targets.push((id, bounds, frame));
                    }
                },
                |_, _, _, _| {},
            )
            // Pinned to the element's corner (see root_mark).
            .absolute()
            .inset_0(),
        )
        .children(halo)
}

// ── What was shown, understood, used ─────────────────────────────────

/// `tips.json`: kept by tvty on its own.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Seen {
    /// The tips marked "got it": never shown again.
    pub got: Vec<String>,
    /// When each tip was last shown (seconds since the epoch).
    pub shown: HashMap<String, u64>,
    /// The surfaces entered once: their first entry shows a tip.
    pub surfaces: Vec<String>,
    /// The commands run once: their tips are not shown.
    pub used: Vec<String>,
}

impl Stored for Seen {
    const PLACE: Place = Place::State;
    const FILE: &'static str = "tips.json";
}

pub fn init(cx: &mut App) {
    config::register::<Seen>(cx);
}

pub fn seen(cx: &App) -> &Seen {
    config::get::<Seen>(cx)
}

pub fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// A tip is not shown again within this.
const REST: u64 = 20 * 3600;

/// The tip to show on `surface`, after `after` when stepping to the next
/// one: none left.
pub fn pick<'a>(tips: &'a [Tip], seen: &Seen, surface: &str, after: Option<&str>, now: u64) -> Option<&'a Tip> {
    tips.iter()
        .filter(|t| t.surface == surface)
        .filter(|t| !seen.got.contains(&t.id))
        .filter(|t| !t.command.as_ref().is_some_and(|c| seen.used.contains(c)))
        .filter(|t| Some(t.id.as_str()) != after)
        .filter(|t| seen.shown.get(&t.id).is_none_or(|at| now.saturating_sub(*at) >= REST))
        // Never shown first, then the one shown longest ago; the files'
        // order breaks the ties.
        .min_by_key(|t| seen.shown.get(&t.id).copied().unwrap_or(0))
}

/// Marks what happened: the tip shown, understood, a surface entered, a
/// command run.
pub fn shown_now(cx: &mut App, id: &str) {
    let (id, at) = (id.to_string(), now());
    config::update::<Seen>(cx, |s| {
        s.shown.insert(id, at);
    });
}

pub fn got(cx: &mut App, id: &str) {
    let id = id.to_string();
    config::update::<Seen>(cx, |s| {
        if !s.got.contains(&id) {
            s.got.push(id);
        }
    });
}

/// Every tip shown again: none understood, none shown.
pub fn forget(cx: &mut App) {
    config::update::<Seen>(cx, |s| {
        s.got.clear();
        s.shown.clear();
    });
}

/// `true` the first time `surface` is entered.
pub fn entered(cx: &mut App, surface: &str) -> bool {
    if seen(cx).surfaces.iter().any(|s| s == surface) {
        return false;
    }
    let surface = surface.to_string();
    config::update::<Seen>(cx, |s| s.surfaces.push(surface));
    true
}

/// A command run: its tips are not shown any more.
pub fn used(cx: &mut App, command: &str) {
    if seen(cx).used.iter().any(|c| c == command) {
        return;
    }
    let command = command.to_string();
    config::update::<Seen>(cx, |s| s.used.push(command));
}

/// The command an action is, by its name: what a keystroke ran.
pub fn command_of(action: &str) -> Option<&'static str> {
    static NAMES: std::sync::OnceLock<HashMap<&'static str, &'static str>> = std::sync::OnceLock::new();
    NAMES
        .get_or_init(|| crate::keymap::COMMANDS.iter().filter_map(|c| Some((crate::keymap::action_name(c.name)?, c.name))).collect())
        .get(action)
        .copied()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{FILES, Mark, REST, Seen, Tip, all, parse, pick, shown};

    fn tip(id: &str, surface: &str, command: Option<&str>) -> Tip {
        Tip { id: id.into(), surface: surface.into(), command: command.map(Into::into), text: String::new(), target: None }
    }

    #[test]
    fn every_file_in_tips_is_built_in_and_reads() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tips");
        let mut files: Vec<String> =
            std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()?.path().file_stem()?.to_str().map(String::from)).collect();
        files.sort();
        let mut listed: Vec<String> = FILES.iter().map(|(id, _)| id.to_string()).collect();
        listed.sort();
        assert_eq!(files, listed, "a file of tips/ not listed in tip_files!, or the reverse");
        for (id, file) in FILES {
            let tip = parse(file).unwrap_or_else(|e| panic!("{id}: {e}"));
            assert_eq!(tip.id, *id, "a tip's id is its file's name");
        }
    }

    #[test]
    fn tips_are_short_and_their_keys_known() {
        let keymap = crate::keymap::current_defaults();
        let surfaces = ["Workspace", "FullList", "Ticket", "Gallery", "Options", "NewTicket", "NewProject"];
        for tip in all() {
            assert!(surfaces.contains(&tip.surface.as_str()), "{}: surface {}", tip.id, tip.surface);
            let (text, _) = shown(tip, |c| keymap.keys_of(c).first().map(|k| k.pretty()));
            assert!(!text.contains('?'), "{}: a key no command has: {text}", tip.id);
            assert!(text.chars().count() <= 150, "{}: {} characters, one idea is shorter", tip.id, text.chars().count());
            if let Some(command) = &tip.command {
                assert!(crate::keymap::COMMANDS.iter().any(|c| c.name == command), "{}: no command {command}", tip.id);
            }
        }
        assert_eq!(all().len(), FILES.len(), "every tip reads");
    }

    #[test]
    fn every_target_is_one_the_code_marks() {
        let src = |path: &str| std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        let code: String = ["src/composer.rs", "src/panel.rs", "src/shell.rs", "src/shell/agentbar.rs"].iter().map(|p| src(p)).collect();
        for tip in all() {
            if let Some(target) = &tip.target {
                assert!(code.contains(&format!("tips::target(\"{target}\"")) || code.contains(&format!("\"{target}\",")), "{}: no element marked {target}", tip.id);
            }
        }
    }

    #[test]
    fn keys_and_marks_are_shown() {
        let t = Tip { text: "**{key}** then `{key:b.c}`, **{key:x}**".into(), ..tip("t", "Workspace", Some("a.b")) };
        let (text, marks) = shown(&t, |c| match c {
            "a.b" => Some("Ctrl+A".into()),
            "b.c" => Some("F2".into()),
            _ => None,
        });
        assert_eq!(text, "Ctrl+A then F2, ?");
        assert_eq!(marks, vec![(0..6, Mark::Bold), (12..14, Mark::Code), (16..17, Mark::Bold)]);
    }

    #[test]
    fn the_pick_skips_what_is_understood_used_or_just_shown() {
        let tips = vec![tip("a", "Workspace", Some("x.y")), tip("b", "Workspace", None), tip("c", "Workspace", None), tip("d", "Ticket", None)];
        let mut seen = Seen::default();
        let now = 1_000_000;
        assert_eq!(pick(&tips, &seen, "Workspace", None, now).unwrap().id, "a");
        seen.used.push("x.y".into());
        assert_eq!(pick(&tips, &seen, "Workspace", None, now).unwrap().id, "b");
        seen.got.push("b".into());
        assert_eq!(pick(&tips, &seen, "Workspace", None, now).unwrap().id, "c");
        seen.shown.insert("c".into(), now - 60);
        assert_eq!(pick(&tips, &seen, "Workspace", None, now), None, "shown a minute ago");
        assert_eq!(pick(&tips, &seen, "Workspace", None, now + REST).unwrap().id, "c", "a day later");
        assert_eq!(pick(&tips, &seen, "Ticket", None, now).unwrap().id, "d");
        assert_eq!(pick(&tips, &seen, "Ticket", Some("d"), now), None, "Next never gives the same one");
    }

    #[test]
    fn the_longest_unseen_comes_first() {
        let tips = vec![tip("a", "Workspace", None), tip("b", "Workspace", None)];
        let seen = Seen { shown: HashMap::from([("a".into(), 10), ("b".into(), 5)]), ..Default::default() };
        assert_eq!(pick(&tips, &seen, "Workspace", None, REST * 10).unwrap().id, "b");
    }
}
