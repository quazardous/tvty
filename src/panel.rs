//! The ticket panel, on the right of a project's terminal: the tickets of
//! the terminal's agent (its current one, then the project's queue), and a
//! ticket's thread with the usual gestures — accept, reject, reply.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Textarea, TextareaState};
use gpui_kit::component::text::TextView;
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::aiball::{Aiball, Thread, TicketRow};

/// Something changed on the board: the shell should read it again.
pub struct BoardChanged;

/// The user folds the panel away.
pub struct CollapsePanel;

/// What the panel is about: a project, and the agent of the terminal shown.
#[derive(Clone, Debug, PartialEq)]
pub struct Scope {
    pub project: String,
    pub agent: Option<String>,
}

struct Detail {
    ticket: u64,
    thread: Option<Thread>,
    error: Option<String>,
    /// A gesture is on its way to aiball.
    busy: bool,
}

pub struct TicketPanel {
    aiball: Aiball,
    scope: Option<Scope>,
    tickets: Vec<TicketRow>,
    critical: Option<u64>,
    detail: Option<Detail>,
    reply: Entity<TextareaState>,
}

impl EventEmitter<BoardChanged> for TicketPanel {}
impl EventEmitter<CollapsePanel> for TicketPanel {}

impl TicketPanel {
    pub fn new(aiball: Aiball, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let reply = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Reply…")
                .auto_grow(2, 8)
        });
        Self {
            aiball,
            scope: None,
            tickets: Vec::new(),
            critical: None,
            detail: None,
            reply,
        }
    }

    /// Shows another terminal's tickets; closes the open ticket when the
    /// project changes.
    pub fn set_scope(&mut self, scope: Option<Scope>, cx: &mut Context<Self>) {
        if self.scope.as_ref().map(|s| &s.project) != scope.as_ref().map(|s| &s.project) {
            self.detail = None;
        }
        self.scope = scope;
        cx.notify();
    }

    /// The board as last read: this project's tickets, and who tvty is.
    pub fn set_board(
        &mut self,
        aiball: &Aiball,
        tickets: Vec<TicketRow>,
        critical: Option<u64>,
        cx: &mut Context<Self>,
    ) {
        self.aiball = aiball.clone();
        if self.tickets != tickets || self.critical != critical {
            self.tickets = tickets;
            self.critical = critical;
            cx.notify();
        }
    }

    pub fn open(&mut self, ticket: u64, cx: &mut Context<Self>) {
        self.detail = Some(Detail {
            ticket,
            thread: None,
            error: None,
            busy: false,
        });
        self.load(ticket, true, cx);
        cx.notify();
    }

    /// Reads the thread again; `mark_read` clears its unread for the user.
    fn load(&mut self, ticket: u64, mark_read: bool, cx: &mut Context<Self>) {
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let thread = cx
                .background_executor()
                .spawn(async move {
                    let thread = aiball.thread(ticket);
                    if mark_read && thread.is_ok() {
                        let _ = aiball.mark_read(ticket);
                    }
                    thread
                })
                .await;
            let _ = this.update(cx, |panel, cx| {
                if let Some(detail) = panel.detail.as_mut().filter(|d| d.ticket == ticket) {
                    match thread {
                        Ok(thread) => {
                            detail.thread = Some(thread);
                            detail.error = None;
                        }
                        Err(error) => detail.error = Some(format!("{error:#}")),
                    }
                    detail.busy = false;
                }
                if mark_read {
                    cx.emit(BoardChanged);
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Sends a gesture to aiball, then reads the thread and the board again.
    fn gesture(
        &mut self,
        run: impl FnOnce(&Aiball) -> anyhow::Result<()> + Send + 'static,
        on_success: impl FnOnce(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(detail) = self.detail.as_mut() else {
            return;
        };
        let ticket = detail.ticket;
        detail.busy = true;
        detail.error = None;
        let aiball = self.aiball.clone();
        let window_handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { run(&aiball) }).await;
            let _ = cx.update_window(window_handle, |_, window, cx| {
                let _ = this.update(cx, |panel, cx| {
                    match result {
                        Ok(()) => {
                            on_success(panel, window, cx);
                            panel.load(ticket, false, cx);
                            cx.emit(BoardChanged);
                        }
                        Err(error) => {
                            if let Some(detail) = panel.detail.as_mut() {
                                detail.busy = false;
                                detail.error = Some(format!("{error:#}"));
                            }
                        }
                    }
                    cx.notify();
                });
            });
        })
        .detach();
        cx.notify();
    }

    fn decide(&mut self, comment: u64, accept: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.gesture(
            move |aiball| aiball.decide(comment, accept),
            |_, _, _| {},
            window,
            cx,
        );
    }

    fn send_reply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let body = self.reply.read(cx).value().trim().to_string();
        let (Some(detail), Some(scope)) = (self.detail.as_ref(), self.scope.as_ref()) else {
            return;
        };
        if body.is_empty() {
            return;
        }
        let (project, ticket) = (scope.project.clone(), detail.ticket);
        self.gesture(
            move |aiball| aiball.reply(&project, ticket, &body),
            |panel, window, cx| {
                panel
                    .reply
                    .update(cx, |reply, cx| reply.set_value("", window, cx));
            },
            window,
            cx,
        );
    }

    fn list(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(scope) = &self.scope else {
            return hint("No aiball project on this terminal.").into_any_element();
        };
        let agent = scope.agent.as_deref();
        let current: Vec<&TicketRow> = self
            .tickets
            .iter()
            .filter(|t| agent.is_some() && t.holder() == agent)
            .collect();
        let queue: Vec<&TicketRow> = self.tickets.iter().filter(|t| t.holder().is_none()).collect();
        let others: Vec<&TicketRow> = self
            .tickets
            .iter()
            .filter(|t| t.holder().is_some() && t.holder() != agent)
            .collect();

        let mut list = div()
            .id("ticket-list")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .pb_2();
        for (title, rows) in [
            (agent.map(|a| format!("{a} is on")), current),
            (Some("Queue".to_string()), queue),
            (Some("Held by other agents".to_string()), others),
        ] {
            let (Some(title), false) = (title, rows.is_empty()) else {
                continue;
            };
            list = list.child(section_title(title));
            for row in rows {
                list = list.child(self.row(row, cx));
            }
        }
        if self.tickets.is_empty() {
            list = list.child(hint("No open ticket."));
        }
        list.into_any_element()
    }

    fn row(&self, ticket: &TicketRow, cx: &mut Context<Self>) -> impl IntoElement {
        let id = ticket.id;
        let critical = self.critical == Some(id);
        div()
            .id(("ticket", id))
            .flex()
            .items_start()
            .gap_2()
            .px_3()
            .py_1p5()
            .cursor_pointer()
            .hover(|d| d.bg(rgb(0x2a2d2e)))
            .child(
                div()
                    .flex_none()
                    .w(px(44.))
                    .text_color(rgb(0x808080))
                    .child(format!("#{id}")),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_1()
                    .child(div().mr_1().child(ticket.title.clone()))
                    .when(critical, |d| d.child(pill("critical", 0xc72e0f)))
                    .when(ticket.pending_decision, |d| d.child(pill("decision", 0xcc6d00)))
                    .when(ticket.urgent(), |d| {
                        d.child(pill(ticket.priority.clone().unwrap_or_default(), 0x8b2f2f))
                    }),
            )
            .when(ticket.unread, |d| d.child(dot(0x3b8eea)))
            .on_click(cx.listener(move |panel, _, _, cx| panel.open(id, cx)))
    }

    fn detail(&self, detail: &Detail, cx: &mut Context<Self>) -> AnyElement {
        let Some(thread) = &detail.thread else {
            return match &detail.error {
                Some(error) => hint(error.clone()).into_any_element(),
                None => hint("Loading…").into_any_element(),
            };
        };
        let ticket = &thread.ticket;
        let mut body = div()
            .id("ticket-thread")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .px_3()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .child(format!("#{} {}", ticket.id, ticket.title)),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(byline(&ticket.by_agent, &ticket.created_at))
                    .when(ticket.closed, |d| d.child(pill("closed", 0x6b6b6b))),
            )
            .when_some(ticket.body.clone(), |d, text| {
                d.child(TextView::markdown(("body", ticket.id), text))
            });

        for comment in &thread.comments {
            let text = match (comment.kind.as_str(), &comment.body) {
                ("comment_added", Some(text)) => text.clone(),
                ("comment_added", None) => continue,
                // Lifecycle events: one line.
                (kind, text) => {
                    body = body.child(
                        div()
                            .flex()
                            .gap_2()
                            .child(byline(&comment.by_agent, &comment.created_at))
                            .child(pill(kind.replace('_', " "), 0x3c3c3c)),
                    );
                    if let Some(text) = text {
                        body = body.child(TextView::markdown(("event", comment.id), text.clone()));
                    }
                    continue;
                }
            };
            let decision = comment.decision();
            body = body.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .pt_2()
                    .border_t_1()
                    .border_color(rgb(0x3c3c3c))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(byline(&comment.by_agent, &comment.created_at))
                            .when_some(decision, |d, decision| {
                                let color = match decision.status.as_str() {
                                    "pending" => 0xcc6d00,
                                    "accepted" => 0x2e7d32,
                                    _ => 0x6b6b6b,
                                };
                                d.child(pill(format!("{} · {}", decision.kind, decision.status), color))
                            }),
                    )
                    .child(TextView::markdown(("comment", comment.id), text)),
            );
        }

        let pending = thread.pending_decision();
        let actions = div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .border_t_1()
            .border_color(rgb(0x3c3c3c))
            .when_some(detail.error.clone(), |d, error| {
                d.child(div().text_color(rgb(0xf14c4c)).child(error))
            })
            .when_some(pending, |d, (comment, decision)| {
                let id = comment.id;
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().flex_1().child(format!("{} by {}", decision.kind, comment.by_agent)))
                        .child(
                            Button::new("accept")
                                .success()
                                .small()
                                .label("Accept")
                                .disabled(detail.busy)
                                .on_click(cx.listener(move |panel, _, window, cx| {
                                    panel.decide(id, true, window, cx)
                                })),
                        )
                        .child(
                            Button::new("reject")
                                .danger()
                                .small()
                                .label("Reject")
                                .disabled(detail.busy)
                                .on_click(cx.listener(move |panel, _, window, cx| {
                                    panel.decide(id, false, window, cx)
                                })),
                        ),
                )
            })
            .child(Textarea::new(&self.reply))
            .child(
                div().flex().justify_end().child(
                    Button::new("send")
                        .primary()
                        .small()
                        .label("Reply")
                        .loading(detail.busy)
                        .on_click(cx.listener(|panel, _, window, cx| panel.send_reply(window, cx))),
                ),
            );

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .child(body)
            .child(actions)
            .into_any_element()
    }
}

impl Render for TicketPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = div()
            .flex()
            .items_center()
            .gap_2()
            .h(px(36.))
            .px_3()
            .border_b_1()
            .border_color(rgb(0x3c3c3c))
            .when(self.detail.is_some(), |d| {
                d.child(
                    div()
                        .id("back")
                        .cursor_pointer()
                        .text_color(rgb(0x3b8eea))
                        .child("← Tickets")
                        .on_click(cx.listener(|panel, _, _, cx| {
                            panel.detail = None;
                            cx.notify();
                        })),
                )
            })
            .when(self.detail.is_none(), |d| {
                d.child(div().font_weight(FontWeight::BOLD).child(
                    self.scope
                        .as_ref()
                        .map(|s| s.project.clone())
                        .unwrap_or_else(|| "Tickets".into()),
                ))
            })
            .child(div().flex_1())
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x808080))
                    .child(format!("as {}", self.aiball.user)),
            )
            .child(
                div()
                    .id("collapse")
                    .px_1()
                    .cursor_pointer()
                    .text_color(rgb(0x3b8eea))
                    .child("›")
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(CollapsePanel))),
            );
        let content = match &self.detail {
            Some(detail) => self.detail(detail, cx),
            None => self.list(cx),
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x252526))
            .text_sm()
            .child(header)
            .child(content)
    }
}

fn section_title(title: impl Into<SharedString>) -> impl IntoElement {
    div()
        .px_3()
        .pt_3()
        .pb_1()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0x9d9d9d))
        .child(title.into().to_uppercase())
}

fn hint(text: impl Into<SharedString>) -> impl IntoElement {
    div().p_3().text_color(rgb(0x808080)).child(text.into())
}

fn byline(who: &str, when: &str) -> impl IntoElement {
    // `2026-09-24T13:01:55.681Z` → `2026-09-24 13:01`
    let when = when.get(..16).map(|w| w.replace('T', " ")).unwrap_or_else(|| when.to_string());
    div()
        .text_xs()
        .text_color(rgb(0x9d9d9d))
        .child(format!("{who} · {when}"))
}

pub fn pill(text: impl Into<SharedString>, color: u32) -> impl IntoElement {
    div()
        .flex_none()
        .px_1p5()
        .rounded_sm()
        .text_xs()
        .bg(rgb(color))
        .text_color(rgb(0xffffff))
        .child(text.into())
}

pub fn dot(color: u32) -> impl IntoElement {
    div().flex_none().mt_1p5().size(px(7.)).rounded_full().bg(rgb(color))
}
