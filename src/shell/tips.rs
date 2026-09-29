//! The tips' card, in a bottom corner, over the work but never in its way: "Did
//! you know?", a tip, and what to do with it — got it (never again), next,
//! × (not now), or no tips at all. The menu's Tips… shows them all, one
//! after the other.

use std::time::Duration;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::component::button::{ButtonCustomVariant, ButtonVariants as _};
use gpui_kit::*;

use super::Shell;
use crate::activity::{self, Activity};
use crate::notify::Kind;
use crate::theme::p;
use crate::tips::{self, Mark};
use crate::ui::buttons;
use tvty_config::Value;

/// The card's width, and the room it needs beside its target.
const CARD_WIDTH: f32 = 380.;
const CARD_ROOM: f32 = 190.;
/// The least room between the card and its target: the target, and what is
/// around it, stay in sight.
const TARGET_GAP: f32 = 24.;

/// The start's own tip comes once the work is on screen.
const AFTER_START: Duration = Duration::from_secs(5);

/// The tip on screen.
pub(super) struct TipCard {
    id: String,
    /// Its surface: Next stays there.
    surface: String,
    /// From the menu: every tip, this one's place among them.
    browsing: Option<usize>,
}

impl Shell {
    /// The tip of the start, and a tip's end once its command is run.
    pub(super) fn start_tips(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(AFTER_START).await;
            let _ = this.update(cx, |shell, cx| shell.offer_tip("Workspace", cx));
        })
        .detach();
        cx.observe_keystrokes(|shell, event, _, cx| {
            let Some(command) = event.action.as_ref().and_then(|a| tips::command_of(a.name())) else { return };
            tips::used(cx, command);
            // Its own tip on screen: it is known now.
            let own = shell.tip.as_ref().is_some_and(|t| {
                t.browsing.is_none() && tips::all().iter().any(|tip| tip.id == t.id && tip.command.as_deref() == Some(command))
            });
            if own {
                shell.tip = None;
                cx.notify();
            }
        })
        .detach();
    }

    /// The surface shown, as drawn: its first entry offers its tip.
    pub(super) fn surface_drawn(&mut self, cx: &mut Context<Self>) {
        // A frame: where the targets are painted from now on, and which one
        // wears the halo (set before they are built).
        tips::next_frame();
        let target = self
            .tip
            .as_ref()
            .filter(|_| self.ask.is_none())
            .and_then(|card| tips::all().iter().find(|t| t.id == card.id))
            .and_then(|t| t.target.clone());
        tips::set_active(target);
        let surface = self.page_shown(cx).unwrap_or("Workspace");
        if self.tip_surface == Some(surface) {
            return;
        }
        self.tip_surface = Some(surface);
        // A page's tip goes with the page (not one browsed from the menu).
        if self.tip.as_ref().is_some_and(|t| t.browsing.is_none() && t.surface != surface) {
            self.tip = None;
        }
        // The workspace's comes after start; a page's, the first time.
        if surface != "Workspace" && tips::entered(cx, surface) {
            self.offer_tip(surface, cx);
        }
    }

    /// A tip for `surface`, if tips are on, none is up, and one is left.
    fn offer_tip(&mut self, surface: &str, cx: &mut Context<Self>) {
        // One at a time, and never over a dialog.
        if !self.applied.tips.show || self.tip.is_some() || self.ask.is_some() || self.stopping_all {
            return;
        }
        self.show_tip(surface, None, cx);
    }

    fn show_tip(&mut self, surface: &str, after: Option<&str>, cx: &mut Context<Self>) {
        let Some(tip) = tips::pick(tips::all(), tips::seen(cx), surface, after, tips::now()) else {
            self.tip = None;
            cx.notify();
            return;
        };
        log::info!("tips: {}", tip.id);
        tips::shown_now(cx, &tip.id);
        self.tip = Some(TipCard { id: tip.id.clone(), surface: surface.to_string(), browsing: None });
        cx.notify();
    }

    /// The menu's Tips…: every tip, from the first.
    pub(super) fn browse_tips(&mut self, cx: &mut Context<Self>) {
        self.browse_tip(0, cx);
    }

    fn browse_tip(&mut self, at: usize, cx: &mut Context<Self>) {
        let all = tips::all();
        let Some(tip) = all.get(at % all.len().max(1)) else { return };
        self.tip = Some(TipCard { id: tip.id.clone(), surface: tip.surface.clone(), browsing: Some(at % all.len()) });
        cx.notify();
    }

    fn tip_next(&mut self, cx: &mut Context<Self>) {
        let Some(card) = self.tip.as_ref() else { return };
        match card.browsing {
            Some(at) => self.browse_tip(at + 1, cx),
            None => {
                let (surface, id) = (card.surface.clone(), card.id.clone());
                self.show_tip(&surface, Some(&id), cx);
            }
        }
    }

    fn tip_back(&mut self, cx: &mut Context<Self>) {
        if let Some(at) = self.tip.as_ref().and_then(|c| c.browsing) {
            self.browse_tip(at + tips::all().len() - 1, cx);
        }
    }

    fn tip_got(&mut self, cx: &mut Context<Self>) {
        if let Some(card) = self.tip.take() {
            tips::got(cx, &card.id);
        }
        cx.notify();
    }

    fn tip_close(&mut self, cx: &mut Context<Self>) {
        self.tip = None;
        cx.notify();
    }

    fn tips_off(&mut self, cx: &mut Context<Self>) {
        self.tip = None;
        self.set_pref("tips.show", Value::Toggle(false), cx);
        activity::publish(cx, Activity::news("tvty", Kind::Info, None, "Tips are off: Options > Layout > Tips turns them back on"));
    }

    fn tips_again(&mut self, cx: &mut Context<Self>) {
        tips::forget(cx);
        if !self.applied.tips.show {
            self.set_pref("tips.show", Value::Toggle(true), cx);
        }
        activity::publish(cx, Activity::news("tvty", Kind::Info, None, "Every tip will show again, one at a time"));
        cx.notify();
    }

    /// The card, bottom left.
    pub(super) fn tip_view(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        // A dialog asks: the tip waits.
        if self.ask.is_some() {
            return None;
        }
        let card = self.tip.as_ref()?;
        let tip = tips::all().iter().find(|t| t.id == card.id)?;
        let keymap = crate::keymap::current(cx);
        let (text, marks) = tips::shown(tip, |command| keymap.keys_of(command).first().map(|k| k.pretty()));
        let highlights: Vec<_> = marks
            .into_iter()
            .map(|(range, mark)| {
                (range, match mark {
                    Mark::Bold => HighlightStyle { font_weight: Some(FontWeight::BOLD), color: Some(p().text), ..Default::default() },
                    Mark::Code => HighlightStyle { color: Some(p().accent), background_color: Some(p().hover), ..Default::default() },
                })
            })
            .collect();
        let title = match card.browsing {
            Some(at) => format!("Tips · {} / {}", at + 1, tips::all().len()),
            None => "Did you know?".to_string(),
        };
        // The tips' own colour (not the accent's blue, which says other things).
        let violet = crate::theme::tip();
        let head = div()
            .flex()
            .items_center()
            .child(div().flex_1().text_xs().font_weight(FontWeight::BOLD).text_color(violet).child(title))
            .child(buttons::remove("tip-close", "✕", "Not now").px_1().on_click(cx.listener(|shell, _, _, cx| shell.tip_close(cx))));
        let actions = match card.browsing {
            Some(_) => div()
                .flex()
                .items_center()
                .gap_3()
                .child(buttons::link("tip-back", "‹ Previous").on_click(cx.listener(|shell, _, _, cx| shell.tip_back(cx))))
                .child(buttons::link("tip-next", "Next ›").on_click(cx.listener(|shell, _, _, cx| shell.tip_next(cx))))
                .child(div().flex_1())
                .child(buttons::link("tips-again", "Show them all again").text_xs().on_click(cx.listener(|shell, _, _, cx| shell.tips_again(cx)))),
            None => div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    buttons::answer("tip-got", "Got it")
                        .custom(ButtonCustomVariant::new(cx).color(violet).foreground(crate::theme::on(violet)).hover(violet.opacity(0.85)).active(violet.opacity(0.7)))
                        .on_click(cx.listener(|shell, _, _, cx| shell.tip_got(cx))),
                )
                .child(buttons::link("tip-next", "Next tip").on_click(cx.listener(|shell, _, _, cx| shell.tip_next(cx))))
                .child(div().flex_1())
                .child(
                    buttons::link("tips-off", "Turn tips off")
                        .text_xs()
                        .text_color(p().muted)
                        .on_click(cx.listener(|shell, _, _, cx| shell.tips_off(cx))),
                ),
        };
        let body = div()
            .id("tip-card")
            .occlude()
            .w(px(CARD_WIDTH))
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .rounded_md()
            .bg(p().surface)
            .border_1()
            .border_color(violet.opacity(0.6))
            .shadow_lg()
            .text_sm()
            .text_color(p().muted)
            .child(head)
            .child(StyledText::new(text).with_highlights(highlights))
            .child(actions);
        // Beside its target when it has one on screen: under it, or above
        // when there is no room below; a little pointer towards it.
        // Beside its target when it has one on screen, never over it: under
        // it or above, at a distance that leaves the target and what is
        // around it in sight. No room either way: its corner. Placed in the
        // shell's root, from where the root and the target were painted.
        let root = tips::bounds_of(tips::ROOT);
        let placed = tip.target.as_deref().and_then(tips::bounds_of).zip(root).and_then(|(target, root)| {
            let (width, margin, gap, room) = (px(CARD_WIDTH), px(8.), px(TARGET_GAP), px(CARD_ROOM));
            let left = (target.left() - root.left()).max(margin).min((root.size.width - width - margin).max(margin));
            let (top, bottom) = (target.top() - root.top(), target.bottom() - root.top());
            if root.size.height - bottom - gap >= room {
                Some(div().absolute().left(left).top(bottom + gap))
            } else if top - gap >= room {
                Some(div().absolute().left(left).bottom(root.size.height - top + gap))
            } else {
                None
            }
        });
        if let Some(place) = placed {
            return Some(place.child(body).into_any_element());
        }
        // Else bottom left; over a full page, where the notifications take
        // that corner, bottom right.
        let page = self.page_shown(cx).is_some();
        Some(
            div()
                .absolute()
                .when(page, |d| d.right(px(12.)))
                .when(!page, |d| d.left(px(12.)))
                .bottom(px(44.))
                .child(body)
                .into_any_element(),
        )
    }
}
