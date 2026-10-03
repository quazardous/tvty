//! The commit tvty is built from, for Options > About: with the version,
//! it says which build runs.

use std::process::Command;

fn main() {
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let dirty = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .ok()
        .is_some_and(|o| !o.stdout.is_empty());
    let commit = if dirty && !commit.is_empty() { format!("{commit}+") } else { commit };
    println!("cargo:rustc-env=TVTY_COMMIT={commit}");
    // Built again when the checked-out commit moves.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
    println!("cargo:rerun-if-changed=.git/index");
    // The words built in (src/i18n.rs): a file added there is one rust-embed
    // does not know yet, so tvty is built again whenever the folder moves.
    println!("cargo:rerun-if-changed=assets/locales");
    windows_icon();
}

/// On Windows, the icon Explorer, the Start menu and the taskbar show.
#[cfg(windows)]
fn windows_icon() {
    println!("cargo:rerun-if-changed=assets/tvty.ico");
    println!("cargo:rerun-if-changed=packaging/windows/tvty.rc");
    // Drawn by `cargo run --example windows_icon`, which builds this first.
    if !std::path::Path::new("assets/tvty.ico").exists() {
        println!("cargo:warning=no assets/tvty.ico: tvty.exe without its icon");
        return;
    }
    embed_resource::compile("packaging/windows/tvty.rc", embed_resource::NONE).manifest_required().expect("the Windows resources");
}

#[cfg(not(windows))]
fn windows_icon() {}
