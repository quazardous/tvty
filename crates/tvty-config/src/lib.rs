//! Settings as data, with no user interface in them.
//!
//! - [`files`]: where a program keeps them (the XDG directories), a set of
//!   settings as a Rust type ([`Stored`]) read with its defaults and written
//!   whole or not at all, and the log;
//! - [`schema`]: each setting declared once — its key, the options page
//!   that shows it, its kind and bounds — to build the pages, reach a value
//!   by its key, and check a file edited by hand.

pub mod files;
pub mod schema;

pub use files::{LogTee, Place, Stored, dir, modified, parse, path, read, render, write_atomic};
pub use schema::{Kind, Schema, Setting, Value};
