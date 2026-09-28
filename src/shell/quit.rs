//! Quitting tvty with Claude Code loops of this machine running: stop them
//! (claude-loop stop: they stay restartable) or keep them, as asked — or as
//! the user's remembered choice says (Options > Layout > Sessions). The
//! loops tvty stopped are kept in the workspace; at the next start, tvty
//! offers to restart them (claude-loop restart --resume), asked or not.

use std::time::Duration;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use tvty_config::Value;

use super::Shell;
use crate::theme::p;
use crate::ui::buttons;

/// The longest tvty waits for the loops to stop before it quits anyway.
const STOP_WAIT: Duration = Duration::from_secs(15);

/// What a dialog of this module asks.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Ask {
    /// Quitting: stop these loops too?
    Quit(Vec<String>),
    /// At start: restart the loops stopped when tvty quit?
    Restart(Vec<String>),
}

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
            _ => {
                self.ask = Some(Ask::Quit(loops));
                self.remember = false;
                cx.notify();
            }
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
                return crate::activity::publish(cx, crate::activity::Activity::failed(None, "restart tvty", error));
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
                crate::activity::publish(cx, crate::activity::Activity::failed(None, "restart tvty", format!("{}: {error}", exe.display())));
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

    fn quit_now(&mut self, cx: &mut Context<Self>) {
        self.quitting = true;
        self.save_workspace(cx);
        self.settings.save(cx);
        cx.quit();
    }

    /// Stops `loops` (at most [`STOP_WAIT`]), keeps them to offer at the next
    /// start, then quits.
    fn stop_and_quit(&mut self, loops: Vec<String>, cx: &mut Context<Self>) {
        self.stopping_all = true;
        self.ask = None;
        cx.notify();
        let aiball = self.aiball.clone();
        let known: Vec<crate::loops::KnownLoop> = self.board.known.iter().filter(|l| loops.contains(&l.name)).cloned().collect();
        cx.spawn(async move |this, cx| {
            let (done, wait) = futures::channel::oneshot::channel();
            std::thread::spawn(move || {
                let stopped: Vec<String> = known
                    .into_iter()
                    .filter(|l| match crate::loops::stop(&aiball, l) {
                        Ok(()) => true,
                        Err(error) => {
                            log::warn!("{error:#}");
                            false
                        }
                    })
                    .map(|l| l.name)
                    .collect();
                let _ = done.send(stopped);
            });
            let timer = cx.background_executor().timer(STOP_WAIT);
            let stopped = futures::select_biased! {
                stopped = futures::FutureExt::fuse(wait) => stopped.unwrap_or_default(),
                _ = futures::FutureExt::fuse(timer) => {
                    log::warn!("quit: the loops took longer than {}s to stop", STOP_WAIT.as_secs());
                    loops
                }
            };
            let _ = this.update(cx, |shell, cx| {
                shell.settings.workspace.stopped_on_quit = stopped;
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
        let names: Vec<String> = std::mem::take(&mut self.settings.workspace.stopped_on_quit)
            .into_iter()
            .filter(|n| self.board.known.iter().any(|l| l.name == *n) && !running.contains(n))
            .collect();
        self.settings.save(cx);
        if names.is_empty() {
            return;
        }
        match self.applied.sessions.on_start.as_deref() {
            Some("restart") => self.restart_loops(names, cx),
            Some("leave") => {}
            _ => {
                self.ask = Some(Ask::Restart(names));
                self.remember = false;
                cx.notify();
            }
        }
    }

    fn restart_loops(&mut self, names: Vec<String>, cx: &mut Context<Self>) {
        self.ask = None;
        cx.notify();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let count = names.len();
            let failed = cx
                .background_executor()
                .spawn(async move {
                    names
                        .into_iter()
                        .filter_map(|name| crate::loops::restart(&aiball, &name).err().map(|e| format!("{e:#}")))
                        .collect::<Vec<_>>()
                })
                .await;
            let _ = this.update(cx, |shell, cx| {
                let activity = match failed.first() {
                    None => crate::activity::Activity::done(None, format!("{count} session(s) restarted, resuming their conversation")),
                    Some(error) => crate::activity::Activity::failed(None, "restart of the stopped sessions", error.clone()),
                };
                crate::activity::publish(cx, activity);
                let _ = shell.refresh_now.unbounded_send(());
            });
        })
        .detach();
    }

    /// The answer: `yes` stops (or restarts) them; remembered if asked.
    fn answer(&mut self, yes: bool, cx: &mut Context<Self>) {
        let Some(ask) = self.ask.take() else { return };
        let remember = self.remember;
        match ask {
            Ask::Quit(loops) => {
                if remember {
                    self.set_pref("sessions.on_quit", Value::Choice(Some(if yes { "stop" } else { "keep" }.into())), cx);
                }
                if yes { self.stop_and_quit(loops, cx) } else { self.quit_now(cx) }
            }
            Ask::Restart(names) => {
                if remember {
                    self.set_pref("sessions.on_start", Value::Choice(Some(if yes { "restart" } else { "leave" }.into())), cx);
                }
                if yes {
                    self.restart_loops(names, cx);
                }
            }
        }
        cx.notify();
    }

    fn cancel_ask(&mut self, cx: &mut Context<Self>) {
        self.ask = None;
        cx.notify();
    }

    /// The dialog, over everything; or "stopping…" while the loops stop.
    pub(super) fn quit_dialog(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.stopping_all {
            return Some(dialog(div().child("Stopping the Claude Code sessions, then quitting…")).into_any_element());
        }
        let ask = self.ask.as_ref()?;
        let (title, loops, yes, no, cancel) = match ask {
            Ask::Quit(loops) => ("Stop the Claude Code sessions too?", loops, "Quit and stop them", "Quit, keep them running", true),
            Ask::Restart(loops) => ("Restart the sessions stopped when tvty quit?", loops, "Restart them", "Not now", false),
        };
        let list = loops.iter().map(|name| {
            let known = self.board.known.iter().find(|l| l.name == *name);
            let who = known.and_then(|l| l.agent().map(str::to_string)).unwrap_or_else(|| name.clone());
            let project = known.and_then(|l| l.project.clone()).unwrap_or_default();
            let place = if known.is_some_and(|l| l.on_host()) { "host" } else { "tmux" };
            div().flex().gap_2().child(div().text_color(p().text).child(who)).child(div().text_color(p().muted).child(format!("{project} · {place}")))
        });
        let remember = self.remember;
        let body = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_lg().font_weight(FontWeight::BOLD).child(title))
            .child(div().flex().flex_col().gap_1().text_sm().children(list))
            .child(
                buttons::link("quit-remember", if remember { "☑" } else { "☐" })
                    .gap_2()
                    .text_sm()
                    .text_color(p().text)
                    .child("Remember this choice (Options > Layout > Sessions)")
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.remember = !shell.remember;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .justify_end()
                    .when(cancel, |d| {
                        d.child(buttons::chip("quit-cancel", "Cancel").px_3().py_1().on_click(cx.listener(|shell, _, _, cx| shell.cancel_ask(cx))))
                    })
                    .child(buttons::chip("quit-no", no).px_3().py_1().on_click(cx.listener(|shell, _, _, cx| shell.answer(false, cx))))
                    .child(buttons::primary("quit-yes", yes).on_click(cx.listener(|shell, _, _, cx| shell.answer(true, cx)))),
            );
        Some(dialog(body).into_any_element())
    }

    /// Esc: the quit dialog is cancelled; the restart one, "not now".
    pub(super) fn escape_ask(&mut self, cx: &mut Context<Self>) -> bool {
        match self.ask {
            Some(Ask::Quit(_)) => self.cancel_ask(cx),
            Some(Ask::Restart(_)) => self.answer(false, cx),
            None => return false,
        }
        true
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
fn dialog(body: impl IntoElement) -> Stateful<Div> {
    div()
        .id("quit-dialog")
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
