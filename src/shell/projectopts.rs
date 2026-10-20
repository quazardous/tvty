//! Options scoped to a project: its "Project" page. What its folders'
//! `.aiball.yaml` set — where its loops run, its Claude's Remote Control —
//! as aiball resolves and writes them (`project.settings`,
//! `project.settings_set`: tvty reads no such file), and the keys of the
//! board's config the project overrides.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use serde_json::{Value as Json, json};
use std::collections::HashMap;

use gpui_kit::component::Sizable as _;
use gpui_kit::component::input::{Input, InputEvent, InputState};

use super::{Away, Shell, option_group, option_note, reset_button, setting_frame};
use crate::aiball::{FolderSetting, ProjectSettings, Setting};
use crate::activity::{self, Activity};
use crate::options::Section;
use crate::theme::p;
use crate::ui::Named as _;
use crate::ui::buttons::{self, Look as _};
use crate::ui::combo::{self, Choice, ComboEvent, ComboState};

/// The project the options are scoped to: its folders, each with its
/// settings once aiball answered, and the one shown.
pub(super) struct ProjectOpts {
    project: String,
    folders: Vec<(String, Option<Result<ProjectSettings, String>>)>,
    chosen: usize,
    /// The `path` settings' inputs, by folder and key, with the value they
    /// were last filled with.
    paths: HashMap<(String, String), (Entity<InputState>, String)>,
    /// Loops still on an older value of a setting written here, which take
    /// it once restarted.
    restart: Vec<String>,
    /// Their restart under way.
    restarting: bool,
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
    /// Settings > Appearance > Language, as a dropdown, made as the options
    /// open: the system's, then each language by its own name.
    pub(super) fn new_language_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut choices = vec![Choice::new("auto", crate::t!("setting-language-auto", lang = crate::i18n::name(crate::i18n::system())))];
        choices.extend(crate::i18n::LANGS.iter().map(|(code, name)| Choice::new(*code, *name)));
        let chosen = self.applied.appearance.language.clone().unwrap_or_else(|| "auto".into());
        let state = combo::new(choices, Some(&chosen), window, cx);
        cx.subscribe_in(&state, window, |shell, _, event: &ComboEvent<combo::Choices>, _, cx| {
            let ComboEvent::Confirm(Some(code)) = event else { return };
            let value = (code != "auto").then(|| code.to_string());
            shell.set_pref("appearance.language", tvty_config::Value::Choice(value), cx);
        })
        .detach();
        self.language_select = Some(state);
    }

    /// The scope list, made as the options open: Global and the board's
    /// projects, the scope shown chosen.
    pub(super) fn new_scope_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Global by its key; said in the interface's language.
        let mut names = vec![Choice::new(GLOBAL, crate::t!("projectopts-global"))];
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
        div().px_1().children(crate::inspect::mark_if("options-scope")).children(
            self.scope_select
                .as_ref()
                .map(|state| combo::view(state, "options-scope", &crate::t!("projectopts-global"), &crate::t!("projectopts-a-project")).menu_max_h(px(360.)).w_full()),
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
                self.rename_for(&project, window, cx);
                if self.project_opts.as_ref().is_none_or(|o| o.project != project) {
                    let folders = folders_of(&project, &self.board.homes, &self.board.known);
                    self.project_opts = Some(ProjectOpts {
                        project,
                        folders: folders.into_iter().map(|f| (f, None)).collect(),
                        chosen: 0,
                        paths: HashMap::new(),
                        restart: Vec::new(),
                        restarting: false,
                    });
                }
                self.load_project_settings(window, cx);
            }
            None => {
                self.project_opts = None;
                self.rename = None;
            }
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
    pub(super) fn load_project_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(opts) = self.project_opts.as_ref() else { return };
        let (project, folders) = (opts.project.clone(), opts.folders.iter().map(|(f, _)| f.clone()).collect::<Vec<_>>());
        let aiball = self.aiball.clone();
        cx.spawn_in(window, async move |this, cx| {
            let read = cx
                .background_executor()
                .spawn(async move { folders.into_iter().map(|f| { let s = aiball.project_settings(&f).map_err(|e| format!("{e:#}")); (f, Some(s)) }).collect::<Vec<_>>() })
                .await;
            let _ = this.update_in(cx, |shell, window, cx| {
                if shell.project_opts.as_ref().is_some_and(|o| o.project == project) {
                    shell.fill_paths(&read, window, cx);
                    if let Some(opts) = shell.project_opts.as_mut() {
                        opts.folders = read;
                    }
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Writes one key in the shown folder's `.aiball.yaml` (null: removed),
    /// then reads every folder again (a file may serve several). The loops
    /// that take it only once restarted are kept, to offer their restart.
    fn write_project_setting(&mut self, key: String, value: Json, window: &mut Window, cx: &mut Context<Self>) {
        let Some(opts) = self.project_opts.as_ref() else { return };
        let Some((cwd, _)) = opts.folders.get(opts.chosen) else { return };
        let cwd = cwd.clone();
        let aiball = self.aiball.clone();
        cx.spawn_in(window, async move |this, cx| {
            let done = {
                let key = key.clone();
                cx.background_executor().spawn(async move { aiball.project_settings_set(&cwd, json!({ "key": key, "value": value })) }).await
            };
            let _ = this.update_in(cx, |shell, window, cx| {
                match done {
                    Err(error) => activity::publish(cx, Activity::failed(None, &crate::t!("projectopts-failed"), format!("{key}: {error:#}"))),
                    Ok(settings) => {
                        if let Some(opts) = shell.project_opts.as_mut() {
                            for name in settings.restart_loops {
                                if !opts.restart.contains(&name) {
                                    opts.restart.push(name);
                                }
                            }
                        }
                    }
                }
                shell.load_project_settings(window, cx);
            });
        })
        .detach();
    }

    /// An input for each `path` setting read, filled with its value; one
    /// the user is typing in is left alone until the value changes.
    fn fill_paths(&mut self, read: &[(String, Option<Result<ProjectSettings, String>>)], window: &mut Window, cx: &mut Context<Self>) {
        let mut wanted = Vec::new();
        for (folder, settings) in read {
            let Some(Ok(settings)) = settings else { continue };
            for setting in settings.settings.iter().filter(|s| s.kind == "path") {
                // Its own value only: unset, the input is empty and says the default.
                let value = setting.value.as_str().filter(|_| setting.from == "file").map(home_tilde).unwrap_or_default();
                let default = setting.default.as_str().map(home_tilde).unwrap_or_else(|| "~/.claude".into());
                wanted.push(((folder.clone(), setting.key.clone()), value, default));
            }
        }
        for (at, value, default) in wanted {
            let known = self.project_opts.as_ref().and_then(|o| o.paths.get(&at)).map(|(input, was)| (input.clone(), was.clone()));
            match known {
                Some((_, was)) if was == value => {}
                Some((input, _)) => {
                    input.update(cx, |input, cx| input.set_value(value.clone(), window, cx));
                    if let Some(opts) = self.project_opts.as_mut() {
                        opts.paths.insert(at, (input, value));
                    }
                }
                None => {
                    let input = cx.new(|cx| InputState::new(window, cx).placeholder(default).default_value(value.clone()));
                    let key = at.1.clone();
                    cx.subscribe_in(&input, window, move |shell: &mut Self, input, event: &InputEvent, window, cx| {
                        if matches!(event, InputEvent::PressEnter { .. }) {
                            let typed = input.read(cx).value().trim().to_string();
                            let value = if typed.is_empty() { Json::Null } else { Json::String(typed) };
                            shell.write_project_setting(key.clone(), value, window, cx);
                        }
                    })
                    .detach();
                    if let Some(opts) = self.project_opts.as_mut() {
                        opts.paths.insert(at, (input, value));
                    }
                }
            }
        }
    }

    /// Restarts the loops that still run on an older value of a setting:
    /// each on a new conversation, as the config dir holds the sessions.
    fn restart_project_loops(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(opts) = self.project_opts.as_mut() else { return };
        if opts.restarting || opts.restart.is_empty() {
            return;
        }
        opts.restarting = true;
        let names = opts.restart.clone();
        let aiball = self.aiball.clone();
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let failed = cx
                .background_executor()
                .spawn(async move { names.iter().filter_map(|name| crate::loops::restart(&aiball, name, None).err().map(|e| format!("{name}: {e:#}"))).collect::<Vec<_>>() })
                .await;
            let _ = this.update_in(cx, |shell, window, cx| {
                if let Some(opts) = shell.project_opts.as_mut() {
                    opts.restarting = false;
                    opts.restart.clear();
                }
                let activity = if failed.is_empty() {
                    Activity::done(None, crate::t!("projectopts-restarted"))
                } else {
                    Activity::failed(None, &crate::t!("projectopts-restart-failed"), failed.join("; "))
                };
                activity::publish(cx, activity);
                shell.load_project_settings(window, cx);
            });
        })
        .detach();
    }

    /// The Project page's groups, for the tree.
    pub(super) fn project_groups(&self) -> Vec<SharedString> {
        vec!["Name".into(), "Folder".into(), "Board".into()]
    }

    /// The Project page: its name, the folder's settings, then the board's
    /// keys the project overrides.
    pub(super) fn project_sections(&self, cx: &mut Context<Self>) -> Vec<(Option<SharedString>, AnyElement)> {
        let Some(opts) = self.project_opts.as_ref() else {
            return vec![(None, option_note(crate::t!("projectopts-choose")).into_any_element())];
        };
        vec![
            (Some("Name".into()), self.rename_section(cx).into_any_element()),
            (Some("Folder".into()), self.folder_section(opts, cx).into_any_element()),
            (Some("Board".into()), self.board_section(opts, cx).into_any_element()),
        ]
    }

    fn folder_section(&self, opts: &ProjectOpts, cx: &mut Context<Self>) -> Div {
        let mut out = div().flex().flex_col().gap_2().max_w(px(720.)).child(option_group("Folder"));
        if opts.folders.is_empty() {
            return out.child(option_note(crate::t!("projectopts-no-folder")));
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
            None => return out.child(div().text_sm().text_color(p().muted).child(crate::t!("projectopts-asking"))),
            Some(Err(error)) => return out.child(div().text_sm().text_color(p().danger).child(crate::t!("projectopts-could-not-say", error = error.to_string()))),
            Some(Ok(settings)) => settings,
        };
        let Some(file) = settings.file.as_ref() else {
            return out.child(div().text_sm().text_color(p().muted).child(crate::t!("projectopts-no-file", folder = super::loopstabs::home_short(folder))));
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
        out = out.child(div().text_sm().text_color(p().muted).child(crate::t!("projectopts-written-in", file = super::loopstabs::home_short(file))));
        if !shared.is_empty() {
            out = out.child(div().text_sm().text_color(p().warning).child(crate::t!("projectopts-also-serves", others = shared.join(", "))));
        }
        // Each setting as aiball describes it.
        for setting in &settings.settings {
            out = out.child(self.folder_row(setting, cx));
        }
        if !opts.restart.is_empty() {
            let count = opts.restart.len();
            out = out.child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .text_sm()
                    .text_color(p().warning)
                    .child(div().flex_1().min_w_0().child(crate::t!("projectopts-restart-needed", loops = opts.restart.join(", "), count = count)))
                    .child(if opts.restarting {
                        div().text_color(p().muted).child(crate::t!("projectopts-restarting")).into_any_element()
                    } else {
                        buttons::chip("options-folder-restart", crate::t!("projectopts-restart", count = count))
                            .py_0p5()
                            .on_click(cx.listener(|shell, _, window, cx| shell.restart_project_loops(window, cx)))
                            .into_any_element()
                    }),
            );
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
                    .when(!chosen, |d| d.on_click(cx.listener(move |shell, _, window, cx| shell.write_project_setting(key.clone(), wanted.clone(), window, cx)))),
            );
        };
        let mut field = None;
        match setting.kind.as_str() {
            "path" => {
                let at = (self.project_opts.as_ref().and_then(|o| o.folders.get(o.chosen)).map(|(f, _)| f.clone()).unwrap_or_default(), setting.key.clone());
                field = self.project_opts.as_ref().and_then(|o| o.paths.get(&at)).map(|(input, _)| {
                    div().named(SharedString::from(format!("options-folder-{}", setting.key))).w(px(280.)).flex_none().child(Input::new(input).small())
                });
            }
            "boolean_or_name" => {
                let named = setting.value.as_str().filter(|n| !n.is_empty()).map(str::to_string);
                let on = setting.value.as_bool() == Some(true) || named.is_some();
                choice(format!("{}-off", setting.key), crate::t!("settings-off"), !on, Json::Bool(false));
                choice(format!("{}-on", setting.key), named.map_or(crate::t!("settings-on"), |n| crate::t!("projectopts-on-named", name = n)), on, Json::Bool(true));
            }
            _ => {
                for option in &setting.options {
                    choice(format!("{}-{option}", setting.key), option.clone(), setting.value.as_str() == Some(option.as_str()), Json::String(option.clone()));
                }
            }
        }
        let reset = {
            let key = setting.key.clone();
            cx.listener(move |shell, _, window, cx| shell.write_project_setting(key.clone(), Json::Null, window, cx))
        };
        setting_frame(key.clone(), label(&setting.label, &setting.from), setting.description.clone().into(), away.as_ref())
            .child(div().flex().flex_wrap().gap_1().flex_none().children(chips).children(field))
            .child(reset_button(&key, away.as_ref(), reset))
    }

    /// The board's config keys this project sets over the board's values.
    fn board_section(&self, opts: &ProjectOpts, cx: &mut Context<Self>) -> Div {
        let mut out = div().flex().flex_col().gap_2().max_w(px(720.)).child(option_group("Board"));
        let config = self.remote.as_ref().filter(|c| c.project.as_deref() == Some(opts.project.as_str()));
        let Some(config) = config else {
            return out.child(div().text_sm().text_color(p().muted).child(crate::t!("settings-aiball-reading")));
        };
        let items = crate::options::remote_items(config);
        let set: Vec<usize> = items.iter().enumerate().filter(|(_, i)| i.modified && !i.inherited).map(|(at, _)| at).collect();
        let note = if set.is_empty() {
            crate::t!("projectopts-none-set", project = opts.project.clone())
        } else {
            crate::t!("settings-aiball-project", project = opts.project.clone())
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
                    buttons::chip("options-project-all", crate::t!("projectopts-all-keys"))
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

/// A path as typed: the home folder as `~`.
fn home_tilde(path: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && (path == home || path.starts_with(&format!("{home}/"))) => format!("~{}", &path[home.len()..]),
        _ => path.to_string(),
    }
}

/// A setting's value as said: `off`, `on`, a name, an option.
fn value_said(value: &Json) -> String {
    match value {
        Json::Bool(true) => crate::t!("settings-on"),
        Json::Bool(false) | Json::Null => crate::t!("settings-off"),
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
