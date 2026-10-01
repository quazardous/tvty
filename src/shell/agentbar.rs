//! The agent's bar, under its terminal: what claude-loop's tmux status line
//! says, drawn by tvty from aiball. When the loop pushes its bar (aiball's
//! `/api/consumers/:id/bar`), all of it: who drives the loop — and a click to
//! hold or free it —, its Claude's phase and passing state, the dialogs and
//! alerts, the prompt and a human typing, the proxy, its counters and its
//! next wake. Before a loop pushes one, what the agent's state says. Then its
//! events, the tickets it holds, where it works; a click on
//! its backlog lists it. Agent-centric: the panel beside it is the project's.

use crate::ui::Named as _;
use std::time::Duration;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::*;

use super::Shell;
use super::loopstabs::home_short;
use crate::ui::buttons;
use crate::ui::chipbar::{self, Edge};
use crate::aiball::AgentBar;
use crate::status::{ago, now, parse_time};
use crate::theme::p;
use crate::tip::Tip as _;

/// The bar's height: the terminal gives it that much, once.
pub const BAR_HEIGHT: f32 = 24.;

/// What the AFK chip offers: aiball's action, its label, the bar's glyph
/// for it (▶ runs, ‖ paused, ■ stopped) and the armed mode it sets.
const AFK_ACTIONS: &[(&str, &str, &str, Tone, &str)] = &[
    ("off", "auto", "▶", Tone::Green, "off"),
    ("arm_10m", "hold 10 min", "‖", Tone::Orange, "wait_10m"),
    ("arm_inf", "hold", "■", Tone::Red, "wait_inf"),
];

/// An agent's backlog, open over the bar: whose, in which project (the
/// backlogs' store keeps it, `crate::kernel::backlog`).
pub(super) struct BacklogView {
    agent: String,
    project: String,
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
/// the mode in force (▶ the loop runs on its own, ‖ held a while, ■ held
/// until let go — typing holds
/// too; its ⌨ is further on, by the prompt), and the little man with the mode armed by
/// F9 (grey: away, so auto; the seconds of a ten-minute hold; ∞). F9 moves
/// the little man at once and the mode in force 3 s after the last press:
/// while they differ, it is arming.
#[derive(Debug, PartialEq)]
struct AfkMarks {
    /// A glyph alone: it is drawn in a glyph's box.
    in_force: Option<(&'static str, Tone)>,
    /// A word instead, while the loop boots: text at the bar's size.
    word: Option<(&'static str, Tone)>,
    armed: Option<(String, Tone)>,
    arming: bool,
}

fn afk_marks(presence: Option<&str>, armed: Option<&str>, left: Option<u64>) -> AfkMarks {
    if presence == Some("boot") {
        return AfkMarks { in_force: None, word: Some(("… boot", Tone::Boot)), armed: None, arming: false };
    }
    let in_force = match (presence, armed) {
        (Some("loop"), _) => Some(("▶", Tone::Green)),
        // Held until let go (not AFK, for good): stopped, as the folded list
        // says it — typing does not make it a pause (stop > pause).
        (Some("wait") | Some("stop"), Some("wait_inf")) => Some(("■", Tone::Red)),
        (Some("wait") | Some("stop"), _) => Some(("‖", Tone::Orange)),
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
    AfkMarks { in_force, word: None, armed: man, arming }
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
        let loud = |text: SharedString| {
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
            .named("agent-afk")
            .relative()
            .px_1p5()
            .rounded_sm()
            .cursor_pointer()
            .hover(|d| d.bg(p().hover))
            .when(self.afk_menu, |d| d.bg(p().active))
            .children(marks.in_force.map(|(glyph, t)| crate::icons::loop_glyph(glyph, tone(t), 9.)))
            .children(marks.word.map(|(word, t)| div().text_color(tone(t)).child(word)))
            .children(marks.armed.map(|(man, t)| {
                div()
                    .flex()
                    .text_color(tone(t))
                    .when(marks.arming, |d| d.border_b_1().border_dashed().border_color(tone(t)))
                    .child(man)
                    .when(marks.arming, |d| d.child("…"))
            }))
            // Silent while its choices are open: the tip would cover them.
            .tip_unless(self.afk_menu, if marks.arming {
                "armed: the little man shows the mode chosen with F9, in force 3 s after the last press — ▶ or ‖ says the one in force until then"
            } else {
                "who drives the loop: ▶ on its own, ‖ held for you (or while you type); the little man is the AFK mode — grey: you are away, the loop runs on its own; the seconds of a 10 min hold; ∞ held. F9 cycles it (auto → 10 min → ∞), in force 3 s after the last press; a click chooses"
            })
            .on_click(cx.listener(|shell, _, _, cx| {
                shell.afk_menu = !shell.afk_menu;
                cx.notify();
            }));
        let afk_choices = self.afk_menu.then(|| {
            let mut row = chipbar::line("agent-afk-bar", p().accent);
            let armed = bar.as_ref().map(|b| b.afk.mode.clone());
            for &(action, label, glyph, t, mode) in AFK_ACTIONS {
                let target = agent.clone();
                let label = div().flex().items_center().gap_1().child(crate::icons::loop_glyph(glyph, tone(t), 9.)).child(label);
                row = row.child(
                    buttons::chip(SharedString::from(format!("agent-afk-{action}")), label)
                        // The mode armed now.
                        .when(armed.as_deref() == Some(mode), |d| d.bg(p().active).border_color(tone(t)))
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
            .saying("agent-state", format!("{what}{since}{info}"))
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
            let asked = self.restart_asked.as_deref() == Some(agent.as_str());
            let target = agent.clone();
            // An offer until clicked: its word names the cause, not a state.
            let (word, tip) = match (restarting, armed) {
                (true, _) => ("restarting…", "its Claude restarts, resuming its conversation"),
                (_, true) => ("restart pending", "a restart is asked: its Claude restarts as soon as it is idle, resuming its conversation"),
                _ => ("update", "its Claude Code installed an update: a click asks to restart it (at once if idle, otherwise as soon as it is idle), resuming its conversation"),
            };
            let chip = buttons::chip_if("agent-restart", "⟳", !restarting && !armed)
                .child(word)
                .border_color(p().warning)
                .text_color(p().warning)
                .when(armed || asked, |d| d.bg(p().active))
                .when(!restarting && !armed, |d| {
                    d.on_click(cx.listener({
                        let target = target.clone();
                        move |shell, _, _, cx| {
                            shell.restart_asked = (shell.restart_asked.as_deref() != Some(target.as_str())).then(|| target.clone());
                            cx.notify();
                        }
                    }))
                })
                .tip(tip);
            let busy = b.phase != "idle";
            // Said before done: a restart interrupts nothing, it waits.
            let line = asked.then(|| {
                chipbar::line("agent-restart-bar", p().warning)
                    .child(item().child(if busy { "Restart its Claude once it is idle?" } else { "Restart its Claude now?" }))
                    .child(
                        buttons::answer("agent-restart-go", "Restart")
                            .warning()
                            .tooltip("its conversation is resumed")
                            .on_click(cx.listener(move |shell, _, _, cx| {
                                shell.restart_asked = None;
                                shell.restart_claude(target.clone(), cx)
                            })),
                    )
                    .child(buttons::secondary("agent-restart-cancel", "Cancel").on_click(cx.listener(|shell, _, _, cx| {
                        shell.restart_asked = None;
                        cx.notify();
                    })))
            });
            chipbar::anchored(chip, line, Edge::Right, cx.listener(|shell, _: &MouseDownEvent, _, cx| {
                shell.restart_asked = None;
                cx.notify();
            }))
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

        // One chip, where it runs and whose hands are on it: "tmux ·
        // controls +1". A click opens the bar of what can change — the
        // controls taken or left, the loop moved — each said before done.
        let (place_chip, place_bar) = {
            let hosted = terminal.attach.is_some();
            let session = terminal.session.clone();
            let known = self.board.known.iter().find(|l| l.session() == session).map(|l| l.name.clone());
            let moving = self.moves.contains_key(&agent);
            let asked = self.move_asked.as_deref() == Some(session.as_str());
            let copy = self.copies.contains(&session);
            // Who else is attached: claude-loop's terminal, another tvty.
            let attached = self.attached(&session).filter(|a| a.others > 0);
            // The multiplexer by its name: tmux, or psmux on Windows.
            let mux = crate::mux::program();
            let place = if hosted { "host" } else { mux };
            let hands = if copy { "copy" } else { "controls" };
            let label = match (moving, attached) {
                (true, _) => "moving…".to_string(),
                (false, Some(a)) => format!("{place} · {hands} +{}", a.others),
                (false, None) => format!("{place} · {hands}"),
            };
            let proxy = bar.as_ref().is_some_and(|b| b.proxy_alive);
            let tip = format!(
                "its loop runs {}; {}{}{} A click: take or leave the controls, move the loop.",
                if hosted { "on aiball's session host".to_string() } else { format!("in {mux} (claude-loop)") },
                if copy {
                    "this terminal is a copy: you watch, nothing you type reaches its Claude, and the session keeps its size."
                } else {
                    "you have the controls, shared with any other client: the size follows who types last."
                },
                attached
                    .map(|a| format!(" {} other client{} attached ({} with the controls).", a.others, if a.others == 1 { "" } else { "s" }, a.typing))
                    .unwrap_or_default(),
                if proxy { " The terminal proxy in front of Claude is alive." } else { "" },
            );
            let chip = buttons::chip_if("agent-place", label, !moving)
                .px_1()
                .border_color(ink(if asked || copy { p().warning } else { p().border }))
                .text_color(ink(if copy { p().warning } else { p().muted }))
                .when(!moving, |d| {
                    d.on_click(cx.listener({
                        let session = session.clone();
                        move |shell, _, _, cx| {
                            shell.move_asked = (shell.move_asked.as_deref() != Some(session.as_str())).then(|| session.clone());
                            cx.notify();
                        }
                    }))
                })
                // Silent while its bar is open: the tip would cover it.
                .tip_unless(asked, tip);
            let busy = bar.as_ref().is_some_and(|b| b.phase != "idle");
            let other = if hosted { format!("into {mux}") } else { "to aiball's host".to_string() };
            // The others can be closed from here, this client having the
            // controls: on a host that says it can, or a loop in tmux (aiball
            // detaches them there, this terminal's client kept).
            let closable = attached.filter(|_| !copy).and_then(|a| {
                let view = self.terminals.get(&session)?.read(cx);
                let how = match view.attachment() {
                    Some(attach) => attach.can_detach_others().then(|| Others::Host(attach.clone()))?,
                    None => Others::Tmux(known.clone()?, view.child_pid?),
                };
                Some((a, how))
            });
            let place_bar = asked.then(|| {
                let hands_session = session.clone();
                let mut row = chipbar::line("agent-place-bar", p().warning)
                    .when_some(attached.filter(|a| a.typing > 0 && !hosted), |d, _| {
                        d.child(item().text_color(p().warning).child("claude-loop's terminal types into it too"))
                    })
                    .child(
                        buttons::answer("agent-hands", if copy { "Take the controls" } else { "Leave for a copy" })
                            .on_click(cx.listener(move |shell, _, window, cx| {
                                shell.move_asked = None;
                                shell.set_copy(hands_session.clone(), !copy, window, cx);
                            })),
                    );
                if let Some((a, how)) = closable {
                    let who = if a.others == 1 { "the other client".to_string() } else { format!("the {} other clients", a.others) };
                    row = row.child(
                        buttons::answer("agent-close-others", format!("Close the others ({})", a.others))
                            .tooltip(format!(
                                "{who} attached to this session leave it (claude-loop's terminal, another Terminal Velocity); its Claude and this terminal go on"
                            ))
                            .on_click(cx.listener(move |shell, _, _, cx| {
                                shell.move_asked = None;
                                shell.close_others(how.clone(), cx);
                            })),
                    );
                }
                // Moved from here only a loop of this machine.
                if let Some(name) = known.clone() {
                    let (agent, to_host) = (agent.clone(), !hosted);
                    row = row
                        .child(
                            buttons::answer("agent-move-go", if hosted { format!("Move into {mux}") } else { "Move to host".to_string() })
                                .warning()
                                .tooltip(format!(
                                    "its Claude restarts {other}, resuming its conversation{}",
                                    if busy { " — it works now: the move interrupts it" } else { "" }
                                ))
                                .on_click(cx.listener(move |shell, _, _, cx| shell.move_loop(agent.clone(), name.clone(), to_host, cx))),
                        );
                }
                row.child(
                    buttons::secondary("agent-move-cancel", "Cancel").on_click(cx.listener(|shell, _, _, cx| {
                        shell.move_asked = None;
                        cx.notify();
                    })),
                )
            });
            (chip, place_bar)
        };

        // Remote Control: whether its Claude can be taken up from claude.ai,
        // as its loop reads it on the screen (the folder's setting or `/rc`
        // typed). Said, not set: the folder's setting is the project's
        // options'. None from a loop too old to say it.
        let rc_chip = bar.as_ref().and_then(|b| b.remote_control.as_ref()).map(|rc| {
            div()
                .named("agent-rc")
                .flex_none()
                .px_1()
                .rounded_sm()
                .border_1()
                .border_color(ink(if rc.on { p().accent } else { p().border }))
                .text_color(ink(if rc.on { p().accent } else { p().muted }))
                .child("RC")
                .tip(if rc.on {
                    "Remote Control is on: this Claude can be taken up from claude.ai and the mobile app"
                } else {
                    "Remote Control is off. /rc in the session turns it on; a folder's loops get it from the project's settings"
                })
        });
        // The model its Claude ran its last turn on; a newer one of its
        // family out: yellow, ↑.
        let model_item = bar.as_ref().and_then(|b| b.model.as_ref()).map(|model| {
            let newer = model.newer.is_some();
            item()
                .saying("agent-model", model.name.clone())
                .text_color(ink(if newer { p().warning } else { p().muted }))
                .child(if newer { format!("{} ↑", model.name) } else { model.name.clone() })
                .tip(model.said())
        });
        Some(
            div()
                .named("agent-bar")
                .relative()
                .flex()
                .flex_none()
                // The column's width, never more: what is long gives way.
                .w_full()
                .min_w_0()
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
                .child(chipbar::anchored(
                    crate::tips::target("agent.afk", afk).flex_none(),
                    afk_choices,
                    Edge::Left,
                    cx.listener(|shell, _: &MouseDownEvent, _, cx| {
                        shell.afk_menu = false;
                        cx.notify();
                    }),
                ))
                .child(sep())
                .child(if online {
                    state.into_any_element()
                } else {
                    item().text_color(ink(p().danger)).child("offline").into_any_element()
                })
                .children(dialog.map(|_| item().text_color(ink(p().warning)).child("waits for an answer")))
                .when_some(bar.as_ref(), |d, b| {
                    let a = &b.alerts;
                    d.when_some(b.limit_said(), |d, limit| d.child(loud(limit.into())))
                        .when(a.trust_dialog, |d| d.child(loud("trust this folder?".into())))
                        .when(a.not_logged_in, |d| d.child(loud("not logged in".into())))
                        .when(a.api_unreachable, |d| d.child(loud("API unreachable".into())))
                        .when(a.link_down, |d| d.child(loud("loop link down".into())))
                        .when(a.daemon_down, |d| d.child(loud("aiball unreachable".into())))
                        .when(b.prompt.visible, |d| {
                            d.child(
                                item()
                                    .named("agent-prompt")
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
                                    .named("agent-typing")
                                    .text_color(ink(p().danger))
                                    .child("⌨")
                                    .tip("a human typed in its terminal a moment ago: the loop holds off"),
                            )
                        })
                        .when(b.zen, |d| d.child(item().named("agent-zen").child("zen").tip("zen mode: the loop keeps quiet")))
                })
                .child(sep())
                // As claude-loop's line counts them (a: b: e:), in words.
                .child(
                    item()
                        .saying("agent-all", format!("all:{}", counts.all.map_or("-".to_string(), |n| n.to_string())))
                        .child(format!("all:{}", counts.all.map_or("-".to_string(), |n| n.to_string())))
                        .tip("a: all the project's open tickets"),
                )
                .child(
                    item()
                        .named("agent-backlog")
                        .px_1()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|d| d.bg(p().hover))
                        .when(backlog_open, |d| d.bg(p().active))
                        .when(backlog.is_some_and(|b| b > 0), |d| d.text_color(ink(p().text)))
                        .child(format!("backlog:{}", backlog.map_or("-".to_string(), |n| n.to_string())))
                        .tip_unless(backlog_open, "b: its backlog, the tickets for it to look at; a click lists them")
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            shell.toggle_backlog(target.0.clone(), target.1.clone(), cx)
                        })),
                )
                .child(
                    item()
                        .named("agent-events")
                        .when(unseen > 0, |d| d.text_color(ink(p().text)))
                        .child(format!("events:{unseen}"))
                        .tip("e: its events not seen yet — pings, answers, decisions waiting for it"),
                )
                .child(
                    item()
                        .named("agent-holds")
                        .when(holds > 0, |d| d.text_color(ink(p().text)))
                        .child(format!("holds:{holds}"))
                        .tip("the tickets it holds"),
                )
                .when(pending || wake.is_some(), |d| {
                    d.child(
                        item()
                            .named("agent-wake")
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
                .children(model_item)
                // Claude Code updated: restarting it is a gesture on the loop,
                // beside the loop's place and hands.
                .children(restart)
                // Where its loop runs, and whose hands are on it.
                .child(chipbar::anchored(
                    crate::tips::target("agent.place", place_chip).flex_none(),
                    place_bar,
                    Edge::Right,
                    cx.listener(|shell, _: &MouseDownEvent, _, cx| {
                        shell.move_asked = None;
                        cx.notify();
                    }),
                ))
                .children(rc_chip.map(|chip| crate::tips::target("agent.rc", chip).flex_none()))
                // Its folder, then its name, give way when the bar is short (the
                // tab says its name too): the controls stay.
                .child(item().flex_shrink(1.).min_w_0().overflow_hidden().text_color(ink(p().text)).child(div().min_w_0().truncate().child(agent.clone())))
                .children(cwd.map(|cwd| item().flex_shrink(1000.).min_w_0().overflow_hidden().child(div().min_w_0().truncate().child(cwd))))
                .children(self.backlog_view.as_ref().filter(|v| v.agent == agent).map(|v| self.backlog_list(v, cx)))
                .into_any_element(),
        )
    }

    /// The agent's own backlog, over its bar: its tickets by tier, a click
    /// opens one in the panel.
    fn backlog_list(&self, view: &BacklogView, cx: &mut Context<Self>) -> AnyElement {
        let mut list = div()
            .named("agent-backlog-list")
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
        let read = crate::kernel::backlog::get(cx, &view.agent, &view.project);
        match &read {
            None => list = list.child(div().text_color(p().muted).child("reading…")),
            Some(Err(error)) => list = list.child(div().text_color(p().danger).child(format!("backlog: {}", short_error(error)))),
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
                            .named(SharedString::from(format!("backlog-{ticket}")))
                            .flex()
                            .gap_2()
                            .px_1()
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|d| d.bg(p().hover))
                            .child(div().flex_none().text_color(p().muted).child(format!("#{ticket}")))
                            .child(div().flex_1().min_w_0().truncate().child(row.title.clone()))
                            .on_click(cx.listener(move |shell, _, window, cx| {
                                shell.backlog_view = None;
                                shell.show_ticket(project.clone(), ticket, window, cx);
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
        crate::kernel::backlog::request(cx, &agent, &project);
        self.backlog_view = Some(BacklogView { agent, project });
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

    /// Turns `agent`'s Claude's Remote Control on or off, off the UI thread:
    /// its folder keeps the choice, its loop restarts with it and opens again.
    /// Moves `agent`'s loop (`name`) to aiball's host or into tmux, through
    /// claude-loop, off the UI thread. Its session ends and comes back as
    /// another one, which opens.
    fn move_loop(&mut self, agent: String, name: String, to_host: bool, cx: &mut Context<Self>) {
        self.move_asked = None;
        let next = if to_host { format!("{}{agent}", crate::sessions::HOSTED_PREFIX) } else { name.clone() };
        self.moves.insert(agent.clone(), (next.clone(), std::time::Instant::now()));
        cx.notify();
        cx.spawn(async move |this, cx| {
            let loop_name = name.clone();
            let Ok(aiball) = this.read_with(cx, |shell, _| shell.aiball.clone()) else { return };
            let done = cx.background_executor().spawn(async move { crate::loops::move_to(&aiball, &loop_name, to_host) }).await;
            let _ = this.update(cx, |shell, cx| {
                let place = if to_host { "to aiball's host".to_string() } else { format!("into {}", crate::mux::program()) };
                let activity = match done {
                    Ok(()) => {
                        shell.open_when_running = Some(next.clone());
                        let _ = shell.refresh_now.unbounded_send(());
                        crate::activity::Activity::done(None, format!("{agent} moved {place}, its conversation resumed"))
                    }
                    Err(error) => {
                        shell.moves.remove(&agent);
                        crate::activity::Activity::failed(None, &format!("move of {agent} {place}"), short_error(&format!("{error:#}")))
                    }
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

/// How the session's other clients are closed: by the host this terminal
/// is attached to, or by aiball for a loop in tmux (its name, and the pid
/// of this terminal's tmux client, kept).
#[derive(Clone)]
pub(super) enum Others {
    Host(std::sync::Arc<crate::attach::Attach>),
    Tmux(String, u32),
}

impl Shell {
    /// The session's other clients are closed, and it is said: on the
    /// host, how many once it answers (one that does not, within a few
    /// seconds, is said to have been asked).
    pub(super) fn close_others(&mut self, how: Others, cx: &mut Context<Self>) {
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let said = match how {
                Others::Host(attach) => {
                    attach.detach_others();
                    let mut count = None;
                    for _ in 0..30 {
                        cx.background_executor().timer(std::time::Duration::from_millis(100)).await;
                        count = attach.take_others_closed();
                        if count.is_some() {
                            break;
                        }
                    }
                    Ok(match count {
                        Some(1) => "closed the other client of this session".to_string(),
                        Some(n) => format!("closed the {n} other clients of this session"),
                        None => "asked the session's host to close its other clients".to_string(),
                    })
                }
                Others::Tmux(name, keep) => cx
                    .background_executor()
                    .spawn(async move { crate::loops::detach_others(&aiball, &name, keep) })
                    .await
                    // More than this one left: psmux does not say its
                    // clients' pids to aiball yet, which closed none.
                    .and_then(|left| match left {
                        0 | 1 => Ok("closed the other clients of this session".to_string()),
                        n => Err(anyhow::anyhow!(
                            "{} other client{} still attached: {} cannot tell them from this one yet",
                            n - 1,
                            if n == 2 { "" } else { "s" },
                            crate::mux::program()
                        )),
                    }),
            };
            let _ = this.update(cx, |shell, cx| {
                crate::activity::publish(
                    cx,
                    match said {
                        Ok(what) => crate::activity::Activity::done(None, what),
                        Err(error) => crate::activity::Activity::failed(None, "close the other clients", short_error(&format!("{error:#}"))),
                    },
                );
                let _ = shell.refresh_now.unbounded_send(());
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

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
        let auto = afk_marks(Some("loop"), Some("off"), None);
        assert_eq!(auto, AfkMarks { in_force: Some(("▶", Tone::Green)), word: None, armed: Some(("웃".into(), Tone::Grey)), arming: false });
        // F9 once: 10 min armed, still ▶ in force — arming.
        let armed = afk_marks(Some("loop"), Some("wait_10m"), Some(599));
        assert_eq!((armed.armed, armed.arming), (Some(("웃599s".into(), Tone::Orange)), true));
        // In force 3 s later: ⏸ and the countdown, no longer arming.
        let held = afk_marks(Some("wait"), Some("wait_10m"), Some(596));
        assert_eq!((held.in_force, held.arming), (Some(("‖", Tone::Orange)), false));
        // F9 twice more from there: ∞ then off armed while ⏸ holds — arming;
        // david's "held" was this: ⏸ in force, the grey man armed.
        assert!(afk_marks(Some("wait"), Some("off"), None).arming);
        assert_eq!(afk_marks(Some("wait"), Some("wait_inf"), None).armed, Some(("웃∞".into(), Tone::Red)));
        // Held until let go: stopped, not paused.
        assert_eq!(afk_marks(Some("wait"), Some("wait_inf"), None).in_force, Some(("■", Tone::Red)));
        // Typing in a loop held until let go: still stopped, not paused.
        assert_eq!(afk_marks(Some("stop"), Some("wait_inf"), None).in_force, Some(("■", Tone::Red)));
        // Typing holds and arms nothing (its ⌨ is by the prompt, once).
        let typing = afk_marks(Some("stop"), Some("wait_10m"), Some(600));
        assert_eq!((typing.in_force, typing.arming), (Some(("‖", Tone::Orange)), false));
        // Booting: the boot alone, as a word — never in a glyph's box.
        let boot = afk_marks(Some("boot"), Some("off"), None);
        assert_eq!((boot.in_force, boot.word), (None, Some(("… boot", Tone::Boot))));
        // Whatever the state, the glyph's box gets a glyph alone.
        for presence in [None, Some("boot"), Some("loop"), Some("wait"), Some("stop")] {
            for armed in [None, Some("off"), Some("wait_10m"), Some("wait_inf")] {
                if let Some((glyph, _)) = afk_marks(presence, armed, Some(30)).in_force {
                    assert_eq!(glyph.chars().count(), 1, "{glyph:?} is not a glyph alone");
                }
            }
        }
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
