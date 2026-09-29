//! Versions: whether a newer Terminal Velocity is out (asked of GitHub at
//! start and once a day, when the user lets it), and whether aiball is
//! recent enough — each said once, in a notification. The updater
//! (`tvty-updater`, the menu's "Updates…") does the installing.

use std::time::Duration;

use gpui_kit::*;

use crate::activity::{self, Activity};
use crate::notify::Kind;

/// The latest release known, when newer than this one.
#[derive(Default)]
pub struct Newer(pub Option<String>);

impl Global for Newer {}

/// The newer release, when one is known.
pub fn newer(cx: &App) -> Option<String> {
    cx.try_global::<Newer>().and_then(|n| n.0.clone())
}

/// Starts the checks: aiball's version once, tvty's latest release at start
/// and every day while `check` holds.
pub fn init(check: impl Fn(&App) -> bool + 'static, cx: &mut App) {
    cx.set_global(Newer::default());
    cx.spawn(async move |cx| {
        // After the start's own work.
        cx.background_executor().timer(Duration::from_secs(10)).await;
        aiball_recent_enough(cx).await;
        loop {
            if cx.update(|cx| check(cx)) {
                latest_release(cx).await;
            }
            cx.background_executor().timer(Duration::from_secs(24 * 3600)).await;
        }
    })
    .detach();
}

async fn latest_release(cx: &mut AsyncApp) {
    let status = cx.background_executor().spawn(async { tvty_updater::tvty_status() }).await;
    let running = env!("CARGO_PKG_VERSION");
    let Some(latest) = status.latest.filter(|latest| tvty_updater::older(running, latest)) else { return };
    let _ = cx.update(|cx| {
        let said_before = cx.global::<Newer>().0.as_deref() == Some(latest.as_str());
        cx.set_global(Newer(Some(latest.clone())));
        if !said_before && !announced(&latest) {
            remember(&latest);
            activity::publish(
                cx,
                Activity::news("tvty", Kind::Info, None, format!("Terminal Velocity {latest} is out (you run {running}): the menu's Updates… installs it")),
            );
        }
        cx.refresh_windows();
    });
}

async fn aiball_recent_enough(cx: &mut AsyncApp) {
    let version = cx.background_executor().spawn(async { crate::aiball::Aiball::from_env().daemon_version() }).await;
    let Ok(version) = version else { return };
    if tvty_updater::older(&version, tvty_updater::MIN_AIBALL) {
        let _ = cx.update(|cx| {
            activity::publish(
                cx,
                Activity::news(
                    "tvty",
                    Kind::Error,
                    None,
                    format!("aiball {version} is older than Terminal Velocity needs ({}): the menu's Updates… updates it", tvty_updater::MIN_AIBALL),
                ),
            )
        });
    }
}

/// The version last said to be out, so that a restart does not say it again.
fn announced_file() -> Option<std::path::PathBuf> {
    crate::config::dir(crate::config::Place::State).map(|d| d.join("update-announced"))
}

fn announced(version: &str) -> bool {
    announced_file().and_then(|f| std::fs::read_to_string(f).ok()).is_some_and(|v| v.trim() == version)
}

fn remember(version: &str) {
    if let Some(file) = announced_file() {
        let _ = std::fs::write(file, version);
    }
}

/// Starts the updater, on its own; `false` when it is not installed.
pub fn open_updater() -> bool {
    std::process::Command::new(tvty_updater::program("tvty-updater"))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .is_ok()
}
