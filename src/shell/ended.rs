//! A session that ended while shown (Claude quit, the loop stopped): its
//! terminal is let go, and the centre says so and offers what comes next —
//! start its loop again where it worked (Enter), or go back to the terminal
//! used before (Esc). The panel keeps the ended agent's project meanwhile.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Shell;
use super::loopstabs::home_short;
use crate::loops::Start;
use crate::terminal::{Ended, TerminalView};
use crate::theme::p;
use crate::ui::buttons;

/// What is known of a session that ended while selected.
pub(super) struct EndedSession {
    pub session: String,
    pub agent: Option<String>,
    pub project: Option<String>,
    /// How to start its loop again; none for a bare tmux session.
    pub start: Option<Start>,
    /// Not an end: another client closed this one (its "Close the
    /// others"). The session goes on; Enter attaches again — never on its
    /// own, or two windows would close each other in turn.
    pub detached: bool,
}

impl Shell {
    /// Hears when `terminal`'s session ends.
    pub(super) fn watch_end(&mut self, session: String, terminal: &Entity<TerminalView>, window: &mut Window, cx: &mut Context<Self>) {
        cx.subscribe_in(terminal, window, move |shell, _, _: &Ended, window, cx| {
            shell.session_ended(&session, window, cx);
        })
        .detach();
    }

    /// Opens `session` again as a copy, or with the controls: its client
    /// leaves and another comes, in the other mode; its Claude goes on.
    pub(super) fn set_copy(&mut self, session: String, copy: bool, window: &mut Window, cx: &mut Context<Self>) {
        if copy {
            self.copies.insert(session.clone());
            self.controls_taken.remove(&session);
        } else {
            self.copies.remove(&session);
            self.controls_taken.insert(session.clone());
        }
        self.terminals.remove(&session);
        self.select(session.clone(), window, cx);
        // The controls taken on a tmux loop: its other clients (claude-loop's
        // terminal) become copies, as aiball does it; this one kept.
        if !copy {
            self.others_to_copies(&session, cx);
        }
        cx.notify();
    }

    /// Makes the other tmux clients of `session`'s loop copies, off the UI
    /// thread: its terminal here, just opened, is the one kept.
    fn others_to_copies(&mut self, session: &str, cx: &mut Context<Self>) {
        let Some(name) = self.board.known.iter().find(|l| l.session() == session && !l.on_host()).map(|l| l.name.clone()) else { return };
        let Some(pid) = self.terminals.get(session).and_then(|t| t.read(cx).child_pid) else { return };
        let aiball = self.aiball.clone();
        cx.spawn(async move |_, cx| {
            let done = cx.background_executor().spawn(async move { crate::loops::others_to_copies(&aiball, &name, pid) }).await;
            if let Err(error) = done {
                let _ = cx.update(|cx| crate::activity::publish(cx, crate::activity::Activity::failed(None, "make the other terminals copies", format!("{error:#}"))));
            }
        })
        .detach();
    }

    /// Lets the ended terminal go: a later selection of the same name
    /// attaches afresh. Shown, it leaves the end screen in its place.
    fn session_ended(&mut self, session: &str, window: &mut Window, cx: &mut Context<Self>) {
        let detached = self.terminals.get(session).and_then(|t| t.read(cx).attachment().map(|a| a.detached_by_other())).unwrap_or(false);
        self.terminals.remove(session);
        self.save_workspace(cx);
        if self.selected.as_deref() != Some(session) {
            return;
        }
        // A restart tvty asked for: the loop starts again under the same
        // name (its host stopped, then a new one), and the terminal opens
        // again once it is back — as many times as the restart takes.
        let agent = self.terminal_of(session).and_then(|(_, t)| t.agent.clone());
        // A loop being moved (host ↔ tmux): it comes back as another session.
        self.moves.retain(|_, (_, at)| at.elapsed() < std::time::Duration::from_secs(90));
        if let Some((next, _)) = agent.as_ref().and_then(|a| self.moves.get(a)) {
            self.open_when_running = Some(next.clone());
            cx.notify();
            return;
        }
        self.restarts_asked.retain(|_, at| at.elapsed() < std::time::Duration::from_secs(60));
        if agent.is_some_and(|a| self.restarts_asked.contains_key(&a)) {
            self.open_when_running = Some(session.to_string());
            cx.notify();
            return;
        }
        let (project, agent) = match self.terminal_of(session) {
            Some((project, terminal)) => (Some(project.to_string()), terminal.agent.clone()),
            None => (None, None),
        };
        // Its loop: in tmux by its name, on aiball's host by its agent.
        let known = self.board.known.iter().find(|l| l.session() == session);
        let start = known.map(|l| Start {
            cwd: l.cwd.clone(),
            project: project.clone().or_else(|| l.project.clone()),
            agent: l.agent().map(str::to_string),
            crew: l.role.as_deref() == Some("crew"),
            // On the host, its host outlives its program: started again
            // there, through claude-loop's restart.
            again: l.on_host().then(|| l.name.clone()),
            mode: None,
        });
        self.ended = Some(EndedSession {
            session: session.to_string(),
            agent: agent.or_else(|| known.and_then(|l| l.agent().map(str::to_string))),
            project: project.or_else(|| known.and_then(|l| l.project.clone())),
            start,
            detached,
        });
        self.focus_home(window, cx);
        self.sync_panel(cx);
        cx.notify();
    }

    /// The end screen is up: the selected session is the one that ended.
    pub(super) fn ended_shown(&self) -> Option<&EndedSession> {
        self.ended.as_ref().filter(|e| self.selected.as_deref() == Some(e.session.as_str()))
    }

    /// Enter on the end screen: its loop starts again; the new session opens
    /// as soon as it runs.
    pub(super) fn restart_ended(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Closed by another client: the session runs, it is attached again.
        if let Some(session) = self.ended_shown().filter(|e| e.detached).map(|e| e.session.clone()) {
            self.ended = None;
            self.select(session, window, cx);
            return;
        }
        if let Some(start) = self.ended_shown().and_then(|e| e.start.clone()) {
            self.start_loop(start, cx);
        }
    }

    /// Esc on the end screen: back to the terminal used before, if any.
    pub(super) fn close_ended(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ended) = self.ended.take() else { return };
        // A terminal of the daemon's whose program ended: closing it removes
        // it from the host, so that it does not linger there.
        if let Some(name) = ended.session.strip_prefix(crate::sessions::HOSTED_PREFIX).map(str::to_string) {
            let hosted_terminal = !self.live.terminals().iter().any(|t| t.name == name)
                && self.live.session_names().contains(&name);
            if hosted_terminal {
                let aiball = self.aiball.clone();
                cx.background_executor()
                    .spawn(async move {
                        if let Err(error) = aiball.stop_terminal(&name) {
                            log::warn!("removing terminal {name}: {error:#}");
                        }
                    })
                    .detach();
            }
        }
        self.recent.retain(|s| *s != ended.session);
        let previous = self.recent.iter().find(|s| self.terminal_of(s).is_some()).cloned();
        match previous {
            Some(session) => self.select(session, window, cx),
            None => {
                self.selected = None;
                self.sync_panel(cx);
                cx.notify();
            }
        }
    }

    pub(super) fn ended_view(&self, ended: &EndedSession, cx: &mut Context<Self>) -> AnyElement {
        // A hosted terminal is named without the key that sets it apart from tmux's.
        let name = ended.session.strip_prefix(crate::sessions::HOSTED_PREFIX).unwrap_or(&ended.session).to_string();
        let who = ended.agent.clone().unwrap_or_else(|| name.clone());
        let starting = self.starting.is_some();
        let button = |id: &'static str, label: &'static str, key: &'static str, primary: bool| {
            buttons::chip(id, label)
                .gap_2()
                .px_3()
                .py_1p5()
                .when(primary, |d| d.border_color(p().accent).text_color(p().accent))
                .child(div().text_xs().text_color(p().muted).child(key))
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            // Where the terminal was: as see-through as one.
            .bg(p().bg.opacity(crate::terminal::opacity()))
            .child(
                div()
                    .flex()
                    .gap_1()
                    .text_lg()
                    .child(div().font_weight(FontWeight::BOLD).child(who))
                    .child(div().text_color(p().muted).child(if ended.detached {
                        "— detached by another client"
                    } else {
                        "— the session ended"
                    })),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(p().muted)
                    .child(match (&ended.project, &ended.start) {
                        (Some(project), Some(start)) => format!("{project} · {}", home_short(&start.cwd)),
                        (Some(project), None) => project.clone(),
                        (None, Some(start)) => home_short(&start.cwd),
                        (None, None) => name.clone(),
                    }),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .when(ended.detached, |d| {
                        d.child(
                            button("ended-reattach", "Attach again", "Enter", true)
                                .on_click(cx.listener(|shell, _, window, cx| shell.restart_ended(window, cx))),
                        )
                    })
                    .when(!ended.detached && ended.start.is_some(), |d| {
                        d.child(
                            button("ended-restart", if starting { "Starting…" } else { "Restart" }, "Enter", true)
                                .on_click(cx.listener(|shell, _, window, cx| shell.restart_ended(window, cx))),
                        )
                    })
                    .child(
                        button("ended-close", "Close", "Esc", false)
                            .on_click(cx.listener(|shell, _, window, cx| shell.close_ended(window, cx))),
                    ),
            )
            .into_any_element()
    }
}
