//! The agent bar's Claude config dir: an icon after the folder, whose
//! popup names the dir the agent's folder runs on (its account, settings
//! and sessions) and offers the other dirs aiball's folders use. A switch
//! restarts the folder's loops on the new dir as a restart for an update
//! does: held until they are idle, their conversations carried along
//! (`project.settings_set` with `move_sessions`) and resumed.
//!
//! tvty reads no Claude file: the dirs are what `project.settings` says of
//! each folder aiball knows.

use std::collections::{BTreeSet, HashMap};

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Shell;
use super::loopstabs::home_short;
use crate::aiball::{ConfigDirState, Setting};
use crate::icons::{self, Icon};
use crate::loops::KnownLoop;
use crate::theme::p;
use crate::tip::Tip as _;
use crate::ui::buttons;
use crate::ui::chipbar::{self, Edge};

/// The projects a dir's line names; the others are counted ("+12").
const PROJECTS_SHOWN: usize = 3;
/// The widest a dir's path is drawn in the popup: a longer one is cut.
const POPUP_PATH_MAX: f32 = 420.;

/// The projects on a dir, as its line says them: the first few, then how
/// many more.
fn projects_said(projects: &[String]) -> String {
    match projects.len() {
        n if n <= PROJECTS_SHOWN => projects.join(", "),
        n => format!("{} +{}", projects[..PROJECTS_SHOWN].join(", "), n - PROJECTS_SHOWN),
    }
}

/// The longest a switch waits for the folder's loops to stop.
const STOP_WAIT: std::time::Duration = std::time::Duration::from_secs(15);

/// A folder's Claude config dir, as aiball resolved it when last read.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct FolderDir {
    pub dir: Setting<String>,
    pub state: Option<ConfigDirState>,
}

/// A dir the popup offers: the projects whose folder runs on it, whether
/// it is the layer below (no dir of their own), whether a loop can start
/// on it.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Choice {
    pub dir: String,
    pub default: bool,
    pub projects: Vec<String>,
    pub ready: bool,
}

impl Choice {
    /// What `claude.config_dir` is set to: none for the layer below.
    fn value(&self) -> Option<String> {
        (!self.default).then(|| self.dir.clone())
    }
}

/// A switch asked: the folder moves to `to` once its loops are idle.
#[derive(Clone, Debug)]
pub(super) struct Switch {
    pub cwd: String,
    pub to: Choice,
    /// Under way: its loops stopped, the sessions moving.
    pub running: bool,
}

/// What a switch did: the loops restarted, the session files moved.
struct Done {
    restarted: usize,
    moved: u64,
}

/// The dirs to offer, `current` left out: one per dir, with the projects
/// on it, the layer below marked when a folder runs on it without a dir of
/// its own. `folders` is each folder's project and dir.
pub(super) fn choices(folders: &[(String, FolderDir)], current: &str) -> Vec<Choice> {
    let mut by_dir: Vec<Choice> = Vec::new();
    for (project, folder) in folders {
        let dir = &folder.dir.value;
        if dir.is_empty() || dir == current {
            continue;
        }
        let ready = folder.state.is_none_or(|s| s.exists && s.first_run_done && s.has_account);
        let default = !folder.dir.set();
        match by_dir.iter_mut().find(|c| c.dir == *dir) {
            Some(choice) => {
                choice.default |= default;
                if !choice.projects.contains(project) {
                    choice.projects.push(project.clone());
                }
            }
            None => by_dir.push(Choice { dir: dir.clone(), default, projects: vec![project.clone()], ready }),
        }
    }
    for choice in &mut by_dir {
        choice.projects.sort();
    }
    // The layer below first, then by path.
    by_dir.sort_by(|a, b| b.default.cmp(&a.default).then_with(|| a.dir.cmp(&b.dir)));
    by_dir
}

impl Shell {
    /// The folders aiball knows, each with its project: its agents' homes
    /// and this machine's loops.
    fn known_folders(&self) -> Vec<(String, String)> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        let homes = self.board.homes.iter().map(|(_, project, cwd)| (project.clone(), cwd.clone()));
        let loops = self.board.known.iter().map(|l| (l.project.clone(), l.cwd.clone()));
        for (project, cwd) in homes.chain(loops) {
            if cwd.is_empty() || !seen.insert(cwd.clone()) {
                continue;
            }
            out.push((project.unwrap_or_else(|| home_short(&cwd)), cwd));
        }
        out
    }

    /// Reads again, off the UI thread, the config dir of every folder known.
    pub(super) fn read_config_dirs(&mut self, cx: &mut Context<Self>) {
        if self.config_reading {
            return;
        }
        self.config_reading = true;
        let folders = self.known_folders();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let read = cx
                .background_executor()
                .spawn(async move {
                    folders
                        .into_iter()
                        .filter_map(|(_, cwd)| match aiball.project_settings(&cwd) {
                            Ok(s) => Some((cwd, FolderDir { dir: s.config_dir, state: s.config_dir_state })),
                            Err(error) => {
                                log::warn!("config dir of {cwd}: {error:#}");
                                None
                            }
                        })
                        .collect::<HashMap<_, _>>()
                })
                .await;
            let _ = this.update(cx, |shell, cx| {
                shell.config_reading = false;
                if shell.config_dirs != read {
                    shell.config_dirs = read;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// The icon after the folder, and its popup when open.
    pub(super) fn config_chip(&self, agent: &str, cwd: &str, cx: &mut Context<Self>) -> AnyElement {
        let folder = self.config_dirs.get(cwd);
        let open = self.config_menu.as_deref() == Some(agent);
        let switch = self.config_switches.values().find(|s| s.cwd == cwd);
        let own = folder.is_some_and(|f| f.dir.set());
        let colour = if switch.is_some() {
            p().warning
        } else if own {
            p().text
        } else {
            p().muted
        };
        let tip = match (switch, folder) {
            (Some(s), _) => crate::t!("agentbar-config-pending-tip", dir = home_short(&s.to.dir)),
            (None, Some(f)) => crate::t!("agentbar-config-tip", dir = home_short(&f.dir.value), from = f.dir.said()),
            (None, None) => crate::t!("agentbar-config-tip-unread"),
        };
        let toggle = agent.to_string();
        let chip = buttons::bare(SharedString::from("agent-config"), Some("config dir".into()))
            .flex_none()
            .rounded_sm()
            .when(open, |d| d.bg(p().active))
            .child(icons::icon(Icon::ConfigDir, colour, 14.))
            .when(switch.is_some(), |d| d.child(div().pl_0p5().text_color(p().warning).child("⟳")))
            .tip_unless(open, tip)
            .on_click(cx.listener(move |shell, _, _, cx| {
                let opening = shell.config_menu.as_deref() != Some(toggle.as_str());
                shell.config_menu = opening.then(|| toggle.clone());
                shell.config_asked = None;
                if opening {
                    shell.read_config_dirs(cx);
                }
                cx.notify();
            }));
        let popup = open.then(|| self.config_popup(agent, cwd, folder, switch, cx));
        chipbar::anchored(
            chip,
            popup,
            Edge::Right,
            cx.listener(|shell, _: &MouseDownEvent, _, cx| {
                shell.config_menu = None;
                shell.config_asked = None;
                cx.notify();
            }),
        )
        .into_any_element()
    }

    fn config_popup(&self, agent: &str, cwd: &str, folder: Option<&FolderDir>, switch: Option<&Switch>, cx: &mut Context<Self>) -> Stateful<Div> {
        let muted = |text: String| div().text_xs().text_color(p().muted).child(text);
        // A column inside the line: the line itself is laid out as a row
        // that wraps (`chipbar::anchored`).
        let line = |col: Div| chipbar::line("agent-config-bar", p().accent).child(col);
        let mut col = div().flex().flex_col().items_start().gap_1();
        // What it runs on now.
        col = col.child(muted(crate::t!("agentbar-config-here"))).child(match folder {
            Some(f) => div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().max_w(px(POPUP_PATH_MAX)).truncate().font_weight(FontWeight::BOLD).child(home_short(&f.dir.value)))
                .child(muted(f.dir.said())),
            None => div().child("…"),
        });
        // A switch held until the loops are idle: said, and taken back.
        if let Some(s) = switch {
            let cancel = cwd.to_string();
            return line(col
                .child(div().text_color(p().warning).child(crate::t!(
                    if s.running { "agentbar-config-switching" } else { "agentbar-config-pending" },
                    dir = home_short(&s.to.dir)
                )))
                .when(!s.running, |d| {
                    d.child(buttons::secondary("agent-config-takeback", crate::t!("agentbar-config-takeback")).on_click(cx.listener(
                        move |shell, _, _, cx| {
                            shell.config_switches.retain(|_, s| s.cwd != cancel);
                            shell.config_menu = None;
                            cx.notify();
                        },
                    )))
                }));
        }
        let current = folder.map(|f| f.dir.value.clone()).unwrap_or_default();
        let folders: Vec<(String, FolderDir)> = self
            .known_folders()
            .into_iter()
            .filter_map(|(project, cwd)| Some((project, self.config_dirs.get(&cwd)?.clone())))
            .collect();
        let offered = if folder.is_some() { choices(&folders, &current) } else { Vec::new() };
        // A choice asked: said before done.
        if let Some((_, choice)) = self.config_asked.as_ref().filter(|(a, _)| a == agent) {
            let loops = self.folder_loops(cwd);
            let busy = loops.iter().any(|l| self.loop_busy(l));
            let (agent, cwd, choice) = (agent.to_string(), cwd.to_string(), choice.clone());
            return line(col
                .child(div().text_color(p().warning).child(crate::t!(
                    if busy { "agentbar-config-ask-busy" } else { "agentbar-config-ask" },
                    dir = home_short(&choice.dir),
                    loops = loops.len()
                )))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            buttons::answer("agent-config-go", crate::t!("agentbar-config-go"))
                                .tooltip(crate::t!("agentbar-config-go-tip"))
                                .on_click(cx.listener(move |shell, _, _, cx| shell.ask_config_switch(agent.clone(), cwd.clone(), choice.clone(), cx))),
                        )
                        .child(buttons::secondary("agent-config-cancel", crate::t!("agentbar-cancel")).on_click(cx.listener(|shell, _, _, cx| {
                            shell.config_asked = None;
                            cx.notify();
                        }))),
                ));
        }
        col = col.child(muted(crate::t!("agentbar-config-others")));
        if folder.is_some() && offered.is_empty() {
            col = col.child(muted(crate::t!("agentbar-config-none")));
        }
        for (i, choice) in offered.into_iter().enumerate() {
            let label = div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().max_w(px(POPUP_PATH_MAX)).truncate().child(home_short(&choice.dir)))
                .when(choice.default, |d| d.child(div().text_xs().text_color(p().muted).child(crate::t!("agentbar-config-default"))))
                .child(div().text_xs().text_color(p().muted).child(projects_said(&choice.projects)))
                .when(!choice.ready, |d| d.child(div().text_xs().text_color(p().warning).child(crate::t!("agentbar-config-not-ready"))));
            let (all, choice_ready) = (choice.projects.clone(), choice.ready);
            let chip = buttons::chip_if(SharedString::from(format!("agent-config-to-{i}")), label, choice.ready);
            let chip = if choice.ready {
                let agent = agent.to_string();
                chip.on_click(cx.listener(move |shell, _, _, cx| {
                    shell.config_asked = Some((agent.clone(), choice.clone()));
                    cx.notify();
                }))
            } else {
                chip.tip(crate::t!("agentbar-config-not-ready-tip"))
            };
            // Every project on it, when the line names only some.
            let chip = if all.len() > PROJECTS_SHOWN && choice_ready { chip.tip(all.join(", ")) } else { chip };
            col = col.child(chip);
        }
        line(col)
    }

    /// This machine's loops running in `cwd`: those a switch restarts.
    fn folder_loops(&self, cwd: &str) -> Vec<KnownLoop> {
        self.board.known.iter().filter(|l| l.running && l.cwd == cwd).cloned().collect()
    }

    /// Whether a loop's Claude is at work (or starting): a switch waits.
    fn loop_busy(&self, known: &KnownLoop) -> bool {
        known.agent().and_then(|a| self.board.bars.get(a)).is_some_and(|read| !read.stale && read.bar.phase != "idle")
    }

    /// A switch confirmed: held until the folder's loops are idle.
    fn ask_config_switch(&mut self, agent: String, cwd: String, to: Choice, cx: &mut Context<Self>) {
        self.config_asked = None;
        self.config_menu = None;
        self.config_switches.insert(agent, Switch { cwd, to, running: false });
        self.advance_config_switches(cx);
        cx.notify();
    }

    /// The switches whose loops are all idle now start: called at each new
    /// board, which carries the bars.
    pub(super) fn advance_config_switches(&mut self, cx: &mut Context<Self>) {
        if self.config_dirs.is_empty() {
            self.read_config_dirs(cx);
        }
        let ready: Vec<String> = self
            .config_switches
            .iter()
            .filter(|(_, s)| !s.running && !self.folder_loops(&s.cwd).iter().any(|l| self.loop_busy(l)))
            .map(|(agent, _)| agent.clone())
            .collect();
        for agent in ready {
            self.run_config_switch(agent, cx);
        }
    }

    /// Stops the folder's loops, moves its sessions to the new dir as its
    /// setting changes, and starts them again on their conversations.
    fn run_config_switch(&mut self, agent: String, cx: &mut Context<Self>) {
        let Some(switch) = self.config_switches.get_mut(&agent) else { return };
        switch.running = true;
        let (cwd, to) = (switch.cwd.clone(), switch.to.clone());
        let loops = self.folder_loops(&cwd);
        // Their sessions end and come back: opened again, no end screen.
        for known in &loops {
            if let Some(a) = known.agent() {
                self.restarts_asked.insert(a.to_string(), std::time::Instant::now());
            }
        }
        cx.notify();
        let aiball = self.aiball.clone();
        let value = to.value();
        cx.spawn(async move |this, cx| {
            let done = cx.background_executor().spawn(async move { switch_folder(&aiball, &cwd, value, &loops) }).await;
            let _ = this.update(cx, |shell, cx| {
                shell.config_switches.remove(&agent);
                let dir = home_short(&to.dir);
                let activity = match done {
                    Ok(done) => crate::activity::Activity::done(
                        None,
                        crate::t!("agentbar-config-done", dir = dir, loops = done.restarted, moved = done.moved),
                    ),
                    Err(error) => crate::activity::Activity::failed(None, &crate::t!("agentbar-config-failed", dir = dir), format!("{error:#}")),
                };
                crate::activity::publish(cx, activity);
                let _ = shell.refresh_now.unbounded_send(());
                shell.read_config_dirs(cx);
                cx.notify();
            });
        })
        .detach();
    }
}

/// The switch itself, off the UI thread: the loops stopped (each waited
/// for, [`STOP_WAIT`] at most), the setting written with the sessions moved,
/// the loops started again — on the old dir when the write was refused.
fn switch_folder(aiball: &crate::aiball::Aiball, cwd: &str, value: Option<String>, loops: &[KnownLoop]) -> anyhow::Result<Done> {
    // Every loop started again, whatever happened before: on the new dir
    // when the write went through, on the old one otherwise.
    let restart = || -> (usize, Vec<String>) {
        let mut restarted = 0;
        let mut failed = Vec::new();
        for known in loops {
            match crate::loops::restart(aiball, &known.name, None) {
                Ok(_) => restarted += 1,
                Err(error) => failed.push(format!("{}: {error:#}", known.name)),
            }
        }
        (restarted, failed)
    };
    let stopped = stop_all(aiball, loops);
    let written = stopped.and_then(|()| {
        aiball.project_settings_set(cwd, serde_json::json!({ "key": "claude.config_dir", "value": value, "move_sessions": true }))
    });
    let (restarted, failed) = restart();
    let settings = written?;
    if !failed.is_empty() {
        anyhow::bail!(failed.join("; "));
    }
    Ok(Done { restarted, moved: settings.sessions.map_or(0, |s| s.moved) })
}

/// Stops the loops and waits, [`STOP_WAIT`] at most, until none runs: the
/// sessions move only once no Claude writes to them.
fn stop_all(aiball: &crate::aiball::Aiball, loops: &[KnownLoop]) -> anyhow::Result<()> {
    for known in loops {
        crate::loops::stop(aiball, known)?;
    }
    let began = std::time::Instant::now();
    let agents: Vec<&str> = loops.iter().filter_map(KnownLoop::agent).collect();
    loop {
        let running = aiball.running_agents()?;
        if !agents.iter().any(|a| running.iter().any(|r| r == a)) {
            return Ok(());
        }
        if began.elapsed() > STOP_WAIT {
            anyhow::bail!(crate::t!("agentbar-config-still-running"));
        }
        std::thread::sleep(std::time::Duration::from_millis(400));
    }
}

#[cfg(test)]
mod tests {
    use super::{Choice, FolderDir, choices};
    use crate::aiball::{ConfigDirState, Setting};

    fn folder(dir: &str, from: &str, ready: bool) -> FolderDir {
        FolderDir {
            dir: Setting { value: dir.into(), from: from.into() },
            state: Some(ConfigDirState { exists: true, first_run_done: ready, has_account: ready }),
        }
    }

    #[test]
    fn the_dirs_offered_are_the_others_once_each_the_layer_below_first() {
        let folders = vec![
            ("tvty".to_string(), folder("/h/.claude-admi", "file", true)),
            ("admi".to_string(), folder("/h/.claude-admi", "file", true)),
            ("aiball".to_string(), folder("/h/.claude", "default", true)),
            ("mcu".to_string(), folder("/h/.claude-new", "file", false)),
            ("helios".to_string(), folder("/h/.claude", "default", true)),
        ];
        let offered = choices(&folders, "/h/.claude-admi");
        assert_eq!(
            offered,
            vec![
                Choice { dir: "/h/.claude".into(), default: true, projects: vec!["aiball".into(), "helios".into()], ready: true },
                Choice { dir: "/h/.claude-new".into(), default: false, projects: vec!["mcu".into()], ready: false },
            ]
        );
        // The layer below is set back by removing the folder's own dir.
        assert_eq!(offered[0].value(), None);
        assert_eq!(offered[1].value().as_deref(), Some("/h/.claude-new"));
    }

    #[test]
    fn a_dir_used_by_many_projects_names_three_then_counts() {
        let names = |n: usize| (0..n).map(|i| format!("p{i}")).collect::<Vec<_>>();
        assert_eq!(super::projects_said(&names(2)), "p0, p1");
        assert_eq!(super::projects_said(&names(3)), "p0, p1, p2");
        assert_eq!(super::projects_said(&names(30)), "p0, p1, p2 +27");
    }

    #[test]
    fn nothing_to_offer_when_every_folder_runs_on_the_same_dir() {
        let folders = vec![("tvty".to_string(), folder("/h/.claude", "default", true))];
        assert!(choices(&folders, "/h/.claude").is_empty());
    }
}
