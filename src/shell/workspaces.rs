//! Workspaces in the window: the left panel's second tab, and the sessions'
//! picker — one dialog, the quit dialog's frame, to choose among sessions
//! by group before anything is done to them: which a new workspace keeps,
//! which stop when tvty quits or a workspace is shut, which start or are
//! let go when one is opened. Nothing is stopped or started unasked.

use crate::ui::Named as _;
use std::collections::HashMap;

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Shell;
use crate::activity::{self, Activity};
use crate::theme::p;
use crate::tip::Tip as _;
use crate::ui::buttons;
use crate::workspaces::{Group, Mode, Opening, Session, Workspace, opening};

/// What the picker chooses for.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum PickFor {
    /// Quitting: the sessions to stop.
    Quit,
    /// A new workspace: the sessions it keeps.
    New,
    /// A workspace kept again as things are now.
    Save(String),
    /// A workspace shut: the sessions to stop.
    Shut(String),
    /// A workspace opened: what to start, let go or hold.
    Open(String),
    /// At start, the sessions stopped when tvty quit: which to restart,
    /// with the AFK mode each had then (by loop).
    Restart,
}

/// A session in the picker.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct PickRow {
    project: String,
    agent: String,
    /// Its loop on this machine, when one is known (to stop or restart it).
    loop_name: Option<String>,
    /// How it runs now; none: no loop of it runs.
    now: Option<Mode>,
    /// What is said beside it.
    note: String,
    checked: bool,
    /// Nothing to choose: listed only.
    fixed: bool,
    /// Opening: what would be done to it.
    opening: Option<Opening>,
    /// Kept in a workspace with this mode (saving one that does not run).
    kept: Option<Mode>,
}

/// The picker: what for, and its sessions.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Picker {
    what: PickFor,
    rows: Vec<PickRow>,
}

impl Picker {
    /// The sessions stopped at quit, offered at start.
    pub(super) fn restarting(&self) -> bool {
        matches!(self.what, PickFor::Restart)
    }

    /// What it asks and of how many sessions, for the debug control.
    pub(super) fn said(&self) -> serde_json::Value {
        serde_json::json!({
            "for": format!("{:?}", self.what),
            "rows": self.rows.len(),
            "checked": self.rows.iter().filter(|r| r.checked).count(),
        })
    }
}


/// The sessions stopped at quit, as the restart sheet lists them: by
/// project, each ticked, its mark as it was (▶ on its own, ■ held).
fn restart_rows(loops: &[String], still: &[String], held: &std::collections::HashSet<String>, known: &[crate::loops::KnownLoop]) -> Vec<PickRow> {
    let mut rows: Vec<PickRow> = loops
        .iter()
        .chain(still)
        .map(|name| {
            let known = known.iter().find(|l| l.name == *name);
            let project = known.and_then(|l| l.project.clone()).unwrap_or_default();
            let agent = known.and_then(|l| l.agent().map(str::to_string)).unwrap_or_else(|| name.clone());
            let place = if known.is_some_and(|l| l.on_host()) { crate::t!("workspaces-host") } else { crate::mux::program().to_string() };
            // Its hold, as aiball keeps it: the one it restarts in.
            let (now, held) = if held.contains(&agent) { (Mode::Stop, crate::t!("workspaces-held-for-good")) } else { (Mode::Auto, String::new()) };
            // Asked to stop when tvty quit, it ran on: listed, not to choose.
            let ran_on = still.contains(name);
            PickRow {
                project,
                agent,
                loop_name: Some(name.clone()),
                now: Some(now),
                note: if ran_on { crate::t!("workspaces-ran-on", place = place) } else { format!("{place}{held}") },
                checked: !ran_on,
                fixed: ran_on,
                opening: None,
                kept: None,
            }
        })
        .collect();
    rows.sort_by(|a, b| (&a.project, &a.agent).cmp(&(&b.project, &b.agent)));
    rows
}

impl Shell {
    /// The name field of a new or renamed workspace: Enter confirms.
    pub(super) fn workspace_name_field(window: &mut Window, cx: &mut Context<Self>) -> Entity<InputState> {
        let field = cx.new(|cx| InputState::new(window, cx).placeholder(crate::t!("workspaces-its-name")));
        cx.subscribe_in(&field, window, |shell: &mut Self, _, event: &InputEvent, window, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                if shell.workspace_renaming.is_some() {
                    shell.rename_workspace(window, cx);
                } else if shell.picker.is_some() {
                    shell.picker_answer(true, cx);
                }
            }
        })
        .detach();
        field
    }

    // ── What is there now ────────────────────────────────────────────────

    /// The agents' sessions that run, by group: (project, agent).
    fn live_sessions(&self) -> Vec<(String, String)> {
        self.board
            .projects
            .iter()
            .flat_map(|p| p.terminals.iter().filter_map(move |t| Some((p.name.clone(), t.agent.clone()?))))
            .collect()
    }

    /// How `agent`'s session runs now; none when no loop of it runs.
    fn mode_now(&self, agent: &str) -> Option<Mode> {
        let live = self.board.projects.iter().any(|p| p.terminals.iter().any(|t| t.agent.as_deref() == Some(agent)));
        live.then(|| Mode::of_afk(self.board.bars.get(agent).filter(|b| !b.stale).map(|b| b.bar.afk.mode.as_str())))
    }

    /// The loop of `agent` that runs, as this machine knows it.
    fn running_loop(&self, agent: &str) -> Option<String> {
        self.board.known.iter().find(|l| l.agent() == Some(agent) && self.terminal_of(&l.session()).is_some()).map(|l| l.name.clone())
    }

    fn row(&self, project: &str, agent: &str) -> PickRow {
        PickRow {
            project: project.to_string(),
            agent: agent.to_string(),
            loop_name: self.running_loop(agent),
            now: self.mode_now(agent),
            note: String::new(),
            checked: true,
            fixed: false,
            opening: None,
            kept: None,
        }
    }

    // ── Opening the picker ───────────────────────────────────────────────

    /// Quitting with loops running: which to stop.
    pub(super) fn pick_quit(&mut self, loops: Vec<String>, cx: &mut Context<Self>) {
        let mut rows: Vec<PickRow> = loops
            .iter()
            .filter_map(|name| {
                let known = self.board.known.iter().find(|l| l.name == *name)?;
                let agent = known.agent()?;
                let project = known.project.clone().unwrap_or_default();
                Some(PickRow { loop_name: Some(name.clone()), ..self.row(&project, agent) })
            })
            .collect();
        rows.sort_by(|a, b| (&a.project, &a.agent).cmp(&(&b.project, &b.agent)));
        self.open_picker(PickFor::Quit, rows, cx);
    }

    /// "+ workspace": the groups that run, all ticked.
    pub(super) fn pick_new_workspace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let rows = self.live_sessions().iter().map(|(project, agent)| self.row(project, agent)).collect();
        let name = self.settings.saved.free_name(&crate::t!("workspaces-default-name"));
        self.workspace_name.update(cx, |field, cx| {
            field.set_value(name, window, cx);
            field.focus(window, cx);
        });
        self.open_picker(PickFor::New, rows, cx);
    }

    /// A workspace kept again: its sessions ticked (those that do not run
    /// keep their mode), the other groups that run offered unticked.
    pub(super) fn pick_save_workspace(&mut self, name: &str, cx: &mut Context<Self>) {
        let Some(workspace) = self.settings.saved.get(name).cloned() else { return };
        let mut rows = Vec::new();
        for group in &workspace.groups {
            for session in &group.sessions {
                let now = self.mode_now(&session.agent);
                rows.push(PickRow {
                    note: if now.is_none() { crate::t!("workspaces-not-running") } else { String::new() },
                    kept: Some(session.mode),
                    ..self.row(&group.project, &session.agent)
                });
            }
        }
        for (project, agent) in self.live_sessions() {
            if rows.iter().any(|r| r.agent == agent) {
                continue;
            }
            // A session new in one of its groups comes with it; another
            // group is an offer.
            let in_group = workspace.groups.iter().any(|g| g.project == project);
            rows.push(PickRow { checked: in_group, note: crate::t!(if in_group { "workspaces-new-in-group" } else { "workspaces-not-in-yet" }), ..self.row(&project, &agent) });
        }
        self.open_picker(PickFor::Save(name.to_string()), rows, cx);
    }

    /// A workspace shut: its sessions that run; ticked to stop when their
    /// group is in no other workspace.
    pub(super) fn pick_shut_workspace(&mut self, name: &str, cx: &mut Context<Self>) {
        let Some(workspace) = self.settings.saved.get(name).cloned() else { return };
        let mut rows = Vec::new();
        for group in &workspace.groups {
            let others = self.settings.saved.others_with(name, &group.project);
            for session in &group.sessions {
                if self.mode_now(&session.agent).is_none() {
                    continue;
                }
                rows.push(PickRow {
                    checked: others.is_empty(),
                    note: if others.is_empty() { String::new() } else { crate::t!("workspaces-also-in", others = others.join(", ")) },
                    ..self.row(&group.project, &session.agent)
                });
            }
        }
        if rows.is_empty() {
            return activity::publish(cx, Activity::done(None, crate::t!("workspaces-none-runs", name = name)));
        }
        self.open_picker(PickFor::Shut(name.to_string()), rows, cx);
    }

    /// A workspace opened: each session with what opening would do to it.
    pub(super) fn pick_open_workspace(&mut self, name: &str, cx: &mut Context<Self>) {
        let Some(workspace) = self.settings.saved.get(name).cloned() else { return };
        let mut rows = Vec::new();
        for group in &workspace.groups {
            for session in &group.sessions {
                let now = self.mode_now(&session.agent);
                let what = opening(session, now);
                rows.push(PickRow {
                    checked: what.by_default(),
                    fixed: what == Opening::AsKept,
                    note: what.said().into(),
                    opening: Some(what),
                    kept: Some(session.mode),
                    ..self.row(&group.project, &session.agent)
                });
            }
        }
        if rows.iter().all(|r| r.fixed) {
            return activity::publish(cx, Activity::done(None, crate::t!("workspaces-all-as-kept", name = name)));
        }
        self.open_picker(PickFor::Open(name.to_string()), rows, cx);
    }

    fn open_picker(&mut self, what: PickFor, rows: Vec<PickRow>, cx: &mut Context<Self>) {
        self.picker = Some(Picker { what, rows });
        self.remember = false;
        cx.notify();
    }

    pub(super) fn cancel_picker(&mut self, cx: &mut Context<Self>) -> bool {
        let open = self.picker.take().is_some();
        if open {
            cx.notify();
        }
        open
    }

    // ── Its answer ───────────────────────────────────────────────────────

    /// The picker's answer: `checked`, the ticked sessions are dealt with;
    /// otherwise none is (quitting keeps them all, shutting stops none).
    pub(super) fn picker_answer(&mut self, checked: bool, cx: &mut Context<Self>) {
        let Some(picker) = self.picker.take() else { return };
        let chosen: Vec<PickRow> = picker.rows.iter().filter(|r| checked && r.checked && !r.fixed).cloned().collect();
        match picker.what {
            PickFor::Quit => {
                if self.remember {
                    self.set_pref("sessions.on_quit", tvty_config::Value::Choice(Some(if checked { "stop" } else { "keep" }.into())), cx);
                }
                let loops: Vec<String> = chosen.iter().filter_map(|r| r.loop_name.clone()).collect();
                if loops.is_empty() { self.quit_now(cx) } else { self.stop_and_quit(loops, cx) }
            }
            PickFor::New | PickFor::Save(_) => {
                let name = match &picker.what {
                    PickFor::Save(name) => name.clone(),
                    _ => self.settings.saved.free_name(&self.workspace_name.read(cx).value()),
                };
                let kept: Vec<&PickRow> = picker.rows.iter().filter(|r| r.checked).collect();
                let mut groups: Vec<Group> = Vec::new();
                for row in kept {
                    // As it runs now; one that does not run, as it was kept.
                    let mode = row.now.or(row.kept).unwrap_or(Mode::Auto);
                    let session = Session { agent: row.agent.clone(), mode };
                    match groups.iter_mut().find(|g| g.project == row.project) {
                        Some(group) => group.sessions.push(session),
                        None => groups.push(Group { project: row.project.clone(), sessions: vec![session] }),
                    }
                }
                let said = crate::t!("workspaces-kept", name = name.clone(), count = groups.len());
                self.settings.saved.put(Workspace { name, groups });
                self.settings.save(cx);
                activity::publish(cx, Activity::done(None, said));
            }
            PickFor::Shut(name) => self.stop_sessions(&name, chosen, cx),
            PickFor::Open(name) => self.open_sessions(&name, chosen, cx),
            // "Not now": none restarted (remembered: never asked again).
            PickFor::Restart => {
                if self.remember && !checked {
                    self.set_pref("sessions.on_start", tvty_config::Value::Choice(Some("leave".into())), cx);
                }
                if checked {
                    let loops: Vec<String> = chosen.iter().filter_map(|r| r.loop_name.clone()).collect();
                    self.restart_loops(loops, true, cx);
                }
            }
        }
        cx.notify();
    }

    /// The restart sheet's answer: the ticked sessions restarted, as they
    /// were (`as_they_were`) or fresh; remembered, for all of them, every
    /// time.
    pub(super) fn picker_restart(&mut self, as_they_were: bool, cx: &mut Context<Self>) {
        let Some(picker) = self.picker.take() else { return };
        let PickFor::Restart = picker.what else {
            self.picker = Some(picker);
            return;
        };
        if self.remember {
            let choice = if as_they_were { "restart" } else { "fresh" };
            self.set_pref("sessions.on_start", tvty_config::Value::Choice(Some(choice.into())), cx);
        }
        let loops: Vec<String> = picker.rows.iter().filter(|r| r.checked && !r.fixed).filter_map(|r| r.loop_name.clone()).collect();
        if !loops.is_empty() {
            self.restart_loops(loops, as_they_were, cx);
        }
        cx.notify();
    }

    /// At start: the sessions stopped when tvty quit, offered to restart —
    /// each ticked, its mark as it was (▶ on its own, ■ held).
    pub(super) fn pick_restart(&mut self, loops: Vec<String>, still: Vec<String>, cx: &mut Context<Self>) {
        let rows = restart_rows(&loops, &still, &self.board.held, &self.board.known);
        self.open_picker(PickFor::Restart, rows, cx);
    }

    /// Stops the loops of `rows` (a workspace shut), off the UI thread.
    fn stop_sessions(&mut self, workspace: &str, rows: Vec<PickRow>, cx: &mut Context<Self>) {
        if rows.is_empty() {
            return activity::publish(cx, Activity::done(None, crate::t!("workspaces-shut-run-on", name = workspace)));
        }
        let known: Vec<crate::loops::KnownLoop> =
            rows.iter().filter_map(|r| self.board.known.iter().find(|l| Some(&l.name) == r.loop_name.as_ref()).cloned()).collect();
        let missing = rows.len() - known.len();
        let (aiball, workspace) = (self.aiball.clone(), workspace.to_string());
        cx.spawn(async move |this, cx| {
            let failed: Vec<String> = cx
                .background_executor()
                .spawn(async move { known.iter().filter_map(|l| crate::loops::stop(&aiball, l).err().map(|e| format!("{e:#}"))).collect() })
                .await;
            let _ = this.update(cx, |shell, cx| {
                let done = rows.len() - failed.len() - missing;
                let said = match failed.first() {
                    None if missing == 0 => Activity::done(None, crate::t!("workspaces-shut-stopped", name = workspace.clone(), count = done)),
                    None => Activity::failed(None, &crate::t!("workspaces-shut"), crate::t!("workspaces-no-loop", count = missing)),
                    Some(error) => Activity::failed(None, &crate::t!("workspaces-shut"), error.clone()),
                };
                activity::publish(cx, said);
                let _ = shell.refresh_now.unbounded_send(());
            });
        })
        .detach();
    }

    /// Does what opening a workspace asked of `rows`: starts, lets go, holds.
    fn open_sessions(&mut self, workspace: &str, rows: Vec<PickRow>, cx: &mut Context<Self>) {
        if rows.is_empty() {
            return;
        }
        // What starts each stopped one: its loop of this machine (the one the
        // idle list would start), else its folder as aiball knows it.
        enum How {
            Restart(String),
            Start { cwd: String, project: Option<String> },
            Nothing(String),
        }
        let idle: HashMap<String, (String, String)> =
            self.inactive(&[]).iter().filter_map(|l| Some((l.agent()?.to_string(), (l.name.clone(), l.cwd.clone())))).collect();
        let plan: Vec<(PickRow, Option<How>)> = rows
            .into_iter()
            .map(|row| {
                let how = matches!(row.opening, Some(Opening::Start(_))).then(|| {
                    let home = self.board.homes.iter().find(|(agent, _, _)| *agent == row.agent);
                    let project = Some(row.project.as_str()).filter(|p| !p.is_empty());
                    let (cwd, how) = match (idle.get(&row.agent), home) {
                        (Some((name, cwd)), _) => (cwd.clone(), How::Restart(name.clone())),
                        (None, Some((_, _, cwd))) => (cwd.clone(), How::Start { cwd: cwd.clone(), project: project.map(String::from) }),
                        (None, None) => return How::Nothing(crate::t!("workspaces-nothing-to-start", agent = row.agent.clone())),
                    };
                    // Never in another project's agent's folder.
                    match crate::loops::stranger(&cwd, Some(&row.agent), project, &self.board.homes) {
                        Some(other) => How::Nothing(crate::t!("workspaces-astray", agent = row.agent.clone(), cwd = cwd.to_string(), other = other.to_string())),
                        None => how,
                    }
                });
                (row, how)
            })
            .collect();
        let (aiball, workspace) = (self.aiball.clone(), workspace.to_string());
        cx.spawn(async move |this, cx| {
            let (done, failed) = cx
                .background_executor()
                .spawn(async move {
                    let (mut done, mut failed) = (0usize, Vec::new());
                    for (row, how) in plan {
                        let mode = match row.opening {
                            Some(Opening::Start(mode)) => mode,
                            Some(Opening::Launch) => Mode::Auto,
                            Some(Opening::Hold) => Mode::Stop,
                            _ => continue,
                        };
                        // Its hold first: a loop started now starts in it,
                        // one running takes it at once.
                        if let Err(error) = aiball.set_afk_hold(&row.agent, mode.hold()) {
                            failed.push(crate::t!("workspaces-mode-failed", agent = row.agent.clone(), error = format!("{error:#}")));
                            continue;
                        }
                        let started = match how {
                            None => Ok(()),
                            Some(How::Restart(name)) => crate::loops::restart(&aiball, &name, None).map(drop),
                            Some(How::Start { cwd, project }) => aiball.start_agent(&cwd, project.as_deref(), Some(&row.agent), false, None).map(drop),
                            Some(How::Nothing(why)) => Err(anyhow::anyhow!(why)),
                        };
                        match started {
                            Ok(()) => done += 1,
                            Err(error) => failed.push(format!("{error:#}")),
                        }
                    }
                    (done, failed)
                })
                .await;
            let _ = this.update(cx, |shell, cx| {
                let said = match failed.first() {
                    None => Activity::done(None, crate::t!("workspaces-opened", name = workspace.clone(), count = done)),
                    Some(error) => Activity::failed(None, &crate::t!("workspaces-opening", name = workspace.clone()), error.clone()),
                };
                activity::publish(cx, said);
                let _ = shell.refresh_now.unbounded_send(());
            });
        })
        .detach();
    }

    // ── The picker, drawn ────────────────────────────────────────────────

    /// The picker over the window, when one is open.
    pub(super) fn picker_dialog(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let picker = self.picker.as_ref()?;
        let (title, summary) = match &picker.what {
            PickFor::Quit => (crate::t!("workspaces-quit-title"), crate::t!("workspaces-quit-summary")),
            PickFor::New => (crate::t!("workspaces-new-title"), crate::t!("workspaces-new-summary")),
            PickFor::Save(name) => (crate::t!("workspaces-save-title", name = name.clone()), crate::t!("workspaces-save-summary")),
            PickFor::Shut(name) => (crate::t!("workspaces-shut-title", name = name.clone()), crate::t!("workspaces-shut-summary")),
            PickFor::Open(name) => (crate::t!("workspaces-open-title", name = name.clone()), crate::t!("workspaces-open-summary")),
            PickFor::Restart => (crate::t!("workspaces-restart-title"), crate::t!("workspaces-restart-summary")),
        };
        // By group, in the order they came.
        let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
        for (i, row) in picker.rows.iter().enumerate() {
            match groups.iter_mut().find(|(project, _)| *project == row.project) {
                Some((_, rows)) => rows.push(i),
                None => groups.push((row.project.clone(), vec![i])),
            }
        }
        let mut list = div().named("picker-list").flex().flex_col().gap_2().p_2().text_sm();
        for (project, indexes) in groups {
            let free: Vec<usize> = indexes.iter().copied().filter(|i| !picker.rows[*i].fixed).collect();
            let ticked = free.iter().filter(|i| picker.rows[**i].checked).count();
            let all = !free.is_empty() && ticked == free.len();
            // The group's own switch: on when all of it is; how many are, when
            // only some, is said after its name.
            let some = (!all && ticked > 0).then(|| crate::t!("workspaces-some", ticked = ticked, of = free.len()));
            let name = div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(p().muted).child(if project.is_empty() { crate::t!("workspaces-no-project") } else { project.to_uppercase() }))
                .children(some.map(|some| div().text_xs().text_color(p().muted).child(some)));
            // The group's name flips its switch too.
            let head = if free.is_empty() {
                name.into_any_element()
            } else {
                buttons::switch_with(
                    format!("pick-group-{project}"),
                    all,
                    name,
                    cx.listener(move |shell, wanted: &bool, _, cx| {
                        if let Some(picker) = shell.picker.as_mut() {
                            for i in &free {
                                picker.rows[*i].checked = *wanted;
                            }
                        }
                        cx.notify();
                    }),
                )
                .tip(crate::t!("workspaces-whole-group"))
                .into_any_element()
            };
            let mut group = div().flex().flex_col().gap_0p5().child(head);
            for i in indexes {
                let row = &picker.rows[i];
                let (glyph, colour) = match row.now {
                    Some(Mode::Auto) => ("▶", p().success),
                    Some(Mode::Stop) => ("■", p().danger),
                    None => ("·", p().muted),
                };
                // Restarting: its mark turns its hold (kept by aiball), the
                // one it restarts in.
                let restart_hold = (matches!(picker.what, PickFor::Restart) && !row.fixed).then(|| (row.agent.clone(), row.now == Some(Mode::Stop)));
                let mark = div()
                    .named(SharedString::from(format!("pick-hold-{}", row.agent)))
                    .flex_none()
                    .px_0p5()
                    .rounded_sm()
                    .child(crate::icons::loop_glyph(glyph, colour, 9.))
                    .when_some(restart_hold, |d, (agent, held)| {
                        d.cursor_pointer()
                            .hover(|d| d.bg(p().active))
                            .tip(crate::t!(if held { "sessions-hold-held" } else { "sessions-hold-free" }))
                            .on_click(cx.listener(move |shell, _, _, cx| {
                                cx.stop_propagation();
                                if let Some(row) = shell.picker.as_mut().and_then(|p| p.rows.get_mut(i)) {
                                    row.now = Some(Mode::from_held(!held));
                                    row.note = row.note.replace(&crate::t!("workspaces-held-for-good"), "");
                                    if !held {
                                        row.note.push_str(&crate::t!("workspaces-held-for-good"));
                                    }
                                }
                                shell.set_hold(agent.clone(), !held, cx);
                            }))
                    });
                // What the row says: the loop's glyph, its agent, a note.
                let said = div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(mark)
                    .child(div().text_color(if row.fixed { p().muted } else { p().text }).child(row.agent.clone()))
                    .child(div().text_xs().text_color(p().muted).child(row.note.clone()));
                let line = if row.fixed {
                    // A switch's width: the names stay in one column.
                    div().flex().items_center().gap_2().child(div().w(px(28.)).flex_none()).child(said).into_any_element()
                } else {
                    // The row's words flip its switch too.
                    buttons::switch_with(
                        format!("pick-{}", row.agent),
                        row.checked,
                        said,
                        cx.listener(move |shell, wanted: &bool, _, cx| {
                            if let Some(row) = shell.picker.as_mut().and_then(|p| p.rows.get_mut(i)) {
                                row.checked = *wanted;
                            }
                            cx.notify();
                        }),
                    )
                    .into_any_element()
                };
                group = group.child(div().pl_2().child(line));
            }
            list = list.child(group);
        }
        let ticked = picker.rows.iter().filter(|r| r.checked && !r.fixed).count();
        let cancel = buttons::secondary("picker-cancel", crate::t!("workspaces-cancel")).on_click(cx.listener(|shell, _, _, cx| {
            shell.cancel_picker(cx);
        }));
        let none = |id: &'static str, label: String, cx: &mut Context<Self>| {
            buttons::secondary(id, label).on_click(cx.listener(|shell, _, _, cx| shell.picker_answer(false, cx)))
        };
        let go = |label: String, cx: &mut Context<Self>| buttons::primary("picker-go", label).on_click(cx.listener(|shell, _, _, cx| shell.picker_answer(true, cx)));
        let answers = match &picker.what {
            PickFor::Quit => div()
                .flex()
                .gap_2()
                .child(cancel)
                .child(none("picker-none", crate::t!("workspaces-quit-keep"), cx))
                .child(go(crate::t!("workspaces-quit-stop", count = ticked), cx)),
            PickFor::New | PickFor::Save(_) => div().flex().gap_2().child(cancel).child(go(crate::t!("workspaces-keep"), cx)),
            PickFor::Shut(_) => div()
                .flex()
                .gap_2()
                .child(cancel)
                .child(none("picker-none", crate::t!("workspaces-shut-keep"), cx))
                .child(go(crate::t!("workspaces-shut-stop", count = ticked), cx)),
            PickFor::Open(_) => div().flex().gap_2().child(cancel).child(go(crate::t!("workspaces-open-do", count = ticked), cx)),
            PickFor::Restart => div()
                .flex()
                .gap_2()
                .child(none("picker-none", crate::t!("workspaces-not-now"), cx))
                .child(buttons::secondary("picker-fresh", crate::t!("workspaces-restart-fresh", count = ticked)).on_click(cx.listener(|shell, _, _, cx| shell.picker_restart(false, cx))))
                .child(buttons::primary("picker-go", crate::t!("workspaces-restart-as-were", count = ticked)).on_click(cx.listener(|shell, _, _, cx| shell.picker_restart(true, cx)))),
        };
        let remember = self.remember;
        let remembered = match &picker.what {
            PickFor::Quit => Some(crate::t!("workspaces-remember-quit")),
            PickFor::Restart => Some(crate::t!("workspaces-remember-restart")),
            _ => None,
        };
        let body = div()
            .flex()
            .flex_col()
            .gap_3()
            .when(picker.what == PickFor::New, |d| d.child(div().flex().items_center().gap_2().child(crate::t!("workspaces-name")).child(div().flex_1().child(Input::new(&self.workspace_name)))))
            .child(div().rounded_md().border_1().border_color(p().border).bg(p().bg).child(list));
        let mut sheet = crate::ui::sheet::Sheet::new("picker", title)
            .summary(summary)
            .body(body)
            .answers(answers)
            .on_close(cx.listener(|shell, _, _, cx| {
                shell.cancel_picker(cx);
            }));
        if let Some(label) = remembered {
            sheet = sheet.setting(
                buttons::switch(
                    "picker-remember",
                    remember,
                    label,
                    cx.listener(|shell, wanted: &bool, _, cx| {
                        shell.remember = *wanted;
                        cx.notify();
                    }),
                )
                .text_sm()
                .tip(crate::t!("workspaces-remember-tip")),
            );
        }
        Some(sheet.render())
    }

    // ── The Workspaces tab ───────────────────────────────────────────────

    fn rename_workspace(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(old) = self.workspace_renaming.take() else { return };
        let wanted = self.workspace_name.read(cx).value().trim().to_string();
        if !wanted.is_empty() && wanted != old {
            let name = self.settings.saved.free_name(&wanted);
            if let Some(workspace) = self.settings.saved.workspaces.iter_mut().find(|w| w.name == old) {
                workspace.name = name;
            }
            self.settings.save(cx);
        }
        cx.notify();
    }

    /// The group shown now: the project of the terminal on screen, and its
    /// agents' sessions as they run.
    fn current_group(&self) -> Option<Group> {
        let (project, _) = self.terminal_of(self.selected.as_deref()?)?;
        let sessions: Vec<Session> = self
            .live_sessions()
            .into_iter()
            .filter(|(of, _)| of == project)
            .map(|(_, agent)| Session { mode: self.mode_now(&agent).unwrap_or(Mode::Auto), agent })
            .collect();
        (!sessions.is_empty()).then(|| Group { project: project.to_string(), sessions })
    }

    /// The group shown now goes into `name`: added, or put again as it runs.
    fn add_current_group(&mut self, name: &str, cx: &mut Context<Self>) {
        let Some(group) = self.current_group() else { return };
        let project = group.project.clone();
        if let Some(workspace) = self.settings.saved.workspaces.iter_mut().find(|w| w.name == name) {
            match workspace.groups.iter_mut().find(|g| g.project == project) {
                Some(kept) => *kept = group,
                None => workspace.groups.push(group),
            }
        }
        self.settings.save(cx);
        activity::publish(cx, Activity::done(None, crate::t!("workspaces-added", project = project.clone(), name = name)));
        cx.notify();
    }

    /// The left panel's second tab: the workspaces kept, each with its
    /// groups and their sessions — kept how, and how they are now.
    pub(super) fn workspaces_tab(&self, cx: &mut Context<Self>) -> AnyElement {
        let current = self.current_group().map(|g| g.project);
        let mut list = div().named("workspaces").flex().flex_col().pb_2();
        list = list.child(
            div().flex().px_3().py_2().child(
                buttons::link("workspace-new", crate::t!("workspaces-new"))
                    .text_sm()
                    .tip(crate::t!("workspaces-new-tip"))
                    .on_click(cx.listener(|shell, _, window, cx| shell.pick_new_workspace(window, cx))),
            ),
        );
        if self.settings.saved.workspaces.is_empty() {
            list = list.child(div().px_3().text_sm().text_color(p().muted).child(crate::t!("workspaces-none")));
        }
        for workspace in &self.settings.saved.workspaces {
            let name = workspace.name.clone();
            let folded = self.settings.layout.workspaces_folded.contains(&name);
            let renaming = self.workspace_renaming.as_deref() == Some(name.as_str());
            let deleting = self.workspace_deleting.as_deref() == Some(name.as_str());
            let act = |id: &str, label: String, tip: String| buttons::link(SharedString::from(format!("workspace-{id}-{name}")), label).text_xs().tip(tip);
            let title = if renaming {
                div().flex_1().min_w_0().child(Input::new(&self.workspace_name).small()).into_any_element()
            } else {
                div()
                    .named(SharedString::from(format!("workspace-{name}")))
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_1()
                    .cursor_pointer()
                    .child(div().text_xs().text_color(p().muted).child(if folded { "▸" } else { "▾" }))
                    .child(div().truncate().font_weight(FontWeight::BOLD).child(name.clone()))
                    .on_click(cx.listener({
                        let name = name.clone();
                        move |shell, _, _, cx| {
                            let folded = &mut shell.settings.layout.workspaces_folded;
                            match folded.iter().position(|f| *f == name) {
                                Some(at) => {
                                    folded.remove(at);
                                }
                                None => folded.push(name.clone()),
                            }
                            shell.settings.save(cx);
                            cx.notify();
                        }
                    }))
                    .into_any_element()
            };
            let head = div()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .border_t_1()
                .border_color(p().border)
                .child(title)
                .child(act("open", crate::t!("workspaces-act-open"), crate::t!("workspaces-act-open-tip")).on_click(cx.listener({
                    let name = name.clone();
                    move |shell, _, _, cx| shell.pick_open_workspace(&name, cx)
                })))
                .child(act("shut", crate::t!("workspaces-act-shut"), crate::t!("workspaces-act-shut-tip")).on_click(cx.listener({
                    let name = name.clone();
                    move |shell, _, _, cx| shell.pick_shut_workspace(&name, cx)
                })));
            let tools = div()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .pb_1()
                .text_color(p().muted)
                .child(act("save", crate::t!("workspaces-act-save"), crate::t!("workspaces-act-save-tip")).on_click(cx.listener({
                    let name = name.clone();
                    move |shell, _, _, cx| shell.pick_save_workspace(&name, cx)
                })))
                // The group on screen, added (or put again as it runs now).
                .when_some(current.clone(), |d, project| {
                    let there = workspace.groups.iter().any(|g| g.project == project);
                    d.child(
                        buttons::link(SharedString::from(format!("workspace-add-{name}")), if there { format!("↻ {project}") } else { format!("+ {project}") })
                            .text_xs()
                            .tip(crate::t!(if there { "workspaces-again-tip" } else { "workspaces-add-tip" }))
                            .on_click(cx.listener({
                                let name = name.clone();
                                move |shell, _, _, cx| shell.add_current_group(&name, cx)
                            })),
                    )
                })
                .child(act("rename", crate::t!("workspaces-act-rename"), crate::t!("workspaces-act-rename-tip")).on_click(cx.listener({
                    let name = name.clone();
                    move |shell, _, window, cx| {
                        shell.workspace_renaming = (shell.workspace_renaming.as_deref() != Some(name.as_str())).then(|| name.clone());
                        shell.workspace_name.update(cx, |field, cx| {
                            field.set_value(name.clone(), window, cx);
                            field.focus(window, cx);
                        });
                        cx.notify();
                    }
                })))
                .child(
                    act("delete", crate::t!(if deleting { "workspaces-act-delete-sure" } else { "workspaces-act-delete" }), crate::t!("workspaces-act-delete-tip"))
                        .when(deleting, |d| d.text_color(p().danger))
                        .on_click(cx.listener({
                            let name = name.clone();
                            move |shell, _, _, cx| {
                                if shell.workspace_deleting.as_deref() == Some(name.as_str()) {
                                    shell.workspace_deleting = None;
                                    shell.settings.saved.remove(&name);
                                    shell.settings.save(cx);
                                } else {
                                    shell.workspace_deleting = Some(name.clone());
                                }
                                cx.notify();
                            }
                        })),
                );
            list = list.child(head);
            if folded {
                continue;
            }
            list = list.child(tools);
            for group in &workspace.groups {
                let project = group.project.clone();
                list = list.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .pt_1()
                        .child(div().flex_1().text_xs().font_weight(FontWeight::BOLD).text_color(p().muted).child(if project.is_empty() { crate::t!("workspaces-no-project").to_uppercase() } else { project.to_uppercase() }))
                        .child(
                            buttons::remove(SharedString::from(format!("workspace-drop-{name}-{project}")), "✕", crate::t!("workspaces-drop-tip"))
                                .text_xs()
                                .on_click(cx.listener({
                                    let (name, project) = (name.clone(), project.clone());
                                    move |shell, _, _, cx| {
                                        if let Some(workspace) = shell.settings.saved.workspaces.iter_mut().find(|w| w.name == name) {
                                            workspace.groups.retain(|g| g.project != project);
                                        }
                                        shell.settings.save(cx);
                                        cx.notify();
                                    }
                                })),
                        ),
                );
                for session in &group.sessions {
                    let now = self.mode_now(&session.agent);
                    let differs = now != Some(session.mode);
                    let said = crate::t!(match now {
                        None => "workspaces-now-stopped",
                        Some(Mode::Auto) => "workspaces-now-runs",
                        Some(Mode::Stop) => "workspaces-now-held",
                    });
                    let live = self.board.projects.iter().flat_map(|p| &p.terminals).find(|t| t.agent.as_deref() == Some(session.agent.as_str())).map(|t| t.session.clone());
                    list = list.child(
                        div()
                            .named(SharedString::from(format!("workspace-session-{name}-{}", session.agent)))
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .pl_5()
                            .py_0p5()
                            .text_sm()
                            .when(live.is_some(), |d| d.cursor_pointer().hover(|d| d.bg(p().hover)))
                            .child(crate::icons::loop_glyph(session.mode.glyph(), if session.mode == Mode::Auto { p().success } else { p().danger }, 9.))
                            .child(div().flex_1().min_w_0().truncate().child(session.agent.clone()))
                            .child(div().text_xs().text_color(if differs { p().warning } else { p().muted }).child(said.clone()))
                            .tip(format!(
                                "{}{}",
                                crate::t!(if session.mode == Mode::Auto { "workspaces-kept-own" } else { "workspaces-kept-held" }, now = said),
                                if differs { crate::t!("workspaces-open-sets") } else { String::new() }
                            ))
                            .when_some(live, |d, live| d.on_click(cx.listener(move |shell, _, window, cx| shell.select(live.clone(), window, cx)))),
                    );
                }
            }
        }
        list.overflow_y_scrollbar().into_any_element()
    }
}

#[cfg(test)]
mod tests {
    // Not `super::*`: the kit's own `test` attribute would come with it.
    use super::{Mode, restart_rows};
    use std::collections::HashSet;
    use crate::loops::KnownLoop;

    #[test]
    fn the_sessions_to_restart_are_listed_by_project_ticked_with_their_hold() {
        let known = vec![
            KnownLoop { name: "cl-z-1".into(), agent: Some("z-claude".into()), project: Some("zeta".into()), mode: "host".into(), ..Default::default() },
            KnownLoop { name: "cl-a-1".into(), agent: Some("a-claude".into()), project: Some("alpha".into()), mode: "tmux".into(), ..Default::default() },
        ];
        // aiball keeps z-claude held: it restarts held.
        let held = HashSet::from(["z-claude".to_string()]);
        let rows = restart_rows(&["cl-z-1".into(), "cl-a-1".into()], &[], &held, &known);
        let said: Vec<(&str, &str, Option<Mode>, &str, bool)> =
            rows.iter().map(|r| (r.project.as_str(), r.agent.as_str(), r.now, r.note.as_str(), r.checked)).collect();
        assert_eq!(
            said,
            vec![("alpha", "a-claude", Some(Mode::Auto), crate::mux::program(), true), ("zeta", "z-claude", Some(Mode::Stop), "host · held until let go", true)]
        );
        assert!(rows.iter().all(|r| r.loop_name.is_some() && !r.fixed));
    }

    #[test]
    fn a_session_whose_stop_did_not_take_is_listed_not_to_choose() {
        let known = vec![KnownLoop { name: "cl-b-1".into(), agent: Some("b-claude".into()), project: Some("beta".into()), mode: "host".into(), ..Default::default() }];
        let rows = restart_rows(&[], &["cl-b-1".into()], &HashSet::new(), &known);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].fixed && !rows[0].checked);
        assert_eq!(rows[0].note, "host · ran on: its stop did not take");
    }
}
