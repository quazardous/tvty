//! Fonts of the user's own, loaded at start: every `.ttf`, `.otf` and `.ttc`
//! in `$XDG_DATA_HOME/tvty/fonts/` (by default `~/.local/share/tvty/fonts/`).
//!
//! What it is for first: colour emoji. GPUI colours an emoji font drawn in
//! bitmaps (CBDT) or COLR v0, not COLR v1, which is what Fedora ships; a
//! `NotoColorEmoji.ttf` in bitmaps put there colours them.

//!
//! And the terminals' fonts, which tvty brings itself (`assets/fonts`, each
//! under the SIL Open Font License): Source Code Pro, and JuliaMono for the
//! symbols it lacks (Claude Code's spinner, its marks), every one of them a
//! cell wide. A system's own fonts for those are of any width, and the word
//! after a spinner moved with each of its characters.

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::OnceLock;

use gpui_kit::*;

/// Where the user's fonts are.
pub fn dir() -> Option<PathBuf> {
    Some(crate::config::dir(crate::config::Place::Data)?.join("fonts"))
}

/// The fonts tvty brings.
const OWN: &[&[u8]] = &[
    include_bytes!("../assets/fonts/SourceCodePro-Regular.ttf"),
    include_bytes!("../assets/fonts/SourceCodePro-Bold.ttf"),
    include_bytes!("../assets/fonts/SourceCodePro-It.ttf"),
    include_bytes!("../assets/fonts/SourceCodePro-BoldIt.ttf"),
    include_bytes!("../assets/fonts/JuliaMono-Regular.ttf"),
];

/// The font of the symbols the terminals' font lacks, a cell wide each.
pub const SYMBOLS: &str = "JuliaMono";

/// Whether `font` (one of tvty's own) draws `c`.
fn draws(font: &'static [u8], c: char) -> bool {
    swash::FontRef::from_index(font, 0).is_some_and(|font| font.charmap().map(c) != 0)
}

thread_local! {
    /// What [`is_symbol`] answered, a character at a time: asked of every
    /// cell at every frame, it reads the fonts' tables once a character.
    static SYMBOL: std::cell::RefCell<std::collections::HashMap<char, bool>> = Default::default();
}

/// Whether `c` is a symbol to draw from [`SYMBOLS`]: one the terminals' own
/// font lacks, and that one has.
pub fn is_symbol(c: char) -> bool {
    if c.is_ascii() || mono() != MONO[0] {
        return false;
    }
    SYMBOL.with_borrow_mut(|known| *known.entry(c).or_insert_with(|| !draws(OWN[0], c) && draws(OWN[4], c)))
}

/// Loads tvty's own fonts, then the user's; answers how many of the user's.
pub fn load(cx: &mut App) -> usize {
    if let Err(error) = cx.text_system().add_fonts(OWN.iter().map(|bytes| Cow::Borrowed(*bytes)).collect()) {
        log::warn!("tvty's own fonts: {error:#}");
    }
    let Some(dir) = dir() else { return 0 };
    let Ok(entries) = std::fs::read_dir(&dir) else { return 0 };
    let fonts: Vec<Cow<'static, [u8]>> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc"))
        })
        .filter_map(|p| match std::fs::read(&p) {
            Ok(bytes) => Some(Cow::Owned(bytes)),
            Err(error) => {
                log::warn!("font {}: {error}", p.display());
                None
            }
        })
        .collect();
    let count = fonts.len();
    if count > 0 {
        match cx.text_system().add_fonts(fonts) {
            Ok(()) => log::info!("{count} font(s) loaded from {}", dir.display()),
            Err(error) => log::warn!("fonts in {}: {error:#}", dir.display()),
        }
    }
    count
}

/// The terminals' font — tvty's own — and those that stand for it should it
/// not load, in order: each system ships some of them, none ships them all.
/// All fixed-width — a terminal's grid is one width per cell, and a font of
/// another kind (what a system gives for a name it does not know) leaves the
/// cursor ahead of the text.
const MONO: &[&str] = &["Source Code Pro", "Cascadia Mono", "Consolas", "Menlo", "DejaVu Sans Mono", "Liberation Mono", "Adwaita Mono"];

static CHOSEN: OnceLock<&'static str> = OnceLock::new();

/// The first of `wanted` among `installed`, whatever its case.
fn first_installed(wanted: &[&'static str], installed: &[String]) -> Option<&'static str> {
    wanted.iter().copied().find(|name| installed.iter().any(|have| have.eq_ignore_ascii_case(name)))
}

/// Chooses the terminals' font among the machine's, once the user's own are
/// loaded; answers it.
pub fn choose_mono(cx: &App) -> &'static str {
    CHOSEN.get_or_init(|| {
        let chosen = first_installed(MONO, &cx.text_system().all_font_names());
        match chosen {
            Some(name) => log::info!("the terminals' font: {name}"),
            None => log::warn!("none of the terminals' fonts is installed ({}): the system chooses", MONO.join(", ")),
        }
        chosen.unwrap_or(MONO[0])
    })
}

/// The terminals' font: the one chosen at start.
pub fn mono() -> &'static str {
    CHOSEN.get().copied().unwrap_or(MONO[0])
}

#[cfg(test)]
mod tests {
    use super::{MONO, first_installed};

    #[test]
    fn a_symbol_is_what_the_terminals_font_lacks_and_the_symbols_font_has() {
        // Claude Code's spinner, and its mark before a message.
        for c in ['\u{2722}', '\u{2733}', '\u{2736}', '\u{273B}', '\u{273D}', '\u{23FA}'] {
            assert!(super::is_symbol(c), "{c}");
        }
        // Its own letters and marks; a character neither font has.
        for c in ['m', 'é', '\u{00B7}', '\u{25CF}', '\u{E000}'] {
            assert!(!super::is_symbol(c), "{c}");
        }
    }

    #[test]
    fn the_terminals_font_is_the_first_the_machine_has() {
        let have = |names: &[&str]| names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
        assert_eq!(first_installed(MONO, &have(&["Consolas", "Cascadia Mono", "Segoe UI"])), Some("Cascadia Mono"));
        assert_eq!(first_installed(MONO, &have(&["source code pro", "Consolas"])), Some("Source Code Pro"));
        assert_eq!(first_installed(MONO, &have(&["Segoe UI"])), None);
    }
}
