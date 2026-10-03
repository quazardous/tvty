//! The 📢: what steers a project's agents, as aiball keeps it — its standing
//! instruction (read at the head of every wake), its wake focus (only these
//! tickets wake them, until a date) — and a message typed into every
//! running agent loop at once, holding them or not. The same as aiball's
//! own page, through its bus. The standings themselves are the kernel's
//! (`crate::kernel::standing`): this is the view.


use crate::ui::Named as _;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::input::{Input, InputState, Textarea, TextareaState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Shell;
use crate::activity::{self, Activity};
use crate::aiball::LoopHold;
use crate::notify::Kind;
use crate::theme::p;
use crate::ui::buttons;

/// Sent when the message box is left empty, as aiball's page does.
const DEFAULT_MESSAGE: &str = "I'll be back. Until then, stabilise: finish or park your current step, commit what is green, \
     revert what is not, post the ticket state (then: continue or handback), and stop.";

/// The instructions given last, offered again.
const HISTORY: usize = 6;

/// The popover, while it is open.
pub(super) struct Megaphone {
    /// A project's standing (its 📢), or none: the message to every agent,
    /// which is the whole board's.
    project: Option<String>,
    prompt: Entity<InputState>,
    tickets: Entity<InputState>,
    until: Entity<InputState>,
    message: Entity<TextareaState>,
    busy: bool,
    error: Option<String>,
    /// What the last message or release did, said.
    said: Option<String>,
}

/// `2026-09-30 18:00` (local) as the ISO date aiball takes.
fn until_iso(typed: &str) -> Result<Option<String>, String> {
    use chrono::TimeZone as _;
    let typed = typed.trim();
    if typed.is_empty() {
        return Ok(None);
    }
    let naive = chrono::NaiveDateTime::parse_from_str(typed, "%Y-%m-%d %H:%M")
        .or_else(|_| chrono::NaiveDate::parse_from_str(typed, "%Y-%m-%d").map(|d| d.and_hms_opt(23, 59, 0).unwrap_or_default()))
        .map_err(|_| crate::t!("megaphone-not-a-date", typed = typed))?;
    let local = chrono::Local.from_local_datetime(&naive).single().ok_or_else(|| crate::t!("megaphone-not-a-time", typed = typed))?;
    Ok(Some(local.to_utc().to_rfc3339()))
}

/// An ISO date as the box shows it: local, `2026-09-30 18:00`.
fn until_local(iso: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(iso)
        .map(|d| d.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default()
}

/// What a message or a release did, loop by loop, said in one line.
fn results_said(results: &[LoopHold]) -> String {
    if results.is_empty() {
        return crate::t!("megaphone-no-loop");
    }
    let names = |f: &dyn Fn(&LoopHold) -> bool| results.iter().filter(|r| f(r)).map(|r| r.consumer_id.clone()).collect::<Vec<_>>();
    let mut said = Vec::new();
    for (label, list) in [
        ("megaphone-typed-into", names(&|r| r.prompt.as_deref() == Some("delivered"))),
        ("megaphone-queued-for", names(&|r| r.prompt.as_deref() == Some("spooled"))),
        ("megaphone-held", names(&|r| r.hold.as_deref() == Some("armed"))),
        ("megaphone-released", names(&|r| r.hold.as_deref() == Some("released"))),
    ] {
        if !list.is_empty() {
            said.push(crate::t!(label, names = list.join(", ")));
        }
    }
    for r in results.iter().filter(|r| r.hold.as_deref() == Some("failed")) {
        said.push(format!("{}: {}", r.consumer_id, r.hold_error.clone().unwrap_or_else(|| crate::t!("megaphone-hold-not-applied"))));
    }
    said.join(" · ")
}

impl Shell {
    /// The 📢 of the ticket panel's project: its standing instruction and
    /// wake focus. No project shown: said, nothing opens.
    pub(super) fn open_megaphone(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(project) = self.panel_scope().map(|s| s.project) else {
            activity::publish(cx, Activity::news("tvty", Kind::Info, None, crate::t!("megaphone-no-project")));
            return;
        };
        self.open_popover(Some(project), window, cx);
    }

    /// The message to every running agent loop: the whole board's.
    pub(super) fn open_message_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_popover(None, window, cx);
    }

    fn open_popover(&mut self, project: Option<String>, window: &mut Window, cx: &mut Context<Self>) {
        self.help_menu = false;
        let standing = project.as_ref().and_then(|p| crate::kernel::standing::get(cx, p)).unwrap_or_default();
        let field = |value: Option<String>, placeholder: &str, window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| {
                let mut state = InputState::new(window, cx).placeholder(placeholder.to_string());
                if let Some(value) = value {
                    state.set_value(value, window, cx);
                }
                state
            })
        };
        let prompt = field(standing.standing_prompt.clone(), &crate::t!("megaphone-prompt-placeholder"), window, cx);
        let tickets = field(standing.focus_tickets.clone(), &crate::t!("megaphone-tickets-placeholder"), window, cx);
        let until = field(standing.focus_until.as_deref().map(until_local), &crate::t!("megaphone-until-placeholder"), window, cx);
        let message = cx.new(|cx| TextareaState::new(window, cx).placeholder(DEFAULT_MESSAGE).auto_grow(3, 8));
        // The first box shown takes the keys: the instruction, or the
        // message.
        let focus = if project.is_some() { prompt.read(cx).focus_handle(cx) } else { message.read(cx).focus_handle(cx) };
        window.focus(&focus, cx);
        self.megaphone = Some(Megaphone { project: project.clone(), prompt, tickets, until, message, busy: false, error: None, said: None });
        // The focus's tickets and end, which the list does not give: read,
        // put in their boxes unless typed in meanwhile.
        if let Some(project) = project {
            let aiball = self.aiball.clone();
            cx.spawn_in(window, async move |this, cx| {
                let read = cx.background_executor().spawn(async move { aiball.standing(&project) }).await;
                let Ok(standing) = read else { return };
                let _ = this.update_in(cx, |shell, window, cx| {
                    if let Some(m) = shell.megaphone.as_ref().filter(|m| m.project.as_deref() == Some(standing.project.as_str())) {
                        let fill = |e: &Entity<InputState>, value: Option<String>, window: &mut Window, cx: &mut Context<Shell>| {
                            if e.read(cx).value().is_empty() {
                                e.update(cx, |e, cx| e.set_value(value.unwrap_or_default(), window, cx));
                            }
                        };
                        fill(&m.tickets.clone(), standing.focus_tickets.clone(), window, cx);
                        fill(&m.until.clone(), standing.focus_until.as_deref().map(until_local), window, cx);
                    }
                    crate::kernel::standing::apply(cx, standing);
                });
            })
            .detach();
        }
        cx.notify();
    }

    pub(super) fn close_megaphone(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.megaphone = None;
        self.focus_terminal(window, cx);
        cx.notify();
    }

    /// Saves the instruction and the focus as typed (`clear`: both removed).
    fn save_standing(&mut self, clear: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(m) = self.megaphone.as_mut() else { return };
        let Some(project) = m.project.clone() else { return };
        if clear {
            for e in [&m.prompt, &m.tickets, &m.until] {
                e.update(cx, |e, cx| e.set_value("", window, cx));
            }
        }
        let text = |e: &Entity<InputState>, cx: &App| Some(e.read(cx).value().trim().to_string()).filter(|t| !t.is_empty() && !clear);
        let (prompt, tickets) = (text(&m.prompt, cx), text(&m.tickets, cx));
        let until = match (clear, until_iso(&m.until.read(cx).value())) {
            (true, _) => None,
            (false, Ok(until)) => until,
            (false, Err(error)) => {
                m.error = Some(error);
                cx.notify();
                return;
            }
        };
        m.busy = true;
        m.error = None;
        cx.notify();
        if let Some(prompt) = prompt.clone() {
            let history = &mut self.settings.workspace.standing_history;
            history.retain(|h| *h != prompt);
            history.insert(0, prompt);
            history.truncate(HISTORY);
            self.settings.save(cx);
        }
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let done = {
                let project = project.clone();
                cx.background_executor()
                    .spawn(async move { aiball.set_standing(&project, prompt.as_deref(), tickets.as_deref(), until.as_deref()) })
                    .await
            };
            let _ = this.update(cx, |shell, cx| {
                if let Some(m) = shell.megaphone.as_mut() {
                    m.busy = false;
                }
                match done {
                    Ok(standing) => {
                        let said = match (&standing.standing_prompt, standing.focus_active) {
                            (None, false) => crate::t!("megaphone-said-nothing", project = project.clone()),
                            (Some(prompt), _) => crate::t!("megaphone-said-prompt", project = project.clone(), prompt = prompt.clone()),
                            (None, true) => crate::t!("megaphone-said-focus", project = project.clone()),
                        };
                        activity::publish(cx, Activity::news("tvty", Kind::Info, None, said));
                        crate::kernel::standing::apply(cx, standing);
                    }
                    Err(error) => {
                        if let Some(m) = shell.megaphone.as_mut() {
                            m.error = Some(format!("{error:#}"));
                        }
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The message to every running loop (`hold`: held too), or `None`:
    /// their holds released.
    fn message_loops(&mut self, hold: Option<bool>, cx: &mut Context<Self>) {
        let Some(m) = self.megaphone.as_mut() else { return };
        let typed = m.message.read(cx).value().trim().to_string();
        let message = if typed.is_empty() { DEFAULT_MESSAGE.to_string() } else { typed };
        m.busy = true;
        m.error = None;
        cx.notify();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let done = cx
                .background_executor()
                .spawn(async move {
                    match hold {
                        Some(hold) => aiball.message_all(&message, hold),
                        None => aiball.release_all(),
                    }
                })
                .await;
            let _ = this.update(cx, |shell, cx| {
                let Some(m) = shell.megaphone.as_mut() else { return };
                m.busy = false;
                match done {
                    Ok(results) => m.said = Some(results_said(&results)),
                    Err(error) => m.error = Some(format!("{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The popover, under the title bar at the ticket panel's side.
    pub(super) fn megaphone_view(&self, window: &Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let m = self.megaphone.as_ref()?;
        let head = |text: String| div().pt_2().text_sm().font_weight(FontWeight::BOLD).child(text);
        let hint = |text: String| div().text_xs().text_color(p().muted).child(text);
        let standing = m.project.as_ref().and_then(|p| crate::kernel::standing::get(cx, p));
        let standing = standing.as_ref();
        let mut card = div().flex().flex_col().gap_1p5().p_3().w(px(480.)).text_sm();
        match &m.project {
            Some(project) => {
                let history = &self.settings.workspace.standing_history;
                card = card
                    .child(head(crate::t!("megaphone-standing", project = project.clone())))
                    .child(hint(crate::t!("megaphone-standing-hint")))
                    .child(Input::new(&m.prompt))
                    // The ones given last, a click away.
                    .when(!history.is_empty(), |d| {
                        let mut chips = div().flex().flex_wrap().gap_1();
                        for (at, given) in history.iter().enumerate() {
                            let given = given.clone();
                            chips = chips.child(
                                buttons::chip(SharedString::from(format!("standing-history-{at}")), given.chars().take(40).collect::<String>())
                                    .text_xs()
                                    .py_0p5()
                                    .on_click(cx.listener(move |shell, _, window, cx| {
                                        if let Some(m) = shell.megaphone.as_ref() {
                                            m.prompt.update(cx, |e, cx| e.set_value(given.clone(), window, cx));
                                        }
                                    })),
                            );
                        }
                        d.child(chips)
                    })
                    .child(head(crate::t!("megaphone-focus")))
                    .child(hint(crate::t!("megaphone-focus-hint")))
                    .child(div().flex().gap_2().child(div().flex_1().child(Input::new(&m.tickets))).child(div().w(px(170.)).child(Input::new(&m.until))))
                    .children(standing.and_then(|s| s.focus_line.clone()).filter(|l| !l.is_empty()).map(|line| {
                        div().text_xs().text_color(if standing.is_some_and(|s| s.focus_active) { p().info } else { p().muted }).child(line)
                    }))
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(buttons::secondary("standing-clear", crate::t!("megaphone-clear")).on_click(cx.listener(|shell, _, window, cx| shell.save_standing(true, window, cx))))
                            .child(buttons::primary("standing-save", crate::t!("megaphone-save")).on_click(cx.listener(|shell, _, window, cx| shell.save_standing(false, window, cx)))),
                    );
            }
            None => card = self.message_section(m, card, cx),
        }
        let card = card
            .children(m.error.clone().map(|error| div().text_xs().text_color(p().danger).child(error)))
            .when(m.busy, |d| d.child(div().text_xs().text_color(p().muted).child("…")));
        let backdrop = div().named("megaphone-backdrop").absolute().inset_0().occlude().on_mouse_down(
            MouseButton::Left,
            cx.listener(|shell, _, window, cx| shell.close_megaphone(window, cx)),
        );
        // A project's under the ticket panel's header; the board's under the
        // title bar, by its button.
        let top = if m.project.is_some() { 80. } else { 40. };
        let card = div()
            .named("megaphone")
            .absolute()
            .occlude()
            .top(px(top))
            .right(px(8.))
            // As tall as it needs, the window's height at most: it scrolls then.
            .max_h(window.viewport_size().height - px(top + 16.))
            .overflow_y_scroll()
            .rounded_md()
            .bg(p().surface)
            .border_1()
            .border_color(p().border)
            .shadow_lg()
            .child(card);
        // Over everything, the panel's deferred layers too.
        Some(deferred(div().absolute().inset_0().child(backdrop).child(card)).with_priority(3).into_any_element())
    }

    /// The message to every running agent loop, and what became of it.
    fn message_section(&self, m: &Megaphone, card: Div, cx: &mut Context<Self>) -> Div {
        let head = |text: String| div().pt_2().text_sm().font_weight(FontWeight::BOLD).child(text);
        let hint = |text: String| div().text_xs().text_color(p().muted).child(text);
        let mut running: Vec<String> = self.board.bars.iter().filter(|(_, b)| !b.stale).map(|(a, _)| a.clone()).collect();
        running.sort();
        card.child(head(crate::t!("megaphone-message")))
            .child(hint(crate::t!("megaphone-message-hint")))
            .child(div().text_xs().text_color(p().muted).child(if running.is_empty() {
                crate::t!("megaphone-no-loop")
            } else {
                crate::t!("megaphone-running", count = running.len(), names = running.join(", "))
            }))
            .child(Textarea::new(&m.message))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(buttons::secondary("loops-release", crate::t!("megaphone-release")).on_click(cx.listener(|shell, _, _, cx| shell.message_loops(None, cx))))
                    .child(buttons::secondary("loops-send", crate::t!("megaphone-send")).on_click(cx.listener(|shell, _, _, cx| shell.message_loops(Some(false), cx))))
                    .child(buttons::answer("loops-hold", crate::t!("megaphone-send-hold")).danger().on_click(cx.listener(|shell, _, _, cx| shell.message_loops(Some(true), cx)))),
            )
            .children(m.said.clone().map(|said| div().text_xs().text_color(p().success).child(said)))
    }

    /// The mark of a steered project in the sessions list: 📢, what steers
    /// it in its tip.
    pub(super) fn steered_mark(&self, project: &str, cx: &App) -> Option<AnyElement> {
        let standing = crate::kernel::standing::get(cx, project).filter(|s| s.active())?;
        let mut tip = Vec::new();
        if let Some(prompt) = standing.standing_prompt.as_deref().filter(|p| !p.trim().is_empty()) {
            tip.push(crate::t!("megaphone-tip-standing", prompt = prompt));
        }
        if standing.focus_active
            && let Some(line) = standing.focus_line.as_deref().or(standing.focus_tickets.as_deref())
        {
            tip.push(crate::t!("megaphone-tip-focus", focus = line.strip_prefix("focus: ").unwrap_or(line)));
        }
        // No padding: the row is full, the project's name gives way to it.
        use crate::tip::Tip as _;
        Some(div().named(SharedString::from(format!("steered-{project}"))).flex_none().child(crate::icons::megaphone(p().accent, 13.)).tip(tip.join("\n")).into_any_element())
    }
}

#[cfg(test)]
mod tests {
    use super::{results_said, until_iso};
    use crate::aiball::LoopHold;

    #[test]
    fn a_typed_date_is_aibals_and_a_wrong_one_said() {
        assert_eq!(until_iso("  "), Ok(None));
        assert!(until_iso("2026-09-30 18:00").unwrap().unwrap().starts_with("2026-09-30T"));
        assert!(until_iso("2026-09-30").unwrap().is_some());
        assert!(until_iso("tomorrow").is_err());
    }

    #[test]
    fn what_a_message_did_is_said_loop_by_loop() {
        let hold = |id: &str, prompt: Option<&str>, hold: Option<&str>, error: Option<&str>| LoopHold {
            consumer_id: id.into(),
            prompt: prompt.map(String::from),
            hold: hold.map(String::from),
            hold_error: error.map(String::from),
        };
        let said = results_said(&[hold("a", Some("delivered"), Some("armed"), None), hold("b", Some("spooled"), Some("failed"), Some("no loop"))]);
        assert_eq!(said, "typed into a · queued for b · held a · b: no loop");
        assert_eq!(results_said(&[]), "No agent loop is running.");
    }
}
