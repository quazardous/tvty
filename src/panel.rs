//! The ticket panel, on the right of a project's terminal: the project's
//! tickets, and a ticket's thread — where it stands and whose turn it is on
//! top, the talk folded up to its latest snapshot, and the gestures in one
//! place under it: accept or reject, moderate, reply, close or reopen.

use std::collections::HashSet;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::text::TextView;
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::aiball::{Aiball, Comment, Thread, TicketRow};
use crate::rowstate::{self, Glyph, RowState, Stripe, Turn};
use crate::thread::{self as reading, DecisionState, Entry, Shape};
use crate::theme::p;
use gpui_kit::component::scroll::{ScrollableElement as _, Scrollbar, ScrollbarAxis};

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
    /// Folded comments the user opened, and the ticket's own body.
    unfolded: HashSet<u64>,
    /// The thread's scroll: it opens on its latest word, next to the reply.
    scroll: ScrollHandle,
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
        // Reject waits for a reason: redraw as the reason is typed.
        cx.subscribe(&reply, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        })
        .detach();
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
            unfolded: HashSet::new(),
            scroll: ScrollHandle::new(),
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
                            // First read, or the answer to a gesture: the
                            // latest word is what to see.
                            let grew = detail
                                .thread
                                .as_ref()
                                .is_none_or(|t| t.comments.len() != thread.comments.len());
                            if grew {
                                detail.scroll.scroll_to_bottom();
                            }
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

    /// Posts what the user typed, then `then`: every gesture carries its
    /// why.
    fn act(
        &mut self,
        then: impl FnOnce(&Aiball, &str, u64) -> anyhow::Result<()> + Send + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let body = self.reply.read(cx).value().trim().to_string();
        let (Some(detail), Some(scope)) = (self.detail.as_ref(), self.scope.as_ref()) else {
            return;
        };
        let (project, ticket) = (scope.project.clone(), detail.ticket);
        self.gesture(
            move |aiball| {
                if !body.is_empty() {
                    aiball.reply(&project, ticket, &body)?;
                }
                then(aiball, &project, ticket)
            },
            |panel, window, cx| {
                panel
                    .reply
                    .update(cx, |reply, cx| reply.set_value("", window, cx));
            },
            window,
            cx,
        );
    }

    fn decide(&mut self, message: u64, accept: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.act(move |aiball, _, _| aiball.decide(message, accept), window, cx);
    }

    fn moderate(&mut self, message: u64, approve: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.act(move |aiball, _, _| aiball.moderate(message, approve), window, cx);
    }

    fn set_closed(&mut self, closed: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.act(move |aiball, project, ticket| aiball.set_closed(project, ticket, closed), window, cx);
    }

    fn send_reply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.reply.read(cx).value().trim().is_empty() {
            return;
        }
        self.act(|_, _, _| Ok(()), window, cx);
    }

    fn toggle_fold(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(detail) = self.detail.as_mut() {
            if !detail.unfolded.remove(&id) {
                detail.unfolded.insert(id);
            }
            cx.notify();
        }
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
        list.overflow_y_scrollbar().into_any_element()
    }

    /// One ticket: a stripe for whose turn, one state glyph, the title
    /// (bold when unread), then who spoke last, who holds it, and when.
    fn row(&self, ticket: &TicketRow, state: RowState, cx: &mut Context<Self>) -> impl IntoElement {
        let id = ticket.id;
        let yours = state.turn == Turn::You;
        let glyph_colour = |glyph: Glyph| glyph_colour(glyph, yours);
        let stripe_colour = stripe_colour(&state);
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
            .child(stripe(state.stripe, stripe_colour))
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
        let user = self.aiball.user.clone();
        let read = reading::read(thread);
        let row = self.tickets.iter().find(|t| t.id == ticket.id);
        let state = row.map(|r| rowstate::of(r, &user));
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let sentence = reading::sentence(
            thread,
            &read,
            state.as_ref(),
            row.is_some_and(|r| r.stalled_step),
            &user,
            now,
        );
        let glyph = state.as_ref().map_or_else(|| reading::glyph(thread, &read), |s| s.glyph);
        let yours = state.as_ref().is_some_and(|s| s.turn == Turn::You);

        // ── Where it stands: said once ──────────────────────────────────
        let title = div()
            .flex()
            .gap_2()
            .text_lg()
            .font_weight(FontWeight::BOLD)
            .when_some(glyph, |d, glyph| {
                d.child(div().flex_none().text_color(glyph_colour(glyph, yours)).child(glyph.symbol()))
            })
            .child(div().flex_1().min_w_0().child(format!("#{} {}", ticket.id, ticket.title)));
        let turn = (!sentence.is_empty()).then(|| {
            let (stripe_kind, colour) = state
                .as_ref()
                .map_or((Stripe::None, p().border), |s| (s.stripe, stripe_colour(s)));
            div()
                .flex()
                .gap_2()
                .child(stripe(stripe_kind, colour))
                .child(
                    div()
                        .flex_1()
                        .when(yours, |d| d.font_weight(FontWeight::BOLD))
                        .text_color(if yours { p().text } else { p().muted })
                        .child(sentence),
                )
        });
        let mut chips: Vec<AnyElement> = Vec::new();
        if let Some(holder) = ticket.holder() {
            let hot = row.is_some_and(|r| r.hot);
            chips.push(div().child(format!("{}held by {holder}", if hot { "🔥 " } else { "" })).into_any_element());
        }
        if matches!(ticket.priority.as_deref(), Some("high" | "urgent")) {
            chips.push(pill(ticket.priority.clone().unwrap_or_default(), p().danger).into_any_element());
        }
        if let Some(critical) = &ticket.critical {
            chips.push(pill(format!("⚠ holds {}", critical.holds), p().danger).into_any_element());
        }
        if let Some(usage) = &ticket.token_usage {
            let total = usage.tokens_in + usage.tokens_out + usage.cache_w;
            if total > 0 {
                chips.push(div().child(format!("{} tok", count(total))).into_any_element());
            }
        }
        for relation in &ticket.relations {
            let verb = match (relation.kind.as_str(), relation.reciprocal) {
                ("depends_on", false) | ("blocks", true) => "depends on",
                ("blocks", false) | ("depends_on", true) => "blocks",
                _ => continue,
            };
            let target = relation.target_ticket_id;
            let state = match self.tickets.iter().find(|t| t.id == target) {
                Some(row) => rowstate::of(row, &user).glyph.map(|g| g.symbol().to_string()),
                None => relation.target_stage.clone(),
            };
            chips.push(
                div()
                    .child(format!("{verb} #{target}{}", state.map(|s| format!(" {s}")).unwrap_or_default()))
                    .into_any_element(),
            );
        }
        let summary = read.summary.clone().map(|(text, by)| {
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .px_2()
                .py_1p5()
                .rounded_md()
                .bg(p().hover)
                .child(div().text_xs().text_color(p().muted).child(format!("Where it stands · {}", who(&by, &user))))
                .child(div().child(text))
        });
        let head = div()
            .flex()
            .flex_col()
            .flex_none()
            .gap_1p5()
            .px_3()
            .pt_2()
            .pb_2()
            .border_b_1()
            .border_color(p().border)
            .child(title)
            .children(turn)
            .when(!chips.is_empty(), |d| {
                d.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_x_3()
                        .gap_y_1()
                        .text_xs()
                        .text_color(p().muted)
                        .children(chips),
                )
            })
            .children(summary);

        // ── The talk, folded up to its latest snapshot ──────────────────
        let has_talk = read.entries.iter().any(|e| !matches!(e.shape, Shape::Event { .. }));
        let body_open = !has_talk || detail.unfolded.contains(&ticket.id);
        let mut body = div()
            .flex()
            .flex_col()
            .px_3()
            .py_2()
            .gap_2()
            .child(self.entry_head(
                ticket.id,
                &ticket.by_agent,
                &ticket.created_at,
                read.ticket_decision.clone(),
                None,
                false,
                has_talk,
                &user,
                cx,
            ))
            .map(|d| match (&ticket.body, body_open) {
                (Some(text), true) => d.child(TextView::markdown(("body", ticket.id), text.clone())),
                (Some(text), false) => d.child(folded_line(reading::first_line(Some(text)))),
                (None, _) => d,
            });
        let comments: std::collections::HashMap<u64, &Comment> =
            thread.comments.iter().map(|c| (c.id, c)).collect();
        for entry in &read.entries {
            let Some(comment) = comments.get(&entry.id) else {
                continue;
            };
            body = body.child(self.entry(entry, comment, &detail.unfolded, &user, detail.busy, cx));
        }

        // ── The gestures, in one place ──────────────────────────────────
        let typed = !self.reply.read(cx).value().trim().is_empty();
        let decision = read.active.clone().filter(|a| a.by != user).map(|active| {
            let message = active.message;
            let accept = match active.kind.as_str() {
                "resolution" => "Accept → close",
                "wontfix" => "Accept → close, no fix",
                "escalation" => "Done → accept",
                _ => "Accept → go",
            };
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(format!("{} {} proposes a {}", symbol_of(&active.kind), active.by, reading::kind_noun(&active.kind))),
                        )
                        .child(
                            Button::new("reject")
                                .danger()
                                .small()
                                .label("Reject")
                                .disabled(detail.busy || !typed)
                                .on_click(cx.listener(move |panel, _, window, cx| panel.decide(message, false, window, cx))),
                        )
                        .child(
                            Button::new("accept")
                                .success()
                                .small()
                                .label(accept)
                                .disabled(detail.busy)
                                .on_click(cx.listener(move |panel, _, window, cx| panel.decide(message, true, window, cx))),
                        ),
                )
                .when(!typed, |d| {
                    d.child(div().text_xs().text_color(p().muted).child("To reject, say why below first."))
                })
        });
        let moderation = (ticket.status == "pending").then(|| {
            let id = ticket.id;
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().flex_1().child("This ticket waits for moderation"))
                .child(
                    Button::new("moderate-reject")
                        .danger()
                        .small()
                        .label("Reject")
                        .disabled(detail.busy)
                        .on_click(cx.listener(move |panel, _, window, cx| panel.moderate(id, false, window, cx))),
                )
                .child(
                    Button::new("moderate-approve")
                        .success()
                        .small()
                        .label("Approve")
                        .disabled(detail.busy)
                        .on_click(cx.listener(move |panel, _, window, cx| panel.moderate(id, true, window, cx))),
                )
        });
        let closed = ticket.closed;
        let actions = div()
            .flex()
            .flex_col()
            .flex_none()
            .gap_2()
            .p_3()
            .border_t_1()
            .border_color(p().border)
            .when_some(detail.error.clone(), |d, error| {
                d.child(div().text_color(p().danger).child(error))
            })
            .children(moderation)
            .children(decision)
            .child(Textarea::new(&self.reply))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("close")
                            .ghost()
                            .small()
                            .label(if closed { "Reopen" } else { "Close" })
                            .disabled(detail.busy)
                            .on_click(cx.listener(move |panel, _, window, cx| panel.set_closed(!closed, window, cx))),
                    )
                    .child(div().flex_1())
                    .child(
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
            .child(head)
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .id("ticket-thread")
                            .size_full()
                            .track_scroll(&detail.scroll)
                            .overflow_y_scroll()
                            .child(body),
                    )
                    .child(
                        div().absolute().inset_0().child(
                            Scrollbar::new(&detail.scroll)
                                .axis(ScrollbarAxis::Vertical)
                                .viewport_from_layout(),
                        ),
                    ),
            )
            .child(actions)
            .into_any_element()
    }

    /// One entry of the thread: an event line, a folded comment, or a whole
    /// one.
    fn entry(
        &self,
        entry: &Entry,
        comment: &Comment,
        unfolded: &HashSet<u64>,
        user: &str,
        busy: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Shape::Event { verb, count } = &entry.shape {
            let times = if *count > 1 { format!(" ×{count}") } else { String::new() };
            return div()
                .text_xs()
                .text_color(p().muted)
                .child(format!(
                    "{} {verb}{times}{}",
                    who(&comment.by_agent, user),
                    ago(&comment.created_at).map(|a| format!(" · {a}")).unwrap_or_default()
                ))
                .into_any_element();
        }
        let foldable = matches!(entry.shape, Shape::Folded(_));
        let open = !foldable || unfolded.contains(&entry.id);
        let moderation = entry.pending.then(|| {
            let id = entry.id;
            div()
                .flex()
                .gap_1()
                .child(
                    Button::new(("comment-reject", id))
                        .danger()
                        .xsmall()
                        .label("Reject")
                        .disabled(busy)
                        .on_click(cx.listener(move |panel, _, window, cx| panel.moderate(id, false, window, cx))),
                )
                .child(
                    Button::new(("comment-approve", id))
                        .success()
                        .xsmall()
                        .label("Approve")
                        .disabled(busy)
                        .on_click(cx.listener(move |panel, _, window, cx| panel.moderate(id, true, window, cx))),
                )
        });
        let commits = comment.commits();
        div()
            .flex()
            .flex_col()
            .gap_1()
            .pt_2()
            .border_t_1()
            .border_color(p().border)
            .child(self.entry_head(
                entry.id,
                &comment.by_agent,
                &comment.created_at,
                entry.decision.clone(),
                entry.step,
                entry.pending,
                foldable,
                user,
                cx,
            ))
            .map(|d| match (&entry.shape, open, &comment.body) {
                (Shape::Folded(line), false, _) => d.child(folded_line(line.clone())),
                (_, _, Some(text)) => d.child(TextView::markdown(("comment", comment.id), text.clone())),
                _ => d,
            })
            .children(moderation)
            .when(open && !commits.is_empty(), |d| {
                d.child(div().text_xs().text_color(p().muted).child(commits.join(" · ")))
            })
            .into_any_element()
    }

    /// Who, when, and the chips of an entry; a click folds or unfolds it.
    #[allow(clippy::too_many_arguments)]
    fn entry_head(
        &self,
        id: u64,
        by: &str,
        when: &str,
        decision: Option<(String, DecisionState)>,
        step: Option<bool>,
        pending: bool,
        foldable: bool,
        user: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        div()
            .id(("entry", id))
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .text_xs()
            .text_color(p().muted)
            .child(who(by, user))
            .when_some(ago(when), |d, when| d.child(when))
            .when_some(decision, |d, (kind, state)| d.child(decision_chip(&kind, state)))
            .when_some(step, |d, latest| {
                d.child(
                    div()
                        .text_color(if latest { p().accent } else { p().muted })
                        .child(format!("{} step", Glyph::Step.symbol())),
                )
            })
            .when(pending, |d| d.child(pill("to moderate", p().warning)))
            .when(foldable, |d| {
                d.cursor_pointer()
                    .hover(|d| d.text_color(p().text))
                    .on_click(cx.listener(move |panel, _, _, cx| panel.toggle_fold(id, cx)))
            })
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

/// "you" for the user, else the name.
fn who(name: &str, user: &str) -> String {
    if name == user { "you".into() } else { name.to_string() }
}

/// A folded comment's one line; its head unfolds it.
fn folded_line(text: String) -> impl IntoElement {
    div().text_color(p().muted).truncate().child(text)
}

/// Coloured when it waits on you, muted otherwise.
fn glyph_colour(glyph: Glyph, yours: bool) -> Hsla {
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
}

fn stripe_colour(state: &RowState) -> Hsla {
    state
        .glyph
        .filter(|_| state.stripe != Stripe::Neutral)
        .map(|g| glyph_colour(g, state.turn == Turn::You))
        .unwrap_or(p().border)
}

/// Whose turn: coloured when a decision waits on you (solid when it is the
/// last message, dashed when the talk went on), neutral when an agent
/// answered you, nothing when the ball is theirs.
fn stripe(stripe: Stripe, colour: Hsla) -> impl IntoElement {
    div()
        .w(px(3.))
        .flex_none()
        .rounded_sm()
        .map(|d| match stripe {
            Stripe::Solid | Stripe::Neutral => d.bg(colour),
            Stripe::Dashed => d.border_l_3().border_dashed().border_color(colour),
            Stripe::None => d,
        })
}

/// A decision, as a chip: the list's glyphs, and "superseded" when a newer
/// decision replaced it.
fn decision_chip(kind: &str, state: DecisionState) -> AnyElement {
    let noun = reading::kind_noun(kind);
    match state {
        DecisionState::Pending => pill(format!("{} {noun} · pending", symbol_of(kind)), p().warning).into_any_element(),
        DecisionState::Accepted => pill(format!("✓ {noun} accepted"), p().success).into_any_element(),
        DecisionState::Rejected => pill(format!("✕ {noun} rejected"), p().danger).into_any_element(),
        DecisionState::Superseded => div()
            .flex_none()
            .px_1p5()
            .rounded_sm()
            .text_xs()
            .border_1()
            .border_color(p().border)
            .text_color(p().muted)
            .child(format!("{noun} · superseded"))
            .into_any_element(),
    }
}

fn symbol_of(kind: &str) -> &'static str {
    match kind {
        "resolution" => Glyph::Resolution.symbol(),
        "wontfix" => Glyph::Wontfix.symbol(),
        "escalation" => Glyph::Escalation.symbol(),
        _ => Glyph::Plan.symbol(),
    }
}

/// `12345` → `12.3k`.
fn count(n: u64) -> String {
    match n {
        0..1000 => n.to_string(),
        1000..1_000_000 => format!("{:.1}k", n as f64 / 1e3),
        _ => format!("{:.1}M", n as f64 / 1e6),
    }
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
