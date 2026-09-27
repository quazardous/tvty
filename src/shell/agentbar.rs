//! The agent's bar, under its terminal: what claude-loop's tmux status line
//! says, drawn by tvty from aiball. When the loop pushes its bar (aiball's
//! `/api/consumers/:id/bar`), all of it: who drives the loop — and a click to
//! hold or free it —, its Claude's phase and passing state, the dialogs and
//! alerts, the prompt and a human typing, the proxy, its counters and its
//! next wake. Before a loop pushes one, what the agent's state says. Then its
//! events, the tickets it holds, where it works; a click on
//! its backlog lists it. Agent-centric: the panel beside it is the project's.

use std::time::Duration;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Shell;
use crate::aiball::{AgentBacklog, AgentBar};
use crate::status::{ago, now, parse_time};
use crate::theme::p;
use crate::tip::Tip as _;

/// The bar's height: the terminal gives it that much, once.
pub const BAR_HEIGHT: f32 = 24.;

/// What the AFK chip offers: aiball's action, and its label.
const AFK_ACTIONS: &[(&str, &str)] = &[("off", "auto"), ("arm_10m", "hold 10 min"), ("arm_inf", "hold")];

/// An agent's backlog, open over the bar: whose, and what aiball answered.
pub(super) struct BacklogView {
    agent: String,
    read: Option<Result<AgentBacklog, String>>,
}

/// Seconds from now to an ISO time: `None` when past or unreadable.
fn until(at: Option<&str>) -> Option<u64> {
    parse_time(at?)?.checked_sub(now()).filter(|s| *s > 0)
}

/// A backlog tier's name.
fn tier(t: Option<i64>) -> &'static str {
    match t {
        Some(-1) => "critical",
        Some(0) => "hot",
        Some(1) => "yours",
        Some(2) => "your decision pending",
        Some(3) => "waiting on them",
        Some(4) => "blocked",
        _ => "other",
    }
}

impl Shell {
    /// The bar of the terminal shown, if an agent runs in it.
    pub(super) fn agent_bar(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let (project, terminal) = self.selected.as_deref().and_then(|s| self.terminal_of(s))?;
        let agent = terminal.agent.clone()?;
        let status = terminal.status.clone();
        let bar = self.board.bars.get(&agent).filter(|b| !b.stale).map(|b| b.bar.clone());
        let holds = self
            .board
            .tickets
            .get(project)
            .map_or(0, |rows| rows.iter().filter(|r| r.holder() == Some(agent.as_str())).count());
        let online = bar.is_some() || status.as_ref().is_some_and(|s| s.online);
        let item = || div().flex().items_center().gap_1().flex_none();
        let sep = || div().text_color(p().border).child("│");
        let loud = |text: &'static str| {
            div()
                .flex_none()
                .px_1p5()
                .rounded_sm()
                .bg(p().danger)
                .text_color(crate::theme::on(p().danger))
                .child(text)
        };

        // ── Who drives the loop ──
        let presence = bar.as_ref().map(|b| b.presence.clone()).or_else(|| status.as_ref().map(|s| s.driver.clone()));
        let (glyph, word, colour) = match (presence.as_deref(), bar.as_ref().map(|b| b.afk.mode.as_str())) {
            (Some("stop"), _) => ("✎", "you type".to_string(), p().danger),
            // Held: claude-loop's little man, as its tmux bar draws it —
            // ∞ held for good, the seconds left of a ten-minute hold.
            (_, Some("wait_inf")) => ("웃", "∞".to_string(), p().danger),
            (_, Some("wait_10m")) => {
                let left = bar.as_ref().and_then(|b| until(b.afk.expires_at.as_deref()));
                ("웃", left.map_or("held".into(), |s| format!("{s}s")), p().warning)
            }
            (Some("wait"), _) => ("웃", "held".to_string(), p().warning),
            (Some("boot"), _) => ("…", "boot".to_string(), p().info),
            (Some("loop"), _) => ("▶", "auto".to_string(), p().success),
            _ => ("·", "—".to_string(), p().muted),
        };
        let afk = item()
            .id("agent-afk")
            .px_1p5()
            .rounded_sm()
            .cursor_pointer()
            .hover(|d| d.bg(p().hover))
            .when(self.afk_menu, |d| d.bg(p().active))
            .child(div().text_color(colour).child(glyph))
            .child(div().text_color(colour).child(word))
            .tip("who drives the loop: on its own (▶ auto), held by you (웃: ∞ for good, or the seconds left), or you typing (✎); a click holds or frees it")
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

        // ── What its Claude does ──
        let phase = bar.as_ref().map(|b| b.phase.clone()).or_else(|| status.as_ref().map(|s| s.state.clone()));
        let what = match phase.as_deref() {
            Some("busy") => "working",
            Some("boot") => "starting",
            _ => "idle",
        };
        let since = match &bar {
            Some(b) if b.phase == "boot" => b
                .boot
                .as_ref()
                .and_then(|boot| parse_time(&boot.started_at))
                .map(|s| format!(" · {}", ago(now().saturating_sub(s)))),
            _ => status
                .as_ref()
                .and_then(|s| s.since)
                .map(|s| format!(" · {}", ago(now().saturating_sub(s)))),
        }
        .unwrap_or_default();
        let info = bar.as_ref().and_then(|b| b.marker.info.clone()).map(|i| format!(" · {i}")).unwrap_or_default();
        let state = item()
            .id("agent-state")
            .text_color(if phase.as_deref() == Some("busy") { p().accent } else { p().muted })
            .child(format!("{what}{since}{info}"))
            .tip("what its Claude does, and since when");
        // Claude Code updated itself: a click restarts it. aiball refuses
        // while Claude works: then a click arms it, and tvty restarts it as
        // soon as it is idle.
        let restart = bar.as_ref().filter(|b| b.alerts.restart_needed).map(|b| {
            let restarting = self.restarting.as_deref() == Some(agent.as_str());
            let armed = self.restart_armed.contains(&agent);
            let idle = b.phase == "idle";
            let target = agent.clone();
            let (word, tip) = match (restarting, armed, idle) {
                (true, _, _) => ("restarting…", "its Claude restarts, resuming its conversation"),
                (_, true, _) => ("armed", "it restarts as soon as its Claude is idle; a click disarms"),
                (_, _, true) => ("restart", "its Claude Code installed an update: a click restarts it, resuming its conversation"),
                _ => ("when idle", "its Claude works: a click restarts it as soon as it is idle"),
            };
            item()
                .id("agent-restart")
                .px_1p5()
                .rounded_sm()
                .border_1()
                .border_color(p().warning)
                .text_color(p().warning)
                .when(armed, |d| d.bg(p().active))
                .when(!restarting, |d| {
                    d.cursor_pointer()
                        .hover(|d| d.bg(p().hover))
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            if !shell.restart_armed.remove(&target) {
                                if idle {
                                    shell.restart_claude(target.clone(), cx);
                                } else {
                                    shell.restart_armed.insert(target.clone());
                                }
                            }
                            cx.notify();
                        }))
                })
                .child("⟳")
                .child(word)
                .tip(tip)
        });
        let dialog = bar.as_ref().filter(|b| b.marker.health_prompt || b.marker.resume_picker || b.marker.resume_mode_picker);

        // ── The rest, from the loop when it tells ──
        let unseen = bar
            .as_ref()
            .and_then(|b| b.counters.as_ref())
            .and_then(|c| c.events)
            .or_else(|| status.as_ref().map(|s| s.unseen))
            .unwrap_or(0);
        let backlog = bar.as_ref().and_then(|b| b.counters.as_ref()).and_then(|c| c.backlog);
        let wake = bar.as_ref().and_then(|b| until(b.next_wake_at.as_deref()));
        // Work waits for the loop (events, backlog): the envelope stands, with
        // the countdown to the next wake once it is armed — as claude-loop's
        // tmux line shows it.
        let pending = unseen > 0 || backlog.is_some_and(|b| b > 0);
        let cwd = status.as_ref().and_then(|s| s.cwd.clone()).map(|cwd| home_short(&cwd));
        let backlog_open = self.backlog_view.as_ref().is_some_and(|v| v.agent == agent);
        let target = (agent.clone(), project.to_string());

        Some(
            div()
                .id("agent-bar")
                .relative()
                .flex()
                .flex_none()
                .items_center()
                .gap_2()
                .h(px(BAR_HEIGHT))
                .px_2()
                .bg(p().surface)
                .border_t_1()
                .border_color(p().border)
                .text_xs()
                .text_color(p().muted)
                .child(afk)
                .children(afk_choices)
                .child(sep())
                .child(if online {
                    state.into_any_element()
                } else {
                    item().text_color(p().danger).child("offline").into_any_element()
                })
                .children(restart)
                .children(dialog.map(|_| item().text_color(p().warning).child("waits for an answer")))
                .when_some(bar.as_ref(), |d, b| {
                    let a = &b.alerts;
                    d.when(a.trust_dialog, |d| d.child(loud("trust this folder?")))
                        .when(a.not_logged_in, |d| d.child(loud("not logged in")))
                        .when(a.api_unreachable, |d| d.child(loud("API unreachable")))
                        .when(a.link_down, |d| d.child(loud("loop link down")))
                        .when(a.daemon_down, |d| d.child(loud("aiball unreachable")))
                        .when(b.prompt.visible, |d| {
                            d.child(
                                item()
                                    .id("agent-prompt")
                                    .text_color(if b.prompt.has_input { p().accent } else { p().muted })
                                    .child("❯")
                                    .tip(if b.prompt.has_input {
                                        "Claude's prompt is on screen, with text not sent yet"
                                    } else {
                                        "Claude's prompt is on screen, empty"
                                    }),
                            )
                        })
                        .when(b.human_typing, |d| {
                            d.child(
                                item()
                                    .id("agent-typing")
                                    .text_color(p().danger)
                                    .child("⌨")
                                    .tip("a human typed in its terminal a moment ago: the loop holds off"),
                            )
                        })
                        .when(b.proxy_alive, |d| {
                            d.child(item().id("agent-proxy").child("⇄").tip("the terminal proxy in front of Claude is alive"))
                        })
                        .when(b.zen, |d| d.child(item().id("agent-zen").child("zen").tip("zen mode: the loop keeps quiet")))
                })
                .child(sep())
                .child(
                    item()
                        .id("agent-events")
                        .when(unseen > 0, |d| d.text_color(p().text))
                        .child(format!("{unseen} event{}", if unseen == 1 { "" } else { "s" }))
                        .tip("its events not seen yet: pings, answers, decisions waiting for it"),
                )
                .child(
                    item()
                        .id("agent-backlog")
                        .px_1()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|d| d.bg(p().hover))
                        .when(backlog_open, |d| d.bg(p().active))
                        .when(holds > 0 || backlog.is_some_and(|b| b > 0), |d| d.text_color(p().text))
                        .child(match backlog {
                            Some(b) => format!("backlog {b} · holds {holds}"),
                            None => format!("holds {holds}"),
                        })
                        .tip("its backlog (tickets for it to look at) and the tickets it holds; a click lists them")
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            shell.toggle_backlog(target.0.clone(), target.1.clone(), cx)
                        })),
                )
                .when(pending || wake.is_some(), |d| {
                    d.child(
                        item()
                            .id("agent-wake")
                            .when(wake.is_some(), |d| d.text_color(p().text))
                            .child("✉")
                            .children(wake.map(ago))
                            .tip(match wake {
                                Some(_) => "work waits for the loop: it wakes its Claude on it when the countdown ends",
                                None => "work waits for the loop (events or backlog)",
                            }),
                    )
                })
                .child(div().flex_1())
                .child(item().text_color(p().text).child(agent.clone()))
                .children(cwd.map(|cwd| item().min_w_0().truncate().child(cwd)))
                .children(self.backlog_view.as_ref().filter(|v| v.agent == agent).map(|v| self.backlog_list(v, cx)))
                .into_any_element(),
        )
    }

    /// The agent's own backlog, over its bar: its tickets by tier, a click
    /// opens one in the panel.
    fn backlog_list(&self, view: &BacklogView, cx: &mut Context<Self>) -> AnyElement {
        let mut list = div()
            .id("agent-backlog-list")
            .absolute()
            .bottom(px(BAR_HEIGHT))
            .left(px(8.))
            .w(px(460.))
            .max_h(px(360.))
            .overflow_y_scroll()
            .occlude()
            .flex()
            .flex_col()
            .p_2()
            .gap_0p5()
            .rounded_md()
            .bg(p().surface)
            .border_1()
            .border_color(p().border)
            .shadow_lg()
            .text_sm()
            .text_color(p().text)
            .child(div().text_xs().text_color(p().muted).pb_1().child(format!("{}'s backlog", view.agent)));
        match &view.read {
            None => list = list.child(div().text_color(p().muted).child("reading…")),
            Some(Err(error)) => list = list.child(div().text_color(p().danger).child(error.clone())),
            Some(Ok(backlog)) if backlog.rows.iter().all(|r| r.backlog_tier.is_none()) => {
                list = list.child(div().text_color(p().muted).child("nothing in its backlog"));
            }
            Some(Ok(backlog)) => {
                let mut rows: Vec<_> = backlog.rows.iter().filter(|r| r.backlog_tier.is_some()).collect();
                rows.sort_by_key(|r| r.backlog_tier);
                let mut last = None;
                for row in rows {
                    if last != Some(row.backlog_tier) {
                        last = Some(row.backlog_tier);
                        list = list.child(
                            div()
                                .pt_1()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(p().muted)
                                .child(tier(row.backlog_tier).to_uppercase()),
                        );
                    }
                    let (project, ticket) = (row.project.clone(), row.id);
                    list = list.child(
                        div()
                            .id(SharedString::from(format!("backlog-{ticket}")))
                            .flex()
                            .gap_2()
                            .px_1()
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|d| d.bg(p().hover))
                            .child(div().flex_none().text_color(p().muted).child(format!("#{ticket}")))
                            .child(div().flex_1().min_w_0().truncate().child(row.title.clone()))
                            .on_click(cx.listener(move |shell, _, _, cx| {
                                shell.backlog_view = None;
                                if !shell.settings.panel_open {
                                    shell.toggle_panel(cx);
                                }
                                shell.panel.update(cx, |panel, cx| panel.open_in(Some(project.clone()), ticket, cx));
                                cx.notify();
                            })),
                    );
                }
            }
        }
        list.into_any_element()
    }

    /// Opens (reading it) or closes the agent's backlog over its bar.
    fn toggle_backlog(&mut self, agent: String, project: String, cx: &mut Context<Self>) {
        if self.backlog_view.as_ref().is_some_and(|v| v.agent == agent) {
            self.backlog_view = None;
            cx.notify();
            return;
        }
        self.backlog_view = Some(BacklogView { agent: agent.clone(), read: None });
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let read = cx
                .background_executor()
                .spawn({
                    let agent = agent.clone();
                    async move { aiball.agent_backlog(&agent, &project) }
                })
                .await;
            let _ = this.update(cx, |shell, cx| {
                if let Some(view) = shell.backlog_view.as_mut().filter(|v| v.agent == agent) {
                    view.read = Some(read.map_err(|e| format!("backlog: {}", short_error(&format!("{e:#}")))));
                    cx.notify();
                }
            });
        })
        .detach();
        cx.notify();
    }

    /// While the bar shows a countdown (a hold, the next wake, a boot), it
    /// is drawn again every second.
    /// The times the window shows, as text: the sessions' states (for how
    /// long), and the selected agent's bar.
    fn clock_face(&self) -> Vec<String> {
        let now = now();
        let mut face: Vec<String> = self
            .board
            .projects
            .iter()
            .flat_map(|p| &p.terminals)
            .filter_map(|t| t.status.as_ref().filter(|s| s.online)?.since)
            .map(|since| ago(now.saturating_sub(since)))
            .collect();
        let bar = self
            .selected
            .as_deref()
            .and_then(|s| self.terminal_of(s))
            .and_then(|(_, t)| t.agent.as_ref())
            .and_then(|a| self.board.bars.get(a))
            .filter(|b| !b.stale);
        if let Some(bar) = bar {
            face.extend(bar_times(&bar.bar));
        }
        face
    }

    /// The clock: every second, the times the window shows (the sessions'
    /// states, the selected agent's bar) are read as text, and the window is
    /// drawn again only when one of them changed — once a minute for most,
    /// every second only for what is under a minute.
    pub(super) fn tick_clock(cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let mut shown = Vec::new();
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let alive = this.update(cx, |shell, cx| {
                    let face = shell.clock_face();
                    if face != shown {
                        shown = face;
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    /// The armed restarts whose Claude is idle now go; those no longer
    /// needed are dropped.
    pub(super) fn fire_armed_restarts(&mut self, cx: &mut Context<Self>) {
        let armed: Vec<String> = self.restart_armed.iter().cloned().collect();
        for agent in armed {
            let Some(bar) = self.board.bars.get(&agent).filter(|b| !b.stale) else { continue };
            if !bar.bar.alerts.restart_needed {
                self.restart_armed.remove(&agent);
            } else if bar.bar.phase == "idle" && self.restarting.is_none() {
                self.restart_armed.remove(&agent);
                self.restart_claude(agent, cx);
            }
        }
    }

    /// Restarts the agent's Claude Code (it installed an update): aiball
    /// waits for it to be idle, restarts it resuming its conversation, and
    /// tells the agent to carry on. Its bar then no longer asks for it.
    fn restart_claude(&mut self, agent: String, cx: &mut Context<Self>) {
        self.restarting = Some(agent.clone());
        cx.notify();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let name = agent.clone();
            let done = cx.background_executor().spawn(async move { aiball.restart_claude(&name) }).await;
            let _ = this.update(cx, |shell, cx| {
                shell.restarting = None;
                let activity = match done {
                    Ok(()) => crate::activity::Activity::done(None, format!("{agent}'s Claude restarts, resuming its conversation")),
                    // Busy again by the time it went: armed again, it goes
                    // once Claude is idle.
                    Err(error) if format!("{error:#}").contains("(NOT_IDLE)") => {
                        shell.restart_armed.insert(agent.clone());
                        cx.notify();
                        return;
                    }
                    Err(error) => crate::activity::Activity::failed(None, &format!("restart of {agent}'s Claude"), short_error(&format!("{error:#}"))),
                };
                crate::activity::publish(cx, activity);
                cx.notify();
            });
        })
        .detach();
    }

    /// Holds or frees the agent's loop through aiball; the bar follows at
    /// the next read of the board.
    fn set_afk(&mut self, agent: String, action: &'static str, cx: &mut Context<Self>) {
        self.afk_menu = false;
        let aiball = self.aiball.clone();
        let agent_name = agent.clone();
        cx.spawn(async move |this, cx| {
            let done = cx.background_executor().spawn(async move { aiball.afk(&agent, action) }).await;
            let _ = this.update(cx, |shell, cx| {
                match done {
                    Ok(()) => {
                        let _ = shell.refresh_now.unbounded_send(());
                    }
                    Err(error) => crate::activity::publish(
                        cx,
                        crate::activity::Activity::failed(None, &format!("hold of {agent_name}"), short_error(&format!("{error:#}"))),
                    ),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}

/// The times a bar shows, as text: a hold's seconds left, which tick.
fn bar_times(bar: &AgentBar) -> impl Iterator<Item = String> {
    let hold = until(bar.afk.expires_at.as_deref()).map(|s| format!("{s}s"));
    let others = [
        until(bar.next_wake_at.as_deref()),
        bar.boot.as_ref().and_then(|b| parse_time(&b.started_at)).map(|s| now().saturating_sub(s)),
    ]
    .into_iter()
    .flatten()
    .map(ago);
    hold.into_iter().chain(others)
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
    message.to_string()
}

#[cfg(test)]
mod tests {
    use super::{short_error, tier};

    #[test]
    fn an_error_says_what_aiball_said() {
        assert_eq!(
            short_error(r#"POST /api/agents/x/afk: 501 {"error":"node-relayed loop"}"#),
            "node-relayed loop"
        );
        assert_eq!(short_error("socket gone"), "socket gone");
    }

    #[test]
    fn tiers_have_names() {
        assert_eq!(tier(Some(-1)), "critical");
        assert_eq!(tier(Some(1)), "yours");
        assert_eq!(tier(None), "other");
    }
}
