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
use crate::aiball::{InitAsk, InitDone};
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

    fn title(self) -> &'static str {
        match self {
            Step::Folder => "Folder",
            Step::Identity => "Who works in it",
            Step::Done => "Set up",
            Step::Next => "What next",
        }
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
    running: bool,
    /// What aiball would do with the choices as they stand (a dry run),
    /// for the `asked`-th of them: or why it would not.
    preview: Option<Result<InitDone, String>>,
    asked: u64,
    /// What aiball did: or why it did not.
    outcome: Option<Result<InitDone, String>>,
    /// The folder aiball was last asked about, and the `.aiball.yaml` that
    /// applies there, if any (aiball's answer: tvty reads no such file).
    found: Option<(PathBuf, Option<String>)>,
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

fn folder_of(typed: &str, found: Option<&(PathBuf, Option<String>)>) -> Folder {
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
        Some((_, Some(file))) => Folder::Configured(file.clone()),
        Some((_, None)) => Folder::Fresh { git: path.join(".git").exists() },
        None => Folder::Checking,
    }
}

/// `~/…` as the home's.
fn expand(typed: &str) -> PathBuf {
    match (typed.strip_prefix("~/"), std::env::var_os("HOME")) {
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

/// What aiball's dry run says it did to a file, as what it will do.
fn will(action: &str) -> &str {
    match action {
        "created" => "creates",
        "added" => "adds its entry to",
        "rewritten" => "rewrites its entry in",
        "patched" => "updates",
        "overwrote" => "overwrites",
        "kept" => "keeps",
        other => other,
    }
}

impl Shell {
    /// Opens the wizard, at its first step.
    pub(super) fn open_new_project(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.help_menu = false;
        let folder = cx.new(|cx| InputState::new(window, cx).placeholder("the project's folder, e.g. ~/dev/app"));
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("project"));
        let agent = cx.new(|cx| InputState::new(window, cx).placeholder("agent"));
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
            running: false,
            preview: None,
            asked: 0,
            outcome: None,
            found: None,
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
            prompt: Some("Choose the project's folder".into()),
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

    /// Asks aiball which `.aiball.yaml` applies to the folder typed (none:
    /// not set up); the answer for the folder still typed only is kept.
    fn ask_folder(&mut self, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_ref() else { return };
        let path = expand(wizard.folder.read(cx).value().trim());
        if !path.is_dir() || wizard.found.as_ref().is_some_and(|(asked, _)| *asked == path) {
            return;
        }
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let cwd = path.display().to_string();
            let file = cx.background_executor().spawn(async move { aiball.project_file(&cwd) }).await;
            let _ = this.update(cx, |shell, cx| {
                let Some(wizard) = shell.new_project.as_mut() else { return };
                if expand(wizard.folder.read(cx).value().trim()) != path {
                    return;
                }
                match file {
                    Ok(file) => wizard.found = Some((path, file)),
                    Err(error) => log::warn!("new project: {} {error:#}", path.display()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The project already set up where `file` applies, as aiball knows its
    /// agents: one working under the file's folder.
    fn project_of_file(&self, file: &str) -> Option<String> {
        let root = Path::new(file).parent()?;
        self.board.homes.iter().find_map(|(_, project, cwd)| project.clone().filter(|_| Path::new(cwd).starts_with(root)))
    }

    /// From the folder to who works in it: the names proposed from the
    /// folder (or the project already set up there), unless already typed.
    fn to_identity(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_ref() else { return };
        let typed = wizard.folder.read(cx).value().to_string();
        let proposed = match folder_of(&typed, wizard.found.as_ref()) {
            Folder::Configured(file) => self.project_of_file(&file).unwrap_or_else(|| name_from(&expand(typed.trim()))),
            Folder::Fresh { .. } => name_from(&expand(typed.trim())),
            _ => return,
        };
        let Some(wizard) = self.new_project.as_mut() else { return };
        if wizard.name.read(cx).value().trim().is_empty() {
            wizard.name.update(cx, |n, cx| n.set_value(proposed.clone(), window, cx));
        }
        if wizard.agent.read(cx).value().trim().is_empty() {
            wizard.agent.update(cx, |a, cx| a.set_value(format!("{proposed}-claude"), window, cx));
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
        wizard.running = true;
        cx.notify();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let said = cx.background_executor().spawn(async move { aiball.project_init(&ask, false).map_err(|e| format!("{e:#}")) }).await;
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

    /// Its first session, on aiball's host; the wizard closes.
    fn start_first_session(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_ref() else { return };
        let start = Start {
            cwd: expand(wizard.folder.read(cx).value().trim()).display().to_string(),
            project: Some(wizard.name.read(cx).value().trim().to_string()),
            agent: Some(wizard.agent.read(cx).value().trim().to_string()),
            crew: wizard.crew,
            again: None,
            mode: None,
        };
        self.close_new_project(window, cx);
        self.start_on_host(start, cx);
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
        let card = div()
            .id("new-project")
            .occlude()
            .w(px(680.))
            .h(px(660.))
            .max_w(relative(0.92))
            .max_h(relative(0.9))
            .flex()
            .flex_col()
            .rounded_lg()
            .border_1()
            .border_color(p().border)
            .bg(p().surface)
            .text_color(p().text)
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .px_5()
                    .pt_4()
                    .child(div().flex_1().text_lg().font_weight(FontWeight::BOLD).child("New project"))
                    .child(buttons::link("new-project-close", "✕  Esc").text_sm().on_click(cx.listener(|shell, _, window, cx| shell.close_new_project(window, cx)))),
            )
            .child(div().flex_none().px_5().py_3().child(steps))
            .child(div().id("new-project-page").flex_1().min_h_0().overflow_y_scroll().px_5().py_2().child(page))
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_end()
                    .gap_3()
                    .px_5()
                    .py_3()
                    .border_t_1()
                    .border_color(p().border)
                    .child(footer),
            );
        Some(
            div()
                .id("new-project-veil")
                .absolute()
                .inset_0()
                .occlude()
                .flex()
                .items_center()
                .justify_center()
                .bg(gpui_kit::black().opacity(0.45))
                .child(card)
                .into_any_element(),
        )
    }

    /// The main button of a step: the accent when it can go.
    fn wizard_go(id: &'static str, label: &'static str, enabled: bool) -> Button {
        buttons::primary(id, label).disabled(!enabled)
    }

    fn folder_step(&self, wizard: &NewProject, cx: &mut Context<Self>) -> (Div, Div) {
        let typed = wizard.folder.read(cx).value().to_string();
        let folder = folder_of(&typed, wizard.found.as_ref());
        let project = match &folder {
            Folder::Configured(file) => self.project_of_file(file),
            _ => None,
        };
        let (said, colour) = match &folder {
            Folder::Empty => ("The folder the agent will work in: the project's root.".to_string(), p().muted),
            Folder::Missing => ("No such folder.".to_string(), p().danger),
            Folder::NotADirectory => ("This is a file, not a folder.".to_string(), p().danger),
            Folder::Checking => ("Asking aiball…".to_string(), p().muted),
            Folder::Configured(file) => match &project {
                Some(name) => (format!("Already an aiball project: {name} ({file}). Next sets it up again."), p().warning),
                None => (format!("Already set up for aiball ({file}). Next sets it up again."), p().warning),
            },
            Folder::Fresh { git: true } => ("A git repository: ready.".to_string(), p().success),
            Folder::Fresh { git: false } => ("Ready (not a git repository).".to_string(), p().success),
        };
        let ready = matches!(folder, Folder::Configured(_) | Folder::Fresh { .. });
        // Already on the board: open it rather than set it up again.
        let open = project.filter(|name| self.board.projects.iter().any(|p| p.name == *name));
        let page = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_sm().text_color(p().muted).child("Which folder becomes the project?"))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1().child(Input::new(&wizard.folder)))
                    .child(buttons::chip("new-project-browse", "Browse…").px_3().on_click(cx.listener(|shell, _, window, cx| shell.browse_folder(window, cx)))),
            )
            .child(div().text_sm().text_color(colour).child(said));
        let footer = div()
            .flex()
            .gap_3()
            .children(open.map(|name| {
                buttons::secondary("new-project-open", format!("Open {name}")).on_click(cx.listener(move |shell, _, window, cx| {
                    shell.close_new_project(window, cx);
                    shell.show_project(name.clone(), cx);
                }))
            }))
            .child(Self::wizard_go("new-project-next", "Next →", ready).when(ready, |d| d.on_click(cx.listener(|shell, _, window, cx| shell.to_identity(window, cx)))));
        (page, footer)
    }

    fn identity_step(&self, wizard: &NewProject, cx: &mut Context<Self>) -> (Div, Div) {
        let typed = wizard.folder.read(cx).value().to_string();
        let name = wizard.name.read(cx).value().trim().to_string();
        let agent = wizard.agent.read(cx).value().trim().to_string();
        // Said at once for a name, else what aiball answered to the dry run.
        let problem = if !name_ok(&name) {
            Some("The project's name: letters, digits, -, _ and .".to_string())
        } else if !name_ok(&agent) {
            Some("The agent's name: letters, digits, -, _ and .".to_string())
        } else {
            match &wizard.preview {
                Some(Err(error)) => Some(error.clone()),
                _ => None,
            }
        };
        let field = |label: &'static str, input: &Entity<InputState>| {
            div().flex().items_center().gap_3().child(div().w(px(90.)).flex_none().text_sm().text_color(p().muted).child(label)).child(div().flex_1().child(Input::new(input)))
        };
        let tick = |id: &'static str, on: bool, label: &'static str, about: &'static str, flip: fn(&mut NewProject)| {
            div()
                .flex()
                .flex_col()
                .child(
                    buttons::link(id, if on { "☑" } else { "☐" })
                        .text_color(p().text)
                        .child(label)
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            if let Some(wizard) = shell.new_project.as_mut() {
                                flip(wizard);
                            }
                            shell.preview_init(cx);
                            cx.notify();
                        })),
                )
                .child(div().pl_6().text_xs().text_color(p().muted).child(about))
        };
        // What aiball would do: each file, and whether the project is new.
        let plan = match &wizard.preview {
            Some(Ok(done)) => done.steps.iter().map(|step| format!("{} {}", will(&step.action), step.file)).collect::<Vec<_>>().join(" · "),
            _ => "…".to_string(),
        };
        let joins = matches!(&wizard.preview, Some(Ok(done)) if done.project_exists);
        let running = wizard.running;
        let go = problem.is_none() && !running && matches!(wizard.preview, Some(Ok(_)));
        let page = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(field("project", &wizard.name))
            .child(field("agent", &wizard.agent))
            .child(tick("new-project-crew", wizard.crew, "a crew agent", "Beside the project's lead, on the tickets it is given; unticked: the lead.", |w| w.crew = !w.crew))
            .child(tick("new-project-private", wizard.private, "a private project", "aiball serves it its private kit (no public tickets, no followers).", |w| w.private = !w.private))
            .child(tick("new-project-noclaim", wizard.no_claim, "no claiming", "The agent works only on the tickets assigned to it, never takes one from the pool.", |w| w.no_claim = !w.no_claim))
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
                    .child(div().truncate().child(format!("In {}, aiball:", expand(typed.trim()).display())))
                    // What aiball will do, or why it will not.
                    .child(match &problem {
                        Some(problem) => div().text_color(p().danger).child(problem.clone()),
                        None => div().text_color(p().text).child(plan),
                    })
                    .when(joins && problem.is_none(), |d| {
                        d.child(div().text_color(p().info).child(format!("{name} is on the board already: this folder joins it (another folder, or a crew agent).")))
                    })
                    .child(".mcp.json: aiball's MCP server for Claude Code · .aiball.yaml: project, agent, role."),
            );
        let footer = div()
            .flex()
            .gap_3()
            .child(buttons::secondary("new-project-back", "← Back").on_click(cx.listener(|shell, _, _, cx| shell.wizard_to(Step::Folder, cx))))
            .child(Self::wizard_go("new-project-go", if running { "Setting it up…" } else { "Set it up" }, go).when(go, |d| d.on_click(cx.listener(|shell, _, _, cx| shell.set_it_up(cx)))));
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
                format!("{name} is set up. aiball said:")
            } else {
                "aiball could not set it up:".to_string()
            }))
            .child(
                div()
                    .p_2()
                    .rounded_md()
                    .bg(p().bg)
                    .text_xs()
                    .font_family("monospace")
                    .text_color(p().muted)
                    .child(if said.is_empty() { "(nothing said)".to_string() } else { said }),
            );
        let footer = if ok {
            div().child(Self::wizard_go("new-project-whatnext", "Next →", true).on_click(cx.listener(|shell, _, _, cx| shell.wizard_to(Step::Next, cx))))
        } else {
            div().flex().gap_3().child(buttons::secondary("new-project-retry", "← Back").on_click(cx.listener(|shell, _, _, cx| shell.wizard_to(Step::Identity, cx))))
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
                "Start the agent".into(),
                format!("\"Start its first session\" starts {agent}'s Claude Code on aiball's host, in the project's folder; its terminal opens here, its tickets beside."),
            ))
            .child(item(
                "2",
                "Accept aiball's MCP server".into(),
                "At its first start in this folder, Claude Code asks whether to use the MCP server that .mcp.json declares (aiball): accept it. Without it the agent can neither read the board nor answer its tickets. Refused by mistake? /mcp in Claude Code turns it on.".into(),
            ))
            // aiball's skill (its good gestures, for Claude Code) lives outside
            // the folder: aiball says whether this machine has it.
            .when(matches!(&wizard.outcome, Some(Ok(done)) if done.skill == "missing"), |d| {
                d.child(item(
                    "·",
                    "Install aiball's skill".into(),
                    "Claude Code has no aiball skill on this machine yet: `aiball init skill`, once, installs it — the agent then knows the board's good gestures.".into(),
                ))
            })
            .child(item(
                "3",
                "Give it work".into(),
                format!("\"+ New\" in the ticket panel files a ticket on {name}; the agent picks it up at its next wake, and its plans and questions come back as notifications."),
            ));
        let footer = div()
            .flex()
            .items_center()
            .gap_3()
            .child(buttons::secondary("new-project-done", "Close").on_click(cx.listener(|shell, _, window, cx| shell.close_new_project(window, cx))))
            .child(Self::wizard_go("new-project-start", "Start its first session", true).on_click(cx.listener(|shell, _, window, cx| shell.start_first_session(window, cx))));
        (page, footer)
    }
}

#[cfg(test)]
mod tests {
    use super::{Folder, folder_of, name_from, name_ok};
    use std::path::Path;

    #[test]
    fn a_folder_says_what_it_is_as_aiball_answers() {
        let dir = std::env::temp_dir().join(format!("tvty-newproject-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(folder_of("", None), Folder::Empty);
        assert_eq!(folder_of(&dir.join("nothing").display().to_string(), None), Folder::Missing);
        // A folder: aiball is asked, its answer said.
        assert_eq!(folder_of(&dir.display().to_string(), None), Folder::Checking);
        assert_eq!(folder_of(&dir.display().to_string(), Some(&(dir.clone(), None))), Folder::Fresh { git: false });
        let file = Some("/w/.aiball.yaml".to_string());
        assert_eq!(folder_of(&dir.display().to_string(), Some(&(dir.clone(), file.clone()))), Folder::Configured("/w/.aiball.yaml".into()));
        // An answer about another folder is not this one's.
        assert_eq!(folder_of(&dir.display().to_string(), Some(&(Path::new("/elsewhere").to_path_buf(), file))), Folder::Checking);
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
}
