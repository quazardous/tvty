//! The sessions list, in three foldable sections ([`crate::accordion`]):
//! the sessions that run (live), the loops this machine knows that are
//! stopped (idle, one click starts one again, where it worked, for its
//! agent), and the agents aiball knows with no loop here (shut, one click
//! opens one where the agent works). "+ session" on a project opens one: a working directory
//! (proposed, checked) and an agent. claude-loop starts it, detached, in
//! that directory — never tvty's —, and tvty opens it once it runs.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::*;

use super::Shell;
use crate::loops::{KnownLoop, Start};
use crate::theme::p;

/// The idle and shut sections.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Other {
    Idle,
    Shut,
}

/// "+ session" on a project: where, and for whom.
pub(super) struct NewSession {
    project: String,
    cwd: Entity<InputState>,
    agent: Entity<InputState>,
    crew: bool,
    /// Started on aiball's host (no tmux), the default; unticked, in
    /// claude-loop's tmux.
    on_host: bool,
    busy: bool,
}

impl Shell {
    /// A loop's project as aiball knows its agent; its plate's otherwise (a
    /// plate may name none: a loop moved onto the host keeps only its agent).
    fn loop_project(&self, l: &KnownLoop) -> Option<String> {
        let agent = l.agent();
        self.board
            .homes
            .iter()
            .find(|(a, _, _)| Some(a.as_str()) == agent)
            .and_then(|(_, project, _)| project.clone())
            .or_else(|| l.project.clone())
    }

    /// The loops this machine knows that run no terminal, by project (one
    /// heading each); those of no aiball project (a folder aiball does not
    /// know) last.
    fn inactive(&self, words: &[String]) -> Vec<&KnownLoop> {
        let mut loops: Vec<&KnownLoop> = self
            .board
            .known
            .iter()
            .filter(|l| self.terminal_of(&l.name).is_none())
            // A loop on aiball's host shows as its agent's hosted terminal.
            .filter(|l| {
                l.host_agent.as_ref().is_none_or(|agent| self.terminal_of(&format!("{}{agent}", crate::sessions::HOSTED_PREFIX)).is_none())
            })
            .filter(|l| {
                let project = self.loop_project(l).unwrap_or_default();
                crate::sessions::found(words, &[&project, l.agent().unwrap_or(""), &l.name, &l.cwd])
            })
            .collect();
        loops.sort_by_cached_key(|l| {
            let project = self.loop_project(l);
            (project.is_none(), project)
        });
        loops
    }

    /// The agents aiball knows with no loop on this machine.
    fn closed(&self, words: &[String]) -> Vec<&(String, Option<String>, String)> {
        self.board
            .homes
            .iter()
            .filter(|(agent, _, cwd)| {
                !self.board.known.iter().any(|l| l.consumer.as_deref() == Some(agent.as_str()) || l.cwd == *cwd)
                    && !self.board.projects.iter().any(|p| p.terminals.iter().any(|t| t.agent.as_deref() == Some(agent.as_str())))
            })
            .filter(|(agent, project, cwd)| crate::sessions::found(words, &[project.as_deref().unwrap_or(""), agent, cwd]))
            .collect()
    }

    /// The three sections, each folded or not as the user left it.
    pub(super) fn sessions_list(&self, cx: &mut Context<Self>) -> AnyElement {
        let words = self.filter_words(cx);
        let running: usize = self.live_found(&words).iter().map(|(_, t)| t.len()).sum();
        let groups = [("live", running), ("idle", self.inactive(&words).len()), ("shut", self.closed(&words).len())];
        let mut list = crate::accordion::list("sessions").pb_1();
        for (i, (word, count)) in groups.into_iter().enumerate() {
            // A folded section with something the filter found opens while
            // it is typed.
            let folded = self.settings.layout.sessions_folded.iter().any(|f| f == word) && (words.is_empty() || count == 0);
            let body = match (folded, i) {
                (true, _) => Vec::new(),
                (false, 0) => vec![self.live_list(cx).into_any_element()],
                (false, 1) => vec![self.other_list(Other::Idle, cx)],
                (false, _) => vec![self.other_list(Other::Shut, cx)],
            };
            let section = crate::accordion::Section {
                list: "sessions".into(),
                above: (i > 0).then(|| SharedString::from(format!("sessions-{}", groups[i - 1].0))),
                id: SharedString::from(format!("sessions-{word}")),
                title: word.to_string(),
                count,
                folded,
                // Two sessions, or the line saying there is none.
                keep: (count.min(2) as f32 * 44.).max(24.),
                scroll: self.session_scrolls[i].clone(),
                body,
                before: 0.,
                after: 0.,
                windowed: None,
            };
            list = list.child(section.render(cx.listener(move |shell, _, _, cx| {
                let folded = &mut shell.settings.layout.sessions_folded;
                match folded.iter().position(|f| f == word) {
                    Some(at) => {
                        folded.remove(at);
                    }
                    None => folded.push(word.to_string()),
                }
                shell.settings.save(cx);
                cx.notify();
            })));
        }
        list.into_any_element()
    }

    /// The idle and shut sections' lists; the live one is the projects'
    /// list itself.
    fn other_list(&self, which: Other, cx: &mut Context<Self>) -> AnyElement {
        let mut list = div().flex().flex_col();
        let words = self.filter_words(cx);
        let heading = |text: String, cx: &mut Context<Self>| div().flex().px_3().pt_2().pb_1().child(self.project_heading(&text, text.to_uppercase(), cx));
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
                        .child(div().flex_1().min_w_0().truncate().child(super::marked(&name, &words)))
                        .child(div().text_xs().text_color(p().accent).child(if busy { "starting…" } else { "▶ start" })),
                )
                .child(div().text_xs().text_color(p().muted).truncate().child(home_short(cwd)))
                .on_click(cx.listener(move |shell, _, _, cx| shell.start_loop(start.clone(), cx)))
        };
        match which {
            Other::Idle => {
                let loops = self.inactive(&words);
                if loops.is_empty() {
                    list = list.child(div().px_3().text_color(p().muted).child(if words.is_empty() { "No stopped loop" } else { "No stopped loop found" }));
                }
                let mut last: Option<String> = None;
                for l in loops {
                    let known = self.loop_project(l);
                    let project = known.clone().unwrap_or_else(|| "No project".into());
                    if last.as_deref() != Some(project.as_str()) {
                        list = list.child(heading(project.clone(), cx));
                        last = Some(project);
                    }
                    let agent = l.agent().map(str::to_string);
                    let name = agent.clone().unwrap_or_else(|| l.name.clone());
                    let start = Start {
                        cwd: l.cwd.clone(),
                        project: known,
                        agent,
                        crew: l.role.as_deref() == Some("crew"),
                        // A loop moved onto aiball's host starts again there,
                        // through claude-loop's restart: its start is refused
                        // while its host is up, its program ended.
                        again: l.host_agent.is_some().then(|| l.name.clone()),
                    };
                    list = list.child(row(format!("idle-{}", l.name), name, &l.cwd, cx, start));
                }
            }
            Other::Shut => {
                let homes = self.closed(&words);
                if homes.is_empty() {
                    list = list.child(div().px_3().text_color(p().muted).child(if words.is_empty() { "No agent without a loop" } else { "No agent found" }));
                }
                let mut homes = homes;
                homes.sort_by_key(|(_, project, _)| (project.is_none(), project.clone().unwrap_or_default().to_lowercase()));
                let mut last: Option<String> = None;
                for (agent, project, cwd) in homes {
                    let heading_of = project.clone().unwrap_or_else(|| "No project".into());
                    if last.as_deref() != Some(heading_of.as_str()) {
                        list = list.child(heading(heading_of.clone(), cx));
                        last = Some(heading_of);
                    }
                    let start = Start { cwd: cwd.clone(), project: project.clone(), agent: Some(agent.clone()), crew: false, again: None };
                    list = list.child(row(format!("shut-{agent}"), agent.clone(), cwd, cx, start));
                }
            }
        }
        list.into_any_element()
    }

    /// Where a project works: its main loop's directory, or one of its
    /// agents' — one that exists on this machine.
    pub(super) fn project_folder(&self, project: &str) -> Option<String> {
        let here = |dir: &str| std::path::Path::new(dir).is_dir();
        let known = self
            .board
            .known
            .iter()
            .filter(|l| l.project.as_deref() == Some(project) && here(&l.cwd))
            .min_by_key(|l| l.role.is_some())
            .map(|l| l.cwd.clone());
        known.or_else(|| {
            self.board
                .homes
                .iter()
                .find(|h| h.1.as_deref() == Some(project) && here(&h.2))
                .map(|h| h.2.clone())
        })
    }

    /// "+ session" on a project: opens its form, prefilled — the directory
    /// of a loop or an agent of the project, and its agent.
    pub(super) fn ask_new_session(&mut self, project: String, window: &mut Window, cx: &mut Context<Self>) {
        let known = self.board.known.iter().find(|l| l.project.as_deref() == Some(project.as_str()) && l.role.is_none());
        let home = self.board.homes.iter().find(|h| h.1.as_deref() == Some(project.as_str()));
        let cwd_value = self.project_folder(&project).unwrap_or_default();
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
        self.new_session = Some(NewSession { project, cwd, agent, crew: false, on_host: true, busy: false });
        cx.notify();
    }

    /// The form under a project's heading, when "+ session" was clicked.
    pub(super) fn new_session_form(&self, project: &str, cx: &mut Context<Self>) -> Option<AnyElement> {
        let form = self.new_session.as_ref().filter(|f| f.project == project)?;
        let crew = form.crew;
        let on_host = form.on_host;
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
                        } else if exists && on_host {
                            "aiball's host starts it here"
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
                        .id("new-session-host")
                        .flex()
                        .items_center()
                        .gap_1()
                        .cursor_pointer()
                        .text_color(if on_host { p().text } else { p().muted })
                        .child(if on_host { "☑" } else { "☐" })
                        .child("on aiball's host, without tmux")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            if let Some(form) = shell.new_session.as_mut() {
                                form.on_host = !form.on_host;
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
                                        again: None,
                                    };
                                    if form.on_host {
                                        shell.start_on_host(start, cx);
                                    } else {
                                        shell.start_loop(start, cx);
                                    }
                                })),
                        ),
                )
                .into_any_element(),
        )
    }

    /// Starts an agent's loop on aiball's host (`session.start`), off the UI
    /// thread; once its session is listed, tvty opens it over its socket.
    pub(super) fn start_on_host(&mut self, start: Start, cx: &mut Context<Self>) {
        if self.starting.is_some() {
            return;
        }
        self.starting = Some(start.cwd.clone());
        cx.notify();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let done = cx.background_executor().spawn({
                let start = start.clone();
                async move { aiball.start_agent(&start.cwd, start.project.as_deref(), start.agent.as_deref(), start.crew) }
            });
            let done = done.await;
            let _ = this.update(cx, |shell, cx| {
                shell.starting = None;
                match done {
                    Ok(agent) => {
                        shell.new_session = None;
                        shell.open_when_running = Some(format!("{}{agent}", crate::sessions::HOSTED_PREFIX));
                        crate::activity::publish(cx, crate::activity::Activity::done(None, format!("started {agent} on aiball's host in {}", home_short(&start.cwd))));
                    }
                    // Its loop runs already (on the host, or claude-loop's
                    // in tmux): opened as a copy, as claude-loop joins one;
                    // the bar's chip takes the controls. Never a second Claude.
                    Err(error) if format!("{error:#}").contains("HOST_BUSY") && shell.running_session(start.agent.as_deref()).is_some() => {
                        let session = shell.running_session(start.agent.as_deref()).unwrap_or_default();
                        shell.new_session = None;
                        shell.copies.insert(session.clone());
                        shell.open_when_running = Some(session);
                        let agent = start.agent.clone().unwrap_or_default();
                        crate::activity::publish(cx, crate::activity::Activity::done(None, format!("{agent} runs already: opened as a copy")));
                    }
                    Err(error) => {
                        if let Some(form) = shell.new_session.as_mut() {
                            form.busy = false;
                        }
                        crate::activity::publish(cx, crate::activity::Activity::failed(None, "start on the host", format!("{error:#}")));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Starts a loop through claude-loop, off the UI thread; once its
    /// session runs, tvty opens it.
    pub(super) fn start_loop(&mut self, start: Start, cx: &mut Context<Self>) {
        if self.starting.is_some() {
            return;
        }
        self.starting = Some(start.cwd.clone());
        cx.notify();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let done = cx.background_executor().spawn({
                let start = start.clone();
                async move {
                    // An agent whose loop runs on the host already is opened,
                    // never started again. Listed idle (it just started, its
                    // agent not there yet), with the controls; else as a
                    // copy, as claude-loop joins a loop that runs.
                    if let Some(agent) = &start.agent {
                        if aiball.host_runs(agent).unwrap_or(false) {
                            return Ok((format!("{}{agent}", crate::sessions::HOSTED_PREFIX), false, start.again.is_none()));
                        }
                    }
                    crate::loops::start(&start).map(|name| (name, true, false))
                }
            });
            let done = done.await;
            let _ = this.update(cx, |shell, cx| {
                shell.starting = None;
                match done {
                    Ok((name, started, copy)) => {
                        shell.new_session = None;
                        shell.open_when_running = Some(name.clone());
                        if started {
                            crate::activity::publish(cx, crate::activity::Activity::done(None, format!("started {name} in {}", home_short(&start.cwd))));
                        }
                        if copy {
                            shell.copies.insert(name.clone());
                            let agent = start.agent.clone().unwrap_or_default();
                            crate::activity::publish(cx, crate::activity::Activity::done(None, format!("{agent} runs already: opened as a copy")));
                        }
                        let _ = shell.refresh_now.unbounded_send(());
                    }
                    // claude-loop says its loop runs already (in tmux): opened
                    // as a copy, as claude-loop itself joins one.
                    Err(error) if format!("{error:#}").contains("already runs") && shell.running_session(start.agent.as_deref()).is_some() => {
                        let session = shell.running_session(start.agent.as_deref()).unwrap_or_default();
                        shell.new_session = None;
                        shell.copies.insert(session.clone());
                        shell.open_when_running = Some(session);
                        let agent = start.agent.clone().unwrap_or_default();
                        crate::activity::publish(cx, crate::activity::Activity::done(None, format!("{agent} runs already: opened as a copy")));
                    }
                    Err(error) => {
                        if let Some(form) = shell.new_session.as_mut() {
                            form.busy = false;
                        }
                        crate::activity::publish(cx, crate::activity::Activity::failed(None, "start", format!("{error:#}")));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Shell {
    /// The terminal of `agent`'s running loop, as the board lists it.
    fn running_session(&self, agent: Option<&str>) -> Option<String> {
        let agent = agent?;
        self.board.projects.iter().flat_map(|p| &p.terminals).find(|t| t.agent.as_deref() == Some(agent)).map(|t| t.session.clone())
    }
}

/// A path with the home directory as `~`.
pub(super) fn home_short(path: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && path.starts_with(&home) => format!("~{}", &path[home.len()..]),
        _ => path.to_string(),
    }
}
