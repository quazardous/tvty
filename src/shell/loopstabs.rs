//! The projects' list, as three vertical tabs: the sessions that run
//! (Active), the loops this machine knows that are stopped (Inactive, one
//! click starts one again, where it worked, for its agent), and the agents
//! aiball knows with no loop here (Closed, one click opens one where the
//! agent works). "+ session" on a project opens one: a working directory
//! (proposed, checked) and an agent. claude-loop starts it, detached, in
//! that directory — never tvty's —, and tvty opens it once it runs.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Shell;
use crate::loops::{KnownLoop, Start};
use crate::notify::{self, Kind, Notice};
use crate::theme::p;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum Tab {
    #[default]
    Active,
    Inactive,
    Closed,
}

/// "+ session" on a project: where, and for whom.
pub(super) struct NewSession {
    project: String,
    cwd: Entity<InputState>,
    agent: Entity<InputState>,
    crew: bool,
    busy: bool,
}

impl Shell {
    /// The loops known here that do not run.
    fn inactive(&self) -> Vec<&KnownLoop> {
        self.board
            .known
            .iter()
            .filter(|l| self.terminal_of(&l.name).is_none())
            .collect()
    }

    /// The agents aiball knows with no loop on this machine.
    fn closed(&self) -> Vec<&(String, Option<String>, String)> {
        self.board
            .homes
            .iter()
            .filter(|(agent, _, cwd)| {
                !self.board.known.iter().any(|l| l.consumer.as_deref() == Some(agent.as_str()) || l.cwd == *cwd)
                    && !self.board.projects.iter().any(|p| p.terminals.iter().any(|t| t.agent.as_deref() == Some(agent.as_str())))
            })
            .collect()
    }

    /// The rail of tabs, on the list's left.
    pub(super) fn tab_rail(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let running: usize = self.board.projects.iter().map(|p| p.terminals.len()).sum();
        let tabs = [
            (Tab::Active, "live", running),
            (Tab::Inactive, "idle", self.inactive().len()),
            (Tab::Closed, "shut", self.closed().len()),
        ];
        let mut rail = div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(44.))
            .h_full()
            .py_2()
            .gap_1()
            .border_r_1()
            .border_color(p().border);
        for (tab, word, count) in tabs {
            let on = self.sidebar_tab == tab;
            rail = rail.child(
                div()
                    .id(SharedString::from(format!("tab-{word}")))
                    .flex()
                    .flex_col()
                    .items_center()
                    .py_1()
                    .mx_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .when(on, |d| d.bg(p().active).text_color(p().text))
                    .when(!on, |d| d.text_color(p().muted).hover(|d| d.bg(p().hover)))
                    .child(div().text_sm().font_weight(FontWeight::BOLD).child(count.to_string()))
                    .child(div().text_xs().child(word))
                    .on_click(cx.listener(move |shell, _, _, cx| {
                        shell.sidebar_tab = tab;
                        cx.notify();
                    })),
            );
        }
        rail
    }

    /// The Inactive and Closed tabs' lists; the Active one is the projects'
    /// list itself.
    pub(super) fn other_tab(&self, width: f32, cx: &mut Context<Self>) -> AnyElement {
        let mut list = div()
            .id("sessions-other")
            .flex()
            .flex_col()
            .w(px(width))
            .flex_none()
            .h_full()
            .py_2()
            .text_sm();
        let heading = |text: String| {
            div()
                .px_3()
                .pt_2()
                .pb_1()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(p().muted)
                .child(text.to_uppercase())
        };
        let row = |id: String, name: String, cwd: &str, cx: &mut Context<Self>, start: Start| {
            let busy = self.starting.as_deref() == Some(start.cwd.as_str());
            div()
                .id(SharedString::from(id))
                .flex()
                .flex_col()
                .px_3()
                .py_1()
                .cursor_pointer()
                .hover(|d| d.bg(p().hover))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(div().flex_1().min_w_0().truncate().child(name))
                        .child(div().text_xs().text_color(p().accent).child(if busy { "starting…" } else { "▶ start" })),
                )
                .child(div().text_xs().text_color(p().muted).truncate().child(home_short(cwd)))
                .on_click(cx.listener(move |shell, _, _, cx| shell.start_loop(start.clone(), cx)))
        };
        match self.sidebar_tab {
            Tab::Inactive => {
                let loops = self.inactive();
                if loops.is_empty() {
                    list = list.child(div().px_3().text_color(p().muted).child("No stopped loop"));
                }
                let mut last: Option<String> = None;
                for l in loops {
                    let project = l.project.clone().unwrap_or_else(|| "—".into());
                    if last.as_deref() != Some(project.as_str()) {
                        list = list.child(heading(project.clone()));
                        last = Some(project);
                    }
                    let name = l.consumer.clone().unwrap_or_else(|| l.name.clone());
                    let start = Start {
                        cwd: l.cwd.clone(),
                        project: l.project.clone(),
                        agent: l.consumer.clone(),
                        crew: l.role.as_deref() == Some("crew"),
                    };
                    list = list.child(row(format!("idle-{}", l.name), name, &l.cwd, cx, start));
                }
            }
            _ => {
                let homes = self.closed();
                if homes.is_empty() {
                    list = list.child(div().px_3().text_color(p().muted).child("No agent without a loop"));
                }
                for (agent, project, cwd) in homes {
                    let start = Start { cwd: cwd.clone(), project: project.clone(), agent: Some(agent.clone()), crew: false };
                    list = list.child(row(format!("shut-{agent}"), agent.clone(), cwd, cx, start));
                }
            }
        }
        list.overflow_y_scroll().into_any_element()
    }

    /// "+ session" on a project: opens its form, prefilled — the directory
    /// of a loop or an agent of the project, and its agent.
    pub(super) fn ask_new_session(&mut self, project: String, window: &mut Window, cx: &mut Context<Self>) {
        let known = self.board.known.iter().find(|l| l.project.as_deref() == Some(project.as_str()) && l.role.is_none());
        let home = self.board.homes.iter().find(|h| h.1.as_deref() == Some(project.as_str()));
        let cwd_value = known.map(|l| l.cwd.clone()).or_else(|| home.map(|h| h.2.clone())).unwrap_or_default();
        let agent_value = known.and_then(|l| l.consumer.clone()).or_else(|| home.map(|h| h.0.clone())).unwrap_or_default();
        let cwd = cx.new(|cx| InputState::new(window, cx).placeholder("working directory").default_value(cwd_value));
        let agent = cx.new(|cx| InputState::new(window, cx).placeholder("agent").default_value(agent_value));
        // The directory is checked as it is typed.
        cx.subscribe(&cwd, |_, _, event: &gpui_kit::component::input::InputEvent, cx| {
            if matches!(event, gpui_kit::component::input::InputEvent::Change) {
                cx.notify();
            }
        })
        .detach();
        self.new_session = Some(NewSession { project, cwd, agent, crew: false, busy: false });
        cx.notify();
    }

    /// The form under a project's heading, when "+ session" was clicked.
    pub(super) fn new_session_form(&self, project: &str, cx: &mut Context<Self>) -> Option<AnyElement> {
        let form = self.new_session.as_ref().filter(|f| f.project == project)?;
        let crew = form.crew;
        let cwd = form.cwd.read(cx).value().to_string();
        let exists = std::path::Path::new(cwd.trim()).is_dir();
        Some(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .mx_2()
                .mb_2()
                .p_2()
                .rounded_md()
                .bg(p().bg)
                .border_1()
                .border_color(p().border)
                .text_xs()
                .child(Input::new(&form.cwd).small())
                .child(
                    div().text_color(if exists || cwd.is_empty() { p().muted } else { p().danger }).child(
                        if cwd.is_empty() {
                            "where its Claude works"
                        } else if exists {
                            "claude-loop starts here"
                        } else {
                            "no such directory"
                        },
                    ),
                )
                .child(Input::new(&form.agent).small())
                .child(
                    div()
                        .id("new-session-crew")
                        .flex()
                        .items_center()
                        .gap_1()
                        .cursor_pointer()
                        .text_color(if crew { p().text } else { p().muted })
                        .child(if crew { "☑" } else { "☐" })
                        .child("a crew agent, next to the main loop")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            if let Some(form) = shell.new_session.as_mut() {
                                form.crew = !form.crew;
                            }
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .justify_end()
                        .child(
                            Button::new("new-session-cancel")
                                .ghost()
                                .xsmall()
                                .label("Cancel")
                                .on_click(cx.listener(|shell, _, _, cx| {
                                    shell.new_session = None;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("new-session-start")
                                .primary()
                                .xsmall()
                                .label("Start")
                                .loading(form.busy)
                                .disabled(form.busy || !exists)
                                .on_click(cx.listener(|shell, _, _, cx| {
                                    let Some(form) = shell.new_session.as_mut() else { return };
                                    form.busy = true;
                                    let agent = form.agent.read(cx).value().trim().to_string();
                                    let start = Start {
                                        cwd: form.cwd.read(cx).value().trim().to_string(),
                                        project: Some(form.project.clone()),
                                        agent: (!agent.is_empty()).then_some(agent),
                                        crew: form.crew,
                                    };
                                    shell.start_loop(start, cx);
                                })),
                        ),
                )
                .into_any_element(),
        )
    }

    /// Starts a loop through claude-loop, off the UI thread; once its
    /// session runs, tvty opens it.
    pub(super) fn start_loop(&mut self, start: Start, cx: &mut Context<Self>) {
        if self.starting.is_some() {
            return;
        }
        self.starting = Some(start.cwd.clone());
        cx.notify();
        cx.spawn(async move |this, cx| {
            let done = cx.background_executor().spawn({
                let start = start.clone();
                async move { crate::loops::start(&start) }
            });
            let done = done.await;
            let _ = this.update(cx, |shell, cx| {
                shell.starting = None;
                match done {
                    Ok(name) => {
                        shell.new_session = None;
                        shell.sidebar_tab = Tab::Active;
                        shell.open_when_running = Some(name.clone());
                        notify::push(cx, Notice::new(Kind::Info, "tvty", format!("started {name} in {}", home_short(&start.cwd))));
                        let _ = shell.refresh_now.unbounded_send(crate::events::Change::All);
                    }
                    Err(error) => {
                        if let Some(form) = shell.new_session.as_mut() {
                            form.busy = false;
                        }
                        notify::push(cx, Notice::new(Kind::Error, "claude-loop", format!("{error:#}")));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

/// A path with the home directory as `~`.
fn home_short(path: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && path.starts_with(&home) => format!("~{}", &path[home.len()..]),
        _ => path.to_string(),
    }
}
