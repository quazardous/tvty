//! Fonts of the user's own, loaded at start: every `.ttf`, `.otf` and `.ttc`
//! in `$XDG_DATA_HOME/tvty/fonts/` (by default `~/.local/share/tvty/fonts/`).
//!
//! What it is for first: colour emoji. GPUI colours an emoji font drawn in
//! bitmaps (CBDT) or COLR v0, not COLR v1, which is what Fedora ships; a
//! `NotoColorEmoji.ttf` in bitmaps put there colours them.

use std::borrow::Cow;
use std::path::PathBuf;

use gpui_kit::*;

/// Where the user's fonts are.
pub fn dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))?;
    Some(base.join("tvty").join("fonts"))
}

/// Loads the user's fonts; answers how many.
pub fn load(cx: &mut App) -> usize {
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
