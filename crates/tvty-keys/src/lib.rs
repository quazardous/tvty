//! Keyboard shortcuts as data, with no user interface in them.
//!
//! - [`Key`]: a key as people write it (`Ctrl+Shift+B`, `ctrl-shift-b`),
//!   in one canonical form, and as people read it;
//! - [`Keymap`]: commands bound to keys in nested contexts — the defaults a
//!   program declares, the user's changes over them, conflicts and masked
//!   keys;
//! - [`KeymapFile`]: those changes as a TOML file, the difference from the
//!   defaults only.

pub mod key;
pub mod keymap;

pub use key::{Key, KeyError, Stroke};
pub use keymap::{Binding, Command, Keymap, KeymapError, KeymapFile, Target};
