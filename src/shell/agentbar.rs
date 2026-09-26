//! The agent's bar, under its terminal: what claude-loop's tmux status line
//! says, drawn by tvty from aiball — who drives the loop (and a click to
//! hold or free it), its Claude's state, its events, the tickets it holds,
//! its wait credit, where it works. Agent-centric: the panel beside it is
//! the project's.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Shell;
use crate::status::{ago, now};
use crate::theme::p;

/// The bar's height: the terminal gives it that much, once.
pub const BAR_HEIGHT: f32 = 24.;

/// What the AFK chip offers: aiball's action, and its label.
const AFK_ACTIONS: &[(&str, &str)] = &[("off", "auto"), ("arm_10m", "hold 10 min"), ("arm_inf", "hold")];

impl Shell {
    /// The bar of the terminal shown, if an agent runs in it.
    pub(super) fn agent_bar(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let (project, terminal) = self.selected.as_deref().and_then(|s| self.terminal_of(s))?;
        let agent = terminal.agent.clone()?;
        let status = terminal.status.clone();
        let holds = self
            .board
            .tickets
            .get(project)
            .map_or(0, |rows| rows.iter().filter(|r| r.holder() == Some(agent.as_str())).count());
        let online = status.as_ref().is_some_and(|s| s.online);
        let driver = status.as_ref().map(|s| s.driver.as_str()).unwrap_or("");
        let (glyph, word, colour) = match driver {
            "wait" => ("‖", "held", p().warning),
            "stop" => ("✎", "you type", p().danger),
            "boot" => ("…", "boot", p().info),
            "loop" => ("▶", "auto", p().success),
            _ => ("·", "—", p().muted),
        };
        let item = || div().flex().items_center().gap_1().flex_none();
        let sep = || div().text_color(p().border).child("│");

        let afk = item()
            .id("agent-afk")
            .px_1p5()
            .rounded_sm()
            .cursor_pointer()
            .hover(|d| d.bg(p().hover))
            .when(self.afk_menu, |d| d.bg(p().active))
            .child(div().text_color(colour).child(glyph))
            .child(div().text_color(colour).child(word))
            .on_click(cx.listener(|shell, _, _, cx| {
                shell.afk_menu = !shell.afk_menu;
                cx.notify();
            }));
        let afk_choices = self.afk_menu.then(|| {
            let mut row = item();
            for (action, label) in AFK_ACTIONS {
                let (action, target) = (*action, agent.clone());
                row = row.child(
                    div()
                        .id(SharedString::from(format!("agent-afk-{action}")))
                        .px_1p5()
                        .rounded_sm()
                        .border_1()
                        .border_color(p().border)
                        .cursor_pointer()
                        .hover(|d| d.bg(p().hover))
                        .child(*label)
                        .on_click(cx.listener(move |shell, _, _, cx| shell.set_afk(target.clone(), action, cx))),
                );
            }
            row
        });

        let state = status.as_ref().filter(|_| online).map(|s| {
            let what = match s.state.as_str() {
                "busy" => "working",
                "boot" => "starting",
                _ => "idle",
            };
            let since = s.since.map(|since| format!(" · {}", ago(now().saturating_sub(since)))).unwrap_or_default();
            item()
                .text_color(if s.state == "busy" { p().accent } else { p().muted })
                .child(format!("{what}{since}"))
        });
        let unseen = status.as_ref().map_or(0, |s| s.unseen);
        let credit = status.as_ref().and_then(|s| s.credit);
        let cwd = status.as_ref().and_then(|s| s.cwd.clone()).map(|cwd| home_short(&cwd));

        Some(
            div()
                .id("agent-bar")
                .flex()
                .flex_none()
                .items_center()
                .gap_2()
                .h(px(BAR_HEIGHT))
                .px_2()
                .overflow_hidden()
                .bg(p().surface)
                .border_t_1()
                .border_color(p().border)
                .text_xs()
                .text_color(p().muted)
                .child(afk)
                .children(afk_choices)
                .child(sep())
                .child(if online {
                    item().children(state).into_any_element()
                } else {
                    item().text_color(p().danger).child("offline").into_any_element()
                })
                .child(sep())
                .child(item().when(unseen > 0, |d| d.text_color(p().text)).child(format!(
                    "{unseen} event{}",
                    if unseen == 1 { "" } else { "s" }
                )))
                .child(item().when(holds > 0, |d| d.text_color(p().text)).child(format!("holds {holds}")))
                .children(credit.map(|c| item().child(format!("credit {c} min"))))
                .children(self.afk_error.clone().map(|e| item().text_color(p().danger).child(e)))
                .child(div().flex_1())
                .child(item().text_color(p().text).child(agent.clone()))
                .children(cwd.map(|cwd| item().min_w_0().truncate().child(cwd)))
                .into_any_element(),
        )
    }

    /// Holds or frees the agent's loop through aiball; the bar follows at
    /// the next read of the board.
    fn set_afk(&mut self, agent: String, action: &'static str, cx: &mut Context<Self>) {
        self.afk_menu = false;
        self.afk_error = None;
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let done = cx.background_executor().spawn(async move { aiball.afk(&agent, action) }).await;
            let _ = this.update(cx, |shell, cx| {
                match done {
                    Ok(()) => {
                        let _ = shell.refresh_now.unbounded_send(crate::events::Change::All);
                    }
                    Err(error) => shell.afk_error = Some(short_error(&format!("{error:#}"))),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}

/// A path with the home directory as `~`.
fn home_short(path: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && path.starts_with(&home) => format!("~{}", &path[home.len()..]),
        _ => path.to_string(),
    }
}

/// An error, short enough for the bar: aiball's message, not the route.
fn short_error(error: &str) -> String {
    let message = error
        .split_once("\"error\":\"")
        .and_then(|(_, rest)| rest.split('"').next())
        .unwrap_or(error);
    format!("AFK: {message}")
}

#[cfg(test)]
mod tests {
    use super::short_error;

    #[test]
    fn an_error_says_what_aiball_said() {
        assert_eq!(
            short_error(r#"POST /api/agents/x/afk: 501 {"error":"node-relayed loop"}"#),
            "AFK: node-relayed loop"
        );
        assert_eq!(short_error("socket gone"), "AFK: socket gone");
    }
}
