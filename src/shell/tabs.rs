//! The tabs over the terminal: the terminals of the shown one's group (a
//! project, "terminals", "tmux") open in tvty, its agents first, then its
//! shells. × closes a tab: tvty's view only, but a shell of aiball's host
//! stops. Ctrl+PgUp / Ctrl+PgDn move along them; "+" opens a shell in the
//! group's folder. The slider moves between groups, the tabs within one.

use crate::ui::Named as _;
use std::collections::HashSet;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::Sizable as _;

use super::Shell;
use crate::panel::dot;
use crate::sessions::{self, Board, Project, Terminal};
use crate::theme::p;
use crate::ui::buttons;
use crate::tip::Tip as _;

/// How long a start waits for the terminals it opens again to be listed.
const RESTORE_FOR: std::time::Duration = std::time::Duration::from_secs(15);

/// The workspace being opened again at start: what is still to open.
pub(super) struct Restoring {
    open: Vec<String>,
    shown: Option<String>,
    until: std::time::Instant,
}

impl Restoring {
    pub(super) fn new(open: Vec<String>, shown: Option<String>) -> Self {
        Self { open, shown, until: std::time::Instant::now() + RESTORE_FOR }
    }

    /// Something to open is listed now (`listed`), or it is time to give up.
    pub(super) fn due(&self, listed: impl Fn(&str) -> bool) -> bool {
        std::time::Instant::now() > self.until || self.open.iter().chain(self.shown.iter()).any(|s| listed(s))
    }

    fn done(&self) -> bool {
        (self.open.is_empty() && self.shown.is_none()) || std::time::Instant::now() > self.until
    }
}

/// The tab bar's height.
pub(super) const TAB_BAR: f32 = 30.;

impl Shell {
    /// The group of the terminal shown.
    fn selected_group(&self) -> Option<&Project> {
        let (project, _) = self.selected.as_deref().and_then(|s| self.terminal_of(s))?;
        self.board.projects.iter().find(|p| p.name == project)
    }

    /// The tabs of the group shown: its terminals open in tvty, and the one
    /// shown. The list keeps them all.
    fn tabs(&self) -> Vec<&Terminal> {
        let Some(group) = self.selected_group() else { return Vec::new() };
        group
            .terminals
            .iter()
            .filter(|t| self.terminals.contains_key(&t.session) || self.selected.as_deref() == Some(t.session.as_str()))
            .collect()
    }

    /// × on a tab: tvty's view goes; the terminal shown goes back to the one
    /// used before it. An agent's Claude, a tmux session, go on without it; a
    /// shell on aiball's host has no other life, and stops.
    /// The field a tab is renamed in: Enter names it, leaving it (or Esc)
    /// keeps the name it had.
    pub(super) fn tab_name_field(window: &mut Window, cx: &mut Context<Self>) -> Entity<InputState> {
        let field = cx.new(|cx| InputState::new(window, cx).placeholder("its name"));
        cx.subscribe_in(&field, window, |shell: &mut Self, _, event: &InputEvent, window, cx| match event {
            InputEvent::PressEnter { .. } => shell.commit_tab_rename(window, cx),
            InputEvent::Blur => {
                if shell.tab_renaming.take().is_some() {
                    cx.notify();
                }
            }
            _ => {}
        })
        .detach();
        field
    }

    /// A terminal of aiball's host without Claude: its name is the user's.
    pub(super) fn renamable(&self, session: &str) -> bool {
        session.starts_with(sessions::HOSTED_PREFIX) && self.terminal_of(session).is_some_and(|(_, t)| t.agent.is_none())
    }

    /// The tab of `session` becomes a field, its name in it, selected.
    pub(super) fn start_tab_rename(&mut self, session: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(label) = self.renamable(&session).then(|| self.terminal_of(&session).map(|(_, t)| t.label.clone())).flatten() else { return };
        self.tab_renaming = Some(session);
        self.tab_name.update(cx, |field, cx| {
            // The cursor at the end, the name kept: typing adds to it.
            field.set_value(label, window, cx);
            field.focus(window, cx);
        });
        cx.notify();
    }

    pub(super) fn cancel_tab_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.tab_renaming = None;
        self.focus_terminal(window, cx);
        cx.notify();
    }

    /// Enter: aiball names the terminal (empty: its own name back); every
    /// tvty shows it once aiball says so.
    fn commit_tab_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(session) = self.tab_renaming.take() else { return };
        let Some(name) = session.strip_prefix(sessions::HOSTED_PREFIX).map(str::to_string) else { return };
        let typed = self.tab_name.read(cx).value().trim().to_string();
        let label = (!typed.is_empty() && typed != name).then_some(typed);
        let aiball = self.aiball.clone();
        cx.spawn(async move |_, cx| {
            let done = cx
                .background_executor()
                .spawn({
                    let name = name.clone();
                    async move { aiball.label_terminal(&name, label.as_deref()) }
                })
                .await;
            if let Err(error) = done {
                let _ = cx.update(|cx| crate::activity::publish(cx, crate::activity::Activity::failed(None, "rename the terminal", format!("{name}: {error:#}"))));
            }
        })
        .detach();
        self.focus_terminal(window, cx);
        cx.notify();
    }

    pub(super) fn close_tab(&mut self, session: String, window: &mut Window, cx: &mut Context<Self>) {
        let shell = self
            .terminal_of(&session)
            .is_some_and(|(_, t)| t.agent.is_none() && t.attach.is_some());
        self.terminals.remove(&session);
        // Back to the terminal used before it, still open in tvty — of this
        // group or another, as the end screen's Close does.
        self.recent.retain(|s| *s != session);
        let next = self.recent.iter().find(|s| self.terminals.contains_key(*s)).cloned();
        if shell {
            // Gone from the list now, not once the host has stopped it.
            self.stopping.insert(session.clone());
            let mut board = std::mem::take(&mut self.board);
            self.forget_stopping(&mut board);
            self.board = board;
            if let Some(name) = session.strip_prefix(sessions::HOSTED_PREFIX).map(str::to_string) {
                let aiball = self.aiball.clone();
                let hidden = session.clone();
                cx.spawn(async move |this, cx| {
                    let stopped = cx.background_executor().spawn({
                        let name = name.clone();
                        async move { aiball.stop_terminal(&name) }
                    });
                    let Err(error) = stopped.await else { return };
                    log::warn!("stopping terminal {name}: {error:#}");
                    // Not stopped: listed again, to be closed again or used,
                    // rather than left running out of sight.
                    let _ = this.update(cx, |shell, cx| {
                        shell.stopping.remove(&hidden);
                        crate::activity::publish(cx, crate::activity::Activity::failed(None, "stop the terminal", format!("{name}: {error:#}")));
                        shell.rebuild(cx);
                    });
                })
                .detach();
            }
        }
        if self.selected.as_deref() == Some(session.as_str()) {
            match next {
                Some(next) => self.select(next, window, cx),
                None => {
                    self.selected = None;
                    self.sync_panel(cx);
                }
            }
        }
        self.save_workspace(cx);
        cx.notify();
    }

    /// The workspace, kept for the next start: the terminals open, the one
    /// used last first, and the one shown.
    pub(super) fn save_workspace(&mut self, cx: &mut App) {
        if self.restoring.is_some() {
            return;
        }
        let mut open: Vec<String> = self.recent.iter().filter(|s| self.terminals.contains_key(*s)).cloned().collect();
        for session in self.terminals.keys() {
            if !open.contains(session) {
                open.push(session.clone());
            }
        }
        if open != self.settings.workspace.open_terminals || self.selected != self.settings.workspace.shown_terminal {
            self.settings.workspace.open_terminals = open;
            self.settings.workspace.shown_terminal = self.selected.clone();
            self.settings.save(cx);
        }
    }

    /// At start: opens again the terminals open when tvty was left, each
    /// once it is listed (aiball's sessions may come after tmux's), the one
    /// used last opened last; the one shown is selected once it is there.
    pub(super) fn restore(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(mut restoring) = self.restoring.take() else { return };
        let there: Vec<String> = restoring.open.iter().rev().filter(|s| self.terminal_of(s).is_some()).cloned().collect();
        log::debug!("restore: listed now {there:?}, still to come {:?}, shown {:?}", restoring.open, restoring.shown);
        restoring.open.retain(|s| !there.contains(s));
        for session in there {
            self.open_terminal(&session, window, cx);
            // In the memory of the tabs, the one used last first.
            self.recent.retain(|s| *s != session);
            self.recent.insert(0, session);
        }
        let shown = restoring.shown.clone().filter(|s| self.terminal_of(s).is_some());
        if shown.is_some() {
            restoring.shown = None;
        }
        // Still restoring: the workspace kept is not overwritten meanwhile.
        if !restoring.done() {
            self.restoring = Some(restoring);
        }
        if let Some(shown) = shown {
            self.select(shown, window, cx);
        }
        self.save_workspace(cx);
        cx.notify();
    }

    /// Leaves out of `board` the shells being stopped; the host gone with
    /// them, they are forgotten. A group left empty goes too.
    pub(super) fn forget_stopping(&mut self, board: &mut Board) {
        if self.stopping.is_empty() {
            return;
        }
        let listed: HashSet<String> = board.projects.iter().flat_map(|p| p.terminals.iter().map(|t| t.session.clone())).collect();
        self.stopping.retain(|s| listed.contains(s));
        for project in &mut board.projects {
            project.terminals.retain(|t| !self.stopping.contains(&t.session));
        }
        board.projects.retain(|p| !p.terminals.is_empty());
    }

    /// Ctrl+PgDn (1) / Ctrl+PgUp (-1): the next tab of the group, round.
    pub(super) fn step_tab(&mut self, step: isize, window: &mut Window, cx: &mut Context<Self>) {
        let sessions: Vec<String> = self.tabs().iter().map(|t| t.session.clone()).collect();
        if sessions.is_empty() {
            return;
        }
        let Some(at) = self.selected.as_ref().and_then(|s| sessions.iter().position(|o| o == s)) else { return };
        let next = sessions[(at as isize + step).rem_euclid(sessions.len() as isize) as usize].clone();
        self.select(next, window, cx);
    }

    /// The bar, when a terminal of a group is shown.
    pub(super) fn tab_bar(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        let group = self.selected_group()?;
        let mut bar = div()
            .named("tab-bar")
            .flex()
            .flex_none()
            .items_end()
            .h(px(TAB_BAR))
            .pl_2()
            .gap_0p5()
            .overflow_x_scroll()
            .bg(p().surface)
            .border_b_1()
            .border_color(p().border);
        for terminal in self.tabs() {
            let session = terminal.session.clone();
            let closing = session.clone();
            let middle = session.clone();
            let selected = self.selected.as_deref() == Some(session.as_str());
            let state = terminal.status.as_ref().and_then(|s| s.colour());
            let shell = terminal.agent.is_none() && terminal.attach.is_some();
            let counts = self.counts_of(&group.name, &terminal);
            let renaming = self.tab_renaming.as_deref() == Some(session.as_str());
            let renamed = session.clone();
            bar = bar.child(
                div()
                    .named(SharedString::from(format!("tab-{session}")))
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap_1p5()
                    .h(px(TAB_BAR - 4.))
                    .px_2p5()
                    .rounded_t_md()
                    .text_sm()
                    .cursor_pointer()
                    .border_b_2()
                    .when(selected, |d| d.bg(p().bg).text_color(p().text).border_color(p().accent))
                    .when(!selected, |d| d.text_color(p().muted).border_color(transparent_black()).hover(|d| d.bg(p().hover)))
                    // Its Claude's state, as in the list.
                    .when_some(state, |d, colour| d.child(dot(colour)))
                    .when(shell, |d| d.child(div().text_xs().text_color(p().muted).child(">_")))
                    .map(|d| {
                        if renaming {
                            d.child(div().w(px(160.)).child(Input::new(&self.tab_name).small()))
                        } else {
                            d.child(terminal.label.clone())
                        }
                    })
                    // ✎: renames a terminal without Claude, shown as × is.
                    .when(shell && !renaming, |d| {
                        let renamed = session.clone();
                        d.child(
                            div()
                                .named(SharedString::from(format!("tab-rename-{session}")))
                                .flex_none()
                                .p_0p5()
                                .rounded_sm()
                                .when(!selected, |d| d.opacity(0.).group_hover(SharedString::from(format!("tab-group-{session}")), |s| s.opacity(1.)))
                                .hover(|d| d.bg(p().hover))
                                .child(crate::icons::icon(crate::icons::Icon::Rename, p().muted, 12.))
                                .tip("rename it (or a double click, F2)")
                                .on_click(cx.listener(move |shell, _, window, cx| {
                                    cx.stop_propagation();
                                    shell.start_tab_rename(renamed.clone(), window, cx);
                                })),
                        )
                    })
                    .children(counts.map(|c| c.badges(format!("tab-{session}"))))
                    // ×: shown on the tab shown, and on the one under the pointer.
                    .child(
                        div()
                            .named(SharedString::from(format!("tab-close-{session}")))
                            .flex_none()
                            .px_1()
                            .rounded_sm()
                            .text_xs()
                            .text_color(p().muted)
                            .when(!selected, |d| d.opacity(0.).group_hover(SharedString::from(format!("tab-group-{session}")), |s| s.opacity(1.)))
                            .hover(|d| d.bg(p().hover).text_color(p().text))
                            .child("×")
                            .tip(if shell { "close: stops this terminal" } else { "close the tab: its Claude goes on" })
                            .on_click(cx.listener(move |shell, _, window, cx| {
                                cx.stop_propagation();
                                shell.close_tab(closing.clone(), window, cx);
                            })),
                    )
                    .group(SharedString::from(format!("tab-group-{session}")))
                    .when(!renaming, |d| d.tip(if shell {
                        "a terminal on aiball's host, without Claude; a double click (or F2) renames it"
                    } else {
                        "ctrl+pgup / ctrl+pgdn: the tab before, after"
                    }))
                    // A double click renames a terminal without Claude.
                    .when(shell, |d| {
                        d.on_mouse_down(MouseButton::Left, cx.listener(move |shell, event: &MouseDownEvent, window, cx| {
                            if event.click_count >= 2 {
                                shell.start_tab_rename(renamed.clone(), window, cx);
                                cx.stop_propagation();
                            }
                        }))
                    })
                    .on_mouse_down(MouseButton::Middle, cx.listener(move |shell, _, window, cx| {
                        shell.close_tab(middle.clone(), window, cx);
                    }))
                    // The second click of a double click renames: it does not
                    // select again (that would take the keys from the field).
                    .on_click(cx.listener(move |shell, event: &ClickEvent, window, cx| {
                        if event.click_count() < 2 && shell.tab_renaming.as_deref() != Some(session.as_str()) {
                            shell.select(session.clone(), window, cx)
                        }
                    })),
            );
        }
        // A shell where the group works: the project's folder, or home for
        // the host's own terminals (no project); nothing for tmux's.
        let new = if group.on_board {
            Some(("a terminal in the project's folder", Some(group.name.clone())))
        } else if group.name == sessions::HOSTED_GROUP {
            Some(("a terminal in the home directory", None))
        } else {
            None
        };
        if let Some((tip, project)) = new {
            bar = bar.child(
                buttons::link("tab-new", "+")
                    .self_center()
                    .px_2()
                    .tip(tip)
                    .on_click(cx.listener(move |shell, _, _, cx| match &project {
                        Some(project) => shell.new_project_terminal(project, cx),
                        None => shell.new_terminal(cx),
                    })),
            );
        }
        Some(bar)
    }
}
