//! A setting's wizard, over the window: what aiball would ask and do
//! (`project.setting_wizard_preview`), the questions answered, then the run
//! followed step by step as aiball says it (`wizard.<run>.state`). aiball
//! runs it: closing the card does not stop it, and the agent bar marks its
//! folder while it runs.

use std::collections::HashMap;

use gpui_kit::component::Disableable as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::stepper::{Stepper, StepperItem};
use gpui_kit::component::button::Button;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use serde_json::Value;

use super::Shell;
use crate::activity::{self, Activity};
use crate::theme::p;
use crate::ui::Named as _;
use crate::ui::buttons::{self, Look as _};
use crate::wizard::{self, Preview, Run};

/// The wizard shown: a setting of a folder, toward a value.
pub(super) struct WizardCard {
    cwd: String,
    key: String,
    value: Value,
    preview: Option<Result<Preview, String>>,
    /// A `choose` question's answer, by its id; its default otherwise.
    chosen: HashMap<String, String>,
    confirmed: bool,
    starting: bool,
    /// The run started: followed on the bus, its start's answer until then.
    run: Option<Run>,
    /// What refused the start or the cancel.
    error: Option<String>,
    /// The value still to choose: the card opens on it.
    choosing: Option<Choosing>,
}

/// A value to choose before aiball is asked: the setting's options, a path
/// typed, or a dir another folder uses.
pub(super) struct Choosing {
    title: String,
    kind: String,
    options: Vec<String>,
    current: String,
    input: Option<Entity<InputState>>,
    /// A `path` value: the folders, browsed in tvty (the field filters
    /// them or takes a path).
    picker: Option<crate::ui::filepicker::FilePicker>,
    suggestions: Vec<(Value, String)>,
}

impl Shell {
    /// Opens a setting's wizard: aiball is asked what it would do.
    pub(super) fn open_wizard(&mut self, cwd: String, key: String, value: Value, cx: &mut Context<Self>) {
        // A run of the folder still under way: followed, not started again.
        let running = self.live.wizard_runs().into_iter().find(|r| r.cwd == cwd && r.running());
        self.wizard = Some(WizardCard {
            cwd: cwd.clone(),
            key: key.clone(),
            value: value.clone(),
            preview: None,
            chosen: HashMap::new(),
            confirmed: false,
            starting: false,
            run: running,
            error: None,
            choosing: None,
        });
        cx.notify();
        if self.wizard.as_ref().is_some_and(|w| w.run.is_some()) {
            return;
        }
        self.ask_wizard_preview(value, cx);
    }

    /// Opens a setting's wizard on the choice of its value: the options
    /// page's wand.
    pub(super) fn open_wizard_choosing(&mut self, cwd: String, setting: crate::aiball::FolderSetting, window: &mut Window, cx: &mut Context<Self>) {
        if self.wizard_running(&cwd).is_some() {
            self.open_wizard(cwd, setting.key, Value::Null, cx);
            return;
        }
        let current = match setting.value.as_str() {
            Some(text) => crate::shell::home_short(text),
            None => setting.value.to_string(),
        };
        let path_kind = setting.path_kind();
        let input = path_kind.is_some().then(|| {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder(crate::t!("wizard-type-path")));
            cx.subscribe_in(&input, window, |shell: &mut Self, input, event: &InputEvent, window, cx| {
                let typed = input.read(cx).value().to_string();
                let Some(picker) = shell.wizard.as_mut().and_then(|w| w.choosing.as_mut()).and_then(|c| c.picker.as_mut()) else { return };
                match event {
                    InputEvent::Change => {
                        picker.text = typed;
                        picker.error = None;
                    }
                    // Enter: into the folder typed or the first one shown.
                    InputEvent::PressEnter { .. } => {
                        picker.text = typed;
                        picker.activate();
                        let left = picker.text.clone();
                        input.update(cx, |input, cx| input.set_value(left, window, cx));
                    }
                    _ => return,
                }
                cx.notify();
            })
            .detach();
            input
        });
        let suggestions = if setting.key == "claude.config_dir" { self.config_dir_suggestions(&cwd) } else { Vec::new() };
        // Browsed from the current value's parent: dirs of config are
        // hidden ones, shown here.
        let picker = path_kind.map(|kind| {
            let from = setting.value.as_str().map(std::path::PathBuf::from).and_then(|p| p.parent().map(|d| d.to_path_buf()));
            let mut picker = crate::ui::filepicker::FilePicker::new(kind, from);
            picker.show_hidden = true;
            picker
        });
        let title = wizard::title_said(&setting.key, setting.wizard.as_ref().map_or(setting.key.as_str(), |w| w.title.as_str()));
        self.wizard = Some(WizardCard {
            cwd,
            key: setting.key.clone(),
            value: Value::Null,
            preview: None,
            chosen: HashMap::new(),
            confirmed: false,
            starting: false,
            run: None,
            error: None,
            choosing: Some(Choosing { title, kind: setting.kind.clone(), options: setting.options.clone(), current, input, picker, suggestions }),
        });
        cx.notify();
    }

    /// The value chosen: aiball is asked what it would do for it.
    fn ask_wizard_preview(&mut self, value: Value, cx: &mut Context<Self>) {
        let Some(card) = self.wizard.as_mut() else { return };
        card.value = value.clone();
        card.choosing = None;
        card.preview = None;
        let (cwd, key) = (card.cwd.clone(), card.key.clone());
        cx.notify();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let read = cx.background_executor().spawn(async move { wizard::preview(&aiball, &cwd, &key, &value).map_err(|e| wizard::refusal_said(&e)) }).await;
            let _ = this.update(cx, |shell, cx| {
                if let Some(card) = shell.wizard.as_mut().filter(|c| c.run.is_none()) {
                    card.preview = Some(read);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn start_wizard(&mut self, cx: &mut Context<Self>) {
        let Some(card) = self.wizard.as_mut() else { return };
        let Some(Ok(preview)) = card.preview.as_ref() else { return };
        let Some(answers) = wizard::answers(&preview.questions, &card.chosen, card.confirmed) else { return };
        card.starting = true;
        card.error = None;
        let (aiball, cwd, key, value) = (self.aiball.clone(), card.cwd.clone(), card.key.clone(), card.value.clone());
        cx.notify();
        cx.spawn(async move |this, cx| {
            let started = cx.background_executor().spawn(async move { wizard::start(&aiball, &cwd, &key, &value, &answers) }).await;
            let _ = this.update(cx, |shell, cx| {
                if let Some(card) = shell.wizard.as_mut() {
                    card.starting = false;
                    match started {
                        Ok(run) => card.run = Some(run),
                        Err(error) => card.error = Some(wizard::refusal_said(&error)),
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn cancel_wizard_run(&mut self, run: String, cx: &mut Context<Self>) {
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let done = cx.background_executor().spawn(async move { wizard::cancel(&aiball, &run) }).await;
            let _ = this.update(cx, |shell, cx| {
                if let (Err(error), Some(card)) = (done, shell.wizard.as_mut()) {
                    card.error = Some(format!("{error:#}"));
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The card's run as aiball says it now.
    fn shown_run(&self, card: &WizardCard) -> Option<Run> {
        let started = card.run.as_ref()?;
        Some(self.live.wizard_runs().into_iter().find(|r| r.run == started.run).unwrap_or_else(|| started.clone()))
    }

    /// The runs of a folder under way: the agent bar marks it.
    pub(super) fn wizard_running(&self, cwd: &str) -> Option<Run> {
        self.live.wizard_runs().into_iter().find(|r| r.cwd == cwd && r.running())
    }

    /// A run seen ending: the dirs and the options are read again, and its
    /// end is said once.
    pub(super) fn wizards_ended(&mut self, cx: &mut Context<Self>) {
        if self.config_dirs.is_empty() {
            self.read_config_dirs(cx);
        }
        let runs = self.live.wizard_runs();
        let mut ended = Vec::new();
        for run in &runs {
            let was = self.wizard_seen.insert(run.run.clone(), run.status.clone());
            if was.as_deref() == Some("running") && !run.running() {
                ended.push(run.clone());
            }
        }
        if ended.is_empty() {
            return;
        }
        for run in ended {
            let title = wizard::title_said(&run.key, &run.key);
            let what = format!("{} · {}", title, crate::shell::home_short(&run.cwd));
            let said = wizard::status_said(&run.status);
            let activity = if run.status == "done" {
                Activity::done(None, format!("{what}: {said}"))
            } else {
                let why = run.steps.iter().find(|s| s.status.as_deref() == Some("failed")).and_then(wizard::step_news).unwrap_or(said);
                Activity::failed(None, &crate::t!("wizard-failed", title = what), why)
            };
            activity::publish(cx, activity);
        }
        let _ = self.refresh_now.unbounded_send(());
        self.read_config_dirs(cx);
    }

    pub(super) fn wizard_view(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let card = self.wizard.as_ref()?;
        let muted = |text: String| div().text_xs().text_color(p().muted).child(text);
        let title = match (card.choosing.as_ref(), card.preview.as_ref()) {
            (Some(choosing), _) => choosing.title.clone(),
            (None, Some(Ok(preview))) => wizard::title_said(&preview.key, &preview.title),
            _ => wizard::title_said(&card.key, &card.key),
        };
        // Closed: the options page, open on a project, reads its folders again.
        let close = |id: &'static str, label: String, cx: &mut Context<Self>| {
            buttons::secondary(id, label).on_click(cx.listener(|shell, _, window, cx| shell.close_wizard(window, cx)))
        };
        let (phase, body, answers): (usize, Div, Div) = if let Some(choosing) = card.choosing.as_ref() {
            let (body, next) = self.choosing_view(choosing, cx);
            (0, body, div().flex().gap_2().child(close("wizard-cancel", crate::t!("wizard-cancel"), cx)).children(next))
        } else if let Some(run) = self.shown_run(card) {
            let mut body = div().flex().flex_col().gap_3().child(div().font_weight(FontWeight::BOLD).child(wizard::status_said(&run.status))).child(steps_view(&run.steps));
            if let Some(fallback) = run.fallback.as_ref().filter(|f| f.ran) {
                body = body.children(wizard::fallback_said(fallback.code.as_deref(), &fallback.details, fallback.said.as_deref()).map(|t| div().text_color(p().warning).child(t)));
            }
            body = body.children(card.error.clone().map(|e| div().text_color(p().danger).child(e)));
            let stop = run.cancellable().then(|| {
                let id = run.run.clone();
                buttons::secondary("wizard-stop", crate::t!("wizard-stop")).on_click(cx.listener(move |shell, _, _, cx| shell.cancel_wizard_run(id.clone(), cx)))
            });
            (2, body, div().flex().gap_2().children(stop).child(close("wizard-close", crate::t!("wizard-close"), cx)))
        } else {
            match card.preview.as_ref() {
                None => (1, div().child(muted(crate::t!("wizard-loading"))), div().child(close("wizard-close", crate::t!("wizard-close"), cx))),
                Some(Err(error)) => (1, div().text_color(p().danger).child(error.clone()), div().child(close("wizard-close", crate::t!("wizard-close"), cx))),
                Some(Ok(preview)) => {
                    let body = self.questions_view(card, preview, cx);
                    let ready = wizard::answers(&preview.questions, &card.chosen, card.confirmed).is_some() && !card.starting;
                    let start = buttons::primary("wizard-start", if card.starting { "…".to_string() } else { crate::t!("wizard-start") })
                        .disabled(!ready)
                        .on_click(cx.listener(|shell, _, _, cx| shell.start_wizard(cx)));
                    (1, body, div().flex().gap_2().child(close("wizard-cancel", crate::t!("wizard-cancel"), cx)).child(start))
                }
            }
        };
        let phases = [crate::t!("wizard-phase-choose"), crate::t!("wizard-phase-what"), crate::t!("wizard-phase-run")];
        let steps = Stepper::new("wizard-steps").selected_index(phase).items(phases.into_iter().map(|t| StepperItem::new().child(t)));
        Some(
            crate::ui::sheet::Sheet::new("wizard", title)
                .summary(crate::shell::home_short(&card.cwd))
                .steps(steps)
                .body(body)
                .answers(answers)
                .on_close(cx.listener(|shell, _, window, cx| shell.close_wizard(window, cx)))
                .render(),
        )
    }

    /// ✕ Esc: the card goes, the run (if any) goes on in aiball.
    pub(super) fn close_wizard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.wizard = None;
        self.load_project_settings(window, cx);
        cx.notify();
    }

    /// The questions, all before the start, then what runs and what happens
    /// if it stops early.
    fn questions_view(&self, card: &WizardCard, preview: &Preview, cx: &mut Context<Self>) -> Div {
        let muted = |text: String| div().text_xs().text_color(p().muted).child(text);
        let mut body = div().flex().flex_col().gap_3();
        for q in &preview.questions {
            let id = q.id.clone();
            match q.kind.as_str() {
                "confirm" => {
                    let on = card.confirmed;
                    body = body.child(
                        div().flex().flex_col().items_start().gap_1().child(div().child(wizard::question_said(q, &preview.loops))).child(
                            buttons::chip(format!("wizard-q-{id}"), format!("{} {}", if on { "☑" } else { "☐" }, crate::t!("wizard-confirm")))
                                .py_0p5()
                                .chosen(on)
                                .on_click(cx.listener(move |shell, _, _, cx| {
                                    if let Some(card) = shell.wizard.as_mut() {
                                        card.confirmed = !card.confirmed;
                                    }
                                    cx.notify();
                                })),
                        ),
                    );
                }
                _ => {
                    let current = card.chosen.get(&q.id).cloned().or_else(|| q.default.clone());
                    let mut row = div().flex().flex_col().items_start().w_full().gap_1();
                    for a in &q.options {
                        let (qid, key) = (q.id.clone(), a.key.clone());
                        let on = current.as_deref() == Some(a.key.as_str());
                        row = row.child(
                            buttons::chip(format!("wizard-q-{}-{}", q.id, a.key), div().min_w_0().whitespace_normal().child(wizard::answer_said(q, a)))
                                .max_w_full()
                                .py_0p5()
                                .text_sm()
                                .chosen(on)
                                .on_click(cx.listener(move |shell, _, _, cx| {
                                    if let Some(card) = shell.wizard.as_mut() {
                                        card.chosen.insert(qid.clone(), key.clone());
                                    }
                                    cx.notify();
                                })),
                        );
                    }
                    body = body.child(div().flex().flex_col().items_start().gap_1().child(div().child(wizard::question_said(q, &preview.loops))).child(row));
                }
            }
        }
        let busy: Vec<String> = preview.loops.iter().filter(|l| l.busy).map(|l| l.agent.clone().unwrap_or_else(|| l.name.clone())).collect();
        body = body.child(muted(crate::t!("wizard-steps"))).child(steps_view(&preview.steps));
        if !busy.is_empty() {
            body = body.child(muted(format!("{}: {}", crate::t!("wizard-busy-mark"), busy.join(", "))));
        }
        body = body.children(wizard::fallback_said(preview.fallback_code.as_deref(), &preview.fallback_facts, preview.fallback.as_deref()).map(muted));
        body.children(card.error.clone().map(|e| div().text_color(p().danger).child(e)))
    }
}

/// The steps, each with its state and what it says, the point of no
/// return marked.
fn steps_view(steps: &[wizard::Step]) -> Div {
    let mut list = div().flex().flex_col().gap_1();
    for (i, step) in steps.iter().enumerate() {
        let (glyph, colour) = match step.status.as_deref() {
            Some("running") => ("…", p().accent),
            Some("done") => ("✓", p().success),
            Some("failed") => ("✗", p().danger),
            Some("skipped") => ("–", p().muted),
            Some(_) => ("○", p().muted),
            None => ("", p().muted),
        };
        let mark = if glyph.is_empty() { format!("{}.", i + 1) } else { glyph.to_string() };
        list = list.child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(18.)).flex_none().text_color(colour).child(mark))
                        .child(wizard::step_said(step))
                        .when(step.point_of_no_return, |d| d.child(div().text_xs().text_color(p().warning).child(crate::t!("wizard-no-return")))),
                )
                .children(wizard::step_news(step).filter(|_| step.status.is_some()).map(|news| div().pl(px(26.)).text_xs().text_color(p().muted).child(news))),
        );
    }
    list
}

impl Shell {
    /// The value to choose: the current one, the setting's options, a path
    /// typed (Enter) or picked in the system's folder dialog, the dirs other
    /// folders use. Answers the page and its Next.
    fn choosing_view(&self, choosing: &Choosing, cx: &mut Context<Self>) -> (Div, Option<Button>) {
        let muted = |text: String| div().text_xs().text_color(p().muted).child(text);
        let mut body = div().flex().flex_col().gap_3().child(div().child(crate::t!("wizard-now", value = choosing.current.clone())));
        if choosing.kind == "enum" {
            let mut row = div().flex().flex_wrap().gap_1();
            for option in &choosing.options {
                let value = Value::String(option.clone());
                row = row.child(
                    buttons::chip(format!("wizard-choose-{option}"), option.clone())
                        .py_0p5()
                        .text_sm()
                        .on_click(cx.listener(move |shell, _, _, cx| shell.ask_wizard_preview(value.clone(), cx))),
                );
            }
            body = body.child(row);
        }
        let mut next = None;
        if let (Some(input), Some(picker)) = (choosing.input.as_ref(), choosing.picker.as_ref()) {
            body = body.child(self.picker_view(input, picker, cx));
            let label = if picker.kind == crate::ui::filepicker::Kind::File { crate::t!("wizard-choose-file") } else { crate::t!("wizard-choose-folder") };
            next = Some(buttons::primary("wizard-next", label).on_click(cx.listener(|shell, _, _, cx| {
                let chosen = shell.wizard.as_mut().and_then(|w| w.choosing.as_mut()).and_then(|c| c.picker.as_mut()).and_then(|p| p.accept());
                if let Some(dir) = chosen {
                    shell.ask_wizard_preview(Value::String(dir.display().to_string()), cx);
                }
            })));
        }
        if !choosing.suggestions.is_empty() {
            body = body.child(muted(crate::t!("wizard-others")));
            let mut list = div().flex().flex_col().items_start().w_full().gap_1();
            for (i, (value, label)) in choosing.suggestions.iter().enumerate() {
                let value = value.clone();
                list = list.child(
                    buttons::chip(format!("wizard-suggest-{i}"), div().min_w_0().truncate().child(label.clone()))
                        .max_w_full()
                        .py_0p5()
                        .text_sm()
                        .on_click(cx.listener(move |shell, _, _, cx| shell.ask_wizard_preview(value.clone(), cx))),
                );
            }
            body = body.child(list);
        }
        (body, next)
    }

    /// The system's dialog, for who prefers it to the picker: the folder
    /// chosen is shown in the picker, a file chosen is the value.
    fn browse_wizard_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let file = self.wizard.as_ref().and_then(|w| w.choosing.as_ref()).and_then(|c| c.picker.as_ref()).is_some_and(|p| p.kind == crate::ui::filepicker::Kind::File);
        let chosen = cx.prompt_for_paths(PathPromptOptions { files: file, directories: !file, multiple: false, prompt: Some(crate::t!("wizard-browse").into()) });
        let handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = chosen.await else { return };
            let Some(path) = paths.into_iter().next() else { return };
            let _ = handle.update(cx, |_, window, cx| {
                let _ = this.update(cx, |shell, cx| {
                    let input = shell.wizard.as_ref().and_then(|w| w.choosing.as_ref()).and_then(|c| c.input.clone());
                    if path.is_file() {
                        shell.ask_wizard_preview(Value::String(path.display().to_string()), cx);
                        return;
                    }
                    if let Some(picker) = shell.wizard.as_mut().and_then(|w| w.choosing.as_mut()).and_then(|c| c.picker.as_mut()) {
                        picker.go(path.clone());
                    }
                    if let Some(input) = input {
                        input.update(cx, |input, cx| input.set_value(String::new(), window, cx));
                    }
                    cx.notify();
                });
            });
        })
        .detach();
    }
}

impl Shell {
    /// The in-app folder picker: the folder shown and the way up, the field
    /// (a filter, or a path; Enter goes in), its sub-folders, and the
    /// system's dialog for who prefers it.
    fn picker_view(&self, input: &Entity<InputState>, picker: &crate::ui::filepicker::FilePicker, cx: &mut Context<Self>) -> Div {
        let mut picker = picker.clone();
        let shown = picker.visible();
        let mut col = div().flex().flex_col().gap_2();
        // Where it is, the way up, home.
        let places = div()
            .flex()
            .items_center()
            .gap_2()
            .child(buttons::secondary("wizard-picker-up", "↑").tooltip(crate::t!("wizard-picker-up")).on_click(cx.listener(|shell, _, window, cx| {
                shell.wizard_picker(window, cx, |p| p.up());
            })))
            .child(buttons::secondary("wizard-picker-home", "~").tooltip(crate::t!("wizard-picker-home")).on_click(cx.listener(|shell, _, window, cx| {
                shell.wizard_picker(window, cx, |p| {
                    if let Some(home) = crate::ui::filepicker::home() {
                        p.go(home);
                    }
                });
            })))
            .child(div().flex_1().min_w_0().truncate().font_weight(FontWeight::BOLD).child(crate::ui::filepicker::shown(&picker.dir)))
            .child(buttons::secondary("wizard-browse", crate::t!("wizard-browse-system")).on_click(cx.listener(|shell, _, window, cx| shell.browse_wizard_folder(window, cx))));
        col = col.child(places).child(Input::new(input).small());
        // A path typed: what it may complete to, a click away.
        let completions = picker.completions();
        if !completions.is_empty() {
            let mut row = div().flex().flex_wrap().gap_1();
            for (i, done) in completions.into_iter().take(8).enumerate() {
                let label = done.rsplit(['/', std::path::MAIN_SEPARATOR]).find(|p| !p.is_empty()).unwrap_or(&done).to_string();
                row = row.child(buttons::chip(format!("wizard-complete-{i}"), format!("{label}/")).py_0p5().text_xs().on_click(cx.listener(move |shell, _, window, cx| {
                    let done = done.clone();
                    shell.wizard_picker(window, cx, move |p| p.text = done);
                })));
            }
            col = col.child(row);
        }
        col = col.children(picker.error.clone().map(|e| div().text_xs().text_color(p().danger).child(crate::t!("wizard-picker-missing", path = e))));
        let mut list = div().flex().flex_col().border_1().border_color(p().border).rounded_md().py_1();
        if shown.is_empty() {
            list = list.child(div().px_2().text_xs().text_color(p().muted).child(crate::t!("wizard-picker-empty")));
        }
        for (i, entry) in shown.into_iter().take(200).enumerate() {
            let path = entry.path.clone();
            let entry = entry.clone();
            list = list.child(
                div()
                    .named(SharedString::from(format!("wizard-picker-{i}")))
                    .px_2()
                    .py_0p5()
                    .text_sm()
                    .cursor_pointer()
                    .hover(|d| d.bg(p().hover))
                    .child(if entry.is_dir { format!("{}/", entry.name) } else { entry.name.clone() })
                    .on_click(cx.listener(move |shell, _, window, cx| {
                        // A folder: into it; a file: chosen.
                        if entry.is_dir {
                            let path = path.clone();
                            shell.wizard_picker(window, cx, move |p| p.go(path));
                        } else {
                            shell.ask_wizard_preview(Value::String(path.display().to_string()), cx);
                        }
                    })),
            );
        }
        col.child(list)
    }

    /// A gesture on the picker: done, its field emptied as the folder changes.
    fn wizard_picker(&mut self, window: &mut Window, cx: &mut Context<Self>, gesture: impl FnOnce(&mut crate::ui::filepicker::FilePicker)) {
        let Some(choosing) = self.wizard.as_mut().and_then(|w| w.choosing.as_mut()) else { return };
        let Some(picker) = choosing.picker.as_mut() else { return };
        gesture(picker);
        let text = picker.text.clone();
        if let Some(input) = choosing.input.clone() {
            input.update(cx, |input, cx| input.set_value(text, window, cx));
        }
        cx.notify();
    }
}
