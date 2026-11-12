//! What tvty-egui cannot do, said when it is asked: it reads a scene, not
//! aiball nor tmux, so an action of theirs (a session started, a project
//! made) shows a toast instead of doing nothing (egui-pop's toasts).

use egui_pop::api::AppHandle;
use egui_pop::api::toasts::{TOASTS, Toast};

/// "<what>: not in tvty-egui, which reads a scene (no aiball, no tmux)".
pub fn not_here(app: &AppHandle, what: &str) {
    if let Some(toasts) = app.service(&TOASTS) {
        toasts.notify(Toast::info(&format!("{what}: not in tvty-egui, which reads a scene (no aiball, no tmux)")));
    }
}
