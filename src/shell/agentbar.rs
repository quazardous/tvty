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

/// A mark's colour, as claude-loop's tmux bar paints it.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tone {
    Green,
    Orange,
    Red,
    Grey,
    Boot,
}

/// What the agent bar shows of who drives the loop, as claude-loop's bar:
/// the mode in force (▶ the loop runs on its own, ‖ held — typing holds
/// too), ⌨ while a human types, and the little man with the mode armed by
/// F9 (grey: away, so auto; the seconds of a ten-minute hold; ∞). F9 moves
/// the little man at once and the mode in force 3 s after the last press:
/// while they differ, it is arming.
#[derive(Debug, PartialEq)]
struct AfkMarks {
    in_force: Option<(&'static str, Tone)>,
    typing: bool,
    armed: Option<(String, Tone)>,
    arming: bool,
}

fn afk_marks(presence: Option<&str>, armed: Option<&str>, left: Option<u64>, typing: bool) -> AfkMarks {
    if presence == Some("boot") {
        return AfkMarks { in_force: Some(("… boot", Tone::Boot)), typing: false, armed: None, arming: false };
    }
    let in_force = match presence {
        Some("loop") => Some(("▶", Tone::Green)),
        Some("wait") | Some("stop") => Some(("‖", Tone::Orange)),
        _ => None,
    };
    let man = match armed {
        Some("wait_inf") => Some(("웃∞".to_string(), Tone::Red)),
        Some("wait_10m") => Some((left.map_or("웃".to_string(), |s| format!("웃{s}s")), Tone::Orange)),
        Some("off") => Some(("웃".to_string(), Tone::Grey)),
        _ => None,
    };
    // Typing holds the loop without a choice of F9's: not arming.
    let arming = match armed {
        Some("off") => presence == Some("wait"),
        Some("wait_10m") | Some("wait_inf") => presence == Some("loop"),
        _ => false,
    };
    AfkMarks { in_force, typing: typing || presence == Some("stop"), armed: man, arming }
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

        // ── Who drives the loop, as claude-loop's bar says it ──
        let presence = bar.as_ref().map(|b| b.presence.clone()).or_else(|| status.as_ref().map(|s| s.driver.clone()));
        let marks = afk_marks(
            presence.as_deref(),
            bar.as_ref().map(|b| b.afk.mode.as_str()),
            bar.as_ref().and_then(|b| until(b.afk.expires_at.as_deref())),
            bar.as_ref().is_some_and(|b| b.human_typing),
        );
        let tone = |t: Tone| match t {
            Tone::Green => p().success,
            Tone::Orange => p().warning,
            Tone::Red => p().danger,
            Tone::Grey => p().muted,
            // On the bar's yellow while it boots.
            Tone::Boot => crate::theme::on(p().warning),
        };
        let afk = item()
            .id("agent-afk")
            .px_1p5()
            .rounded_sm()
            .cursor_pointer()
            .hover(|d| d.bg(p().hover))
            .when(self.afk_menu, |d| d.bg(p().active))
            .children(marks.in_force.map(|(glyph, t)| div().text_color(tone(t)).child(glyph)))
            .when(marks.typing, |d| d.child(div().text_color(p().danger).child("⌨")))
            .children(marks.armed.map(|(man, t)| {
                div()
                    .flex()
                    .text_color(tone(t))
                    .when(marks.arming, |d| d.border_b_1().border_dashed().border_color(tone(t)))
                    .child(man)
                    .when(marks.arming, |d| d.child("…"))
            }))
            .tip(if marks.arming {
                "armed: the little man shows the mode chosen with F9, in force 3 s after the last press — ▶ or ‖ says the one in force until then"
            } else {
                "who drives the loop: ▶ on its own, ‖ held for you, ⌨ you are typing; the little man is the AFK mode — grey: you are away, the loop runs on its own; the seconds of a 10 min hold; ∞ held. F9 cycles it (auto → 10 min → ∞), in force 3 s after the last press; a click chooses"
            })
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
        let booting = phase.as_deref() == Some("boot");
        // On the bar's yellow while its loop boots, every text in one ink.
        let ink = |colour: Hsla| if booting { crate::theme::on(p().warning) } else { colour };
        let since = match &bar {
            // As claude-loop's 🚀: since when, and how long it has left —
            // the boot lasts until its deadline, pushed back while a resume
            // or a compaction shows.
            Some(b) if b.phase == "boot" => b.boot.as_ref().and_then(|boot| {
                let started = parse_time(&boot.started_at)?;
                let left = until(boot.deadline_at.as_deref()).map(|s| format!(" · {}s left", s)).unwrap_or_default();
                Some(format!(" · {}{left}", ago(now().saturating_sub(started))))
            }),
            _ => status
                .as_ref()
                .and_then(|s| s.since)
                .map(|s| format!(" · {}", ago(now().saturating_sub(s)))),
        }
        .unwrap_or_default();
        let info = bar.as_ref().and_then(|b| b.marker.info.clone()).map(|i| format!(" · {i}")).unwrap_or_default();
        let state = item()
            .id("agent-state")
            .text_color(match phase.as_deref() {
                Some("busy") => p().accent,
                Some("boot") => crate::theme::on(p().warning),
                _ => p().muted,
            })
            .child(format!("{what}{since}{info}"))
            .tip(if booting {
                "its loop is starting: Claude loads, may resume its conversation or compact; the loop wakes it once the boot ends (30 s at least, longer while a resume or a compaction shows)"
            } else {
                "what its Claude does, and since when"
            });
        // Claude Code updated itself: a click restarts it — its loop waits
        // for Claude's next idle (at once when it is), and its bar says a
        // restart is pending meanwhile, whoever asked for it.
        let restart = bar.as_ref().filter(|b| b.alerts.restart_needed || b.alerts.restart_pending).map(|b| {
            let restarting = self.restarting.as_deref() == Some(agent.as_str());
            let armed = b.alerts.restart_pending;
            let idle = b.phase == "idle";
            let target = agent.clone();
            let (word, tip) = match (restarting, armed, idle) {
                (true, _, _) => ("restarting…", "its Claude restarts, resuming its conversation"),
                (_, true, _) => ("when idle", "a restart is asked: its Claude restarts as soon as it is idle, resuming its conversation"),
                (_, _, true) => ("restart", "its Claude Code installed an update: a click restarts it, resuming its conversation"),
                _ => ("restart when idle", "its Claude works: a click restarts it as soon as it is idle, resuming its conversation"),
            };
            item()
                .id("agent-restart")
                .px_1p5()
                .rounded_sm()
                .border_1()
                .border_color(p().warning)
                .text_color(p().warning)
                .when(armed, |d| d.bg(p().active))
                .when(!restarting && !armed, |d| {
                    d.cursor_pointer()
                        .hover(|d| d.bg(p().hover))
                        .on_click(cx.listener(move |shell, _, _, cx| shell.restart_claude(target.clone(), cx)))
                })
                .child("⟳")
                .child(word)
                .tip(tip)
        });
        let dialog = bar.as_ref().filter(|b| b.marker.health_prompt || b.marker.resume_picker || b.marker.resume_mode_picker);

        // ── The rest, from the loop when it tells ──
        let counts = self.counts_of(project, terminal).unwrap_or_default();
        let (unseen, backlog) = (counts.events, counts.backlog);
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
                // Yellow while its loop boots, as claude-loop's bar.
                .when(booting, |d| d.bg(p().warning).text_color(crate::theme::on(p().warning)))
                .border_t_1()
                .border_color(p().border)
                .text_xs()
                .text_color(ink(p().muted))
                .child(afk)
                .children(afk_choices)
                .child(sep())
                .child(if online {
                    state.into_any_element()
                } else {
                    item().text_color(ink(p().danger)).child("offline").into_any_element()
                })
                .children(restart)
                .children(dialog.map(|_| item().text_color(ink(p().warning)).child("waits for an answer")))
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
                                    .text_color(ink(if b.prompt.has_input { p().accent } else { p().muted }))
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
                                    .text_color(ink(p().danger))
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
                // As claude-loop's line counts them (a: b: e:), in words.
                .child(
                    item()
                        .id("agent-all")
                        .child(format!("all:{}", counts.all.map_or("-".to_string(), |n| n.to_string())))
                        .tip("a: all the project's open tickets"),
                )
                .child(
                    item()
                        .id("agent-backlog")
                        .px_1()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|d| d.bg(p().hover))
                        .when(backlog_open, |d| d.bg(p().active))
                        .when(backlog.is_some_and(|b| b > 0), |d| d.text_color(ink(p().text)))
                        .child(format!("backlog:{}", backlog.map_or("-".to_string(), |n| n.to_string())))
                        .tip("b: its backlog, the tickets for it to look at; a click lists them")
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            shell.toggle_backlog(target.0.clone(), target.1.clone(), cx)
                        })),
                )
                .child(
                    item()
                        .id("agent-events")
                        .when(unseen > 0, |d| d.text_color(ink(p().text)))
                        .child(format!("events:{unseen}"))
                        .tip("e: its events not seen yet — pings, answers, decisions waiting for it"),
                )
                .child(
                    item()
                        .id("agent-holds")
                        .when(holds > 0, |d| d.text_color(ink(p().text)))
                        .child(format!("holds {holds}"))
                        .tip("the tickets it holds"),
                )
                .when(pending || wake.is_some(), |d| {
                    d.child(
                        item()
                            .id("agent-wake")
                            .when(wake.is_some(), |d| d.text_color(ink(p().text)))
                            .child("✉")
                            .children(wake.map(ago))
                            .tip(match wake {
                                Some(_) => "work waits for the loop: it wakes its Claude on it when the countdown ends",
                                None => "work waits for the loop (events or backlog)",
                            }),
                    )
                })
                .child(div().flex_1())
                // Where its loop runs: on aiball's host, or in tmux.
                .child({
                    let hosted = terminal.attach.is_some();
                    item()
                        .id("agent-mode")
                        .px_1()
                        .rounded_sm()
                        .border_1()
                        .border_color(ink(p().border))
                        .text_color(ink(p().muted))
                        .child(if hosted { "host" } else { "tmux" })
                        .tip(if hosted {
                            "its loop runs on aiball's session host: tvty attaches to it directly"
                        } else {
                            "its loop runs in tmux (claude-loop): tvty opens it through tmux"
                        })
                })
                .child(item().text_color(ink(p().text)).child(agent.clone()))
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
                                if !shell.settings.layout.panel_open {
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

    /// Restarts the agent's Claude Code (it installed an update): its loop
    /// waits for it to be idle, restarts it resuming its conversation, and
    /// tells the agent to carry on. Its bar says so meanwhile, then no
    /// longer asks for it.
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
                    Ok(()) => {
                        shell.restarts_asked.insert(agent.clone(), std::time::Instant::now());
                        crate::activity::Activity::done(None, format!("{agent}'s Claude restarts once idle, resuming its conversation"))
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
    pub(super) fn set_afk(&mut self, agent: String, action: &'static str, cx: &mut Context<Self>) {
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

impl Shell {
    /// F9, wherever the focus is: the shown agent's AFK mode moves one
    /// step (aiball's toggle, claude-loop's own cycle and 3 s arming); a
    /// terminal without an agent gets the key, as it would have.
    pub(super) fn afk_cycle(&mut self, cx: &mut Context<Self>) {
        let Some(session) = self.selected.clone() else { return };
        match self.terminal_of(&session).and_then(|(_, t)| t.agent.clone()) {
            Some(agent) => self.set_afk(agent, "toggle", cx),
            None => {
                if let Some(terminal) = self.terminals.get(&session) {
                    terminal.read(cx).send_bytes(b"\x1b[20~");
                }
            }
        }
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
    use super::{AfkMarks, Tone, afk_marks};

    #[test]
    fn f9_arms_a_mode_before_it_is_in_force() {
        // Auto, and nothing armed: ▶ and a grey little man.
        let auto = afk_marks(Some("loop"), Some("off"), None, false);
        assert_eq!(auto, AfkMarks { in_force: Some(("▶", Tone::Green)), typing: false, armed: Some(("웃".into(), Tone::Grey)), arming: false });
        // F9 once: 10 min armed, still ▶ in force — arming.
        let armed = afk_marks(Some("loop"), Some("wait_10m"), Some(599), false);
        assert_eq!((armed.armed, armed.arming), (Some(("웃599s".into(), Tone::Orange)), true));
        // In force 3 s later: ⏸ and the countdown, no longer arming.
        let held = afk_marks(Some("wait"), Some("wait_10m"), Some(596), false);
        assert_eq!((held.in_force, held.arming), (Some(("‖", Tone::Orange)), false));
        // F9 twice more from there: ∞ then off armed while ⏸ holds — arming;
        // david's "held" was this: ⏸ in force, the grey man armed.
        assert!(afk_marks(Some("wait"), Some("off"), None, false).arming);
        assert_eq!(afk_marks(Some("wait"), Some("wait_inf"), None, false).armed, Some(("웃∞".into(), Tone::Red)));
        // Typing holds, marks ⌨, arms nothing.
        let typing = afk_marks(Some("stop"), Some("wait_10m"), Some(600), true);
        assert_eq!((typing.in_force, typing.typing, typing.arming), (Some(("‖", Tone::Orange)), true, false));
        // Booting: the boot alone.
        assert_eq!(afk_marks(Some("boot"), Some("off"), None, false).in_force, Some(("… boot", Tone::Boot)));
    }
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
