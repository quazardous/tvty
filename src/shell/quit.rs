//! Quitting tvty with Claude Code loops of this machine running: stop them
//! (claude-loop stop: they stay restartable) or keep them, as asked — or as
//! the user's remembered choice says (Options > Layout > Sessions). The
//! loops tvty stopped are kept in the workspace, with their AFK hold; at
//! the next start, tvty offers to restart them, resuming their
//! conversation — as they were (a held one held again) or fresh (on their
//! own) — asked or not.

use std::collections::HashMap;
use std::time::Duration;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use tvty_config::Value;

use super::Shell;
use crate::theme::p;
use crate::tip::Tip as _;
use crate::ui::buttons;

/// The longest tvty waits for the loops to stop before it quits anyway.
const STOP_WAIT: Duration = Duration::from_secs(15);

/// What a dialog of this module asks.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Ask {
    /// Quitting: stop these loops too?
    Quit(Vec<String>),
    /// At start: restart the loops stopped when tvty quit? With their AFK
    /// mode then, by loop.
    Restart(Vec<String>, HashMap<String, String>),
}

/// A dialog's answer.
#[derive(Clone, Copy, PartialEq)]
enum Answer {
    /// Stop them too; restart them as they were.
    Yes,
    /// Restart them fresh: on their own, no hold.
    Fresh,
    /// Keep them running; leave them stopped.
    No,
}

/// The AFK action that puts a loop back as it was: held until let go,
/// held again ten minutes, or on its own.
fn afk_action(mode: Option<&str>) -> &'static str {
    match mode {
        Some("wait_inf") => "arm_inf",
        Some("wait_10m") => "arm_10m",
        _ => "off",
    }
}

/// A session's mark, as the lists say it: ▶ on its own, ‖ held for now,
/// ■ held until let go.
fn hold_mark(presence: Option<&str>, mode: Option<&str>) -> (&'static str, Hsla) {
    match (presence, mode) {
        (_, Some("wait_inf")) => ("■", p().danger),
        (Some("wait") | Some("stop"), _) | (_, Some("wait_10m")) => ("‖", p().warning),
        (Some("loop"), _) => ("▶", p().success),
        _ => ("·", p().muted),
    }
}

/// "1 session", "3 sessions".
fn count(n: usize, what: &str) -> String {
    format!("{n} {what}{}", if n == 1 { "" } else { "s" })
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
        // Their hold as it is: put back when they restart as they were.
        let holds: HashMap<String, String> = known
            .iter()
            .filter_map(|l| {
                let bar = self.board.bars.get(l.agent()?).filter(|b| !b.stale)?;
                let mode = bar.bar.afk.mode.clone();
                (!mode.is_empty()).then(|| (l.name.clone(), mode))
            })
            .collect();
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
                shell.settings.workspace.holds_on_quit = holds.into_iter().filter(|(name, _)| stopped.contains(name)).collect();
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
        let holds = std::mem::take(&mut self.settings.workspace.holds_on_quit);
        self.settings.save(cx);
        if names.is_empty() {
            return;
        }
        match self.applied.sessions.on_start.as_deref() {
            Some("restart") => self.restart_loops(names, holds, true, cx),
            Some("fresh") => self.restart_loops(names, holds, false, cx),
            Some("leave") => {}
            _ => {
                self.ask = Some(Ask::Restart(names, holds));
                self.remember = false;
                cx.notify();
            }
        }
    }

    /// Restarts `names`, resuming their conversation; then puts each back
    /// on hold as it was (`as_they_were`), or frees them all.
    fn restart_loops(&mut self, names: Vec<String>, holds: HashMap<String, String>, as_they_were: bool, cx: &mut Context<Self>) {
        self.ask = None;
        cx.notify();
        let aiball = self.aiball.clone();
        let agents: HashMap<String, String> =
            self.board.known.iter().filter_map(|l| Some((l.name.clone(), l.agent()?.to_string()))).collect();
        cx.spawn(async move |this, cx| {
            let count = names.len();
            let executor = cx.background_executor().clone();
            let failed = cx
                .background_executor()
                .spawn(async move {
                    let mut failed = Vec::new();
                    let mut started = Vec::new();
                    for name in names {
                        match crate::loops::restart(&aiball, &name) {
                            Ok(_) => started.push(name),
                            Err(error) => failed.push(format!("{error:#}")),
                        }
                    }
                    // Its hold, once the loop answers (it boots first).
                    for name in started {
                        let Some(agent) = agents.get(&name) else { continue };
                        let action = if as_they_were { afk_action(holds.get(&name).map(String::as_str)) } else { "off" };
                        let mut tries = 0;
                        loop {
                            match aiball.afk(agent, action) {
                                Ok(()) => {
                                    log::info!("restart: {name} put back {action}");
                                    break;
                                }
                                Err(error) if tries >= 15 => {
                                    failed.push(format!("{name}: its hold ({action}): {error:#}"));
                                    break;
                                }
                                Err(_) => {
                                    tries += 1;
                                    executor.timer(Duration::from_secs(2)).await;
                                }
                            }
                        }
                    }
                    failed
                })
                .await;
            let _ = this.update(cx, |shell, cx| {
                let how = if as_they_were { "as they were" } else { "fresh" };
                let activity = match failed.first() {
                    None => crate::activity::Activity::done(None, format!("{} restarted {how}, resuming their conversation", count_of(count))),
                    Some(error) => crate::activity::Activity::failed(None, "restart of the stopped sessions", error.clone()),
                };
                crate::activity::publish(cx, activity);
                let _ = shell.refresh_now.unbounded_send(());
            });
        })
        .detach();
    }

    /// The answer, remembered if asked.
    fn answer(&mut self, answer: Answer, cx: &mut Context<Self>) {
        let Some(ask) = self.ask.take() else { return };
        let remember = self.remember;
        match ask {
            Ask::Quit(loops) => {
                let yes = answer == Answer::Yes;
                if remember {
                    self.set_pref("sessions.on_quit", Value::Choice(Some(if yes { "stop" } else { "keep" }.into())), cx);
                }
                if yes { self.stop_and_quit(loops, cx) } else { self.quit_now(cx) }
            }
            Ask::Restart(names, holds) => {
                if remember {
                    let choice = match answer {
                        Answer::Yes => "restart",
                        Answer::Fresh => "fresh",
                        Answer::No => "leave",
                    };
                    self.set_pref("sessions.on_start", Value::Choice(Some(choice.into())), cx);
                }
                if answer != Answer::No {
                    self.restart_loops(names, holds, answer == Answer::Yes, cx);
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
        let (title, loops) = match ask {
            Ask::Quit(loops) => ("Stop the Claude Code sessions too?", loops),
            Ask::Restart(loops, _) => ("Restart the sessions stopped when tvty quit?", loops),
        };
        // The sessions by project, each with its mark: live when quitting,
        // as it was when restarting.
        let mut projects: Vec<(String, Vec<AnyElement>)> = Vec::new();
        let mut names: Vec<&String> = loops.iter().collect();
        let project_of = |name: &str| self.board.known.iter().find(|l| l.name == name).and_then(|l| l.project.clone()).unwrap_or_default();
        names.sort_by_key(|name| (project_of(name), name.to_string()));
        for name in names {
            let known = self.board.known.iter().find(|l| l.name == *name);
            let agent = known.and_then(|l| l.agent().map(str::to_string));
            let (glyph, colour) = match ask {
                Ask::Quit(_) => {
                    let bar = agent.as_ref().and_then(|a| self.board.bars.get(a)).filter(|b| !b.stale).map(|b| &b.bar);
                    hold_mark(bar.map(|b| b.presence.as_str()), bar.map(|b| b.afk.mode.as_str()))
                }
                Ask::Restart(_, holds) => {
                    let mode = holds.get(name).map(String::as_str);
                    hold_mark(mode.map(|m| if m == "off" { "loop" } else { "wait" }), mode)
                }
            };
            let place = if known.is_some_and(|l| l.on_host()) { "host" } else { "tmux" };
            let row = div()
                .flex()
                .items_center()
                .gap_2()
                .pl_2()
                .child(div().w(px(14.)).text_color(colour).child(glyph))
                .child(div().text_color(p().text).child(agent.unwrap_or_else(|| name.clone())))
                .child(div().text_xs().text_color(p().muted).child(place))
                .into_any_element();
            let project = project_of(name);
            match projects.last_mut() {
                Some((last, rows)) if *last == project => rows.push(row),
                _ => projects.push((project, vec![row])),
            }
        }
        let summary = match ask {
            Ask::Quit(_) => format!(
                "{} of {} still {} on this machine.",
                count_of(loops.len()),
                count(projects.len(), "project"),
                if loops.len() == 1 { "runs" } else { "run" }
            ),
            Ask::Restart(..) => format!(
                "{} of {}; {} its conversation.",
                count_of(loops.len()),
                count(projects.len(), "project"),
                if loops.len() == 1 { "it resumes" } else { "each resumes" }
            ),
        };
        let mut list = div().id("quit-list").flex().flex_col().gap_2().max_h(px(240.)).overflow_y_scroll().p_2().rounded_md().border_1().border_color(p().border).bg(p().bg).text_sm();
        for (project, rows) in projects {
            list = list.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(p().muted).child(if project.is_empty() { "no project".into() } else { project.to_uppercase() }))
                    .children(rows),
            );
        }
        let remember = self.remember;
        let answers = match ask {
            Ask::Quit(_) => div()
                .flex()
                .gap_2()
                .ml_auto()
                .child(buttons::secondary("quit-cancel", "Cancel").on_click(cx.listener(|shell, _, _, cx| shell.cancel_ask(cx))))
                .child(buttons::secondary("quit-no", "Quit, keep them running").on_click(cx.listener(|shell, _, _, cx| shell.answer(Answer::No, cx))))
                .child(buttons::primary("quit-yes", "Quit and stop them").on_click(cx.listener(|shell, _, _, cx| shell.answer(Answer::Yes, cx)))),
            Ask::Restart(..) => div()
                .flex()
                .gap_2()
                .ml_auto()
                .child(buttons::secondary("quit-no", "Not now").on_click(cx.listener(|shell, _, _, cx| shell.answer(Answer::No, cx))))
                .child(buttons::secondary("quit-fresh", "Restart fresh").on_click(cx.listener(|shell, _, _, cx| shell.answer(Answer::Fresh, cx))))
                .child(buttons::primary("quit-yes", "Restart as they were").on_click(cx.listener(|shell, _, _, cx| shell.answer(Answer::Yes, cx)))),
        };
        let body = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_lg().font_weight(FontWeight::BOLD).child(title))
                    .child(div().text_sm().text_color(p().muted).child(summary)),
            )
            .child(list)
            .when(matches!(ask, Ask::Restart(..)), |d| {
                d.child(div().text_xs().text_color(p().muted).child("As they were: a held session is held again. Fresh: each boots, then runs on its own."))
            })
            .child(div().h(px(1.)).bg(p().border))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        buttons::link("quit-remember", if remember { "☑" } else { "☐" })
                            .flex_1()
                            .gap_2()
                            .text_sm()
                            .text_color(p().text)
                            .child("Remember this choice")
                            .tip("Options > Layout > Sessions changes it")
                            .on_click(cx.listener(|shell, _, _, cx| {
                                shell.remember = !shell.remember;
                                cx.notify();
                            })),
                    )
                    .child(answers),
            );
        Some(dialog(body).into_any_element())
    }

    /// Esc: the quit dialog is cancelled; the restart one, "not now".
    pub(super) fn escape_ask(&mut self, cx: &mut Context<Self>) -> bool {
        match self.ask {
            Some(Ask::Quit(_)) => self.cancel_ask(cx),
            Some(Ask::Restart(..)) => self.answer(Answer::No, cx),
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

/// "1 session", "3 sessions".
fn count_of(n: usize) -> String {
    count(n, "session")
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
