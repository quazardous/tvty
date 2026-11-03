//! The agent bar's Claude config dir: an icon after the folder, whose
//! popup names the dir the agent's folder runs on (its account, settings
//! and sessions) and offers the other dirs aiball's folders use. A switch
//! goes through the setting's wizard, which aiball runs: it waits for the
//! loops to be idle, stops them, moves their conversations and starts them
//! again (`super::wizard`).
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
    pub(super) fn value(&self) -> serde_json::Value {
        if self.default { serde_json::Value::Null } else { serde_json::Value::String(self.dir.clone()) }
    }
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
    /// The dirs other folders use, as the wizard offers them: their value
    /// and how they are said.
    pub(super) fn config_dir_suggestions(&self, cwd: &str) -> Vec<(serde_json::Value, String)> {
        let current = self.config_dirs.get(cwd).map(|f| f.dir.value.clone()).unwrap_or_default();
        let folders: Vec<(String, FolderDir)> = self
            .known_folders()
            .into_iter()
            .filter_map(|(project, cwd)| Some((project, self.config_dirs.get(&cwd)?.clone())))
            .collect();
        // A dir never run is not offered: a loop would refuse it.
        choices(&folders, &current)
            .into_iter()
            .filter(|c| c.ready)
            .map(|c| {
                let mut said = format!("{} · {}", home_short(&c.dir), projects_said(&c.projects));
                if c.default {
                    said = format!("{said} · {}", crate::t!("agentbar-config-default"));
                }
                (c.value(), said)
            })
            .collect()
    }

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
        let switch = self.wizard_running(cwd);
        let own = folder.is_some_and(|f| f.dir.set());
        let colour = if switch.is_some() {
            p().warning
        } else if own {
            p().text
        } else {
            p().muted
        };
        let tip = match (&switch, folder) {
            (Some(run), _) => crate::t!("wizard-running-tip", title = crate::wizard::title_said(&run.key, &run.key)),
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
                if opening {
                    shell.read_config_dirs(cx);
                }
                cx.notify();
            }));
        let popup = open.then(|| self.config_popup(cwd, folder, switch.is_some(), cx));
        chipbar::anchored(
            chip,
            popup,
            Edge::Right,
            cx.listener(|shell, _: &MouseDownEvent, _, cx| {
                shell.config_menu = None;
                cx.notify();
            }),
        )
        .into_any_element()
    }

    fn config_popup(&self, cwd: &str, folder: Option<&FolderDir>, running: bool, cx: &mut Context<Self>) -> Stateful<Div> {
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
        // A switch under way: its wizard, followed.
        if running {
            let cwd = cwd.to_string();
            return line(col.child(buttons::secondary("agent-config-follow", crate::t!("wizard-status-running")).on_click(cx.listener(move |shell, _, _, cx| {
                shell.config_menu = None;
                shell.open_wizard(cwd.clone(), "claude.config_dir".into(), serde_json::Value::Null, cx);
            }))));
        }
        let current = folder.map(|f| f.dir.value.clone()).unwrap_or_default();
        let folders: Vec<(String, FolderDir)> = self
            .known_folders()
            .into_iter()
            .filter_map(|(project, cwd)| Some((project, self.config_dirs.get(&cwd)?.clone())))
            .collect();
        let offered = if folder.is_some() { choices(&folders, &current) } else { Vec::new() };
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
                let cwd = cwd.to_string();
                chip.on_click(cx.listener(move |shell, _, _, cx| {
                    shell.config_menu = None;
                    shell.open_wizard(cwd.clone(), "claude.config_dir".into(), choice.value(), cx);
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
        assert_eq!(offered[0].value(), serde_json::Value::Null);
        assert_eq!(offered[1].value().as_str(), Some("/h/.claude-new"));
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
