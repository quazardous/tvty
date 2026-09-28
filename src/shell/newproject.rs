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
use gpui_kit::*;

use super::Shell;
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
    /// What `aiball init` said: its output, or its error.
    outcome: Option<Result<String, String>>,
}

/// What the folder typed is, for aiball.
#[derive(Debug, PartialEq)]
enum Folder {
    Empty,
    Missing,
    NotADirectory,
    /// Already an aiball project (its `.aiball.yaml`), named if it says so.
    Configured(Option<String>),
    /// Ready to be one; `git`: a git repository.
    Fresh { git: bool },
}

fn folder_of(typed: &str) -> Folder {
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
    match std::fs::read_to_string(path.join(".aiball.yaml")) {
        Ok(yaml) => Folder::Configured(project_in(&yaml)),
        Err(_) => Folder::Fresh { git: path.join(".git").exists() },
    }
}

/// `~/…` as the home's.
fn expand(typed: &str) -> PathBuf {
    match (typed.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => Path::new(&home).join(rest),
        _ => PathBuf::from(typed),
    }
}

/// The project a `.aiball.yaml` names (`project: name`, under `consumer:`
/// or alone).
fn project_in(yaml: &str) -> Option<String> {
    yaml.lines()
        .filter_map(|line| line.trim().strip_prefix("project:"))
        .map(|value| value.trim().trim_matches(['"', '\'']).to_string())
        .find(|value| !value.is_empty())
}

/// A project's or an agent's name as aiball takes it: letters, digits,
/// `-` and `_`.
fn name_ok(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// A name for the folder's project: its own name, made acceptable.
fn name_from(folder: &Path) -> String {
    folder
        .file_name()
        .map(|n| n.to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect())
        .unwrap_or_default()
}

/// `aiball init`'s arguments.
fn init_args(name: &str, agent: &str, crew: bool, private: bool, no_claim: bool) -> Vec<String> {
    let mut args = vec!["init".into(), "--project".into(), name.into(), "--agent".into(), agent.into()];
    if crew {
        args.extend(["--role".into(), "crew".into()]);
    }
    if private {
        args.push("--private".into());
    }
    if no_claim {
        args.push("--no-claim".into());
    }
    args
}

impl Shell {
    /// Opens the wizard, at its first step.
    pub(super) fn open_new_project(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.help_menu = false;
        let folder = cx.new(|cx| InputState::new(window, cx).placeholder("the project's folder, e.g. ~/dev/app"));
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("project"));
        let agent = cx.new(|cx| InputState::new(window, cx).placeholder("agent"));
        // What the folder is, said as it is typed.
        for input in [&folder, &name, &agent] {
            cx.subscribe(input, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
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
            outcome: None,
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
                    cx.notify();
                });
            });
        })
        .detach();
    }

    /// From the folder to who works in it: the names proposed from the
    /// folder (or its `.aiball.yaml`), unless already typed.
    fn to_identity(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_mut() else { return };
        let typed = wizard.folder.read(cx).value().to_string();
        let proposed = match folder_of(&typed) {
            Folder::Configured(Some(name)) => name,
            Folder::Configured(None) | Folder::Fresh { .. } => name_from(&expand(typed.trim())),
            _ => return,
        };
        if wizard.name.read(cx).value().trim().is_empty() {
            wizard.name.update(cx, |n, cx| n.set_value(proposed.clone(), window, cx));
        }
        if wizard.agent.read(cx).value().trim().is_empty() {
            wizard.agent.update(cx, |a, cx| a.set_value(format!("{proposed}-claude"), window, cx));
        }
        wizard.step = Step::Identity;
        cx.notify();
    }

    /// A project of the board by that name, other than the folder's own.
    fn name_taken(&self, name: &str, folder: &Folder) -> bool {
        let on_board = self.board.projects.iter().any(|p| p.name == name) || self.live.tickets().contains_key(name);
        on_board && *folder != Folder::Configured(Some(name.to_string()))
    }

    /// Runs `aiball init` in the folder, with the choices made.
    fn set_it_up(&mut self, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_mut() else { return };
        if wizard.running {
            return;
        }
        let folder = expand(wizard.folder.read(cx).value().trim());
        let (name, agent) = (wizard.name.read(cx).value().trim().to_string(), wizard.agent.read(cx).value().trim().to_string());
        let args = init_args(&name, &agent, wizard.crew, wizard.private, wizard.no_claim);
        wizard.running = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let said = cx
                .background_executor()
                .spawn(async move {
                    let output = std::process::Command::new("aiball")
                        .args(&args)
                        .current_dir(&folder)
                        .env_remove("TMUX")
                        .output()
                        .map_err(|e| format!("aiball: {e}"))?;
                    let text = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr);
                    if output.status.success() { Ok(text.trim().to_string()) } else { Err(text.trim().to_string()) }
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

    /// Its first session, on aiball's host; the wizard closes.
    fn start_first_session(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(wizard) = self.new_project.as_ref() else { return };
        let start = Start {
            cwd: expand(wizard.folder.read(cx).value().trim()).display().to_string(),
            project: Some(wizard.name.read(cx).value().trim().to_string()),
            agent: Some(wizard.agent.read(cx).value().trim().to_string()),
            crew: wizard.crew,
            again: None,
        };
        self.close_new_project(window, cx);
        self.start_on_host(start, cx);
    }

    /// Back to an earlier step (the stepper's), never forward past what
    /// was done, never while aiball init runs.
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
            .h(px(580.))
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
    fn wizard_go(id: &'static str, label: &'static str, enabled: bool) -> Stateful<Div> {
        buttons::chip_if(id, label, enabled).px_3().py_1().when(enabled, |d| d.border_color(p().accent).text_color(p().accent))
    }

    fn folder_step(&self, wizard: &NewProject, cx: &mut Context<Self>) -> (Div, Div) {
        let typed = wizard.folder.read(cx).value().to_string();
        let folder = folder_of(&typed);
        let (said, colour) = match &folder {
            Folder::Empty => ("The folder the agent will work in: the project's root.".to_string(), p().muted),
            Folder::Missing => ("No such folder.".to_string(), p().danger),
            Folder::NotADirectory => ("This is a file, not a folder.".to_string(), p().danger),
            Folder::Configured(Some(name)) => (format!("Already an aiball project: {name}. Next sets it up again."), p().warning),
            Folder::Configured(None) => ("Already set up for aiball (.aiball.yaml). Next sets it up again.".to_string(), p().warning),
            Folder::Fresh { git: true } => ("A git repository: ready.".to_string(), p().success),
            Folder::Fresh { git: false } => ("Ready (not a git repository).".to_string(), p().success),
        };
        let ready = matches!(folder, Folder::Configured(_) | Folder::Fresh { .. });
        // Already on the board: open it rather than set it up again.
        let open = match &folder {
            Folder::Configured(Some(name)) if self.board.projects.iter().any(|p| p.name == *name) => Some(name.clone()),
            _ => None,
        };
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
                buttons::chip("new-project-open", format!("Open {name}")).px_3().py_1().on_click(cx.listener(move |shell, _, window, cx| {
                    shell.close_new_project(window, cx);
                    shell.show_project(name.clone(), cx);
                }))
            }))
            .child(Self::wizard_go("new-project-next", "Next →", ready).when(ready, |d| d.on_click(cx.listener(|shell, _, window, cx| shell.to_identity(window, cx)))));
        (page, footer)
    }

    fn identity_step(&self, wizard: &NewProject, cx: &mut Context<Self>) -> (Div, Div) {
        let typed = wizard.folder.read(cx).value().to_string();
        let folder = folder_of(&typed);
        let name = wizard.name.read(cx).value().trim().to_string();
        let agent = wizard.agent.read(cx).value().trim().to_string();
        let problem = if !name_ok(&name) {
            Some("The project's name: letters, digits, - and _.")
        } else if !name_ok(&agent) {
            Some("The agent's name: letters, digits, - and _.")
        } else if self.name_taken(&name, &folder) {
            Some("A project of the board has that name already.")
        } else {
            None
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
                            cx.notify();
                        })),
                )
                .child(div().pl_6().text_xs().text_color(p().muted).child(about))
        };
        let command = format!("aiball {}", init_args(&name, &agent, wizard.crew, wizard.private, wizard.no_claim).join(" "));
        let running = wizard.running;
        let go = problem.is_none() && !running;
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
                    .child(format!("In {}:", expand(typed.trim()).display()))
                    .child(div().text_color(p().text).child(command))
                    .child("writes .mcp.json (aiball's MCP server, for Claude Code) and .aiball.yaml (the project, the agent, its role)."),
            )
            .children(problem.map(|p_| div().text_sm().text_color(p().danger).child(p_)));
        let footer = div()
            .flex()
            .gap_3()
            .child(buttons::link("new-project-back", "← Back").on_click(cx.listener(|shell, _, _, cx| shell.wizard_to(Step::Folder, cx))))
            .child(Self::wizard_go("new-project-go", if running { "Setting it up…" } else { "Set it up" }, go).when(go, |d| d.on_click(cx.listener(|shell, _, _, cx| shell.set_it_up(cx)))));
        (page, footer)
    }

    fn done_step(&self, wizard: &NewProject, cx: &mut Context<Self>) -> (Div, Div) {
        let (ok, said) = match &wizard.outcome {
            Some(Ok(text)) => (true, text.clone()),
            Some(Err(text)) => (false, text.clone()),
            None => (false, String::new()),
        };
        let name = wizard.name.read(cx).value().trim().to_string();
        let page = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_color(if ok { p().success } else { p().danger }).child(if ok {
                format!("{name} is set up. aiball init said:")
            } else {
                "aiball init did not go through:".to_string()
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
            div().flex().gap_3().child(buttons::link("new-project-retry", "← Back").on_click(cx.listener(|shell, _, _, cx| shell.wizard_to(Step::Identity, cx))))
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
            .child(item(
                "3",
                "Give it work".into(),
                format!("\"+ New\" in the ticket panel files a ticket on {name}; the agent picks it up at its next wake, and its plans and questions come back as notifications."),
            ));
        let footer = div()
            .flex()
            .items_center()
            .gap_3()
            .child(buttons::link("new-project-done", "Close").on_click(cx.listener(|shell, _, window, cx| shell.close_new_project(window, cx))))
            .child(Self::wizard_go("new-project-start", "Start its first session", true).on_click(cx.listener(|shell, _, window, cx| shell.start_first_session(window, cx))));
        (page, footer)
    }
}

#[cfg(test)]
mod tests {
    use super::{Folder, folder_of, init_args, name_from, name_ok, project_in};
    use std::path::Path;

    #[test]
    fn a_folder_says_what_it_is() {
        let dir = std::env::temp_dir().join(format!("tvty-newproject-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(folder_of(""), Folder::Empty);
        assert_eq!(folder_of(&dir.join("nothing").display().to_string()), Folder::Missing);
        assert_eq!(folder_of(&dir.display().to_string()), Folder::Fresh { git: false });
        std::fs::write(dir.join(".aiball.yaml"), "consumer:\n  agent: app-claude\n  project: \"app\"\n").unwrap();
        assert_eq!(folder_of(&dir.display().to_string()), Folder::Configured(Some("app".into())));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn names_and_the_command() {
        assert_eq!(project_in("project: demo\n"), Some("demo".into()));
        assert_eq!(project_in("consumer:\n  agent: x\n"), None);
        assert!(name_ok("my-app_2") && !name_ok("my app") && !name_ok(""));
        assert_eq!(name_from(Path::new("/tmp/My App")), "My-App");
        assert_eq!(init_args("app", "app-claude", true, false, true).join(" "), "init --project app --agent app-claude --role crew --no-claim");
    }
}
