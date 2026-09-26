//! The tabs over the terminal: one per terminal of the selected terminal's
//! group (a project, "terminals", "tmux"), its agents first, then its
//! shells. Ctrl+PgUp / Ctrl+PgDn move along them; "+" opens a shell in the
//! group's folder. The slider moves between groups, the tabs within one.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Shell;
use crate::panel::dot;
use crate::sessions::{self, Project};
use crate::theme::p;
use crate::tip::Tip as _;

/// The tab bar's height.
pub(super) const TAB_BAR: f32 = 30.;

impl Shell {
    /// The group of the terminal shown.
    fn selected_group(&self) -> Option<&Project> {
        let (project, _) = self.selected.as_deref().and_then(|s| self.terminal_of(s))?;
        self.board.projects.iter().find(|p| p.name == project)
    }

    /// Ctrl+PgDn (1) / Ctrl+PgUp (-1): the next tab of the group, round.
    pub(super) fn step_tab(&mut self, step: isize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(group) = self.selected_group() else { return };
        let sessions: Vec<String> = group.terminals.iter().map(|t| t.session.clone()).collect();
        let Some(at) = self.selected.as_ref().and_then(|s| sessions.iter().position(|o| o == s)) else { return };
        let next = sessions[(at as isize + step).rem_euclid(sessions.len() as isize) as usize].clone();
        self.select(next, window, cx);
    }

    /// The bar, when a terminal of a group is shown.
    pub(super) fn tab_bar(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        let group = self.selected_group()?;
        let mut bar = div()
            .id("tab-bar")
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
        for terminal in &group.terminals {
            let session = terminal.session.clone();
            let selected = self.selected.as_deref() == Some(session.as_str());
            let state = terminal.status.as_ref().and_then(|s| s.colour());
            let shell = terminal.agent.is_none() && terminal.attach.is_some();
            let alerts = self.alerts_of(&group.name, terminal.agent.as_deref());
            bar = bar.child(
                div()
                    .id(SharedString::from(format!("tab-{session}")))
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
                    .child(terminal.label.clone())
                    .child(alerts.badges(format!("tab-{session}")))
                    .tip(if shell { "a terminal on aiball's host, without Claude" } else { "ctrl+pgup / ctrl+pgdn: the tab before, after" })
                    .on_click(cx.listener(move |shell, _, window, cx| shell.select(session.clone(), window, cx))),
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
                div()
                    .id("tab-new")
                    .flex_none()
                    .self_center()
                    .px_2()
                    .rounded_sm()
                    .text_color(p().accent)
                    .cursor_pointer()
                    .hover(|d| d.bg(p().hover))
                    .child("+")
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
