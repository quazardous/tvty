//! The usage arrow in the title bar ([`crate::usage`]): red and up when the
//! subscription goes faster than the pace that would use it up right at the
//! window's end, green and down when slower, deeper as the gap grows. A
//! click says the gap another way, until tvty restarts.

use gpui_kit::*;

use super::{Shell, press_kept};
use crate::theme::p;
use crate::ui::buttons;

/// The arrow's colour at no gap: a hint, deeper up to [`crate::usage::FULL_AT`].
const FAINTEST: f32 = 0.4;

/// The steady pace moves with the clock (a third of a point a minute over
/// five hours): the arrow follows it while no reading comes.
const REDRAW_EVERY: std::time::Duration = std::time::Duration::from_secs(60);

impl Shell {
    pub(super) fn follow_usage_pace(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(REDRAW_EVERY).await;
                let shown = this.update(cx, |shell, cx| {
                    if crate::usage::freshest(shell.board.bars.values()).is_some() {
                        cx.notify();
                    }
                });
                if shown.is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    /// None until a loop read the usage from its Claude.
    pub(super) fn usage_arrow(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        let now = crate::status::now();
        let usage = crate::usage::freshest(self.board.bars.values())?;
        let pace = crate::usage::pace(usage, now)?;
        let display = self.usage_display.unwrap_or_else(|| self.applied.usage.display());
        let (arrow, colour) = if pace.above() { ("↑", p().danger) } else { ("↓", p().success) };
        let depth = FAINTEST + (1. - FAINTEST) * pace.heat().abs() as f32;
        let figure = crate::usage::said(&pace, display, now);
        Some(
            press_kept(buttons::coloured("usage-arrow", format!("{arrow} {figure}"), crate::usage::tip(usage, now)))
                .text_xs()
                .text_color(colour.opacity(depth))
                .flex_none()
                .mr_2()
                .on_click(cx.listener(move |shell, _, _, cx| {
                    cx.stop_propagation();
                    shell.usage_display = Some(display.next());
                    cx.notify();
                })),
        )
    }
}
