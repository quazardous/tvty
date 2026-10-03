//! A first start in a folder where Claude Code already has conversations
//! that no loop follows (someone worked there by hand): which one its
//! Claude takes up — the last one, or a new one. Asked once, on a sheet; a
//! folder with none, or one a loop already follows, starts without a word.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::ui::Named as _;

use super::Shell;
use crate::aiball::Conversation;
use crate::loops::{Resume, Start};
use crate::theme::p;
use crate::ui::buttons;

/// The question on screen: the start it waits for, and the conversation it
/// would take up.
pub(super) struct ResumeQuestion {
    pub start: Start,
    pub last: Conversation,
    pub others: usize,
}

impl Shell {
    /// Before a first start: the folder's conversations read (off the UI
    /// thread). Something to resume: asked; nothing, or an aiball that
    /// cannot say: started as before.
    pub(super) fn ask_resume(&mut self, start: Start, cx: &mut Context<Self>) {
        let aiball = self.aiball.clone();
        self.starting = Some(start.cwd.clone());
        cx.notify();
        cx.spawn(async move |this, cx| {
            let (cwd, crew) = (start.cwd.clone(), start.crew.then(|| start.agent.clone()).flatten());
            let found = cx.background_executor().spawn(async move { aiball.conversations(&cwd, crew.as_deref()) }).await;
            let _ = this.update(cx, |shell, cx| {
                shell.starting = None;
                let question = match found {
                    Ok(found) if found.tracked.is_none() => {
                        let free: Vec<Conversation> = found.conversations.into_iter().filter(|c| c.held_by.is_none()).collect();
                        let others = free.len().saturating_sub(1);
                        free.into_iter().next().map(|last| ResumeQuestion { start: start.clone(), last, others })
                    }
                    Ok(_) => None,
                    Err(error) => {
                        log::info!("start in {}: its conversations not read ({error:#}); started as before", start.cwd);
                        None
                    }
                };
                match question {
                    Some(question) => {
                        shell.resume_question = Some(question);
                        cx.notify();
                    }
                    None => shell.start_loop(Start { resume: Resume::Fresh, ..start }, cx),
                }
            });
        })
        .detach();
    }

    /// The answer: the last conversation taken up, or a new one.
    pub(super) fn answer_resume(&mut self, resume: bool, cx: &mut Context<Self>) {
        let Some(question) = self.resume_question.take() else { return };
        let choice = if resume { Resume::Conversation(question.last.id.clone()) } else { Resume::Fresh };
        self.start_loop(Start { resume: choice, ..question.start }, cx);
    }

    /// Esc, or ✕: nothing is started.
    pub(super) fn cancel_resume(&mut self, cx: &mut Context<Self>) -> bool {
        let open = self.resume_question.take().is_some();
        if open {
            cx.notify();
        }
        open
    }

    pub(super) fn resume_dialog(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let question = self.resume_question.as_ref()?;
        let who = question.start.agent.clone().unwrap_or_else(|| "its agent".into());
        let when = question
            .last
            .updated_at
            .as_deref()
            .and_then(crate::status::parse_time)
            .map(|at| format!("{} ago", crate::status::ago(crate::status::now().saturating_sub(at))))
            .unwrap_or_else(|| "some time ago".into());
        let said = question.last.first_prompt.clone().unwrap_or_default();
        let body = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_sm().text_color(p().muted).child(format!("The last one, {when}:")))
            .child(
                div()
                    .named("resume-last")
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(p().border)
                    .bg(p().bg)
                    .text_sm()
                    .child(if said.is_empty() { "(it says nothing yet)".to_string() } else { format!("“{said}”") }),
            )
            .when(question.others > 0, |d| {
                d.child(div().text_xs().text_color(p().muted).child(format!(
                    "{} older one{} too: /resume in Claude Code picks among them.",
                    question.others,
                    if question.others == 1 { "" } else { "s" }
                )))
            });
        let answers = div()
            .flex()
            .gap_2()
            .child(buttons::secondary("resume-new", "New conversation").on_click(cx.listener(|shell, _, _, cx| shell.answer_resume(false, cx))))
            .child(buttons::primary("resume-last-go", "Resume the last conversation").on_click(cx.listener(|shell, _, _, cx| shell.answer_resume(true, cx))));
        Some(
            crate::ui::sheet::Sheet::new("resume", format!("Start {who}"))
                .summary(format!(
                    "Claude Code already has conversations in {}, none of them a loop's: which one does {who} take up?",
                    super::loopstabs::home_short(&question.start.cwd)
                ))
                .body(body)
                .answers(answers)
                .on_close(cx.listener(|shell, _, _, cx| {
                    shell.cancel_resume(cx);
                }))
                .render(),
        )
    }
}
