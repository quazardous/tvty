//! Options scoped to a project: its "Project" page. What its folders'
//! `.aiball.yaml` set — where its loops run, its Claude's Remote Control —
//! as aiball resolves and writes them (`project.settings`,
//! `project.settings_set`: tvty reads no such file), and the keys of the
//! board's config the project overrides.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use serde_json::{Value as Json, json};

use super::{Away, Shell, option_group, option_note, reset_button, setting_frame};
use crate::aiball::{FolderSetting, ProjectSettings, Setting};
use crate::activity::{self, Activity};
use crate::options::Section;
use crate::theme::p;
use crate::ui::buttons::{self, Look as _};
use crate::ui::combo::{self, Choice, ComboEvent, ComboState};

/// The project the options are scoped to: its folders, each with its
/// settings once aiball answered, and the one shown.
pub(super) struct ProjectOpts {
    project: String,
    folders: Vec<(String, Option<Result<ProjectSettings, String>>)>,
    chosen: usize,
}

/// The scope list atop the options: Global, then the board's projects,
/// typed to be found.
pub(super) type ScopeSelect = ComboState;

/// The scope that is not a project.
const GLOBAL: &str = "Global";

/// A project's folders, as aiball knows its agents and loops: each once.
fn folders_of(project: &str, homes: &[(String, Option<String>, String)], known: &[crate::loops::KnownLoop]) -> Vec<String> {
    let mut folders: Vec<String> = Vec::new();
    let homes = homes.iter().filter(|(_, p, _)| p.as_deref() == Some(project)).map(|(_, _, cwd)| cwd);
    let loops = known.iter().filter(|l| l.project.as_deref() == Some(project)).map(|l| &l.cwd);
    for cwd in homes.chain(loops) {
        if !folders.contains(cwd) {
            folders.push(cwd.clone());
        }
    }
    folders.sort();
    folders
}

/// Several folders as told apart: each past the folder they share (the
/// lead's `app`, a crew's `app-crew`, their parent's name dropped); one, or
/// none shared, from home.
fn short_folders(folders: &[String]) -> Vec<String> {
    let parts: Vec<Vec<&str>> = folders.iter().map(|f| f.split('/').collect()).collect();
    let shared = match parts.split_first() {
        Some((first, rest)) if !rest.is_empty() => {
            (0..first.len()).take_while(|&at| rest.iter().all(|p| p.len() > at + 1 && p[at] == first[at]) && first.len() > at + 1).count()
        }
        _ => 0,
    };
    if shared <= 1 {
        return folders.iter().map(|f| super::loopstabs::home_short(f)).collect();
    }
    parts.iter().map(|p| p[shared..].join("/")).collect()
}

impl Shell {
    /// The scope list, made as the options open: Global and the board's
    /// projects, the scope shown chosen.
    pub(super) fn new_scope_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut names = vec![Choice::plain(GLOBAL)];
        names.extend(self.board.projects.iter().filter(|p| p.on_board).map(|p| Choice::plain(p.name.clone())));
        let shown = self.remote_layer.clone().unwrap_or_else(|| GLOBAL.into());
        let state = combo::new(names, Some(&shown), window, cx);
        cx.subscribe_in(&state, window, |shell, _, event: &ComboEvent<combo::Choices>, window, cx| {
            let ComboEvent::Confirm(Some(name)) = event else { return };
            let project = (name != GLOBAL).then(|| name.to_string());
            shell.scope_options(project, window, cx);
        })
        .detach();
        self.scope_select = Some(state);
    }

    /// The scope list, atop the options.
    pub(super) fn options_scope(&self) -> impl IntoElement + use<> {
        div().px_1().children(
            self.scope_select
                .as_ref()
                .map(|state| combo::view(state, "options-scope", GLOBAL, "a project…").menu_max_h(px(360.)).w_full()),
        )
    }

    /// The options scoped to `project` (none: Global); the Project page
    /// comes and goes with it.
    pub(super) fn scope_options(&mut self, project: Option<String>, window: &mut Window, cx: &mut Context<Self>) {
        let to_project = project.is_some();
        // The list says it, wherever the scope was chosen (a project's ⚙).
        if let Some(state) = self.scope_select.clone() {
            let name = project.clone().unwrap_or_else(|| GLOBAL.into());
            state.update(cx, |state, cx| state.set_selected_value(&name, window, cx));
        }
        self.show_layer(project.clone(), cx);
        match project {
            Some(project) => {
                if self.project_opts.as_ref().is_none_or(|o| o.project != project) {
                    let folders = folders_of(&project, &self.board.homes, &self.board.known);
                    self.project_opts = Some(ProjectOpts { project, folders: folders.into_iter().map(|f| (f, None)).collect(), chosen: 0 });
                }
                self.load_project_settings(cx);
            }
            None => self.project_opts = None,
        }
        // Scoped to a project: its page first; back to Global, off it.
        match (to_project, self.options) {
            (true, Some(Section::Aiball)) => {}
            (true, _) => self.options = Some(Section::Project),
            (false, Some(Section::Project)) => self.options = Some(Section::Appearance),
            _ => {}
        }
        self.options_scroll.set_offset(point(px(0.), px(0.)));
        cx.notify();
    }

    /// Options opened on a project's page (its ⚙ in the sessions list).
    pub(super) fn open_project_options(&mut self, project: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.options.is_none() {
            self.toggle_options(window, cx);
        }
        self.scope_options(Some(project), window, cx);
        self.open_options_page(Section::Project, None, window, cx);
    }

    /// Asks aiball how each of the project's folders is configured.
    pub(super) fn load_project_settings(&mut self, cx: &mut Context<Self>) {
        let Some(opts) = self.project_opts.as_ref() else { return };
        let (project, folders) = (opts.project.clone(), opts.folders.iter().map(|(f, _)| f.clone()).collect::<Vec<_>>());
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let read = cx
                .background_executor()
                .spawn(async move { folders.into_iter().map(|f| { let s = aiball.project_settings(&f).map_err(|e| format!("{e:#}")); (f, Some(s)) }).collect::<Vec<_>>() })
                .await;
            let _ = this.update(cx, |shell, cx| {
                if let Some(opts) = shell.project_opts.as_mut().filter(|o| o.project == project) {
                    opts.folders = read;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Writes one key in the shown folder's `.aiball.yaml` (null: removed),
    /// then reads every folder again (a file may serve several).
    fn write_project_setting(&mut self, key: String, value: Json, cx: &mut Context<Self>) {
        let Some(opts) = self.project_opts.as_ref() else { return };
        let Some((cwd, _)) = opts.folders.get(opts.chosen) else { return };
        let cwd = cwd.clone();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let done = {
                let key = key.clone();
                cx.background_executor().spawn(async move { aiball.project_settings_set(&cwd, json!({ "key": key, "value": value })) }).await
            };
            let _ = this.update(cx, |shell, cx| {
                if let Err(error) = done {
                    activity::publish(cx, Activity::failed(None, "project settings", format!("{key}: {error:#}")));
                }
                shell.load_project_settings(cx);
            });
        })
        .detach();
    }

    /// The Project page's groups, for the tree.
    pub(super) fn project_groups(&self) -> Vec<SharedString> {
        vec!["Folder".into(), "Board".into()]
    }

    /// The Project page: the folder's settings, then the board's keys the
    /// project overrides.
    pub(super) fn project_sections(&self, cx: &mut Context<Self>) -> Vec<(Option<SharedString>, AnyElement)> {
        let Some(opts) = self.project_opts.as_ref() else {
            return vec![(None, option_note("Choose a project above.").into_any_element())];
        };
        vec![(Some("Folder".into()), self.folder_section(opts, cx).into_any_element()), (Some("Board".into()), self.board_section(opts, cx).into_any_element())]
    }

    fn folder_section(&self, opts: &ProjectOpts, cx: &mut Context<Self>) -> Div {
        let mut out = div().flex().flex_col().gap_2().max_w(px(720.)).child(option_group("Folder"));
        if opts.folders.is_empty() {
            return out.child(option_note("aiball knows no folder of this project on this machine: none of its agents works here."));
        }
        let names = short_folders(&opts.folders.iter().map(|(f, _)| f.clone()).collect::<Vec<_>>());
        // Several folders (the lead's, a crew's worktree…): one at a time.
        if opts.folders.len() > 1 {
            let mut chooser = div().flex().flex_wrap().gap_1();
            for (at, name) in names.iter().enumerate() {
                chooser = chooser.child(
                    buttons::chip(SharedString::from(format!("options-folder-{at}")), name.clone())
                        .py_0p5()
                        .text_sm()
                        .chosen(at == opts.chosen)
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            if let Some(opts) = shell.project_opts.as_mut() {
                                opts.chosen = at;
                            }
                            cx.notify();
                        })),
                );
            }
            out = out.child(chooser);
        }
        let Some((folder, settings)) = opts.folders.get(opts.chosen) else { return out };
        let settings = match settings {
            None => return out.child(div().text_sm().text_color(p().muted).child("Asking aiball…")),
            Some(Err(error)) => return out.child(div().text_sm().text_color(p().danger).child(format!("aiball could not say: {error}"))),
            Some(Ok(settings)) => settings,
        };
        let Some(file) = settings.file.as_ref() else {
            return out.child(div().text_sm().text_color(p().muted).child(format!(
                "{} has no .aiball.yaml, nor any folder above it: aiball's defaults apply. New project… sets it up.",
                super::loopstabs::home_short(folder)
            )));
        };
        // The file written, and the other folders it serves: a change here
        // is theirs too.
        let shared: Vec<String> = opts
            .folders
            .iter()
            .zip(&names)
            .filter(|((f, s), _)| f != folder && matches!(s, Some(Ok(s)) if s.file.as_ref() == Some(file)))
            .map(|(_, name)| name.clone())
            .collect();
        out = out.child(div().text_sm().text_color(p().muted).child(format!("Written in {}.", super::loopstabs::home_short(file))));
        if !shared.is_empty() {
            out = out.child(div().text_sm().text_color(p().warning).child(format!("It also serves {}: a change here is theirs too.", shared.join(", "))));
        }
        // Each setting as aiball describes it.
        for setting in &settings.settings {
            out = out.child(self.folder_row(setting, cx));
        }
        out
    }

    /// One folder setting, drawn from its description: an `enum` as its
    /// choices, a `boolean_or_name` as off / on (on with a name: that name).
    /// Set in the file: ↺ removes it, back to the layer below.
    fn folder_row(&self, setting: &FolderSetting, cx: &mut Context<Self>) -> Div {
        let key: SharedString = setting.key.clone().into();
        let away = (setting.from == "file").then(|| Away::default(value_said(&setting.default)));
        let mut chips = Vec::new();
        let mut choice = |id: String, text: String, chosen: bool, wanted: Json| {
            let key = setting.key.clone();
            chips.push(
                buttons::chip(SharedString::from(format!("options-folder-{id}")), text)
                    .py_0p5()
                    .text_sm()
                    .chosen(chosen)
                    .when(!chosen, |d| d.on_click(cx.listener(move |shell, _, _, cx| shell.write_project_setting(key.clone(), wanted.clone(), cx)))),
            );
        };
        match setting.kind.as_str() {
            "boolean_or_name" => {
                let named = setting.value.as_str().filter(|n| !n.is_empty()).map(str::to_string);
                let on = setting.value.as_bool() == Some(true) || named.is_some();
                choice(format!("{}-off", setting.key), "off".into(), !on, Json::Bool(false));
                choice(format!("{}-on", setting.key), named.map_or("on".into(), |n| format!("on: {n}")), on, Json::Bool(true));
            }
            _ => {
                for option in &setting.options {
                    choice(format!("{}-{option}", setting.key), option.clone(), setting.value.as_str() == Some(option.as_str()), Json::String(option.clone()));
                }
            }
        }
        let reset = {
            let key = setting.key.clone();
            cx.listener(move |shell, _, _, cx| shell.write_project_setting(key.clone(), Json::Null, cx))
        };
        setting_frame(key.clone(), label(&setting.label, &setting.from), setting.description.clone().into(), away.as_ref())
            .child(div().flex().flex_wrap().gap_1().flex_none().children(chips))
            .child(reset_button(&key, away.as_ref(), reset))
    }

    /// The board's config keys this project sets over the board's values.
    fn board_section(&self, opts: &ProjectOpts, cx: &mut Context<Self>) -> Div {
        let mut out = div().flex().flex_col().gap_2().max_w(px(720.)).child(option_group("Board"));
        let config = self.remote.as_ref().filter(|c| c.project.as_deref() == Some(opts.project.as_str()));
        let Some(config) = config else {
            return out.child(div().text_sm().text_color(p().muted).child("Reading aiball's config…"));
        };
        let items = crate::options::remote_items(config);
        let set: Vec<usize> = items.iter().enumerate().filter(|(_, i)| i.modified && !i.inherited).map(|(at, _)| at).collect();
        let note = if set.is_empty() {
            format!("{} sets none of the board's config: it has the board's values.", opts.project)
        } else {
            format!("What {} sets over the board's config; ↺ gives a key back to the board's value.", opts.project)
        };
        out = out.child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .text_sm()
                .text_color(p().muted)
                .child(div().flex_1().min_w_0().child(note))
                .child(
                    buttons::chip("options-project-all", "All its keys")
                        .py_0p5()
                        .border_color(p().accent.opacity(0.6))
                        .text_color(p().accent)
                        .on_click(cx.listener(|shell, _, window, cx| shell.open_options_page(Section::Aiball, None, window, cx))),
                ),
        );
        for at in set {
            out = out.child(self.remote_row(&config.config[at], &items[at], true, &[], cx));
        }
        out
    }
}

/// A setting's name and where its value comes from.
fn label(name: &str, from: &str) -> Div {
    let from = Setting { value: (), from: from.to_string() };
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(name.to_string())
        .child(div().text_xs().font_weight(FontWeight::NORMAL).text_color(if from.set() { crate::theme::imported() } else { p().muted }).child(from.said().to_string()))
}

/// A setting's value as said: `off`, `on`, a name, an option.
fn value_said(value: &Json) -> String {
    match value {
        Json::Bool(true) => "on".into(),
        Json::Bool(false) | Json::Null => "off".into(),
        Json::String(text) => text.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{folders_of, short_folders};
    use crate::loops::KnownLoop;

    #[test]
    fn a_projects_folders_are_its_agents_and_its_loops_each_once() {
        let homes = vec![
            ("a-claude".to_string(), Some("a".to_string()), "/w/a".to_string()),
            ("a-crew".to_string(), Some("a".to_string()), "/w/a-crew".to_string()),
            ("b-claude".to_string(), Some("b".to_string()), "/w/b".to_string()),
        ];
        let known = vec![KnownLoop { name: "cl-a".into(), cwd: "/w/a".into(), project: Some("a".into()), ..Default::default() }];
        assert_eq!(folders_of("a", &homes, &known), vec!["/w/a", "/w/a-crew"]);
        assert!(folders_of("c", &homes, &known).is_empty());
    }

    #[test]
    fn folders_are_told_apart_by_what_differs() {
        let two = vec!["/w/demo/crew".to_string(), "/w/demo/lead".to_string()];
        assert_eq!(short_folders(&two), vec!["crew", "lead"]);
        let nested = vec!["/w/app".to_string(), "/w/app/sub".to_string()];
        assert_eq!(short_folders(&nested), vec!["app", "app/sub"]);
        // One: said in full (from home).
        assert_eq!(short_folders(&["/w/app".to_string()]), vec!["/w/app"]);
    }
}
