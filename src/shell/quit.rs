//! Quitting tvty with Claude Code loops of this machine running: stop them
//! (claude-loop stop: they stay restartable) or keep them, as asked — or as
//! the user's remembered choice says (Options > Layout > Sessions). The
//! loops tvty stopped are kept in the workspace, with their AFK hold; at
//! the next start, tvty offers to restart them, resuming their
//! conversation — as they were (a held one held again) or fresh (on their
//! own) — asked or not.

use crate::ui::Named as _;
use std::time::Duration;

use gpui_kit::*;

use super::Shell;
use crate::theme::p;

/// The longest tvty waits for the loops to stop before it quits anyway.
const STOP_WAIT: Duration = Duration::from_secs(15);

impl Shell {
    /// This machine's loops that run now: known here, their session live.
    fn running_loops(&self) -> Vec<String> {
        self.board
            .known
            .iter()
            .filter(|l| self.terminal_of(&l.session()).is_some())
            .map(|l| l.name.clone())
            .collect()
    }

    /// The window is asked to close: tvty quits at once, or asks first.
    /// Answers whether it may close now.
    pub(super) fn close_asked(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.remember_window(window);
        if self.quitting {
            return true;
        }
        self.ask_quit(window, cx);
        false
    }

    /// Quits, having dealt with the running loops as the user chose.
    pub(super) fn ask_quit(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.quitting || self.stopping_all {
            return;
        }
        let loops = self.running_loops();
        if loops.is_empty() {
            return self.quit_now(cx);
        }
        match self.applied.sessions.on_quit.as_deref() {
            Some("stop") => self.stop_and_quit(loops, cx),
            Some("keep") => self.quit_now(cx),
            // Asked: which stop, which run on (the sessions' picker).
            _ => self.pick_quit(loops, cx),
        }
    }

    /// Restarts tvty: the window only — the Claude Code sessions and the
    /// terminals' tmux sessions run on, and the new tvty opens the same
    /// terminals again. Its binary is the one on disk now (a new build
    /// included).
    pub(super) fn restart_tvty(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.help_menu = false;
        self.remember_window(window);
        let exe = match own_binary() {
            Ok(exe) => exe,
            Err(error) => {
                return crate::activity::publish(cx, crate::activity::Activity::failed(None, &crate::t!("messages-restart-tvty-failed"), error));
            }
        };
        self.quitting = true;
        self.save_workspace(cx);
        self.settings.save(cx);
        crate::settings::flush(cx);
        crate::instance::release();
        match std::process::Command::new(&exe).spawn() {
            Ok(child) => {
                log::info!("restart: {} started (pid {}), this one quits", exe.display(), child.id());
                cx.quit();
            }
            Err(error) => {
                self.quitting = false;
                crate::activity::publish(cx, crate::activity::Activity::failed(None, &crate::t!("messages-restart-tvty-failed"), format!("{}: {error}", exe.display())));
            }
        }
    }

    /// The window's state, kept with the layout: the next start opens it
    /// so (maximized, full screen, or windowed at its size).
    fn remember_window(&mut self, window: &Window) {
        let (state, bounds) = match window.window_bounds() {
            WindowBounds::Windowed(b) => ("windowed", b),
            WindowBounds::Maximized(b) => ("maximized", b),
            WindowBounds::Fullscreen(b) => ("fullscreen", b),
        };
        self.settings.layout.window = Some(crate::settings::WindowState {
            state: state.into(),
            width: f32::from(bounds.size.width),
            height: f32::from(bounds.size.height),
        });
    }

    pub(super) fn quit_now(&mut self, cx: &mut Context<Self>) {
        self.quitting = true;
        self.save_workspace(cx);
        self.settings.save(cx);
        cx.quit();
    }

    /// Stops `loops` (at most [`STOP_WAIT`]), keeps them to offer at the next
    /// start, then quits.
    pub(super) fn stop_and_quit(&mut self, loops: Vec<String>, cx: &mut Context<Self>) {
        self.stopping_all = true;
        self.picker = None;
        cx.notify();
        let aiball = self.aiball.clone();
        let known: Vec<crate::loops::KnownLoop> = self.board.known.iter().filter(|l| loops.contains(&l.name)).cloned().collect();
        // Their hold is aiball's to keep, from their bars: a restart as
        // they were starts each in it.
        cx.spawn(async move |this, cx| {
            let (done, wait) = futures::channel::oneshot::channel();
            std::thread::spawn(move || {
                let began = std::time::Instant::now();
                let asked: Vec<crate::loops::KnownLoop> = known
                    .into_iter()
                    .filter(|l| match crate::loops::stop(&aiball, l) {
                        Ok(()) => true,
                        Err(error) => {
                            log::warn!("{error:#}");
                            false
                        }
                    })
                    .collect();
                // Asked is not stopped: a loop the order did not reach runs
                // on. Gone once its session is no longer listed.
                let mut still: Vec<crate::loops::KnownLoop> = asked;
                let mut stopped: Vec<String> = Vec::new();
                while !still.is_empty() && began.elapsed() < STOP_WAIT {
                    match aiball.running_agents() {
                        Ok(running) => {
                            let (gone, on): (Vec<_>, Vec<_>) = still.into_iter().partition(|l| l.agent().is_none_or(|a| !running.iter().any(|r| r == a)));
                            stopped.extend(gone.into_iter().map(|l| l.name));
                            still = on;
                        }
                        Err(error) => log::warn!("quit: the sessions could not be read: {error:#}"),
                    }
                    if !still.is_empty() {
                        std::thread::sleep(Duration::from_millis(400));
                    }
                }
                let still: Vec<String> = still.into_iter().map(|l| l.name).collect();
                if !still.is_empty() {
                    log::warn!("quit: still running when tvty quits (their stop did not take): {}", still.join(", "));
                }
                let _ = done.send((stopped, still));
            });
            let timer = cx.background_executor().timer(STOP_WAIT + Duration::from_secs(2));
            let (stopped, still) = futures::select_biased! {
                ended = futures::FutureExt::fuse(wait) => ended.unwrap_or_default(),
                _ = futures::FutureExt::fuse(timer) => {
                    log::warn!("quit: the loops took longer than {}s to stop", STOP_WAIT.as_secs());
                    (Vec::new(), loops)
                }
            };
            let _ = this.update(cx, |shell, cx| {
                shell.settings.workspace.stopped_on_quit = stopped;
                shell.settings.workspace.still_on_quit = still;
                shell.quit_now(cx);
            });
        })
        .detach();
    }

    /// Once the board is there, at start: the loops stopped when tvty quit,
    /// restarted, left, or offered — once.
    pub(super) fn offer_restart(&mut self, cx: &mut Context<Self>) {
        if self.restart_offered || !self.live.ready() {
            return;
        }
        self.restart_offered = true;
        let running = self.running_loops();
        let known = |n: &String| self.board.known.iter().any(|l| l.name == *n);
        // Those whose stop did not take when tvty quit: offered with the
        // others if they stopped since, said as still running otherwise.
        // (A loop aiball no longer lists as stopped is said, not dropped.)
        let (still_stopped, still_running): (Vec<String>, Vec<String>) =
            std::mem::take(&mut self.settings.workspace.still_on_quit).into_iter().partition(|n| known(n) && !running.contains(n));
        let mut names: Vec<String> = std::mem::take(&mut self.settings.workspace.stopped_on_quit)
            .into_iter()
            .filter(|n| known(n) && !running.contains(n))
            .collect();
        names.extend(still_stopped);
        self.settings.save(cx);
        if !still_running.is_empty() {
            let agents: Vec<String> = still_running
                .iter()
                .map(|n| self.board.known.iter().find(|l| l.name == *n).and_then(|l| l.agent().map(str::to_string)).unwrap_or_else(|| n.clone()))
                .collect();
            // Said as a failure: it is one, and one's own notices may be off.
            crate::activity::publish(
                cx,
                crate::activity::Activity::failed(None, &crate::t!("messages-quit-stop-failed"), crate::t!("messages-ran-on", agents = agents.join(", "))),
            );
        }
        if names.is_empty() {
            return;
        }
        match self.applied.sessions.on_start.as_deref() {
            Some("restart") => self.restart_loops(names, true, cx),
            Some("fresh") => self.restart_loops(names, false, cx),
            Some("leave") => {}
            _ => self.pick_restart(names, still_running, cx),
        }
    }

    /// Restarts `names`, resuming their conversation: each in its hold as
    /// aiball keeps it (`as_they_were`: held ones held from their first
    /// breath, no order sent after), or all on their own.
    pub(super) fn restart_loops(&mut self, names: Vec<String>, as_they_were: bool, cx: &mut Context<Self>) {
        cx.notify();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let count = names.len();
            let failed = cx
                .background_executor()
                .spawn(async move {
                    let afk = (!as_they_were).then_some("off");
                    names.iter().filter_map(|name| crate::loops::restart(&aiball, name, afk).err().map(|error| format!("{error:#}"))).collect::<Vec<_>>()
                })
                .await;
            let _ = this.update(cx, |shell, cx| {
                let activity = match failed.first() {
                    None => crate::activity::Activity::done(
                        None,
                        crate::t!(if as_they_were { "messages-restarted-as-were" } else { "messages-restarted-fresh" }, count = count),
                    ),
                    Some(error) => crate::activity::Activity::failed(None, &crate::t!("messages-restart-failed"), error.clone()),
                };
                crate::activity::publish(cx, activity);
                let _ = shell.refresh_now.unbounded_send(());
            });
        })
        .detach();
    }

    /// "Stopping…" over everything while the loops stop, before quitting.
    pub(super) fn quit_dialog(&self) -> Option<AnyElement> {
        self.stopping_all.then(|| dialog(div().child(crate::t!("messages-stopping"))).into_any_element())
    }
}

/// This tvty's binary as it is on disk now: rebuilt since it started, the
/// running one's path reads "… (deleted)", and the new file is the one.
fn own_binary() -> Result<std::path::PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let text = exe.to_string_lossy();
    let path = std::path::PathBuf::from(text.strip_suffix(" (deleted)").unwrap_or(&text));
    if path.is_file() { Ok(path) } else { Err(format!("{} is gone", path.display())) }
}


/// A card over a dimmed window, which takes every click.
pub(super) fn dialog(body: impl IntoElement) -> Stateful<Div> {
    div()
        .named("quit-dialog")
        .occlude()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(gpui_kit::black().opacity(0.45))
        .child(
            div()
                .w(px(620.))
                .max_w(relative(0.9))
                .p_5()
                .rounded_lg()
                .border_1()
                .border_color(p().border)
                .bg(p().surface)
                .text_color(p().text)
                .child(body),
        )
}
