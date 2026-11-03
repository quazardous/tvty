//! A setting's wizard, over the window: what aiball would ask and do
//! (`project.setting_wizard_preview`), the questions answered, then the run
//! followed step by step as aiball says it (`wizard.<run>.state`). aiball
//! runs it: closing the card does not stop it, and the agent bar marks its
//! folder while it runs.

use std::collections::HashMap;

use gpui_kit::component::Disableable as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use serde_json::Value;

use super::Shell;
use crate::activity::{self, Activity};
use crate::theme::p;
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
        });
        cx.notify();
        if self.wizard.as_ref().is_some_and(|w| w.run.is_some()) {
            return;
        }
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let read = cx.background_executor().spawn(async move { wizard::preview(&aiball, &cwd, &key, &value).map_err(|e| format!("{e:#}")) }).await;
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
                        Err(error) => card.error = Some(format!("{error:#}")),
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
        let title = match card.preview.as_ref() {
            Some(Ok(preview)) => wizard::title_said(&preview.key, &preview.title),
            _ => wizard::title_said(&card.key, &card.key),
        };
        let mut body = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_lg().font_weight(FontWeight::BOLD).child(title))
            .child(muted(crate::shell::home_short(&card.cwd)));
        // Closed: the options page, open on a project, reads its folders again.
        let close = buttons::secondary("wizard-close", crate::t!("wizard-close")).on_click(cx.listener(|shell, _, window, cx| {
            shell.wizard = None;
            shell.load_project_settings(window, cx);
            cx.notify();
        }));
        if let Some(run) = self.shown_run(card) {
            body = body.child(div().font_weight(FontWeight::BOLD).child(wizard::status_said(&run.status)));
            body = body.child(steps_view(&run.steps));
            if let Some(fallback) = run.fallback.as_ref().filter(|f| f.ran) {
                body = body.children(wizard::fallback_said(fallback.code.as_deref(), &fallback.details, fallback.said.as_deref()).map(|t| div().text_color(p().warning).child(t)));
            }
            body = body.children(card.error.clone().map(|e| div().text_color(p().danger).child(e)));
            let stop = run.cancellable().then(|| {
                let id = run.run.clone();
                buttons::secondary("wizard-stop", crate::t!("wizard-stop")).on_click(cx.listener(move |shell, _, _, cx| shell.cancel_wizard_run(id.clone(), cx)))
            });
            body = body.child(div().flex().justify_end().gap_2().children(stop).child(close));
            return Some(super::quit::dialog(body).into_any_element());
        }
        let preview = match card.preview.as_ref() {
            None => return Some(super::quit::dialog(body.child(muted(crate::t!("wizard-loading"))).child(div().flex().justify_end().child(close))).into_any_element()),
            Some(Err(error)) => {
                return Some(super::quit::dialog(body.child(div().text_color(p().danger).child(error.clone())).child(div().flex().justify_end().child(close))).into_any_element());
            }
            Some(Ok(preview)) => preview,
        };
        // The questions, all before the start.
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
                    let mut row = div().flex().flex_wrap().gap_1();
                    for a in &q.options {
                        let (qid, key) = (q.id.clone(), a.key.clone());
                        let on = current.as_deref() == Some(a.key.as_str());
                        row = row.child(
                            buttons::chip(format!("wizard-q-{}-{}", q.id, a.key), wizard::answer_said(q, a))
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
        // What runs then, and what happens if it stops early.
        let busy: Vec<String> = preview.loops.iter().filter(|l| l.busy).map(|l| l.agent.clone().unwrap_or_else(|| l.name.clone())).collect();
        body = body.child(muted(crate::t!("wizard-steps"))).child(steps_view(&preview.steps));
        if !busy.is_empty() {
            body = body.child(muted(format!("{}: {}", crate::t!("wizard-busy-mark"), busy.join(", "))));
        }
        body = body.children(wizard::fallback_said(preview.fallback_code.as_deref(), &preview.fallback_facts, preview.fallback.as_deref()).map(muted));
        body = body.children(card.error.clone().map(|e| div().text_color(p().danger).child(e)));
        let ready = wizard::answers(&preview.questions, &card.chosen, card.confirmed).is_some() && !card.starting;
        let start = buttons::primary("wizard-start", if card.starting { "…".to_string() } else { crate::t!("wizard-start") })
            .disabled(!ready)
            .on_click(cx.listener(|shell, _, _, cx| shell.start_wizard(cx)));
        let cancel = buttons::secondary("wizard-cancel", crate::t!("wizard-cancel")).on_click(cx.listener(|shell, _, _, cx| {
            shell.wizard = None;
            cx.notify();
        }));
        body = body.child(div().flex().justify_end().gap_2().child(cancel).child(start));
        Some(super::quit::dialog(body).into_any_element())
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
