//! The slider (ctrl+tab) goes by group: each group — a project, "terminals",
//! "tmux" — is a stack of cards, one per terminal, offset down and to the
//! right, under a single header with the group's counters. The card on top
//! is the group's terminal used last; releasing ctrl opens it, and the tabs
//! move within the group.

use std::time::Duration;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{Alerts, CARD_COLUMNS, CARD_LINES, SLIDER_CARD, SLIDER_CHOSEN, Shell};
use crate::sessions::{Project, Terminal};
use crate::theme::p;

/// How far each card behind shows, down and right, at a card's size.
const STACK_STEP: f32 = 7.;
/// The cards a stack shows at most; more are counted.
const STACK_MAX: usize = 4;

impl Shell {
    /// A group's terminal used last, or its first.
    pub(super) fn group_top<'a>(&self, group: &'a Project) -> Option<&'a Terminal> {
        self.recent
            .iter()
            .find_map(|s| group.terminals.iter().find(|t| t.session == *s))
            .or_else(|| group.terminals.first())
    }

    /// The slider's order: the groups used lately, most recent first, then
    /// the others as the list shows them; each by its card on top.
    pub(super) fn slider_order(&self) -> Vec<String> {
        let mut groups: Vec<&Project> = Vec::new();
        for session in &self.recent {
            if let Some(group) = self.board.projects.iter().find(|p| p.terminals.iter().any(|t| t.session == *session)) {
                if !groups.iter().any(|g| g.name == group.name) {
                    groups.push(group);
                }
            }
        }
        for group in &self.board.projects {
            if !groups.iter().any(|g| g.name == group.name) {
                groups.push(group);
            }
        }
        groups.into_iter().filter_map(|g| self.group_top(g).map(|t| t.session.clone())).collect()
    }

    /// The stacks as laid out, in the list's order, one card each: what the
    /// arrows move along.
    pub(super) fn slider_groups(&self) -> Vec<Vec<String>> {
        self.board
            .projects
            .iter()
            .filter_map(|g| Some(vec![self.group_top(g)?.session.clone()]))
            .collect()
    }

    /// A group's counters: its project's tickets, as its heading in the list.
    fn group_alerts(&self, group: &Project) -> Alerts {
        if !group.on_board {
            return Alerts::default();
        }
        let tickets = self.board.tickets.get(&group.name).into_iter().flatten();
        Alerts::of(tickets, self.board.critical.get(&group.name).copied())
    }

    /// The slider as a portfolio of stacks: the window fades behind, the
    /// stacks come forward one after another, the chosen one enlarged over
    /// its own slot — nothing else moves.
    pub(super) fn portfolio(&self, chosen: &str, cx: &App) -> impl IntoElement + use<> {
        let chosen_group = self.terminal_of(chosen).map(|(p, _)| p.to_string());
        let mut body = div()
            .flex()
            .flex_wrap()
            .content_start()
            .justify_center()
            .gap_x_10()
            .gap_y_8()
            .px_6()
            .pt_6()
            .flex_1()
            .min_h_0()
            .overflow_hidden();
        let count = self.board.projects.len().max(1) as f32;
        for (i, group) in self.board.projects.iter().enumerate() {
            let Some(top) = self.group_top(group) else { continue };
            let (width, height) = SLIDER_CARD;
            let slot = if chosen_group.as_deref() == Some(group.name.as_str()) {
                // The slot keeps a stack's size (a ghost, without its screen);
                // the enlarged stack is painted over it, over its neighbours too.
                let (big_width, big_height) = SLIDER_CHOSEN;
                div()
                    .relative()
                    .child(self.stack(group, top, false, false, width, height, cx).opacity(0.))
                    .child(
                        deferred(
                            div()
                                .absolute()
                                .left(px(-(big_width - width) / 2.))
                                .top(px(-(big_height - height) / 2.))
                                .child(self.stack(group, top, true, true, big_width, big_height, cx)),
                        )
                        .with_priority(1),
                    )
            } else {
                div().child(self.stack(group, top, false, true, width, height, cx))
            };
            // Staggered entrance: each stack starts a little after the one before.
            let delay = 0.35 * i as f32 / count;
            body = body.child(slot.with_animation(
                SharedString::from(format!("portfolio-{}-{}", self.slider_shown, group.name)),
                Animation::new(Duration::from_millis(420)).with_easing(move |t| {
                    let t = ((t - delay) / (1. - delay)).clamp(0., 1.);
                    1. - (1. - t).powi(3)
                }),
                |slot, t| slot.opacity(t).mt(px(40. * (1. - t))),
            ));
        }

        div()
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .flex_col()
            // The frosted glass, for now without the frost: GPUI cannot blur
            // what lies under an element.
            .bg(p().veil)
            .child(body)
            .child(
                div()
                    .flex()
                    .justify_center()
                    .py_3()
                    .text_sm()
                    .text_color(p().muted)
                    .child("tab: next group · shift+tab: back · arrows: move · release ctrl to open · esc cancels · ctrl+pgup/pgdn: the tabs of a group"),
            )
            .with_animation(
                SharedString::from(format!("portfolio-veil-{}", self.slider_shown)),
                Animation::new(Duration::from_millis(180)),
                |veil, t| veil.opacity(t),
            )
    }

    /// A group as a stack: its terminal on top (header, state, live screen),
    /// the others as edges showing down and to the right behind it.
    #[allow(clippy::too_many_arguments)]
    fn stack(&self, group: &Project, top: &Terminal, chosen: bool, live: bool, width: f32, height: f32, cx: &App) -> Div {
        let depth = group.terminals.len().clamp(1, STACK_MAX);
        let more = group.terminals.len().saturating_sub(STACK_MAX);
        let step = STACK_STEP * width / SLIDER_CARD.0;
        let behind = step * (depth - 1) as f32;
        let alerts = self.group_alerts(group);
        let centres = self.card_centres.clone();
        let key = top.session.clone();
        // Where the stack lands, for the arrows; the enlarged one is centred
        // on its slot, so both give the same place.
        let spot = canvas(
            move |bounds, _, _| {
                centres.borrow_mut().insert(key, bounds.center());
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();
        let border = if chosen { p().accent } else { p().border };
        // The edges behind: readable on the veil, fainter the farther.
        let edge = if chosen { p().accent } else { p().muted };
        let front = div()
            .id(SharedString::from(format!("stack-{}{}", group.name, if live { "" } else { "-ghost" })))
            .relative()
            .child(spot)
            .flex()
            .flex_col()
            .w(px(width))
            .rounded_md()
            .overflow_hidden()
            .border_2()
            .border_color(border)
            .bg(p().bg)
            .when(chosen, |d| d.shadow_lg())
            // The one header: the group, its terminal on top, the counters.
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .bg(if chosen { p().active } else { p().surface })
                    .text_sm()
                    .child(div().flex_none().font_weight(FontWeight::BOLD).child(group.name.to_uppercase()))
                    .child(div().flex_1().min_w_0().truncate().text_color(p().muted).child(top.label.clone()))
                    .when(more > 0, |d| d.child(div().text_xs().text_color(p().muted).child(format!("+{more}"))))
                    .child(alerts.badges(format!("stack-{}", group.name))),
            )
            .when_some(top.status.as_ref(), |d, status| d.child(div().px_2().pb_1().bg(p().surface).child(status.line())))
            .child(
                div()
                    .h(px(height))
                    .overflow_hidden()
                    .bg(p().bg)
                    .when_some(self.screen(&top.session, cx).filter(|_| live), |d, screen| {
                        d.child(screen.viewport(CARD_LINES, CARD_COLUMNS))
                    }),
            );
        // The cards behind, farthest first: each the front's size, shifted.
        let mut stack = div().relative().pr(px(behind)).pb(px(behind));
        for i in (1..depth).rev() {
            let at = step * i as f32;
            stack = stack.child(
                div()
                    .absolute()
                    .left(px(at))
                    .top(px(at))
                    .right(px(behind - at))
                    .bottom(px(behind - at))
                    .rounded_md()
                    .border_2()
                    .border_color(edge.opacity(0.25 + 0.5 * (depth - i) as f32 / depth as f32))
                    .bg(p().surface),
            );
        }
        stack.child(front)
    }
}
