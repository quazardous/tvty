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
use crate::rowstate::{self, Glyph, RowState, Stripe, Turn};
use crate::theme::p;

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
        if self.scope.is_none() {
            return hint("No aiball project on this terminal.").into_any_element();
        }
        let user = self.aiball.user.as_str();
        // Bands first, from "it is yours" to "it runs by itself"; the most
        // recent activity first within a band.
        let mut rows: Vec<(RowState, &TicketRow)> =
            self.tickets.iter().map(|t| (rowstate::of(t, user), t)).collect();
        rows.sort_by(|(a, ta), (b, tb)| {
            a.band
                .cmp(&b.band)
                .then_with(|| tb.last_activity.cmp(&ta.last_activity))
        });

        let mut list = div()
            .id("ticket-list")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .pb_2();
        let mut band = None;
        for (state, ticket) in rows {
            if band != Some(state.band) {
                band = Some(state.band);
                list = list.child(section_title(state.band.title()));
            }
            list = list.child(self.row(ticket, state, cx));
        }
        if self.tickets.is_empty() {
            list = list.child(hint("No open ticket."));
        }
        list.into_any_element()
    }

    /// One ticket: a stripe for whose turn, one state glyph, the title
    /// (bold when unread), then who spoke last, who holds it, and when.
    fn row(&self, ticket: &TicketRow, state: RowState, cx: &mut Context<Self>) -> impl IntoElement {
        let id = ticket.id;
        let yours = state.turn == Turn::You;
        let glyph_colour = |glyph: Glyph| {
            // Coloured when it waits on you, muted otherwise.
            if !yours {
                return p().muted;
            }
            match glyph {
                Glyph::Escalation | Glyph::Rejected => p().danger,
                Glyph::Plan | Glyph::StalledStep => p().warning,
                Glyph::Resolution => p().success,
                Glyph::Step => p().accent,
                Glyph::Wontfix | Glyph::ClosedResolved | Glyph::Closed => p().muted,
            }
        };
        let stripe_colour = state
            .glyph
            .filter(|_| state.stripe != Stripe::Neutral)
            .map(glyph_colour)
            .unwrap_or(p().border);
        let speaker = ticket.last_speaker.as_deref().map(|s| {
            if s == self.aiball.user {
                "you".to_string()
            } else {
                s.to_string()
            }
        });
        let meta = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_x_2()
            .text_xs()
            .text_color(p().muted)
            .when_some(speaker, |d, who| {
                d.child(format!("{who} · {} msg", ticket.comment_count))
            })
            .when_some(ticket.holder().map(str::to_string), |d, holder| {
                d.child(if ticket.hot { format!("🔥 {holder}") } else { holder })
            })
            .when_some(ticket.critical.as_ref(), |d, critical| {
                d.child(pill(format!("⚠ {}", critical.holds), p().danger))
            })
            .when(ticket.urgent(), |d| {
                d.child(pill(ticket.priority.clone().unwrap_or_default(), p().danger))
            })
            .child(div().flex_1())
            .when_some(ticket.last_activity.as_deref().and_then(ago), |d, when| d.child(when));

        div()
            .id(("ticket", id))
            .flex()
            .gap_2()
            .pl_2()
            .pr_3()
            .py_1p5()
            .cursor_pointer()
            .hover(|d| d.bg(p().hover))
            // Whose turn: coloured when a decision waits on you (solid when
            // it is the last message, dashed when the talk went on), neutral
            // when an agent answered you, nothing when the ball is theirs.
            .child(
                div()
                    .w(px(3.))
                    .flex_none()
                    .rounded_sm()
                    .map(|d| match state.stripe {
                        Stripe::Solid | Stripe::Neutral => d.bg(stripe_colour),
                        Stripe::Dashed => d.border_l_3().border_dashed().border_color(stripe_colour),
                        Stripe::None => d,
                    }),
            )
            .child(
                div()
                    .w(px(14.))
                    .flex_none()
                    .font_weight(FontWeight::BOLD)
                    .when_some(state.glyph, |d, glyph| {
                        d.text_color(glyph_colour(glyph)).child(glyph.symbol())
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(div().flex_none().text_color(p().muted).child(format!("#{id}")))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    // Weight says unread, and nothing else does: read
                                    // titles step back so bold stands out.
                                    .map(|d| {
                                        if ticket.unread {
                                            d.font_weight(FontWeight::BOLD).text_color(p().text)
                                        } else {
                                            d.font_weight(FontWeight::NORMAL).text_color(p().text.opacity(0.72))
                                        }
                                    })
                                    .child(ticket.title.clone()),
                            ),
                    )
                    .child(meta),
            )
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
                    .when(ticket.closed, |d| d.child(pill("closed", p().muted))),
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
                            .child(pill(kind.replace('_', " "), p().border)),
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
                    .border_color(p().border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(byline(&comment.by_agent, &comment.created_at))
                            .when_some(decision, |d, decision| {
                                let color = match decision.status.as_str() {
                                    "pending" => p().warning,
                                    "accepted" => p().success,
                                    _ => p().muted,
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
            .border_color(p().border)
            .when_some(detail.error.clone(), |d, error| {
                d.child(div().text_color(p().danger).child(error))
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
            .border_color(p().border)
            .when(self.detail.is_some(), |d| {
                d.child(
                    div()
                        .id("back")
                        .cursor_pointer()
                        .text_color(p().accent)
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
                    .text_color(p().muted)
                    .child(format!("as {}", self.aiball.user)),
            )
            .child(
                div()
                    .id("collapse")
                    .px_1()
                    .cursor_pointer()
                    .text_color(p().accent)
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
            .bg(p().surface)
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
        .text_color(p().muted)
        .child(title.into().to_uppercase())
}

fn hint(text: impl Into<SharedString>) -> impl IntoElement {
    div().p_3().text_color(p().muted).child(text.into())
}

fn byline(who: &str, when: &str) -> impl IntoElement {
    // `2026-09-24T13:01:55.681Z` → `2026-09-24 13:01`
    let when = when.get(..16).map(|w| w.replace('T', " ")).unwrap_or_else(|| when.to_string());
    div()
        .text_xs()
        .text_color(p().muted)
        .child(format!("{who} · {when}"))
}

pub fn pill(text: impl Into<SharedString>, color: Hsla) -> impl IntoElement {
    div()
        .flex_none()
        .px_1p5()
        .rounded_sm()
        .text_xs()
        .bg(color)
        .text_color(crate::theme::on(color))
        .child(text.into())
}

pub fn dot(color: Hsla) -> impl IntoElement {
    div().flex_none().mt_1p5().size(px(7.)).rounded_full().bg(color)
}

/// `2026-09-24T13:01:55.681Z` → `3m`, `2h`, `5d` ago.
fn ago(when: &str) -> Option<String> {
    let then = crate::status::parse_time(when)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let seconds = now.saturating_sub(then);
    Some(match seconds {
        0..60 => "now".into(),
        60..3600 => format!("{}m", seconds / 60),
        3600..86400 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    })
}
