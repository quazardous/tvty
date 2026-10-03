//! "New project": a folder made an aiball project, the way `aiball init`
//! does it — tvty runs aiball's command, it does not copy what it does (nor
//! goes through claude-loop, the terminal's side) —
//! then its first session started. Four steps in one frame of one size:
//! the folder, who works in it, the outcome, what is left to do (accept
//! aiball's MCP server in Claude Code). Nothing is written before "Set it
//! up".

use std::path::{Path, PathBuf};

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::stepper::{Stepper, StepperItem};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::component::Disableable as _;
use gpui_kit::component::button::Button;
use gpui_kit::*;

use super::Shell;
use crate::aiball::{InitAsk, InitDone, ProjectSettings, Setting};
use crate::loops::Start;
use crate::theme::p;
use crate::ui::buttons;

#[derive(Clone, Copy, PartialEq)]
enum Step {
    Folder,
    Identity,
    Done,
    Next,
}

impl Step {
    const ALL: [Step; 4] = [Step::Folder, Step::Identity, Step::Done, Step::Next];

    fn index(self) -> usize {
        Step::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    fn title(self) -> String {
        crate::t!(match self {
            Step::Folder => "newproject-step-folder",
            Step::Identity => "newproject-step-identity",
            Step::Done => "newproject-step-done",
            Step::Next => "newproject-step-next",
        })
    }
}

/// The wizard, while it is open.
pub(super) struct NewProject {
    step: Step,
    folder: Entity<InputState>,
    name: Entity<InputState>,
    agent: Entity<InputState>,
    crew: bool,
    private: bool,
    no_claim: bool,
    /// Where its loops run: on aiball's host, else in tmux.
    on_host: bool,
    /// Its Claude's Remote Control.
    remote_control: bool,
    running: bool,
    /// What aiball would do with the choices as they stand (a dry run),
    /// for the `asked`-th of them: or why it would not.
    preview: Option<Result<InitDone, String>>,
    asked: u64,
    /// What aiball did: or why it did not.
    outcome: Option<Result<InitDone, String>>,
    /// The folder aiball was last asked about, and its configuration as
    /// aiball resolves it (tvty reads no `.aiball.yaml` itself).
    found: Option<(PathBuf, ProjectSettings)>,
    /// That configuration, as the choices were filled from it: a choice
    /// still as it says wears the "imported" colour.
    imported: Option<ProjectSettings>,
}

/// What the folder typed is, for aiball.
#[derive(Debug, PartialEq)]
enum Folder {
    Empty,
    Missing,
    NotADirectory,
    /// aiball not asked yet, or not answered.
    Checking,
    /// Already set up: the `.aiball.yaml` that applies (aiball's answer).
    Configured(String),
    /// Ready to be one; `git`: a git repository.
    Fresh { git: bool },
}

fn folder_of(typed: &str, found: Option<&(PathBuf, ProjectSettings)>) -> Folder {
    let typed = typed.trim();
    if typed.is_empty() {
        return Folder::Empty;
    }
    let path = expand(typed);
    if !path.exists() {
        return Folder::Missing;
    }
    if !path.is_dir() {
        return Folder::NotADirectory;
    }
    match found.filter(|(asked, _)| *asked == path) {
        Some((_, ProjectSettings { file: Some(file), .. })) => Folder::Configured(file.clone()),
        Some(_) => Folder::Fresh { git: path.join(".git").exists() },
        None => Folder::Checking,
    }
}

/// `~/…` as the home's.
fn expand(typed: &str) -> PathBuf {
    match (typed.strip_prefix("~/").or_else(|| typed.strip_prefix("~\\")), tvty_config::home()) {
        (Some(rest), Some(home)) => Path::new(&home).join(rest),
        _ => PathBuf::from(typed),
    }
}

/// A project's or an agent's name as aiball takes it: letters, digits,
/// `-`, `_` and `.`.
fn name_ok(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// A name for the folder's project: its own name, made acceptable.
fn name_from(folder: &Path) -> String {
    folder
        .file_name()
        .map(|n| n.to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect())
        .unwrap_or_default()
}

/// The choices a folder's configuration fills in: what it sets (its
/// `.aiball.yaml`, the machine's config), else names from the folder
/// (`fallback`) and aiball's defaults.
#[derive(Debug, PartialEq)]
struct Filled {
    project: String,
    agent: String,
    crew: bool,
    on_host: bool,
    remote_control: bool,
}

impl Filled {
    fn of(settings: &ProjectSettings, fallback: &str) -> Filled {
        let consumer = &settings.consumer;
        let project = if consumer.project.set() { consumer.project.value.clone() } else { fallback.to_string() };
        let agent = if consumer.agent.set() { consumer.agent.value.clone() } else { format!("{project}-claude") };
        Filled {
            crew: consumer.role.value.as_deref() == Some("crew"),
            on_host: settings.session.value != "tmux",
            remote_control: remote_control_on(&settings.remote_control),
            project,
            agent,
        }
    }
}

/// Remote Control on: `true`, or the name its Claude is found by.
fn remote_control_on(setting: &Setting<serde_json::Value>) -> bool {
    match &setting.value {
        serde_json::Value::Bool(on) => *on,
        serde_json::Value::String(name) => !name.is_empty(),
        _ => false,
    }
}

/// The Remote Control chosen, as it goes in the file: as imported while its
/// tick is (a name stays a name), else on or off.
fn remote_control_of(imported: &ProjectSettings, on: bool) -> serde_json::Value {
    if on == remote_control_on(&imported.remote_control) && imported.remote_control.set() {
        imported.remote_control.value.clone()
    } else {
        serde_json::Value::Bool(on)
    }
}

/// What to write in the folder's `.aiball.yaml` once it is set up: where
/// its loops run and its Remote Control, only where the choice differs from
/// what aiball resolves there (`now`: after the set up, which may have
/// written a file of the folder's own, hiding a parent's).
fn to_write(now: &ProjectSettings, on_host: bool, remote_control: serde_json::Value) -> (Option<&'static str>, Option<serde_json::Value>) {
    let session = (on_host != (now.session.value != "tmux")).then_some(if on_host { "host" } else { "tmux" });
    let remote_control = (remote_control != now.remote_control.value).then_some(remote_control);
    (session, remote_control)
}

/// What aiball's dry run says it did to a file, as what it will do.
fn will(action: &str) -> String {
    match action {
        "created" | "added" | "rewritten" | "patched" | "overwrote" | "kept" => crate::t!(&format!("newproject-will-{action}")),
        other => other.to_string(),
    }
}

impl Shell {
    /// Opens the wizard, at its first step.
    pub(super) fn open_new_project(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.help_menu = false;
        let folder = cx.new(|cx| InputState::new(window, cx).placeholder(crate::t!("newproject-folder-placeholder")));
        let name = cx.new(|cx| InputState::new(window, cx).placeholder(crate::t!("newproject-project-placeholder")));
        let agent = cx.new(|cx| InputState::new(window, cx).placeholder(crate::t!("newproject-agent-placeholder")));
        // What the folder is, said as it is typed; the names, aiball
        // asked again what it would do with them.
        for input in [&folder, &name, &agent] {
            cx.subscribe(input, |shell, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    match shell.new_project.as_ref().map(|w| w.step) {
                        Some(Step::Identity) => shell.preview_init(cx),
                        Some(Step::Folder) => shell.ask_folder(cx),
                        _ => {}
                    }
                    cx.notify();
                }
            })
            .detach();
        }
        let focus = folder.read(cx).focus_handle(cx);
        window.focus(&focus, cx);
        self.new_project = Some(NewProject {
            step: Step::Folder,
            folder,
            name,
            agent,
            crew: false,
            private: false,
            no_claim: false,
            on_host: true,
            remote_control: false,
            running: false,
            preview: None,
            asked: 0,
            outcome: None,
            found: None,
            imported: None,
        });
        cx.notify();
    }

    pub(super) fn close_new_project(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.new_project = None;
        self.focus_terminal(window, cx);
        cx.notify();
    }

    /// The system's folder picker; the folder chosen goes in the box.
    fn browse_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(crate::t!("newproject-choose-folder").into()),
        });
        let handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = chosen.await else { return };
            let Some(path) = paths.into_iter().next() else { return };
            let _ = cx.update_window(handle, |_, window, cx| {
                let _ = this.update(cx, |shell, cx| {
                    if let Some(wizard) = shell.new_project.as_ref() {
                        wizard.folder.update(cx, |f, cx| f.set_value(path.display().to_string(), window, cx));
                    }
                    shell.ask_folder(cx);
                    cx.notify();
                });
            });
        })
        .detach();
    }

    /// Asks aiball how the folder typed is configured (which `.aiball.yaml`
    /// applies, if any, and what it says); the answer for the folder still
    /// typed only is kept.
    fn ask_folder(&mut self, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_ref() else { return };
        let path = expand(wizard.folder.read(cx).value().trim());
        if !path.is_dir() || wizard.found.as_ref().is_some_and(|(asked, _)| *asked == path) {
            return;
        }
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let cwd = path.display().to_string();
            let settings = cx.background_executor().spawn(async move { aiball.project_settings(&cwd) }).await;
            let _ = this.update(cx, |shell, cx| {
                let Some(wizard) = shell.new_project.as_mut() else { return };
                if expand(wizard.folder.read(cx).value().trim()) != path {
                    return;
                }
                match settings {
                    Ok(settings) => wizard.found = Some((path, settings)),
                    Err(error) => log::warn!("new project: {} {error:#}", path.display()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// From the folder to who works in it: the choices filled from the
    /// folder's configuration (what its `.aiball.yaml` says, else aiball's
    /// defaults, else names from the folder), unless already made for this
    /// folder.
    fn to_identity(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_mut() else { return };
        let typed = wizard.folder.read(cx).value().to_string();
        if !matches!(folder_of(&typed, wizard.found.as_ref()), Folder::Configured(_) | Folder::Fresh { .. }) {
            return;
        }
        let settings = wizard.found.as_ref().map(|(_, s)| s.clone()).unwrap_or_default();
        // The same folder again (back, then next): the choices made stay.
        if wizard.imported.as_ref() != Some(&settings) {
            let fallback = name_from(&expand(typed.trim()));
            let filled = Filled::of(&settings, &fallback);
            wizard.name.update(cx, |n, cx| n.set_value(filled.project.clone(), window, cx));
            wizard.agent.update(cx, |a, cx| a.set_value(filled.agent.clone(), window, cx));
            wizard.crew = filled.crew;
            wizard.on_host = filled.on_host;
            wizard.remote_control = filled.remote_control;
            wizard.imported = Some(settings);
        }
        wizard.step = Step::Identity;
        self.preview_init(cx);
        cx.notify();
    }

    /// The choices as they stand, for aiball.
    fn init_ask(wizard: &NewProject, cx: &App) -> InitAsk {
        InitAsk {
            cwd: expand(wizard.folder.read(cx).value().trim()).display().to_string(),
            project: wizard.name.read(cx).value().trim().to_string(),
            agent: wizard.agent.read(cx).value().trim().to_string(),
            crew: wizard.crew,
            private: wizard.private,
            no_claim: wizard.no_claim,
        }
    }

    /// Asks aiball what it would do with the choices (a dry run: nothing
    /// written); the latest ask's answer only is kept.
    fn preview_init(&mut self, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_mut() else { return };
        let ask = Self::init_ask(wizard, cx);
        wizard.asked += 1;
        let asked = wizard.asked;
        if !name_ok(&ask.project) || !name_ok(&ask.agent) {
            wizard.preview = None;
            return;
        }
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let said = cx.background_executor().spawn(async move { aiball.project_init(&ask, true).map_err(|e| format!("{e:#}")) }).await;
            let _ = this.update(cx, |shell, cx| {
                if let Some(wizard) = shell.new_project.as_mut().filter(|w| w.asked == asked) {
                    wizard.preview = Some(said);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Has aiball set the folder up, with the choices made.
    fn set_it_up(&mut self, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_mut() else { return };
        if wizard.running {
            return;
        }
        let ask = Self::init_ask(wizard, cx);
        let (on_host, remote_control) = (wizard.on_host, remote_control_of(&wizard.imported.clone().unwrap_or_default(), wizard.remote_control));
        wizard.running = true;
        cx.notify();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let said = cx
                .background_executor()
                .spawn(async move {
                    let mut done = aiball.project_init(&ask, false).map_err(|e| format!("{e:#}"))?;
                    // Then where its loops run and its Remote Control, in the
                    // file that now applies: only what differs from it.
                    let unsaved = |e: anyhow::Error| crate::t!("newproject-unsaved", error = format!("{e:#}"));
                    let now = aiball.project_settings(&ask.cwd).map_err(unsaved)?;
                    let (session, remote_control) = to_write(&now, on_host, remote_control);
                    if session.is_some() || remote_control.is_some() {
                        let mut patch = serde_json::json!({});
                        if let Some(session) = session {
                            patch["session"] = serde_json::json!(session);
                        }
                        if let Some(remote_control) = &remote_control {
                            patch["remote_control"] = remote_control.clone();
                        }
                        aiball.project_settings_set(&ask.cwd, patch).map_err(unsaved)?;
                        // Said with what aiball did.
                        let mut set = Vec::new();
                        set.extend(session.map(|s| format!("claude_loop.session: {s}")));
                        set.extend(remote_control.map(|rc| format!("claude.remote_control: {rc}")));
                        let file = now.file.clone().unwrap_or_default();
                        done.steps.push(crate::aiball::InitStep { message: crate::t!("newproject-set-in", what = set.join(", "), file = file.clone()), action: "patched".into(), file });
                    }
                    Ok(done)
                })
                .await;
            let _ = this.update(cx, |shell, cx| {
                if let Some(wizard) = shell.new_project.as_mut() {
                    wizard.running = false;
                    wizard.outcome = Some(said);
                    wizard.step = Step::Done;
                }
                let _ = shell.refresh_now.unbounded_send(());
                cx.notify();
            });
        })
        .detach();
    }

    /// Its first session, where the folder now says; the wizard closes.
    fn start_first_session(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_ref() else { return };
        // Its agent runs already in this folder: that session, opened.
        if let Some((session, _)) = self.running_session_in(&expand(wizard.folder.read(cx).value().trim())) {
            self.close_new_project(window, cx);
            self.select(session, window, cx);
            return;
        }
        let start = Start {
            cwd: expand(wizard.folder.read(cx).value().trim()).display().to_string(),
            project: Some(wizard.name.read(cx).value().trim().to_string()),
            agent: Some(wizard.agent.read(cx).value().trim().to_string()),
            crew: wizard.crew,
            again: None,
            mode: None,
            resume: crate::loops::Resume::Ask,
        };
        self.close_new_project(window, cx);
        self.start_loop(start, cx);
    }

    /// Back to an earlier step (the stepper's), never forward past what
    /// was done, never while aiball sets it up.
    fn wizard_back_to(&mut self, step: usize, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_mut() else { return };
        if wizard.running || step >= wizard.step.index() {
            return;
        }
        wizard.step = Step::ALL[step];
        cx.notify();
    }

    fn wizard_to(&mut self, step: Step, cx: &mut Context<Self>) {
        if let Some(wizard) = self.new_project.as_mut() {
            wizard.step = step;
        }
        cx.notify();
    }

    /// The wizard over the window, or nothing: one frame for every step —
    /// the same size, the steps on top, the page in the middle, its
    /// buttons at the bottom right.
    pub(super) fn new_project_view(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let wizard = self.new_project.as_ref()?;
        let (page, footer) = match wizard.step {
            Step::Folder => self.folder_step(wizard, cx),
            Step::Identity => self.identity_step(wizard, cx),
            Step::Done => self.done_step(wizard, cx),
            Step::Next => self.next_step(wizard, cx),
        };
        let steps = Stepper::new("new-project-steps")
            .selected_index(wizard.step.index())
            .items(Step::ALL.iter().map(|s| StepperItem::new().child(s.title())))
            .on_click(cx.listener(|shell, step: &usize, _, cx| shell.wizard_back_to(*step, cx)));
        // Its parts named new-project-…: not "new-project" alone, the list's
        // "+ project" link.
        Some(
            crate::ui::sheet::Sheet::new("new-project", crate::t!("newproject-title"))
                .steps(steps)
                .body(page)
                .answers(footer)
                .on_close(cx.listener(|shell, _, window, cx| shell.close_new_project(window, cx)))
                .render(),
        )
    }

    /// The main button of a step: the accent when it can go.
    fn wizard_go(id: &'static str, label: String, enabled: bool) -> Button {
        buttons::primary(id, label).disabled(!enabled)
    }

    fn folder_step(&self, wizard: &NewProject, cx: &mut Context<Self>) -> (Div, Div) {
        let typed = wizard.folder.read(cx).value().to_string();
        let folder = folder_of(&typed, wizard.found.as_ref());
        // The project the folder's file names (not a default).
        let project = match &folder {
            Folder::Configured(_) => wizard.found.as_ref().map(|(_, s)| &s.consumer.project).filter(|p| p.set()).map(|p| p.value.clone()),
            _ => None,
        };
        let (said, colour) = match &folder {
            Folder::Empty => (crate::t!("newproject-folder-empty"), p().muted),
            Folder::Missing => (crate::t!("newproject-folder-missing"), p().danger),
            Folder::NotADirectory => (crate::t!("newproject-folder-file"), p().danger),
            Folder::Checking => (crate::t!("newproject-folder-checking"), p().muted),
            Folder::Configured(file) => match &project {
                Some(name) => (crate::t!("newproject-folder-project", name = name.clone(), file = file.clone()), p().warning),
                None => (crate::t!("newproject-folder-configured", file = file.clone()), p().warning),
            },
            Folder::Fresh { git: true } => (crate::t!("newproject-folder-git"), p().success),
            Folder::Fresh { git: false } => (crate::t!("newproject-folder-ready"), p().success),
        };
        let ready = matches!(folder, Folder::Configured(_) | Folder::Fresh { .. });
        // Already on the board: open it rather than set it up again.
        let open = project.filter(|name| self.board.projects.iter().any(|p| p.name == *name));
        // A session already runs in this folder: opened, not set up again
        // (nor a second one started), even if the board does not show it.
        let running = ready.then(|| self.running_session_in(&expand(&typed))).flatten();
        let page = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_sm().text_color(p().muted).child(crate::t!("newproject-which-folder")))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1().child(Input::new(&wizard.folder)))
                    .child(buttons::chip("new-project-browse", crate::t!("newproject-browse")).px_3().on_click(cx.listener(|shell, _, window, cx| shell.browse_folder(window, cx)))),
            )
            .child(div().text_sm().text_color(colour).child(said));
        let footer = div()
            .flex()
            .gap_3()
            .children(running.map(|(session, label)| {
                buttons::secondary("new-project-running", crate::t!("newproject-open-running", label = label.clone())).on_click(cx.listener(move |shell, _, window, cx| {
                    shell.close_new_project(window, cx);
                    shell.select(session.clone(), window, cx);
                }))
            }))
            .children(open.map(|name| {
                buttons::secondary("new-project-open", crate::t!("newproject-open", name = name.clone())).on_click(cx.listener(move |shell, _, window, cx| {
                    shell.close_new_project(window, cx);
                    shell.show_project(name.clone(), cx);
                }))
            }))
            .child(Self::wizard_go("new-project-next", crate::t!("newproject-next"), ready).when(ready, |d| d.on_click(cx.listener(|shell, _, window, cx| shell.to_identity(window, cx)))));
        (page, footer)
    }

    fn identity_step(&self, wizard: &NewProject, cx: &mut Context<Self>) -> (Div, Div) {
        let typed = wizard.folder.read(cx).value().to_string();
        let name = wizard.name.read(cx).value().trim().to_string();
        let agent = wizard.agent.read(cx).value().trim().to_string();
        // Said at once for a name, else what aiball answered to the dry run.
        let problem = if !name_ok(&name) {
            Some(crate::t!("newproject-bad-project"))
        } else if !name_ok(&agent) {
            Some(crate::t!("newproject-bad-agent"))
        } else {
            match &wizard.preview {
                Some(Err(error)) => Some(error.clone()),
                _ => None,
            }
        };
        // A choice still as the folder's configuration sets it wears the
        // "imported" colour, and says where it comes from; changed, or at
        // aiball's default, it is plain.
        let was = wizard.imported.clone().unwrap_or_default();
        let filled = Filled::of(&was, "");
        let imported = crate::theme::imported();
        let origin = |said: Option<&str>| said.map(|said| div().flex_none().text_xs().text_color(imported).child(said.to_string()));
        let field = |label: String, input: &Entity<InputState>, said: Option<&str>| {
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(div().w(px(90.)).flex_none().text_sm().text_color(if said.is_some() { imported } else { p().muted }).child(label))
                .child(div().flex_1().child(Input::new(input)))
                .children(origin(said))
        };
        let tick = |id: &'static str, on: bool, label: String, about: String, said: Option<&str>, flip: fn(&mut NewProject)| {
            div()
                .flex()
                .flex_col()
                .child(
                    // Its words flip it too, as an HTML label does.
                    buttons::switch_with(
                        id,
                        on,
                        div().flex().items_center().gap_3().child(div().text_color(if said.is_some() { imported } else { p().text }).child(label)).children(origin(said)),
                        cx.listener(move |shell, _: &bool, _, cx| {
                            if let Some(wizard) = shell.new_project.as_mut() {
                                flip(wizard);
                            }
                            shell.preview_init(cx);
                            cx.notify();
                        }),
                    ),
                )
                .child(div().pl_10().text_xs().text_color(p().muted).child(about))
        };
        let kept = |set: bool, same: bool, said: &str| (set && same).then(|| said.to_string());
        let project_from = kept(was.consumer.project.set(), name == filled.project, &was.consumer.project.said());
        let agent_from = kept(was.consumer.agent.set(), agent == filled.agent, &was.consumer.agent.said());
        let crew_from = kept(was.consumer.role.set(), wizard.crew == filled.crew, &was.consumer.role.said());
        let host_from = kept(was.session.set(), wizard.on_host == filled.on_host, &was.session.said());
        let rc_from = kept(was.remote_control.set(), wizard.remote_control == filled.remote_control, &was.remote_control.said());
        // What will be written after: against the folder's file, or, when
        // the set up makes one of the folder's own, aiball's defaults.
        let new_file = matches!(&wizard.preview, Some(Ok(done)) if done.steps.iter().any(|s| s.action == "created" && s.file.ends_with(".aiball.yaml")));
        let against = if new_file { ProjectSettings { session: Setting { value: "host".into(), from: "default".into() }, ..Default::default() } } else { was.clone() };
        let (session_write, rc_write) = to_write(&against, wizard.on_host, remote_control_of(&was, wizard.remote_control));
        // What aiball would do: each file, and whether the project is new.
        let plan = match &wizard.preview {
            Some(Ok(done)) => done.steps.iter().map(|step| crate::t!("newproject-will-file", will = will(&step.action), file = step.file.clone())).collect::<Vec<_>>().join(" · "),
            _ => "…".to_string(),
        };
        let joins = matches!(&wizard.preview, Some(Ok(done)) if done.project_exists);
        let running = wizard.running;
        let go = problem.is_none() && !running && matches!(wizard.preview, Some(Ok(_)));
        let page = div()
            .flex()
            .flex_col()
            .gap_3()
            // The file the choices were filled from.
            .children(was.file.as_ref().map(|file| div().text_xs().text_color(imported).child(crate::t!("newproject-filled-from", file = file.clone()))))
            .child(field(crate::t!("newproject-field-project"), &wizard.name, project_from.as_deref()))
            .child(field(crate::t!("newproject-field-agent"), &wizard.agent, agent_from.as_deref()))
            .child(tick(
                "new-project-crew",
                wizard.crew,
                crate::t!("newproject-crew"),
                crate::t!("newproject-crew-about"),
                crew_from.as_deref(),
                |w| w.crew = !w.crew,
            ))
            .child(tick(
                "new-project-host",
                wizard.on_host,
                crate::t!("newproject-host"),
                crate::t!("newproject-host-about", mux = crate::mux::program()),
                host_from.as_deref(),
                |w| w.on_host = !w.on_host,
            ))
            .child(tick(
                "new-project-rc",
                wizard.remote_control,
                crate::t!("newproject-rc"),
                crate::t!("newproject-rc-about"),
                rc_from.as_deref(),
                |w| w.remote_control = !w.remote_control,
            ))
            .child(tick(
                "new-project-private",
                wizard.private,
                crate::t!("newproject-private"),
                crate::t!("newproject-private-about"),
                None,
                |w| w.private = !w.private,
            ))
            .child(tick(
                "new-project-noclaim",
                wizard.no_claim,
                crate::t!("newproject-noclaim"),
                crate::t!("newproject-noclaim-about"),
                None,
                |w| w.no_claim = !w.no_claim,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_2()
                    .rounded_md()
                    .bg(p().bg)
                    .text_xs()
                    .text_color(p().muted)
                    .child(div().truncate().child(crate::t!("newproject-in", folder = expand(typed.trim()).display().to_string())))
                    // What aiball will do, or why it will not.
                    .child(match &problem {
                        Some(problem) => div().text_color(p().danger).child(problem.clone()),
                        None => div().text_color(p().text).child(plan),
                    })
                    .when(joins && problem.is_none(), |d| {
                        d.child(div().text_color(p().info).child(crate::t!("newproject-joins", name = name.clone())))
                    })
                    // Then, where its loops run and its Remote Control, when
                    // they change what the folder had.
                    .when(problem.is_none() && (session_write.is_some() || rc_write.is_some()), |d| {
                        let mut said = Vec::new();
                        if let Some(session) = session_write {
                            said.push(format!("claude_loop.session: {session}"));
                        }
                        if let Some(rc) = &rc_write {
                            said.push(format!("claude.remote_control: {rc}"));
                        }
                        d.child(div().text_color(p().text).child(crate::t!("newproject-then-sets", what = said.join(", "))))
                    })
                    .child(crate::t!("newproject-files")),
            );
        let footer = div()
            .flex()
            .gap_3()
            .child(buttons::secondary("new-project-back", crate::t!("newproject-back")).on_click(cx.listener(|shell, _, _, cx| shell.wizard_to(Step::Folder, cx))))
            .child(Self::wizard_go("new-project-go", crate::t!(if running { "newproject-setting-up" } else { "newproject-set-up" }), go).when(go, |d| d.on_click(cx.listener(|shell, _, _, cx| shell.set_it_up(cx)))));
        (page, footer)
    }

    fn done_step(&self, wizard: &NewProject, cx: &mut Context<Self>) -> (Div, Div) {
        let (ok, said) = match &wizard.outcome {
            Some(Ok(done)) => (true, done.steps.iter().map(|step| step.message.clone()).collect::<Vec<_>>().join("\n")),
            Some(Err(error)) => (false, error.clone()),
            None => (false, String::new()),
        };
        let name = wizard.name.read(cx).value().trim().to_string();
        let page = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_color(if ok { p().success } else { p().danger }).child(if ok {
                crate::t!("newproject-done", name = name.clone())
            } else {
                crate::t!("newproject-failed")
            }))
            .child(
                div()
                    .p_2()
                    .rounded_md()
                    .bg(p().bg)
                    .text_xs()
                    .font_family("monospace")
                    .text_color(p().muted)
                    .child(if said.is_empty() { crate::t!("newproject-nothing-said") } else { said }),
            );
        let footer = if ok {
            div().child(Self::wizard_go("new-project-whatnext", crate::t!("newproject-next"), true).on_click(cx.listener(|shell, _, _, cx| shell.wizard_to(Step::Next, cx))))
        } else {
            div().flex().gap_3().child(buttons::secondary("new-project-retry", crate::t!("newproject-back")).on_click(cx.listener(|shell, _, _, cx| shell.wizard_to(Step::Identity, cx))))
        };
        (page, footer)
    }

    /// What is left to the user, once the folder is set up.
    fn next_step(&self, wizard: &NewProject, cx: &mut Context<Self>) -> (Div, Div) {
        let name = wizard.name.read(cx).value().trim().to_string();
        let agent = wizard.agent.read(cx).value().trim().to_string();
        let item = |n: &'static str, title: String, text: String| {
            div()
                .flex()
                .gap_3()
                .child(div().flex_none().w(px(22.)).h(px(22.)).rounded_full().flex().items_center().justify_center().text_xs().bg(p().active).text_color(p().accent).child(n))
                .child(div().flex_1().min_w_0().flex().flex_col().gap_0p5().child(div().font_weight(FontWeight::MEDIUM).child(title)).child(div().text_sm().text_color(p().muted).child(text)))
        };
        let page = div()
            .flex()
            .flex_col()
            .gap_4()
            .child(item(
                "1",
                crate::t!("newproject-next-start"),
                if wizard.on_host {
                    crate::t!("newproject-next-start-host", agent = agent.clone())
                } else {
                    crate::t!("newproject-next-start-mux", agent = agent.clone(), mux = crate::mux::program())
                },
            ))
            .child(item(
                "2",
                crate::t!("newproject-next-mcp"),
                crate::t!("newproject-next-mcp-about"),
            ))
            // aiball's skill (its good gestures, for Claude Code) lives outside
            // the folder: aiball says whether this machine has it.
            .when(matches!(&wizard.outcome, Some(Ok(done)) if done.skill == "missing"), |d| {
                d.child(item(
                    "·",
                    crate::t!("newproject-next-skill"),
                    crate::t!("newproject-next-skill-about"),
                ))
            })
            .child(item(
                "3",
                crate::t!("newproject-next-work"),
                crate::t!("newproject-next-work-about", name = name.clone()),
            ));
        let footer = div()
            .flex()
            .items_center()
            .gap_3()
            .child(buttons::secondary("new-project-done", crate::t!("newproject-close")).on_click(cx.listener(|shell, _, window, cx| shell.close_new_project(window, cx))))
            .child(Self::wizard_go("new-project-start", crate::t!("newproject-start-first"), true).on_click(cx.listener(|shell, _, window, cx| shell.start_first_session(window, cx))));
        (page, footer)
    }
}

impl Shell {
    /// The session a running loop of this machine has in `folder`, and its
    /// agent's name (else the loop's): what to open instead of starting one.
    fn running_session_in(&self, folder: &Path) -> Option<(String, String)> {
        let found = self.live.known_loops().into_iter().find(|l| l.running && same_folder(Path::new(&l.cwd), folder))?;
        let label = found.agent.clone().unwrap_or_else(|| found.name.clone());
        Some((found.session(), label))
    }
}

/// Whether two paths name one folder: as written, else as the file system
/// resolves them (on Windows, case and separators aside).
fn same_folder(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    let plain = |p: &Path| p.display().to_string().replace('\\', "/").trim_end_matches('/').to_string();
    if cfg!(windows) && plain(a).eq_ignore_ascii_case(&plain(b)) {
        return true;
    }
    matches!((a.canonicalize(), b.canonicalize()), (Ok(x), Ok(y)) if x == y)
}

#[cfg(test)]
mod tests {
    use super::{Filled, Folder, folder_of, name_from, name_ok, remote_control_of, same_folder, to_write};
    use crate::aiball::{ProjectSettings, Setting};
    use serde_json::json;
    use std::path::Path;

    /// `project.settings` as aiball answers it.
    fn settings(answer: serde_json::Value) -> ProjectSettings {
        serde_json::from_value(answer).unwrap()
    }

    #[test]
    fn a_folder_says_what_it_is_as_aiball_answers() {
        let dir = std::env::temp_dir().join(format!("tvty-newproject-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(folder_of("", None), Folder::Empty);
        assert_eq!(folder_of(&dir.join("nothing").display().to_string(), None), Folder::Missing);
        // A folder: aiball is asked, its answer said.
        assert_eq!(folder_of(&dir.display().to_string(), None), Folder::Checking);
        assert_eq!(folder_of(&dir.display().to_string(), Some(&(dir.clone(), ProjectSettings::default()))), Folder::Fresh { git: false });
        let set = ProjectSettings { file: Some("/w/.aiball.yaml".to_string()), ..Default::default() };
        assert_eq!(folder_of(&dir.display().to_string(), Some(&(dir.clone(), set.clone()))), Folder::Configured("/w/.aiball.yaml".into()));
        // The folder a loop runs in, however written.
        assert!(same_folder(Path::new("/w/tvty"), Path::new("/w/tvty")));
        assert!(!same_folder(Path::new("/w/tvty"), Path::new("/w/aiball")));
        if cfg!(windows) {
            assert!(same_folder(Path::new("C:\\Users\\d\\Tvty\\"), Path::new("c:/users/d/tvty")));
        }
        // An answer about another folder is not this one's.
        assert_eq!(folder_of(&dir.display().to_string(), Some(&(Path::new("/elsewhere").to_path_buf(), set))), Folder::Checking);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn names_and_the_folder() {
        assert!(name_ok("my-app_2.x") && !name_ok("my app") && !name_ok(""));
        assert_eq!(name_from(Path::new("/tmp/My App")), "My-App");
    }

    #[test]
    fn tvty_never_reads_an_aiball_yaml_itself() {
        // aiball's folder configuration is aiball's: asked over the bus
        // (project.settings), never opened by tvty.
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut stack = vec![src];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let code = std::fs::read_to_string(&path).unwrap();
                    assert!(!code.contains(&format!("join(\"{}\")", ".aiball.yaml")), "{} opens .aiball.yaml", path.display());
                }
            }
        }
    }

    #[test]
    fn a_configured_folder_fills_the_choices_and_a_bare_one_its_names() {
        let configured = settings(json!({
            "file": "/w/app/.aiball.yaml", "configured": true,
            "consumer": { "project": { "value": "app", "from": "file" }, "agent": { "value": "app-dev", "from": "file" }, "role": { "value": "crew", "from": "file" } },
            "session": { "value": "tmux", "from": "file" }, "remote_control": { "value": "app-rc", "from": "file" }
        }));
        let filled = Filled::of(&configured, "folder");
        assert_eq!((filled.project.as_str(), filled.agent.as_str()), ("app", "app-dev"));
        assert!(filled.crew && !filled.on_host && filled.remote_control);
        assert!(configured.session.set() && configured.session.said() == "from .aiball.yaml");
        // aiball's defaults: the folder's names, on the host, no Remote Control.
        let bare = settings(json!({
            "file": null, "configured": false,
            "consumer": { "project": { "value": "x", "from": "default" }, "agent": { "value": "x", "from": "default" }, "role": { "value": null, "from": "default" } },
            "session": { "value": "host", "from": "default" }, "remote_control": { "value": false, "from": "default" }
        }));
        let filled = Filled::of(&bare, "my-app");
        assert_eq!((filled.project.as_str(), filled.agent.as_str()), ("my-app", "my-app-claude"));
        assert!(!filled.crew && filled.on_host && !filled.remote_control && !bare.session.set());
        // The machine's config sets where loops run too.
        let global = Setting { value: "tmux".to_string(), from: "global".into() };
        assert!(global.set() && global.said() == "from aiball's global config");
    }

    #[test]
    fn only_what_differs_from_the_folder_is_written() {
        let named = settings(json!({ "session": { "value": "tmux", "from": "file" }, "remote_control": { "value": "app-rc", "from": "file" } }));
        // A named Remote Control, still on, stays as it is named.
        assert_eq!(remote_control_of(&named, true), json!("app-rc"));
        assert_eq!(remote_control_of(&named, false), json!(false));
        // As the folder says: nothing to write.
        assert_eq!(to_write(&named, false, json!("app-rc")), (None, None));
        assert_eq!(to_write(&named, true, json!(false)), (Some("host"), Some(json!(false))));
        // A sub-folder given a file of its own, which hides the parent's:
        // what was imported from the parent is written in it.
        let bare = settings(json!({ "session": { "value": "host", "from": "default" }, "remote_control": { "value": false, "from": "default" } }));
        assert_eq!(to_write(&bare, false, remote_control_of(&named, true)), (Some("tmux"), Some(json!("app-rc"))));
        assert_eq!(to_write(&bare, true, json!(false)), (None, None));
    }
}

