//! The sessions list, in three foldable sections ([`crate::accordion`]):
//! the sessions that run (live), the loops this machine knows that are
//! stopped (idle, one click starts one again, where it worked, for its
//! agent), and the agents aiball knows with no loop here (shut, one click
//! opens one where the agent works). "+ session" on a project opens one: a working directory
//! (proposed, checked) and an agent. claude-loop starts it, detached, in
//! that directory — never tvty's —, and tvty opens it once it runs.

use crate::ui::Named as _;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Shell;
use crate::loops::{KnownLoop, Start};
use crate::theme::p;
use crate::tip::Tip as _;
use crate::ui::buttons;

/// One stopped loop per agent (aiball keeps the ones an agent had before,
/// under other names): one in its own project's folder first (`astray`: in
/// another project's agent's folder, where it would resume that one's
/// conversation), then one aiball does not say superseded, then one on
/// aiball's host, where tvty starts them; loops with no agent all kept.
fn one_per_agent<'a>(loops: Vec<&'a KnownLoop>, astray: impl Fn(&KnownLoop) -> bool) -> Vec<&'a KnownLoop> {
    let rank = |l: &KnownLoop| (astray(l), l.superseded, !l.on_host());
    let mut kept: Vec<&KnownLoop> = Vec::new();
    for l in loops {
        match l.agent().and_then(|agent| kept.iter().position(|k| k.agent() == Some(agent))) {
            Some(at) if rank(l) < rank(kept[at]) => kept[at] = l,
            Some(_) => {}
            None => kept.push(l),
        }
    }
    kept
}

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
    pub(super) fn inactive(&self, words: &[String]) -> Vec<&KnownLoop> {
        let loops: Vec<&KnownLoop> = self
            .board
            .known
            .iter()
            .filter(|l| self.terminal_of(&l.name).is_none())
            // A loop on aiball's host shows as its agent's hosted terminal.
            .filter(|l| {
                !l.on_host() || self.terminal_of(&l.session()).is_none()
            })
            // An agent live under another loop is not idle: its older,
            // stopped loops stay out of the list.
            .filter(|l| !l.agent().is_some_and(|agent| self.agent_live(agent)))
            .filter(|l| {
                let project = self.loop_project(l).unwrap_or_default();
                crate::sessions::found(words, &[&project, l.agent().unwrap_or(""), &l.name, &l.cwd])
            })
            .collect();
        let mut loops = one_per_agent(loops, |l| self.astray(l).is_some());
        loops.sort_by_cached_key(|l| {
            let project = self.loop_project(l);
            (project.is_none(), project)
        });
        loops
    }

    /// The agent of another project whose folder `l` works in, if it does.
    fn astray(&self, l: &KnownLoop) -> Option<&str> {
        crate::loops::stranger(&l.cwd, l.agent(), self.loop_project(l).as_deref(), &self.board.homes)
    }

    /// `agent` runs: a loop of it runs, or a terminal of it is live.
    fn agent_live(&self, agent: &str) -> bool {
        self.board.known.iter().any(|k| k.running && k.agent() == Some(agent))
            || self.board.projects.iter().any(|p| p.terminals.iter().any(|t| t.agent.as_deref() == Some(agent) && t.status.as_ref().is_some_and(|s| s.online)))
    }

    /// The agents aiball knows with no loop on this machine.
    fn closed(&self, words: &[String]) -> Vec<&(String, Option<String>, String)> {
        self.board
            .homes
            .iter()
            .filter(|(agent, _, cwd)| {
                !self.board.known.iter().any(|l| l.agent() == Some(agent.as_str()) || l.cwd == *cwd)
                    && !self.board.projects.iter().any(|p| p.terminals.iter().any(|t| t.agent.as_deref() == Some(agent.as_str())))
            })
            .filter(|(agent, project, cwd)| crate::sessions::found(words, &[project.as_deref().unwrap_or(""), agent, cwd]))
            .collect()
    }

    /// The hub's agents the filter finds.
    fn hub_found(&self, words: &[String]) -> Vec<crate::sessions::HubAgent> {
        self.hub_agents().iter().filter(|h| crate::sessions::found(words, &[h.project.as_deref().unwrap_or(""), &h.agent])).cloned().collect()
    }

    /// The hub's sessions, by project: read from here, never opened — no
    /// start, no stop. A click shows one in the terminal's place.
    fn hub_list(&self, found: &[crate::sessions::HubAgent], words: &[String], cx: &mut Context<Self>) -> AnyElement {
        let mut list = div().flex().flex_col();
        if found.is_empty() {
            list = list.child(div().px_3().text_color(p().muted).child(if words.is_empty() { "No session on the hub" } else { "No session found on the hub" }));
        }
        let mut last: Option<String> = None;
        for hub in found {
            let project = hub.project.clone().unwrap_or_else(|| "No project".into());
            if last.as_deref() != Some(project.as_str()) {
                list = list.child(div().flex().px_3().pt_2().pb_1().text_xs().text_color(p().muted).child(project.to_uppercase()));
                last = Some(project);
            }
            let shown = self.hub_shown.as_deref() == Some(hub.agent.as_str());
            let agent = hub.agent.clone();
            list = list.child(
                div()
                    .saying(SharedString::from(format!("hub-{}", hub.agent)), hub.agent.clone())
                    .relative()
                    .overflow_hidden()
                    // Behind the row's text.
                    .child(super::hub_mark(SharedString::from(format!("hub-{}-mark", hub.agent))))
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .px_3()
                    .py_1()
                    .cursor_pointer()
                    .when(shown, |d| d.bg(p().active))
                    .when(!shown, |d| d.hover(|d| d.bg(p().hover)))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().flex_1().min_w_0().truncate().child(super::marked(&hub.agent, words))),
                    )
                    .children(hub.status.as_ref().map(|status| status.line(None)))
                    .on_click(cx.listener(move |shell, _, _, cx| shell.show_hub_agent(agent.clone(), cx))),
            );
        }
        list.into_any_element()
    }

    /// The sections, each folded or not as the user left it.
    pub(super) fn sessions_list(&self, cx: &mut Context<Self>) -> AnyElement {
        let words = self.filter_words(cx);
        let running: usize = self.live_found(&words).iter().map(|(_, t)| t.len()).sum();
        let on_hub = self.hub_found(&words);
        let mut groups = vec![("live", running), ("idle", self.inactive(&words).len()), ("shut", self.closed(&words).len())];
        // Behind a proxy node, when asked: the hub's sessions, apart.
        if self.away_from_hub() && self.applied.sessions.show_hub {
            groups.push(("on hub", on_hub.len()));
        }
        let sections = groups.len();
        let mut list = crate::accordion::list("sessions").pb_1();
        for (i, &(word, count)) in groups.iter().enumerate() {
            // A folded section with something the filter found opens while
            // it is typed.
            let folded = self.settings.layout.sessions_folded.iter().any(|f| f == word) && (words.is_empty() || count == 0);
            let body = match (folded, i) {
                (true, _) => Vec::new(),
                (false, 0) => vec![self.live_list(cx).into_any_element()],
                (false, 1) => vec![self.other_list(Other::Idle, cx)],
                (false, 2) => vec![self.other_list(Other::Shut, cx)],
                (false, _) => vec![self.hub_list(&on_hub, &words, cx)],
            };
            let section = crate::accordion::Section {
                list: "sessions".into(),
                above: (i > 0).then(|| SharedString::from(format!("sessions-{}", groups[i - 1].0))),
                last: i + 1 == sections,
                id: SharedString::from(format!("sessions-{}", word.replace(' ', "-"))),
                title: word.to_string(),
                count,
                folded,
                // Two sessions, or the line saying there is none.
                keep: (count.min(2) as f32 * 44.).max(24.),
                scroll: self.session_scrolls[i.min(self.session_scrolls.len() - 1)].clone(),
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
        let row = |id: String, name: String, cwd: &str, astray: Option<&str>, cx: &mut Context<Self>, start: Start| {
            let busy = self.starting.as_deref() == Some(start.cwd.as_str());
            let action = match (busy, astray) {
                (true, _) => div().saying(SharedString::from(format!("{id}-action")), "starting…").text_xs().text_color(p().accent).child("starting…").into_any_element(),
                // Refused: said why on hover.
                (false, Some(other)) => div()
                    .saying(SharedString::from(format!("{id}-astray")), format!("⚠ {other}'s folder"))
                    .text_xs()
                    .text_color(p().danger)
                    .child(format!("⚠ {other}'s folder"))
                    .tip(format!("{other} works in {}: started here, this agent would resume its conversation. Not started.", home_short(cwd)))
                    .into_any_element(),
                (false, None) => div().saying(SharedString::from(format!("{id}-action")), "▶ start").text_xs().text_color(p().accent).child("▶ start").into_any_element(),
            };
            div()
                .named(SharedString::from(id.clone()))
                .relative()
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
                        .child(action),
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
                        again: l.on_host().then(|| l.name.clone()),
                        mode: None,
                    };
                    list = list.child(row(format!("idle-{}", l.name), name, &l.cwd, self.astray(l), cx, start));
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
                    let start = Start { cwd: cwd.clone(), project: project.clone(), agent: Some(agent.clone()), crew: false, again: None, mode: None };
                    let astray = crate::loops::stranger(cwd, Some(agent.as_str()), project.as_deref(), &self.board.homes);
                    list = list.child(row(format!("shut-{agent}"), agent.clone(), cwd, astray, cx, start));
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
        let agent_value = known.and_then(|l| l.agent.clone()).or_else(|| home.map(|h| h.0.clone())).unwrap_or_default();
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
                    buttons::link("new-session-crew", if crew { "☑" } else { "☐" })
                        .text_color(if crew { p().text } else { p().muted })
                        .child("a crew agent, next to the main loop")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            if let Some(form) = shell.new_session.as_mut() {
                                form.crew = !form.crew;
                            }
                            cx.notify();
                        })),
                )
                .child(
                    buttons::link("new-session-host", if on_host { "☑" } else { "☐" })
                        .text_color(if on_host { p().text } else { p().muted })
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
                            buttons::secondary("new-session-cancel", "Cancel")
                                .on_click(cx.listener(|shell, _, _, cx| {
                                    shell.new_session = None;
                                    cx.notify();
                                })),
                        )
                        .child(
                            buttons::primary("new-session-start", "Start")
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
                                        // Unchecked "on aiball's host": tmux, asked.
                                        mode: (!form.on_host).then_some("tmux"),
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

    /// A start in another project's agent's folder: its Claude would resume
    /// that agent's conversation, two Claudes writing in one. Refused, and
    /// said.
    fn astray_start(&self, start: &Start, cx: &mut Context<Self>) -> bool {
        let Some(other) = crate::loops::stranger(&start.cwd, start.agent.as_deref(), start.project.as_deref(), &self.board.homes) else { return false };
        let agent = start.agent.clone().unwrap_or_else(|| "a loop".into());
        let why = format!("{} is {other}'s folder: {agent} would resume its conversation", home_short(&start.cwd));
        crate::activity::publish(cx, crate::activity::Activity::failed(None, "start", why));
        true
    }

    /// Starts an agent's loop on aiball's host (`session.start`), off the UI
    /// thread; once its session is listed, tvty opens it over its socket.
    pub(super) fn start_on_host(&mut self, start: Start, cx: &mut Context<Self>) {
        if self.starting.is_some() || self.astray_start(&start, cx) {
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
        if self.astray_start(&start, cx) {
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
                    crate::loops::start(&aiball, &start).map(|name| (name, true, false))
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
    match tvty_config::home() {
        Some(home) => shorten(path, &home),
        None => path.to_string(),
    }
}

/// `path` with `home` as `~`, compared as paths: on Windows a folder is the
/// same whichever slashes name it.
fn shorten(path: &str, home: &std::path::Path) -> String {
    match std::path::Path::new(path).strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".into(),
        Ok(rest) => format!("~/{}", rest.to_string_lossy().replace('\\', "/")),
        Err(_) => path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{one_per_agent, shorten};
    use crate::loops::KnownLoop;
    use std::path::Path;

    #[test]
    fn a_path_under_home_reads_from_tilde() {
        assert_eq!(shorten("/srv/u/dev/p", Path::new("/srv/u")), "~/dev/p");
        assert_eq!(shorten("/srv/u", Path::new("/srv/u")), "~");
        // A sibling folder that only starts with the same letters is not home.
        assert_eq!(shorten("/srv/user2/p", Path::new("/srv/u")), "/srv/user2/p");
    }

    #[test]
    fn an_agent_is_idle_once_in_its_folder_first_then_not_superseded_then_on_its_host() {
        let known = |name: &str, agent: Option<&str>, mode: &str| KnownLoop {
            name: name.into(),
            agent: agent.map(String::from),
            mode: mode.into(),
            ..Default::default()
        };
        let loops = [
            known("cl-b-1", Some("b"), "tmux"),
            known("cl-b-2", Some("b"), "host"),
            known("cl-a-1", Some("a"), "tmux"),
            known("cl-a-2", Some("a"), "tmux"),
            known("cl-x", None, "tmux"),
            known("cl-y", None, "tmux"),
            // Newer, on the host, but in another agent's folder.
            KnownLoop { cwd: "/w/other".into(), ..known("cl-c-2", Some("c"), "host") },
            KnownLoop { superseded: true, ..known("cl-c-1", Some("c"), "host") },
            KnownLoop { superseded: true, ..known("cl-d-1", Some("d"), "host") },
            known("cl-d-2", Some("d"), "tmux"),
        ];
        let kept: Vec<&str> = one_per_agent(loops.iter().collect(), |l| l.cwd == "/w/other").iter().map(|l| l.name.as_str()).collect();
        assert_eq!(kept, vec!["cl-b-2", "cl-a-1", "cl-x", "cl-y", "cl-c-1", "cl-d-2"]);
    }
}
