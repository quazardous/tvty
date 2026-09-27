//! The window: projects and their terminals on the left, the selected
//! terminal in the middle, the ticket panel on the right (resizable,
//! collapsible), and two ways to switch — the slider (ctrl+tab, most recent
//! first) and the gallery (ctrl+shift+space, every terminal as a thumbnail).
//! See docs/UX.md.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use gpui_kit::component::{TitleBar, window_paddings};

use crate::aiball::{Aiball, TicketRow};
use crate::keymap;
use crate::tip::Tip as _;
use crate::theme::{self, p};
use gpui_kit::component::scroll::ScrollableElement as _;
use crate::options::{FIXED_KEYS, Section};
use crate::activity::{self, Activity};
use crate::bus::{self, Signal};
use crate::fulllist::{CloseFullList, FullList};
use crate::newticket::{CloseNewTicket, Created, NewTicketForm};
use crate::notify::{self, Kind, Notice};

mod agentbar;
mod ended;
mod frame;
mod loopstabs;
mod stacks;
mod tabs;
use tabs::Restoring;
mod viewer;
use crate::panel::{CollapsePanel, FullChanged, OpenFullList, OrderChanged, Scope, TicketPanel, dot, pill};
use crate::sessions::{self, Board, Terminal};
use crate::settings::{Preferences, SCHEMA, Settings};
use gpui_kit::component::switch::Switch;
use tvty_config::schema::shown;
use tvty_config::{Kind as SettingKind, Setting, Value};

/// The terminals' font size, which shortcuts step too.
const TERMINAL_FONT: &str = "appearance.terminal_font_size";

/// The shortcuts' page listening for a key for a command: the key it
/// replaces (none: one more), the key pressed when it runs another command
/// (to replace, or not), or why the key was refused.
struct Capture {
    command: &'static str,
    replacing: Option<keymap::Key>,
    pending: Option<(keymap::Key, &'static str)>,
    refused: Option<String>,
}
use crate::terminal::TerminalView;

/// How often the tmux sessions are listed (cheap: one `tmux ls`).
const SESSIONS_EVERY: Duration = Duration::from_secs(3);
/// How long the board waits after what the bus pushed before it is built
/// again: a burst of events is built once.
const REBUILD_AFTER: Duration = Duration::from_millis(100);
/// How often the cards redraw while the slider or the gallery is up: they
/// are live, this only paces them.
const CARDS_EVERY: Duration = Duration::from_millis(100);
/// A card's viewport: the bottom of the screen, where Claude Code writes.
const CARD_LINES: usize = 30;
const CARD_COLUMNS: usize = 100;
/// The slider's cards, and its chosen one, enlarged: a card's screen size.
const SLIDER_CARD: (f32, f32) = (230., 138.);
const SLIDER_CHOSEN: (f32, f32) = (340., 205.);

const TITLE_BAR_HEIGHT: f32 = 41.;
/// The projects' list, its tabs' rail (44 px) included.
const SIDEBAR_WIDTH: f32 = 290.;
const SIDEBAR_MIN: f32 = 230.;
const PANEL_MIN: f32 = 260.;
const CENTER_MIN: f32 = 320.;
/// The sessions list's rows: a project's heading, a session with its
/// state's line, a bare tmux session.
const SESSION_HEADING: f32 = 30.;
const SESSION_ROW: f32 = 51.;
const SESSION_ROW_BARE: f32 = 32.;
/// A folded side: just enough for a few dots saying what waits, beside
/// the window's resize band (6 px) when the side is at the window's edge.
const FOLDED_WIDTH: f32 = 14.;
/// Between the notices and the terminal's edges.
const NOTICE_MARGIN: f32 = 12.;
/// The strip between a side and the terminal, holding the toggle grip.
const EDGE_WIDTH: f32 = 8.;

pub struct Shell {
    aiball: Aiball,
    board: Board,
    /// Terminals opened so far, by tmux session: they stay alive when hidden.
    terminals: HashMap<String, Entity<TerminalView>>,
    selected: Option<String>,
    /// Sessions by last use, most recent first: the slider's order.
    recent: Vec<String>,
    panel: Entity<TicketPanel>,
    settings: Settings,
    /// The preferences in force, to tell what a change changes.
    applied: Preferences,
    /// The shortcuts' page listening for a key.
    capture: Option<Capture>,
    /// A side's edge is being dragged.
    resizing: Option<Side>,
    /// The edge has moved since it was pressed.
    dragged: bool,
    /// The options page is open, on this section.
    options: Option<Section>,
    /// The ticket list, full screen, and whether it is shown: hidden, it
    /// keeps its scope and filters for the next time.
    full_list: Option<Entity<FullList>>,
    full_list_shown: bool,
    /// The full detail was opened from the full list: leaving it returns
    /// there.
    full_list_return: bool,
    /// The panel as it was before going full screen, to find it so when
    /// coming back: `Some(None)` its list, `Some(Some(..))` a ticket.
    compact: Option<Option<(Option<String>, u64)>>,
    /// The agent bar's AFK choices are open; its agent's backlog is open.
    afk_menu: bool,
    /// The agent whose Claude is being restarted.
    restarting: Option<String>,
    /// Agents whose Claude restarts as soon as it is idle (asked while it
    /// worked).
    restart_armed: HashSet<String>,
    backlog_view: Option<agentbar::BacklogView>,
    /// The projects' list's tab, the "+ session" form open, the directory
    /// a loop is starting in, and the loop to open once it runs.
    /// The sessions list's sections' scrolls: live, idle, shut.
    session_scrolls: [ScrollHandle; 3],
    new_session: Option<loopstabs::NewSession>,
    starting: Option<String>,
    open_when_running: Option<String>,
    /// The selected session ended: the end screen stands in its place.
    ended: Option<ended::EndedSession>,
    /// A thread's image, over the whole window.
    viewer: Option<viewer::Viewer>,
    /// aiball's bus, once the user is known.
    wire: Option<crate::wire::Wire>,
    /// What aiball's bus says the connection is (`bus.whoami`).
    wire_whoami: Option<String>,
    /// The new ticket's form, and whether it is shown: hidden, it keeps
    /// its draft.
    new_ticket: Option<Entity<NewTicketForm>>,
    new_ticket_shown: bool,
    /// The board as aiball pushes it on the bus.
    live: crate::live::Live,
    /// The tmux sessions and the loops of this machine, as last listed.
    local: (Vec<(String, String)>, Vec<crate::loops::KnownLoop>),
    /// The board is to be built again shortly.
    rebuild_pending: bool,
    /// Shells closed from their tab, until the host says they are gone:
    /// the list lets them go at once.
    stopping: HashSet<String>,
    /// The workspace to open again at start, once the board is there.
    restoring: Option<Restoring>,
    /// The accordions' heights moved by a drag, counted: kept once it rests.
    heights_moved: u64,
    /// The tmux sessions and loops of this machine were listed once.
    local_seen: bool,
    /// The slider is up, on this session.
    slider: Option<String>,
    /// Counts the slider's openings: each one replays its entrance.
    slider_shown: usize,
    gallery: Option<Gallery>,
    /// Screens of every terminal, for the gallery and the slider; kept
    /// between openings so they open already filled.
    /// Read-only clients (`tmux attach -r`, which never resizes) of the
    /// sessions not open here, while the slider or the gallery shows them.
    watchers: HashMap<String, Entity<TerminalView>>,
    /// A capture loop is running.
    watching: bool,
    /// Where each card of the slider or the gallery was last drawn, by
    /// session: what up and down go by.
    card_centres: Rc<RefCell<HashMap<String, Point<Pixels>>>>,
    /// What was already waiting at the last read, so that only what is new
    /// raises a notification. `None` before the first read.
    waiting: Option<HashSet<(u64, Wait)>>,
    /// Counts terminal switches: each one replays the slide.
    switches: usize,
    /// The theme list is open, under the title bar.
    theme_menu: bool,

    focus: FocusHandle,
    /// Lists the local sessions again now, before the next tick.
    refresh_now: futures::channel::mpsc::UnboundedSender<()>,
}

/// An arrow over the cards: along them, or to the group above or below.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Move {
    Back,
    Forth,
    Up,
    Down,
}

impl Move {
    fn of(key: &str) -> Option<Move> {
        match key {
            "left" => Some(Move::Back),
            "right" => Some(Move::Forth),
            "up" => Some(Move::Up),
            "down" => Some(Move::Down),
            _ => None,
        }
    }
}

/// Where an arrow leads from `from` over cards laid out in `groups`: left and
/// right follow the cards, wrapping around; up and down go to the card seen
/// nearest above or below, by `centres` (where each was drawn) — or, past the
/// edge or unmeasured, to the group before or after, at the same place in it
/// (or its last card). From nowhere, the first card.
fn moved(
    groups: &[Vec<String>],
    from: Option<&str>,
    step: Move,
    centres: &HashMap<String, Point<Pixels>>,
) -> Option<String> {
    let at = from.and_then(|from| {
        groups
            .iter()
            .enumerate()
            .find_map(|(g, sessions)| sessions.iter().position(|s| s == from).map(|i| (g, i)))
    });
    let Some((g, i)) = at else {
        return groups.first()?.first().cloned();
    };
    match step {
        Move::Back | Move::Forth => {
            let flat: Vec<&String> = groups.iter().flatten().collect();
            let here = groups[..g].iter().map(Vec::len).sum::<usize>() + i;
            let step = if step == Move::Back { -1 } else { 1 };
            let next = (here as isize + step).rem_euclid(flat.len() as isize) as usize;
            Some(flat[next].clone())
        }
        Move::Up | Move::Down => {
            let here = centres.get(groups[g][i].as_str());
            let nearest = here.and_then(|here| {
                let (hx, hy) = (f32::from(here.x), f32::from(here.y));
                groups
                    .iter()
                    .flatten()
                    .filter_map(|s| centres.get(s).map(|c| (s, f32::from(c.x) - hx, f32::from(c.y) - hy)))
                    // A row away at least: cards of one row differ by a few pixels.
                    .filter(|(_, _, dy)| if step == Move::Up { *dy < -20. } else { *dy > 20. })
                    .min_by(|a, b| {
                        let score = |(_, dx, dy): &(&String, f32, f32)| dy.abs() + dx.abs() / 2.;
                        score(a).total_cmp(&score(b))
                    })
                    .map(|(s, _, _)| s.clone())
            });
            if nearest.is_some() {
                return nearest;
            }
            let step = if step == Move::Up { -1 } else { 1 };
            let group = &groups[(g as isize + step).rem_euclid(groups.len() as isize) as usize];
            group.get(i).or_else(|| group.last()).cloned()
        }
    }
}

struct Gallery {
    filter: String,
    /// The card enter opens; when the filter hides it, the first one shown.
    chosen: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Wait {
    Decision,
    Unread,
}

/// Something new an agent waits on from the user.
#[derive(Clone)]
struct Need {
    project: String,
    session: String,
    agent: String,
    ticket: u64,
    title: String,
    wait: Wait,
}

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Left,
    Right,
}

fn critical() -> Hsla {
    p().danger
}
fn decision() -> Hsla {
    p().warning
}
fn unread() -> Hsla {
    p().accent
}

/// What a set of tickets asks of the user.
#[derive(Default)]
pub(crate) struct Alerts {
    decisions: usize,
    unread: usize,
    pub(crate) critical: bool,
}

impl Alerts {
    pub(crate) fn of<'a>(tickets: impl Iterator<Item = &'a TicketRow>, critical: Option<u64>) -> Self {
        let mut alerts = Self::default();
        for ticket in tickets {
            alerts.decisions += ticket.pending_decision() as usize;
            alerts.unread += ticket.unread as usize;
            alerts.critical |= critical == Some(ticket.id);
        }
        alerts
    }

    /// One dot per kind of thing waiting, most pressing first.
    fn colors(&self) -> Vec<Hsla> {
        let mut colors = Vec::new();
        if self.critical {
            colors.push(critical());
        }
        if self.decisions > 0 {
            colors.push(decision());
        }
        if self.unread > 0 {
            colors.push(unread());
        }
        colors
    }

    /// The most pressing thing waiting, if any.
    fn color(&self) -> Option<Hsla> {
        self.colors().first().copied()
    }

    /// The badges, each saying what it counts under the pointer; `key` sets
    /// them apart from the other badges of the window.
    pub(crate) fn badges<K: Into<SharedString>>(&self, key: K) -> impl IntoElement + use<K> {
        let key: SharedString = key.into();
        let badge = |what: &str, text: String, colour: Hsla, tip: String| {
            div().id(SharedString::from(format!("{key}-{what}"))).child(pill(text, colour)).tip(tip)
        };
        let plural = |n: usize, one: &str, many: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {many}") };
        div()
            .flex()
            .items_center()
            .gap_1()
            .when(self.critical, |d| {
                d.child(badge("critical", "!".into(), critical(), "the critical ticket is here: it holds the most open tickets".into()))
            })
            .when(self.decisions > 0, |d| {
                let tip = format!("{} waiting for your decision", plural(self.decisions, "ticket", "tickets"));
                d.child(badge("decisions", self.decisions.to_string(), decision(), tip))
            })
            .when(self.unread > 0, |d| {
                let tip = format!("{} with something new for you", plural(self.unread, "ticket", "tickets"));
                d.child(badge("unread", self.unread.to_string(), unread(), tip))
            })
    }
}

impl Shell {
    pub fn new(selected: Option<String>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let aiball = Aiball::from_env();
        // aiball's bus first: the first read of the board goes through it.
        let (notices, wire_notices) = futures::channel::mpsc::unbounded::<crate::wire::Notification>();
        let wire = crate::aiball::start_wire(&aiball.user, notices);
        let panel = cx.new(|cx| TicketPanel::new(aiball.clone(), window, cx));
        // What the views say to whom it may concern.
        cx.subscribe_in(&bus::bus(cx), window, |shell, _, signal: &Signal, window, cx| match signal {
            // A gesture moved the board: aiball pushes what changed.
            Signal::BoardChanged => {}
            Signal::Notices => cx.notify(),
            Signal::OpenNotice(notice) => shell.open_notice(notice.clone(), window, cx),
            Signal::OpenTicket { project, ticket } => shell.open_full_ticket(project.clone(), *ticket, window, cx),
            Signal::AskNewTicket { project, parent } => shell.open_new_ticket(project.clone(), *parent, window, cx),
            Signal::OpenPictures { pictures, index } => {
                shell.viewer = Some(viewer::Viewer::new(pictures.clone(), *index));
                window.focus(&shell.focus.clone(), cx);
                cx.notify();
            }
            // The activity service's.
            Signal::Activity(_) => {}
        })
        .detach();
        cx.subscribe(&panel, |shell, _, _: &CollapsePanel, cx| shell.toggle_panel(cx))
            .detach();
        cx.subscribe_in(&panel, window, |shell, panel, _: &FullChanged, window, cx| {
            // Leaving the full detail opened from the full list goes back
            // to the list.
            if panel.read(cx).is_full() {
                // Full screen from the panel itself (its ⤢ more): what
                // to come back to is what it shows.
                shell.remember_compact(cx);
            } else {
                if shell.full_list_return {
                    shell.full_list_return = false;
                    shell.full_list_shown = true;
                } else {
                    shell.restore_compact(cx);
                }
                cx.defer_in(window, |shell, window, cx| window.focus(&shell.focus.clone(), cx));
            }
            cx.notify();
        })
        .detach();
        cx.subscribe(&panel, |shell, _, order: &OrderChanged, cx| {
            shell.set_pref("tickets.newest_first", Value::Toggle(order.0), cx);
        })
        .detach();
        cx.subscribe_in(&panel, window, |shell, _, _: &OpenFullList, window, cx| {
            shell.go_full(window, cx)
        })
        .detach();

        Self::tick_clock(cx);
        let (refresh_now, wake) = futures::channel::mpsc::unbounded::<()>();
        Self::local_loop(wake, cx);

        let mut shell = Self {
            aiball,
            board: Board::default(),
            terminals: HashMap::new(),
            selected: None,
            recent: Vec::new(),
            panel,
            settings: Settings::current(cx),
            applied: crate::config::get::<Preferences>(cx).clone(),
            capture: None,
            resizing: None,
            dragged: false,
            options: None,
            full_list: None,
            full_list_shown: false,
            full_list_return: false,
            compact: None,
            afk_menu: false,
            restarting: None,
            restart_armed: HashSet::new(),
            backlog_view: None,
            session_scrolls: Default::default(),
            new_session: None,
            starting: None,
            open_when_running: None,
            ended: None,
            viewer: None,
            wire: None,
            wire_whoami: None,
            new_ticket: None,
            new_ticket_shown: false,
            live: Default::default(),
            local: Default::default(),
            rebuild_pending: false,
            stopping: HashSet::new(),
            restoring: None,
            heights_moved: 0,
            local_seen: false,
            slider: None,
            slider_shown: 0,
            gallery: None,
            watchers: HashMap::new(),
            watching: false,
            card_centres: Rc::default(),
            waiting: None,
            switches: 0,
            theme_menu: false,

            focus: cx.focus_handle(),
            refresh_now,
        };
        shell.wire = Some(wire);
        shell.follow_wire(wire_notices, cx);
        // aiball not running: start it (detached), or say it is missing.
        cx.spawn(async move |_, cx| {
            let found = cx.background_executor().spawn(async { crate::daemon::ensure() }).await;
            let said = match found {
                crate::daemon::Start::Running => return,
                crate::daemon::Start::Started => Activity::news("tvty", Kind::Info, None, "aiball was not running: started it"),
                crate::daemon::Start::Missing(why) => Activity::news("tvty", Kind::Error, None, why),
            };
            let _ = cx.update(|cx| activity::publish(cx, said));
        })
        .detach();
        // The accordions' heights set by hand: kept once a drag ends or a
        // double click resets them.
        crate::accordion::set_heights(shell.settings.layout.section_heights.clone());
        cx.observe_global::<crate::accordion::HeightsChanged>(|shell, cx| {
            shell.settings.layout.section_heights = crate::accordion::heights();
            shell.settings.save(cx);
        })
        .detach();
        // settings.toml edited by hand: what changed is put in force.
        cx.observe_global_in::<crate::config::Store<crate::settings::Preferences>>(window, |shell, window, cx| {
            shell.preferences_changed(window, cx)
        })
        .detach();
        let newest_first = shell.applied.tickets.newest_first;
        shell.panel.update(cx, |panel, cx| panel.set_newest_first(newest_first, cx));
        match selected {
            Some(session) => shell.select(session, window, cx),
            None => {
                // The workspace as it was left, once the board is known.
                let open = shell.settings.workspace.open_terminals.clone();
                if !open.is_empty() {
                    shell.restoring = Some(tabs::Restoring::new(open, shell.settings.workspace.shown_terminal.clone()));
                }
                window.focus(&shell.focus.clone(), cx)
            }
        }
        shell
    }

    /// Lists the tmux sessions and the loops of this machine every few
    /// seconds, or at once when asked, and builds the board again when they
    /// changed. What aiball says comes on its own, over the bus.
    fn local_loop(mut wake: futures::channel::mpsc::UnboundedReceiver<()>, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            use futures::{FutureExt as _, StreamExt as _};
            loop {
                let local = cx
                    .background_executor()
                    .spawn(async { (sessions::tmux_sessions(), crate::loops::known()) })
                    .await;
                let alive = this.update(cx, |shell, cx| {
                    if !shell.local_seen {
                        shell.local_seen = true;
                        cx.notify();
                    }
                    if shell.local != local {
                        shell.local = local;
                        shell.rebuild(cx);
                    }
                });
                if alive.is_err() {
                    break;
                }
                let timer = cx.background_executor().timer(SESSIONS_EVERY).fuse();
                futures::pin_mut!(timer);
                futures::select! {
                    _ = timer => {}
                    nudge = wake.next() => if nudge.is_none() { break },
                };
                while wake.try_recv().is_ok() {}
            }
        })
        .detach();
    }

    /// A new ticket on a project of the board: blue from an agent, orange
    /// when it waits for moderation, green when the user filed it (from the
    /// web UI, say; filed from tvty, the same notice replaces tvty's own).
    fn announce_filed(&mut self, filed: crate::live::Filed, cx: &mut Context<Self>) {
        if !self.board.projects.iter().any(|p| p.on_board && p.name == filed.project) {
            return;
        }
        let about = Some((filed.project.clone(), filed.ticket));
        let activity = if filed.by == self.aiball.user {
            Activity::done(about, format!("filed — {}", filed.title))
        } else if filed.pending {
            Activity::news(filed.by, Kind::Decision, about, format!("{} — a new ticket to moderate", filed.title))
        } else {
            Activity::news(filed.by, Kind::News, about, format!("{} — a new ticket", filed.title))
        };
        activity::publish(cx, activity);
    }

    /// Builds the board again from what aiball pushed and the local sessions.
    fn rebuild(&mut self, cx: &mut Context<Self>) {
        // Who the user is, from the humans aiball knows; the bus runs as them.
        self.aiball.find_user(&self.live.consumers());
        let mut board = sessions::build(&self.live, self.local.0.clone(), self.local.1.clone());
        self.forget_stopping(&mut board);
        if self.board != board {
            self.board = board;
            self.fire_armed_restarts(cx);
            self.update_needs(cx);
            cx.notify();
        }
        self.sync_panel(cx);
        self.sync_full_list(cx);
    }

    /// The board moved on the bus: built again shortly, once for a burst.
    fn board_moved(&mut self, cx: &mut Context<Self>) {
        if self.rebuild_pending {
            return;
        }
        self.rebuild_pending = true;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(REBUILD_AFTER).await;
            let _ = this.update(cx, |shell, cx| {
                shell.rebuild_pending = false;
                shell.rebuild(cx);
            });
        })
        .detach();
    }

    /// What the bus's subscriptions said.
    fn take_updates(&mut self, updates: Vec<crate::live::Update>, cx: &mut Context<Self>) {
        use crate::live::Update;
        for update in updates {
            match update {
                Update::Board => self.board_moved(cx),
                Update::Filed(filed) => self.announce_filed(filed, cx),
                Update::Ping(ping) => self.announce_ping(ping, cx),
            }
        }
    }

    // ── aiball's bus ────────────────────────────────────────────────────

    /// Follows aiball's bus, opened at start as the local owner until the
    /// user is known (the humans aiball knows tell who), then as them. On
    /// each greeting — the first, after a drop, as another user — tvty
    /// subscribes to the board and to the user's pings, resuming where it
    /// was; their events keep the board up to date.
    fn follow_wire(&mut self, mut incoming: futures::channel::mpsc::UnboundedReceiver<crate::wire::Notification>, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            use futures::StreamExt as _;
            while let Some(notice) = incoming.next().await {
                let alive = match notice.method.as_str() {
                    "bus.event" => this.update(cx, |shell, cx| {
                        let updates = shell.live.event(&notice.params);
                        shell.take_updates(updates, cx);
                    }),
                    "bus.hello" => {
                        // The user's pings are one's own: subscribed as a human.
                        let text = |key: &str| notice.params.get(key).and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
                        let user = if text("kind") == "human" { text("consumer") } else { String::new() };
                        let planned = this.update(cx, |shell, _| {
                            let plan = shell.live.plan(&user);
                            // Another user's board is coming: what waits
                            // starts again from it.
                            if !shell.live.ready() {
                                shell.waiting = None;
                            }
                            (shell.wire.clone(), plan)
                        });
                        let Ok((Some(wire), plan)) = planned else { break };
                        let (plan, answers, whoami) = cx
                            .background_executor()
                            .spawn(async move {
                                let answers = crate::live::subscribe(&wire, &plan);
                                let whoami = wire.call("bus.whoami", serde_json::json!({}));
                                (plan, answers, whoami)
                            })
                            .await;
                        let said = match whoami {
                            Ok(v) => format!(
                                "{} ({}) over {}",
                                v["consumer"].as_str().unwrap_or("?"),
                                v["kind"].as_str().unwrap_or("?"),
                                v["transport"].as_str().unwrap_or("?")
                            ),
                            Err(error) => format!("whoami failed: {error:#}"),
                        };
                        this.update(cx, |shell, cx| {
                            shell.wire_whoami = Some(said);
                            let updates = shell.live.subscribed(plan, answers);
                            shell.take_updates(updates, cx);
                            cx.notify();
                        })
                    }
                    _ => {
                        log::debug!("aiball bus: {} {}", notice.method, notice.params);
                        Ok(())
                    }
                };
                if alive.is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    /// A ping to the user, as a notification: urgent in red, waiting for
    /// moderation in orange, else blue.
    fn announce_ping(&mut self, ping: crate::pings::PingInfo, cx: &mut Context<Self>) {
        let kind = if ping.urgent {
            Kind::Error
        } else if ping.pending {
            Kind::Decision
        } else {
            Kind::News
        };
        // The agent's terminal, when it has one here.
        let session = self
            .board
            .projects
            .iter()
            .flat_map(|p| &p.terminals)
            .find(|t| t.agent.as_deref() == Some(ping.from.as_str()))
            .map(|t| t.session.clone());
        let text = if ping.title.is_empty() { ping.what.clone() } else { format!("{} — {}", ping.title, ping.what) };
        let about = (!ping.project.is_empty()).then(|| (ping.project.clone(), ping.ticket));
        activity::publish(cx, Activity::news(ping.from.clone(), kind, about, text).on(session));
    }

    // ── The ticket list, full screen ────────────────────────────────────

    /// The panel's ⤢ and ctrl+shift+l: full screen on what the panel
    /// shows — its ticket, else its project's list —; from full screen, back.
    fn go_full(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.full_list_shown {
            return self.toggle_full_list(window, cx);
        }
        if self.panel.read(cx).is_full() {
            self.full_list_return = false;
            self.panel.update(cx, |panel, cx| panel.set_full(false, cx));
            return;
        }
        self.remember_compact(cx);
        if self.panel.read(cx).snapshot().is_some() {
            if !self.settings.layout.panel_open {
                self.toggle_panel(cx);
            }
            self.panel.update(cx, |panel, cx| panel.set_full(true, cx));
            cx.defer_in(window, |shell, window, cx| window.focus(&shell.focus.clone(), cx));
            return;
        }
        let scope = self.panel.read(cx).scope_project();
        self.toggle_full_list(window, cx);
        if let (Some(list), Some(scope)) = (self.full_list.clone(), scope) {
            list.update(cx, |list, cx| list.set_scope(Some(scope), cx));
        }
    }

    /// Keeps what the panel shows before full screen, once.
    fn remember_compact(&mut self, cx: &App) {
        if self.compact.is_none() {
            self.compact = Some(self.panel.read(cx).snapshot());
        }
    }

    /// Back from full screen: the panel shows again what it showed.
    fn restore_compact(&mut self, cx: &mut Context<Self>) {
        if let Some(snapshot) = self.compact.take() {
            self.panel.update(cx, |panel, cx| panel.restore(snapshot, cx));
        }
    }

    /// Shows it — as it was left, else on the selected terminal's project —
    /// or hides it, back to the panel as it was.
    fn toggle_full_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.full_list_shown {
            self.full_list_shown = false;
            self.restore_compact(cx);
            self.focus_terminal(window, cx);
            cx.notify();
            return;
        }
        self.remember_compact(cx);
        self.full_list_shown = true;
        // Keys go to the list, not to the terminal: after the click that
        // opened it, if a click did, has settled focus.
        cx.defer_in(window, |shell, window, cx| window.focus(&shell.focus.clone(), cx));
        if self.full_list.is_some() {
            self.sync_full_list(cx);
            cx.notify();
            return;
        }
        let scope = self
            .selected
            .as_deref()
            .and_then(|s| self.terminal_of(s))
            .map(|(project, _)| project.to_string())
            .filter(|project| self.board.tickets.contains_key(project));
        let aiball = self.aiball.clone();
        let list = cx.new(|cx| FullList::new(aiball, scope, window, cx));
        cx.subscribe_in(&list, window, |shell, _, _: &CloseFullList, window, cx| {
            shell.toggle_full_list(window, cx)
        })
        .detach();
        self.full_list = Some(list);
        self.sync_full_list(cx);
        cx.notify();
    }

    // ── A new ticket ────────────────────────────────────────────────────

    /// Shows the form, on `project` (else what is shown: the panel's, the
    /// terminal's, the last used) and under `parent`.
    fn open_new_ticket(&mut self, project: Option<String>, parent: Option<u64>, window: &mut Window, cx: &mut Context<Self>) {
        let mut board: Vec<String> = self.board.tickets.keys().cloned().collect();
        board.sort_by_key(|p| p.to_lowercase());
        let terminal = self.selected.as_deref().and_then(|s| self.terminal_of(s)).map(|(p, _)| p.to_string());
        let panel = self.panel.read(cx).scope_project();
        let project = project.or_else(|| {
            crate::newticket::default_project(
                panel.as_deref(),
                terminal.as_deref(),
                self.settings.workspace.last_ticket_project.as_deref(),
                &board,
            )
        });
        self.new_ticket_shown = true;
        let form = match self.new_ticket.clone() {
            Some(form) => form,
            None => {
                let aiball = self.aiball.clone();
                let start = project.clone().unwrap_or_default();
                let form = cx.new(|cx| NewTicketForm::new(aiball, start, window, cx));
                cx.subscribe_in(&form, window, |shell, _, _: &CloseNewTicket, window, cx| shell.close_new_ticket(window, cx))
                    .detach();
                cx.subscribe_in(&form, window, |shell, _, created: &Created, window, cx| {
                    shell.settings.workspace.last_ticket_project = Some(created.project.clone());
                    shell.remember_compact(cx);
                    shell.settings.save(cx);
                    shell.new_ticket_shown = false;
                    shell.full_list_shown = false;
                    if !shell.settings.layout.panel_open {
                        shell.toggle_panel(cx);
                    }
                    let (project, ticket) = (created.project.clone(), created.ticket);
                    activity::publish(cx, Activity::done(Some((project.clone(), ticket)), "filed"));
                    shell.panel.update(cx, |panel, cx| {
                        panel.open_in(Some(project.clone()), ticket, cx);
                        panel.set_full(true, cx);
                    });
                    let _ = shell.refresh_now.unbounded_send(());
                    cx.defer_in(window, |shell, window, cx| window.focus(&shell.focus.clone(), cx));
                    cx.notify();
                })
                .detach();
                self.new_ticket = Some(form.clone());
                form
            }
        };
        form.update(cx, |form, cx| form.prefill(project, parent, window, cx));
        cx.notify();
    }

    fn close_new_ticket(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.new_ticket_shown = false;
        if !self.full_list_shown && !self.panel.read(cx).is_full() {
            self.focus_terminal(window, cx);
        } else {
            window.focus(&self.focus.clone(), cx);
        }
        cx.notify();
    }

    /// A ticket the full list opened: full screen too, back to the list on
    /// leaving it.
    fn open_full_ticket(&mut self, project: String, ticket: u64, window: &mut Window, cx: &mut Context<Self>) {
        self.full_list_shown = false;
        if !self.settings.layout.panel_open {
            self.toggle_panel(cx);
        }
        self.full_list_return = true;
        self.panel.update(cx, |panel, cx| {
            panel.open_in(Some(project), ticket, cx);
            panel.set_full(true, cx);
        });
        cx.defer_in(window, |shell, window, cx| window.focus(&shell.focus.clone(), cx));
        cx.notify();
    }

    fn sync_full_list(&mut self, cx: &mut Context<Self>) {
        let Some(list) = self.full_list.clone() else { return };
        let mut projects: Vec<String> = self.board.tickets.keys().cloned().collect();
        projects.sort_by_key(|p| p.to_lowercase());
        let (aiball, open, critical) = (self.aiball.clone(), self.board.tickets.clone(), self.board.critical.clone());
        list.update(cx, |list, cx| list.set_board(&aiball, projects, open, critical, cx));
    }

    fn focus_terminal(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(terminal) = self.selected.as_ref().and_then(|s| self.terminals.get(s)) {
            let focus = terminal.read(cx).focus_handle().clone();
            window.focus(&focus, cx);
        }
    }

    /// Notifies what newly waits on the user: a decision or
    /// something unread on a ticket an agent with a terminal holds.
    fn update_needs(&mut self, cx: &mut Context<Self>) {
        // Before aiball's board arrived (or arrived again as another
        // user), everything would look new: what waits then is where to
        // start from.
        if !self.live.ready() {
            self.waiting = None;
            return;
        }
        let mut now = HashSet::new();
        let mut found = Vec::new();
        for project in &self.board.projects {
            for terminal in &project.terminals {
                let Some(agent) = &terminal.agent else { continue };
                let tickets = self.board.tickets.get(&project.name).into_iter().flatten();
                for ticket in tickets.filter(|t| t.holder() == Some(agent.as_str())) {
                    // One entry per ticket: a decision says more than "new".
                    let wait = if ticket.pending_decision() {
                        Wait::Decision
                    } else if ticket.unread {
                        Wait::Unread
                    } else {
                        continue;
                    };
                    now.insert((ticket.id, wait));
                    found.push(Need {
                        project: project.name.clone(),
                        session: terminal.session.clone(),
                        agent: agent.clone(),
                        ticket: ticket.id,
                        title: ticket.title.clone(),
                        wait,
                    });
                }
            }
        }
        if let Some(before) = &self.waiting {
            for need in found {
                let key = (need.ticket, need.wait);
                let shown = self.selected.as_deref() == Some(need.session.as_str());
                if !before.contains(&key) && !shown {
                    let (kind, what) = match need.wait {
                        Wait::Decision => (Kind::Decision, "a decision waits on you"),
                        Wait::Unread => (Kind::News, "something new"),
                    };
                    activity::publish(
                        cx,
                        Activity::news(need.agent.clone(), kind, Some((need.project.clone(), need.ticket)), format!("{} — {what}", need.title))
                            .on(Some(need.session.clone())),
                    );
                }
            }
        }
        self.waiting = Some(now);
    }

    /// Goes where a notification points: the agent's terminal, the panel
    /// open on the ticket.
    fn open_notice(&mut self, notice: Notice, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(session) = notice.session.clone() {
            self.select(session, window, cx);
        }
        if let Some((project, ticket)) = notice.ticket {
            if !self.settings.layout.panel_open {
                self.toggle_panel(cx);
            }
            self.panel.update(cx, |panel, cx| panel.open_in(Some(project), ticket, cx));
        }
        cx.notify();
    }

    fn terminal_of(&self, session: &str) -> Option<(&str, &Terminal)> {
        self.board.projects.iter().find_map(|p| {
            p.terminals
                .iter()
                .find(|t| t.session == session)
                .map(|t| (p.name.as_str(), t))
        })
    }

    /// Points the panel at the selected terminal's project and agent.
    fn sync_panel(&mut self, cx: &mut Context<Self>) {
        let on_board = |name: &str| self.board.projects.iter().any(|p| p.name == name && p.on_board);
        let scope = self
            .selected
            .as_deref()
            .and_then(|s| self.terminal_of(s))
            .filter(|(project, _)| on_board(project))
            .map(|(project, terminal)| Scope {
                project: project.to_string(),
                agent: terminal.agent.clone(),
            })
            // An ended session keeps its project's tickets in view.
            .or_else(|| {
                let ended = self.ended_shown()?;
                let project = ended.project.clone().filter(|p| on_board(p))?;
                Some(Scope { project, agent: ended.agent.clone() })
            });
        let tickets = scope
            .as_ref()
            .and_then(|s| self.board.tickets.get(&s.project).cloned())
            .unwrap_or_default();
        let critical = scope
            .as_ref()
            .and_then(|s| self.board.critical.get(&s.project).copied());
        let aiball = self.aiball.clone();
        self.panel.update(cx, |panel, cx| {
            panel.set_scope(scope, cx);
            panel.set_board(&aiball, tickets, critical, cx);
        });
    }

    fn select(&mut self, session: String, window: &mut Window, cx: &mut Context<Self>) {
        // Shown again (or started again under its name): attach afresh; a
        // session still gone ends at once and brings the end screen back.
        self.ended = None;
        let Some(terminal) = self.open_terminal(&session, window, cx) else { return };
        let focus = terminal.read(cx).focus_handle().clone();
        window.focus(&focus, cx);
        if self.selected.as_deref() != Some(session.as_str()) {
            self.switches += 1;
        }
        notify::dismiss_session(cx, &session);
        self.recent.retain(|s| *s != session);
        self.recent.insert(0, session.clone());
        self.selected = Some(session);
        self.save_workspace(cx);
        self.sync_panel(cx);
        cx.notify();
    }

    /// The session's terminal in tvty, opened if it is not: over its socket
    /// when a host holds it, through tmux otherwise.
    fn open_terminal(&mut self, session: &str, window: &mut Window, cx: &mut Context<Self>) -> Option<Entity<TerminalView>> {
        if let Some(terminal) = self.terminals.get(session) {
            return Some(terminal.clone());
        }
        let socket = self.terminal_of(session).and_then(|(_, t)| t.attach.clone());
        // A hosted session never goes through tmux: not listed (yet, or any
        // more), nothing to open.
        if socket.is_none() && session.starts_with(sessions::HOSTED_PREFIX) {
            return None;
        }
        let terminal = cx.new(|cx| match &socket {
            Some(socket) => TerminalView::attach(std::path::Path::new(socket), cx),
            None => TerminalView::tmux(session, cx).expect("failed to spawn the terminal"),
        });
        self.watch_end(session.to_string(), &terminal, window, cx);
        self.terminals.insert(session.to_string(), terminal.clone());
        Some(terminal)
    }

    fn alerts_of(&self, project: &str, agent: Option<&str>) -> Alerts {
        let tickets = self.board.tickets.get(project).into_iter().flatten();
        let critical = self.board.critical.get(project).copied();
        match agent {
            Some(agent) => Alerts::of(tickets.filter(|t| t.holder() == Some(agent)), critical),
            None => Alerts::default(),
        }
    }

    // ── The panel ───────────────────────────────────────────────────────

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.settings.layout.sidebar_open = !self.settings.layout.sidebar_open;
        self.settings.save(cx);
        cx.notify();
    }

    fn toggle_panel(&mut self, cx: &mut Context<Self>) {
        self.settings.layout.panel_open = !self.settings.layout.panel_open;
        self.settings.save(cx);
        cx.notify();
    }

    fn inner_width(window: &Window) -> f32 {
        let paddings = window_paddings(window);
        f32::from(window.viewport_size().width - paddings.left - paddings.right)
    }

    /// The projects' list lies over the terminal: it only needs to leave
    /// some of it visible.
    fn sidebar_width(&self, window: &Window) -> f32 {
        let max = (Self::inner_width(window) * 0.6).max(SIDEBAR_MIN);
        self.settings
            .layout
            .sidebar_width
            .unwrap_or(SIDEBAR_WIDTH)
            .clamp(SIDEBAR_MIN, max)
    }

    fn panel_width(&self, window: &Window) -> f32 {
        let total = Self::inner_width(window);
        // The projects' list lies over the terminal: only its strip takes
        // room from the layout.
        let max = (total - FOLDED_WIDTH - EDGE_WIDTH - CENTER_MIN).max(PANEL_MIN);
        self.settings
            .layout
            .panel_width
            .unwrap_or(total / 3.)
            .clamp(PANEL_MIN, max)
    }

    /// A side's edge is dragged. GPUI's drag, not a mouse move: its moves
    /// reach the shell even over the projects' list, which lies over the
    /// terminal and stops the mouse.
    fn on_drag_move(&mut self, event: &DragMoveEvent<ResizeDrag>, window: &mut Window, cx: &mut Context<Self>) {
        let paddings = window_paddings(window);
        let x = f32::from(event.event.position.x);
        self.dragged = true;
        match self.resizing {
            None => return,
            // The list starts after its strip, where the frame's content does.
            Some(Side::Left) => {
                self.settings.layout.sidebar_width =
                    Some(x - f32::from(paddings.left) - FOLDED_WIDTH - EDGE_WIDTH / 2.)
            }
            // The panel ends where the frame's content does.
            Some(Side::Right) => {
                let right = f32::from(window.viewport_size().width - paddings.right);
                self.settings.layout.panel_width = Some(right - x);
            }
        }
        cx.notify();
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.end_resize(window, cx);
    }

    fn end_resize(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.dragged = false;
        if self.resizing.take().is_some() {
            // Store what is shown, not an out-of-range drag.
            self.settings.layout.panel_width = Some(self.panel_width(window));
            self.settings.layout.sidebar_width = Some(self.sidebar_width(window));
            self.settings.save(cx);
            cx.notify();
        }
    }

    // ── Keys: the slider and the gallery ────────────────────────────────

    /// Before the terminal sees a key: the window's own shortcuts.
    fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.capture.is_some() {
            self.capture_key(event, cx);
            cx.stop_propagation();
            return;
        }
        let keystroke = &event.keystroke;
        let m = &keystroke.modifiers;
        let key = keystroke.key.as_str();
        if self.viewer.is_some() {
            if self.viewer_key(key, window, cx) {
                cx.stop_propagation();
            }
            return;
        }
        if self.ended_shown().is_some() && !m.control && !m.alt && (key == "enter" || key == "escape") {
            if key == "enter" {
                self.restart_ended(cx);
            } else {
                self.close_ended(window, cx);
            }
            cx.stop_propagation();
            return;
        }
        // The shortcuts are commands bound in `crate::keymap` (they come
        // before this); here, the keys of what is up: Esc, the slider's and
        // the gallery's.
        if key == "escape" && self.options.is_some() {
            self.toggle_options(window, cx);
        } else if key == "escape" && (self.backlog_view.is_some() || self.afk_menu) {
            self.backlog_view = None;
            self.afk_menu = false;
            cx.notify();
        } else if key == "escape" && self.new_ticket_shown {
            self.close_new_ticket(window, cx);
        } else if key == "escape" && self.panel.read(cx).is_full() {
            self.panel.update(cx, |panel, cx| panel.set_full(false, cx));
        } else if key == "escape" && self.full_list_shown {
            self.toggle_full_list(window, cx);
        } else if key == "escape" && self.theme_menu {
            self.theme_menu = false;
            cx.notify();
        } else if key == "escape" && (self.slider.is_some() || self.gallery.is_some()) {
            self.slider = None;
            self.close_gallery(window, cx);
        } else if let (Some(chosen), Some(step)) = (self.slider.clone(), Move::of(key)) {
            let groups = self.slider_groups();
            let centres = self.card_centres.borrow().clone();
            self.slider = moved(&groups, Some(&chosen), step, &centres).or(Some(chosen));
            cx.notify();
        } else if self.gallery.is_some() {
            // The gallery is up: keys filter it, arrows move along its cards.
            let chosen = self.gallery_chosen();
            let groups = self.card_sessions(&self.gallery_filter());
            let centres = self.card_centres.borrow().clone();
            let Some(gallery) = self.gallery.as_mut() else { return };
            match key {
                "backspace" => {
                    gallery.filter.pop();
                }
                "enter" => {
                    if let Some(session) = chosen {
                        self.gallery = None;
                        self.select(session, window, cx);
                    }
                }
                _ if Move::of(key).is_some() => {
                    gallery.chosen = moved(&groups, chosen.as_deref(), Move::of(key).unwrap(), &centres).or(chosen);
                }
                _ if !m.control && !m.alt && !m.platform => {
                    if let Some(text) = &keystroke.key_char {
                        gallery.filter.push_str(text);
                    }
                }
                _ => return,
            }
            cx.notify();
        } else {
            return;
        }
        cx.stop_propagation();
    }

    /// The slider commits when ctrl is released, like alt-tab.
    fn on_modifiers(&mut self, event: &ModifiersChangedEvent, window: &mut Window, cx: &mut Context<Self>) {
        log::debug!("modifiers {:?}, slider {:?}", event.modifiers, self.slider);
        if self.slider.is_some() && !event.modifiers.control {
            if let Some(session) = self.slider.take() {
                self.select(session, window, cx);
            }
            cx.notify();
        }
    }

    fn step_slider(&mut self, step: isize, cx: &mut Context<Self>) {
        log::debug!("slider step {step} from {:?}", self.slider);
        let order = self.slider_order();
        let len = order.len() as isize;
        if len == 0 {
            return;
        }
        if self.slider.is_none() {
            self.slider_shown += 1;
            self.card_centres.borrow_mut().clear();
        }
        // From the current terminal (index 0), the first tap goes to the
        // previous one: the quick hop between two. The cards stay in place:
        // only the chosen one changes.
        let index = self
            .slider
            .as_ref()
            .and_then(|s| order.iter().position(|o| o == s))
            .unwrap_or(0) as isize;
        self.slider = order.get((index + step).rem_euclid(len) as usize).cloned();
        self.watch_cards(cx);
        cx.notify();
    }

    fn toggle_gallery(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.gallery.is_some() {
            self.close_gallery(window, cx);
            return;
        }
        self.card_centres.borrow_mut().clear();
        self.gallery = Some(Gallery {
            filter: String::new(),
            chosen: self.selected.clone(),
        });
        // Keys go to the window while the gallery is up.
        window.focus(&self.focus.clone(), cx);
        self.watch_cards(cx);
        cx.notify();
    }

    fn close_gallery(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.gallery = None;
        if let Some(terminal) = self.selected.as_ref().and_then(|s| self.terminals.get(s)) {
            let focus = terminal.read(cx).focus_handle().clone();
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    /// Makes the cards live while the slider or the gallery is up: the
    /// terminals open here draw their own screen; every other session gets
    /// a read-only client that never resizes it, closed with the view.
    fn watch_cards(&mut self, cx: &mut Context<Self>) {
        if self.watching {
            return;
        }
        self.watching = true;
        let sessions: Vec<String> = self
            .board
            .projects
            .iter()
            .flat_map(|p| p.terminals.iter().map(|t| t.session.clone()))
            .filter(|s| !self.terminals.contains_key(s))
            .filter(|s| !s.starts_with(sessions::HOSTED_PREFIX))
            .collect();
        // The host's sessions: an observer over their socket, no tmux.
        let hosted: Vec<(String, String)> = self
            .board
            .projects
            .iter()
            .flat_map(|p| p.terminals.iter())
            .filter(|t| !self.terminals.contains_key(&t.session) && !self.watchers.contains_key(&t.session))
            .filter_map(|t| Some((t.session.clone(), t.attach.clone()?)))
            .collect();
        for (session, socket) in hosted {
            let watcher = cx.new(|cx| TerminalView::observe(std::path::Path::new(&socket), cx));
            self.watchers.insert(session, watcher);
        }
        cx.spawn(async move |this, cx| {
            let sized = cx
                .background_executor()
                .spawn(async move {
                    sessions
                        .into_iter()
                        .filter_map(|s| sessions::window_size(&s).map(|(c, l)| (s, c, l)))
                        .collect::<Vec<_>>()
                })
                .await;
            let _ = this.update(cx, |shell, cx| {
                for (session, columns, lines) in sized {
                    if shell.terminals.contains_key(&session) || shell.watchers.contains_key(&session) {
                        continue;
                    }
                    let watcher = cx.new(|cx| {
                        TerminalView::watch(&session, columns, lines, cx).expect("failed to spawn a watcher")
                    });
                    shell.watchers.insert(session, watcher);
                }
            });
            // Redraw at a steady pace while shown; then let the watchers go.
            loop {
                cx.background_executor().timer(CARDS_EVERY).await;
                let shown = this.update(cx, |shell, cx| {
                    let shown = shell.gallery.is_some() || shell.slider.is_some();
                    if shown {
                        cx.notify();
                    } else {
                        shell.watchers.clear();
                        shell.watching = false;
                    }
                    shown
                });
                if !matches!(shown, Ok(true)) {
                    break;
                }
            }
        })
        .detach();
    }

    /// A session's screen, live: its terminal here, else its watcher.
    fn screen(&self, session: &str, cx: &App) -> Option<crate::terminal::Snapshot> {
        self.terminals
            .get(session)
            .or_else(|| self.watchers.get(session))
            .map(|view| view.read(cx).screen())
    }

    /// The cards as the slider and the gallery lay them out: the projects
    /// in the list's order, each with its terminals, kept by `filter`
    /// (label or project, lowercase; empty keeps all).
    fn card_groups(&self, filter: &str) -> Vec<(&str, Vec<&Terminal>)> {
        self.board
            .projects
            .iter()
            .map(|p| {
                let shown = p
                    .terminals
                    .iter()
                    .filter(|t| {
                        filter.is_empty()
                            || t.label.to_lowercase().contains(filter)
                            || p.name.to_lowercase().contains(filter)
                    })
                    .collect::<Vec<_>>();
                (p.name.as_str(), shown)
            })
            .filter(|(_, shown)| !shown.is_empty())
            .collect()
    }

    /// [`Shell::card_groups`], by session: what the arrows move along.
    fn card_sessions(&self, filter: &str) -> Vec<Vec<String>> {
        self.card_groups(filter)
            .into_iter()
            .map(|(_, terminals)| terminals.into_iter().map(|t| t.session.clone()).collect())
            .collect()
    }

    fn gallery_filter(&self) -> String {
        self.gallery.as_ref().map(|g| g.filter.to_lowercase()).unwrap_or_default()
    }

    /// The gallery's chosen card, if the filter shows it; else the first shown.
    fn gallery_chosen(&self) -> Option<String> {
        let groups = self.card_sessions(&self.gallery_filter());
        let chosen = self.gallery.as_ref()?.chosen.as_ref();
        chosen
            .filter(|c| groups.iter().flatten().any(|s| s == *c))
            .or_else(|| groups.first()?.first())
            .cloned()
    }

    // ── Options ─────────────────────────────────────────────────────────

    fn toggle_options(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.end_capture(cx);
        if self.options.take().is_none() {
            self.options = Some(Section::Appearance);
            theme::load(cx);
            // Keys go to the page, not to the terminal.
            window.focus(&self.focus.clone(), cx);
        } else if let Some(terminal) = self.selected.as_ref().and_then(|s| self.terminals.get(s)) {
            let focus = terminal.read(cx).focus_handle().clone();
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    fn options_view(&self, section: Section, window: &Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let mut nav = div()
            .flex()
            .flex_col()
            .gap_0p5()
            .w(px(220.))
            .flex_none()
            .h_full()
            .p_3()
            .bg(p().surface)
            .border_r_1()
            .border_color(p().border)
            .child(
                div()
                    .px_2()
                    .pb_3()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .child("Options"),
            );
        for item in Section::ALL {
            let chosen = item == section;
            nav = nav.child(
                div()
                    .id(SharedString::from(format!("options-{}", item.title())))
                    .px_2()
                    .py_1p5()
                    .rounded_md()
                    .cursor_pointer()
                    .when(chosen, |d| d.bg(p().active).text_color(p().text))
                    .when(!chosen, |d| d.text_color(p().muted).hover(|d| d.bg(p().hover)))
                    .child(item.title())
                    .on_click(cx.listener(move |shell, _, _, cx| {
                        shell.end_capture(cx);
                        shell.options = Some(item);
                        cx.notify();
                    })),
            );
        }
        let content = match section {
            Section::Appearance => self
                .options_settings("Appearance", cx)
                .child(option_note(
                    "Your own themes (gpui-component's theme format) go in ~/.config/tvty/themes/: they show here the next time this page opens.",
                ))
                .into_any_element(),
            Section::Layout => self.options_layout(window, cx).into_any_element(),
            Section::TicketList => self.options_settings("Ticket list", cx).child(options_ticket_list()).into_any_element(),
            Section::Shortcuts => self.options_shortcuts(cx).into_any_element(),
            Section::About => self.options_about(cx).into_any_element(),
        };
        div()
            .id("options")
            .absolute()
            .inset_0()
            // A page over the window: the mouse stops here.
            .occlude()
            .flex()
            .bg(p().bg)
            .text_color(p().text)
            .child(nav)
            .child(
                div()
                    // One per page: each opens at its top.
                    .id(SharedString::from(format!("options-content-{}", section.title())))
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .p_6()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .pb_4()
                            .child(div().flex_1().text_xl().font_weight(FontWeight::BOLD).child(section.title()))
                            .child(
                                div()
                                    .id("options-close")
                                    .px_2()
                                    .rounded_sm()
                                    .text_sm()
                                    .text_color(p().muted)
                                    .cursor_pointer()
                                    .hover(|d| d.bg(p().hover).text_color(p().text))
                                    .child("✕  Esc")
                                    .on_click(cx.listener(|shell, _, window, cx| shell.toggle_options(window, cx))),
                            ),
                    )
                    .child(content)
                    .overflow_y_scrollbar(),
            )
    }

    /// A page of settings, built from the schema: its groups, and a row per
    /// setting as its kind says — a number steps, a toggle switches, choices
    /// are lists side by side. Every change goes through [`Self::set_pref`].
    fn options_settings(&self, page: &str, cx: &mut Context<Self>) -> Div {
        let mut out = div().flex().flex_col().gap_2().max_w(px(720.));
        for (group, settings) in SCHEMA.groups(page) {
            out = out.child(option_group(group));
            if settings.iter().all(|s| s.kind == SettingKind::Choice) {
                let mut lists = div().flex().gap_6();
                for setting in settings {
                    lists = lists.child(self.choice_list(setting, cx));
                }
                out = out.child(lists);
                continue;
            }
            for setting in settings {
                let key = setting.key;
                let row = match (setting.kind, SCHEMA.value(&self.applied, key)) {
                    (SettingKind::Number { unit, .. }, Some(Value::Number(n))) => option_stepper(
                        setting.label,
                        setting.about,
                        shown(n, unit),
                        cx.listener(move |shell, _, _, cx| shell.step_pref(key, -1, cx)),
                        cx.listener(move |shell, _, _, cx| shell.step_pref(key, 1, cx)),
                        cx.listener(move |shell, _, _, cx| shell.reset_pref(key, cx)),
                    )
                    .into_any_element(),
                    (SettingKind::Toggle { on, off, .. }, Some(Value::Toggle(checked))) => option_switch(
                        setting.label,
                        setting.about,
                        if checked { on } else { off },
                        Switch::new(SharedString::from(format!("options-switch-{key}")))
                            .checked(checked)
                            .on_click(cx.listener(move |shell, wanted: &bool, _, cx| shell.set_pref(key, Value::Toggle(*wanted), cx))),
                    )
                    .into_any_element(),
                    _ => continue,
                };
                out = out.child(row);
            }
        }
        out
    }

    /// A choice's list: its label, what it says, the choices (the themes,
    /// dark ones first), the one in force ticked.
    fn choice_list(&self, setting: &'static Setting, cx: &mut Context<Self>) -> Div {
        let key = setting.key;
        // The window's theme is always one; the terminals' may be the
        // window's own (none).
        let (chosen, default) = match key {
            "appearance.theme" => (Some(theme::current(cx)), None),
            _ => (theme::current_terminal(), Some("Same as the window")),
        };
        let column = div()
            .flex()
            .flex_col()
            .gap_1()
            .flex_1()
            .min_w_0()
            .child(div().font_weight(FontWeight::BOLD).child(setting.label))
            .child(div().pb_1().text_sm().text_color(p().muted).child(setting.about));
        let choice = |name: SharedString, label: SharedString, value: Option<SharedString>, on: bool, cx: &mut Context<Self>| {
            div()
                .id(SharedString::from(format!("options-{key}-{name}")))
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .rounded_md()
                .cursor_pointer()
                .when(on, |d| d.bg(p().active))
                .hover(|d| d.bg(p().hover))
                .child(div().w(px(12.)).child(if on { "✓" } else { "" }))
                .child(label)
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell.set_pref(key, Value::Choice(value.as_ref().map(|v| v.to_string())), cx);
                }))
        };
        let mut column = column;
        if let Some(label) = default {
            column = column.child(choice("default".into(), label.into(), None, chosen.is_none(), cx));
        }
        let mut last_dark = None;
        for (name, dark) in theme::names(cx) {
            if last_dark != Some(dark) {
                last_dark = Some(dark);
                column = column.child(div().px_3().pt_2().text_xs().text_color(p().muted).child(if dark { "Dark" } else { "Light" }));
            }
            let on = chosen.as_ref() == Some(&name);
            column = column.child(choice(name.clone(), name.clone(), Some(name), on, cx));
        }
        column
    }

    fn options_layout(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sidebar = format!(
            "{} · {} px",
            if self.settings.layout.sidebar_open { "open" } else { "folded" },
            self.sidebar_width(window).round()
        );
        let panel = format!(
            "{} · {} px",
            if self.settings.layout.panel_open { "open" } else { "folded" },
            self.panel_width(window).round()
        );
        div()
            .flex()
            .flex_col()
            .gap_2()
            .max_w(px(640.))
            .child(option_row(
                "Projects' list",
                "Over the terminal, on the left. Drag its edge to resize it; its grip folds it.",
                sidebar,
                "Fold / unfold",
                cx.listener(|shell, _, _, cx| shell.toggle_sidebar(cx)),
            ))
            .child(option_row(
                "Ticket panel",
                "On the right of the terminal. Drag its edge to resize it; its grip folds it.",
                panel,
                "Fold / unfold",
                cx.listener(|shell, _, _, cx| shell.toggle_panel(cx)),
            ))
            .child(option_row(
                "Widths",
                "Back to the defaults: a list of 290 px, a panel a third of the window.",
                String::new(),
                "Reset",
                cx.listener(|shell, _, _, cx| {
                    shell.settings.layout.sidebar_width = None;
                    shell.settings.layout.panel_width = None;
                    shell.settings.save(cx);
                    cx.notify();
                }),
            ))
    }

    fn options_about(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        use crate::config::{Place, dir};
        let shown = |place| dir(place).map(|d| d.display().to_string()).unwrap_or_default();
        let rows = [
            // The version and the commit it was built from (`+`: with changes not committed).
            ("Version", match env!("TVTY_COMMIT") {
                "" => env!("CARGO_PKG_VERSION").to_string(),
                commit => format!("{} · {commit}", env!("CARGO_PKG_VERSION")),
            }),
            ("aiball socket", crate::aiball::socket_path().display().to_string()),
            ("Acting as", self.aiball.user.clone()),
            (
                "aiball bus",
                match (self.wire.as_ref().and_then(|w| w.hello()), &self.wire_whoami) {
                    (Some(hello), Some(whoami)) => format!("version {}, as {whoami}", hello.version),
                    (Some(hello), None) => format!("version {}, as {} ({})", hello.version, hello.consumer, hello.kind),
                    (None, _) => "not connected".to_string(),
                },
            ),
            (
                "Live board",
                match self.live.subscriptions() {
                    0 => "not subscribed".to_string(),
                    n => format!("{n} subscriptions on the bus"),
                },
            ),
            ("Theme", theme::current(cx).to_string()),
            (
                "Terminal theme",
                theme::current_terminal().map_or("the window's".to_string(), |name| name.to_string()),
            ),
            ("Settings, shortcuts, themes", shown(Place::Config)),
            ("Layout and workspace", shown(Place::State)),
            ("Fonts", shown(Place::Data)),
        ];
        let mut table = div().flex().flex_col().gap_1().max_w(px(720.));
        for (label, value) in rows {
            table = table.child(
                div()
                    .flex()
                    .gap_4()
                    .py_1()
                    .border_b_1()
                    .border_color(p().border)
                    .child(div().w(px(180.)).flex_none().text_color(p().muted).child(label))
                    .child(div().flex_1().min_w_0().child(value)),
            );
        }
        table.child(option_note(
            "tvty — Terminal Velocity. MIT licence. Bundled themes: see themes/README.md.",
        ))
    }

    // ── Shortcuts ───────────────────────────────────────────────────────

    /// Listens for a key for `command` (replacing `replacing`, or one more):
    /// every binding off meanwhile, so that the key comes here whatever it
    /// runs.
    fn start_capture(&mut self, command: &'static str, replacing: Option<keymap::Key>, cx: &mut Context<Self>) {
        if self.capture.is_none() {
            keymap::suspend(cx);
        }
        self.capture = Some(Capture { command, replacing, pending: None, refused: None });
        cx.notify();
    }

    fn end_capture(&mut self, cx: &mut Context<Self>) {
        if self.capture.take().is_some() {
            keymap::resume(cx);
            cx.notify();
        }
    }

    /// A key pressed while listening: Esc gives up; a key a terminal's
    /// program types is refused there; a key another command runs asks
    /// first; any other is bound.
    fn capture_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let Some(key) = keymap::key_of(&event.keystroke) else { return };
        if key.canonical() == "escape" {
            self.end_capture(cx);
            return;
        }
        let Some(capture) = self.capture.as_mut() else { return };
        let Some(command) = keymap::COMMANDS.iter().find(|c| c.name == capture.command) else { return };
        capture.pending = None;
        capture.refused = None;
        if command.context == keymap::TERMINAL && key.is_typing() {
            capture.refused = Some(format!("{} is typing: in a terminal it stays the program's", key.pretty()));
        } else if let Some(other) = keymap::current(cx).conflict(command.name, &key) {
            capture.pending = Some((key, other));
        } else {
            self.bind_captured(key, cx);
            return;
        }
        cx.notify();
    }

    /// The key heard bound to the command listened for — taken from the
    /// command that ran it, if any.
    fn bind_captured(&mut self, key: keymap::Key, cx: &mut Context<Self>) {
        let Some(capture) = self.capture.as_ref() else { return };
        let mut map = keymap::current(cx);
        let bound = match &capture.replacing {
            Some(old) => map.rebind(capture.command, old, &key),
            None => map.bind(capture.command, &key),
        };
        match bound {
            Ok(()) => keymap::save(cx, &map),
            Err(error) => log::warn!("shortcuts: {error}"),
        }
        self.end_capture(cx);
    }

    /// The keymap changed by `change`, and kept.
    fn change_keys(&mut self, cx: &mut Context<Self>, change: impl FnOnce(&mut keymap::Keymap)) {
        self.end_capture(cx);
        let mut map = keymap::current(cx);
        change(&mut map);
        keymap::save(cx, &map);
        cx.notify();
    }

    /// The shortcuts in force, by context: a key clicked listens for its
    /// replacement, × removes it, + adds one, Default puts a command's keys
    /// back; the keys given back to the program; those a terminal masks;
    /// then the keys that are not commands. Every change goes to
    /// keymap.toml, which the user may edit too.
    fn options_shortcuts(&self, cx: &mut Context<Self>) -> Div {
        let (map, error) = keymap::in_force(cx);
        let bindings = map.bindings();
        let listening = |command: &str, key: Option<&keymap::Key>| {
            self.capture.as_ref().is_some_and(|c| c.command == command && c.replacing.as_ref() == key)
        };
        let small = |id: SharedString, label: &'static str| {
            div()
                .id(id)
                .px_2()
                .rounded_sm()
                .text_xs()
                .border_1()
                .border_color(p().border)
                .cursor_pointer()
                .hover(|d| d.bg(p().hover))
                .child(label)
        };
        let listening_chip = |id: SharedString| {
            div()
                .id(id)
                .px_1p5()
                .rounded_sm()
                .border_1()
                .border_color(p().accent)
                .text_sm()
                .text_color(p().accent)
                .child("Press a key… (Esc gives up)")
        };
        let file = keymap::path().map(|p| p.display().to_string()).unwrap_or_else(|| "keymap.toml".into());
        let changed = map.commands().iter().any(|c| map.is_changed(c.name)) || bindings.iter().any(|b| b.command.is_none());
        let mut page = div()
            .flex()
            .flex_col()
            .max_w(px(960.))
            .child(option_note("Each shortcut is a command, bound in a context: Terminal when a terminal has the focus, Window anywhere else. The deepest binding wins; a key bound nowhere goes to the terminal's program."))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .pb_2()
                    .child(div().flex_1().min_w_0().text_sm().text_color(p().muted).child(format!(
                        "Click a key to change it, + to add one, × to remove it. Kept in {file}, which you may edit too."
                    )))
                    .when(changed, |d| {
                        d.child(small("keys-reset-all".into(), "Reset all").on_click(cx.listener(|shell, _, _, cx| shell.change_keys(cx, |map| map.reset_all()))))
                    }),
            )
            .when_some(error, |d, error| {
                d.child(div().p_2().mb_2().rounded_md().border_1().border_color(p().danger).text_sm().text_color(p().danger).child(format!("Not applied: {error}")))
            });
        let masked = map.masked();
        for (context, title) in [(keymap::WINDOW, "Window — anywhere in tvty"), (keymap::TERMINAL, "Terminal — when a terminal has the focus")] {
            page = page.child(option_group(title));
            for command in keymap::COMMANDS.iter().filter(|c| c.context == context) {
                let name = command.name;
                let mut chips = div().flex().flex_wrap().items_center().gap_1();
                for binding in bindings.iter().filter(|b| b.context == context && b.command == Some(name)) {
                    let key = binding.key.clone();
                    let id = format!("key-{name}-{}", key.canonical());
                    if listening(name, Some(&key)) {
                        chips = chips.child(listening_chip(id.into()));
                        continue;
                    }
                    let (again, gone) = (key.clone(), key.clone());
                    chips = chips.child(
                        div()
                            .flex()
                            .items_center()
                            .rounded_sm()
                            .border_1()
                            .border_color(p().border)
                            .bg(p().surface)
                            .text_sm()
                            .child(
                                div()
                                    .id(SharedString::from(id.clone()))
                                    .px_1p5()
                                    .cursor_pointer()
                                    .hover(|d| d.bg(p().hover))
                                    .child(key.pretty())
                                    .on_click(cx.listener(move |shell, _, _, cx| shell.start_capture(name, Some(again.clone()), cx))),
                            )
                            .child(
                                div()
                                    .id(SharedString::from(format!("{id}-x")))
                                    .px_1()
                                    .text_color(p().muted)
                                    .cursor_pointer()
                                    .hover(|d| d.bg(p().hover).text_color(p().danger))
                                    .child("×")
                                    .on_click(cx.listener(move |shell, _, _, cx| {
                                        shell.change_keys(cx, |map| map.unbind(context, &gone))
                                    })),
                            ),
                    );
                }
                chips = if listening(name, None) {
                    chips.child(listening_chip(format!("key-{name}-new").into()))
                } else {
                    chips.child(small(format!("key-{name}-add").into(), "+").on_click(cx.listener(move |shell, _, _, cx| shell.start_capture(name, None, cx))))
                };
                let mut what = command.what.to_string();
                if let Some((_, by)) = masked.iter().find(|(b, _)| b.command == Some(name)) {
                    what = format!("{what} — masked in a terminal by {}", by.command.unwrap_or("the program"));
                }
                let custom = map.is_changed(name);
                let mut row = div().flex().flex_col().py_1p5().border_b_1().border_color(p().border).child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(div().w(px(300.)).flex_none().child(chips))
                        .child(div().flex_1().min_w_0().text_sm().child(what))
                        .when(custom, |d| {
                            d.child(small(format!("key-{name}-default").into(), "Default").on_click(cx.listener(move |shell, _, _, cx| {
                                shell.change_keys(cx, |map| map.reset(name))
                            })))
                        })
                        .child(div().w(px(130.)).flex_none().text_xs().text_color(p().muted).child(name)),
                );
                // What the key heard would do, or why it was refused.
                if let Some(capture) = self.capture.as_ref().filter(|c| c.command == name) {
                    if let Some((key, other)) = capture.pending.clone() {
                        row = row.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .pt_1()
                                .text_sm()
                                .child(format!("{} runs {other}: take it for {name}?", key.pretty()))
                                .child(small("key-replace".into(), "Replace").on_click(cx.listener(move |shell, _, _, cx| shell.bind_captured(key.clone(), cx))))
                                .child(small("key-cancel".into(), "Cancel").on_click(cx.listener(|shell, _, _, cx| shell.end_capture(cx)))),
                        );
                    }
                    if let Some(refused) = capture.refused.clone() {
                        row = row.child(div().pt_1().text_sm().text_color(p().danger).child(refused));
                    }
                }
                page = page.child(row);
            }
            // The keys given back to what has the focus: × binds them again.
            for freed in bindings.iter().filter(|b| b.context == context && b.command.is_none()) {
                let key = freed.key.clone();
                let to = if context == keymap::TERMINAL { "Given back to the terminal's program" } else { "Given back to what has the focus" };
                page = page.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .py_1p5()
                        .border_b_1()
                        .border_color(p().border)
                        .child(
                            div().w(px(300.)).flex_none().child(
                                div().flex().items_center().gap_1().child(key_chip(key.pretty())).child(
                                    small(format!("freed-{context}-{}", key.canonical()).into(), "×").on_click(cx.listener(move |shell, _, _, cx| {
                                        shell.change_keys(cx, |map| map.restore(context, &key))
                                    })),
                                ),
                            ),
                        )
                        .child(div().flex_1().min_w_0().text_sm().child(to)),
                );
            }
        }
        page = page.child(option_group("Fixed keys"));
        for (keys, what) in FIXED_KEYS {
            page = page.child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .py_1p5()
                    .border_b_1()
                    .border_color(p().border)
                    .child(div().w(px(300.)).flex_none().child(div().flex().flex_wrap().gap_1().children(keys.split(" · ").map(|k| key_chip(k.to_string())))))
                    .child(div().flex_1().min_w_0().text_sm().child(*what)),
            );
        }
        page
    }

    // ── Preferences ─────────────────────────────────────────────────────

    /// The one way a preference changes, from the options, a shortcut or a
    /// menu: the store takes it (and writes settings.toml), and
    /// [`Self::preferences_changed`] puts it in force — as for a hand edit.
    fn set_pref(&mut self, key: &str, value: Value, cx: &mut Context<Self>) {
        crate::config::update::<Preferences>(cx, |prefs| match SCHEMA.with(prefs, key, value) {
            Ok(changed) => *prefs = changed,
            Err(error) => log::warn!("settings: {error}"),
        });
    }

    /// A number preference moved by `steps` steps, within its bounds.
    fn step_pref(&mut self, key: &str, steps: i32, cx: &mut Context<Self>) {
        crate::config::update::<Preferences>(cx, |prefs| match SCHEMA.stepped(prefs, key, steps) {
            Ok(changed) => *prefs = changed,
            Err(error) => log::warn!("settings: {error}"),
        });
    }

    /// A preference back to its default.
    fn reset_pref(&mut self, key: &str, cx: &mut Context<Self>) {
        crate::config::update::<Preferences>(cx, |prefs| match SCHEMA.reset(prefs, key) {
            Ok(changed) => *prefs = changed,
            Err(error) => log::warn!("settings: {error}"),
        });
    }

    /// The preferences changed (from tvty or a hand edit of settings.toml):
    /// what differs from what is in force is put in force.
    fn preferences_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let new = crate::config::get::<Preferences>(cx).clone();
        let old = std::mem::replace(&mut self.applied, new.clone());
        if old == new {
            return;
        }
        let (was, now) = (&old.appearance, &new.appearance);
        if was.theme != now.theme {
            let name = theme::known_or_default(now.theme.as_deref(), cx);
            theme::apply(&name, Some(window), cx);
        }
        if was.terminal_theme != now.terminal_theme {
            theme::apply_terminal(now.terminal_theme.as_deref(), cx);
        }
        if was.terminal_font_size != now.terminal_font_size {
            crate::terminal::set_font_size(now.terminal_font_size.unwrap_or(crate::terminal::FONT_SIZE_DEFAULT));
        }
        if was.window_font_size != now.window_font_size {
            theme::set_window_font(now.window_font_size, cx);
        }
        let notifications = &new.notifications;
        notify::set_limits(
            cx,
            notifications.max.unwrap_or(notify::MAX_DEFAULT),
            notifications.seconds.unwrap_or(notify::SECONDS_DEFAULT),
        );
        crate::activity::set_own(cx, notifications.own);
        crate::wheel::set_speed(new.scroll.speed);
        let newest_first = new.tickets.newest_first;
        self.panel.update(cx, |panel, cx| panel.set_newest_first(newest_first, cx));
        self.redraw_terminals(cx);
        window.refresh();
        cx.notify();
    }

    // ── Themes ──────────────────────────────────────────────────────────

    fn set_theme(&mut self, name: SharedString, cx: &mut Context<Self>) {
        self.set_pref("appearance.theme", Value::Choice(Some(name.to_string())), cx);
        self.theme_menu = false;
        cx.notify();
    }

    /// Every terminal (and card) drawn again now: each is a cached view,
    /// which otherwise waits for its program's next output to take new
    /// colours or a new font size.
    fn redraw_terminals(&self, cx: &mut Context<Self>) {
        for terminal in self.terminals.values().chain(self.watchers.values()) {
            terminal.update(cx, |_, cx| cx.notify());
        }
    }

    fn next_theme(&mut self, cx: &mut Context<Self>) {
        let names = theme::names(cx);
        let current = theme::current(cx);
        let at = names.iter().position(|(n, _)| *n == current).map_or(0, |i| i + 1);
        if let Some((name, _)) = names.get(at % names.len().max(1)).cloned() {
            self.set_theme(name, cx);
        }
    }

    fn toggle_theme_menu(&mut self, cx: &mut Context<Self>) {
        self.theme_menu = !self.theme_menu;
        if self.theme_menu {
            // A theme file dropped or edited meanwhile shows now.
            theme::load(cx);
        }
        cx.notify();
    }

    fn theme_menu_view(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let current = theme::current(cx);
        let mut list = div().id("theme-menu-list").flex().flex_col().py_1().text_sm();
        let mut last_dark = None;
        for (name, dark) in theme::names(cx) {
            if last_dark != Some(dark) {
                last_dark = Some(dark);
                list = list.child(
                    div()
                        .px_3()
                        .pt_2()
                        .pb_1()
                        .text_xs()
                        .text_color(p().muted)
                        .child(if dark { "DARK" } else { "LIGHT" }),
                );
            }
            let chosen = name == current;
            let pick = name.clone();
            list = list.child(
                div()
                    .id(SharedString::from(format!("theme-{name}")))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_1()
                    .cursor_pointer()
                    .when(chosen, |d| d.bg(p().active))
                    .hover(|d| d.bg(p().hover))
                    .child(div().w(px(10.)).child(if chosen { "✓" } else { "" }))
                    .child(name.clone())
                    .on_click(cx.listener(move |shell, _, _, cx| shell.set_theme(pick.clone(), cx))),
            );
        }
        // The box holds the place under the title bar; the list scrolls in
        // it (the scrollbar's wrapper keeps a size, not a position).
        div()
            .id("theme-menu")
            .absolute()
            .occlude()
            .top(px(36.))
            .right_2()
            .w(px(240.))
            .h(px(420.))
            .flex()
            .flex_col()
            .rounded_md()
            .bg(p().surface)
            .border_1()
            .border_color(p().border)
            .shadow_lg()
            .overflow_hidden()
            .child(list.overflow_y_scrollbar())
    }

    // ── Rendering ───────────────────────────────────────────────────────

    /// The strip between a side and the terminal: a grip that folds the
    /// side away; the rest of it drags the side's width.
    fn edge(&self, side: Side, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let resizing = self.resizing == Some(side);
        div()
            .id(match side {
                Side::Left => "sidebar-edge",
                Side::Right => "panel-edge",
            })
            .flex()
            .items_center()
            .justify_center()
            .w(px(EDGE_WIDTH))
            .flex_none()
            .h_full()
            .bg(p().bg)
            .cursor(CursorStyle::ResizeColumn)
            .when(resizing, |d| d.bg(p().active))
            .hover(|d| d.bg(p().active))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |shell, _, _, cx| {
                    shell.resizing = Some(side);
                    cx.notify();
                }),
            )
            .on_mouse_up(MouseButton::Left, cx.listener(|shell, _, window, cx| shell.end_resize(window, cx)))
            .on_drag(ResizeDrag, |_, _, _, cx| cx.new(|_| NoPreview))
            .child(
                div()
                    .id(match side {
                        Side::Left => "sidebar-grip",
                        Side::Right => "panel-grip",
                    })
                    .w(px(6.))
                    .h(px(44.))
                    .rounded_full()
                    .bg(p().border)
                    .cursor_pointer()
                    .hover(|d| d.bg(p().accent))
                    // A press on the grip is a toggle, not the start of a drag.
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(cx.listener(move |shell, _, _, cx| match side {
                        Side::Left => shell.toggle_sidebar(cx),
                        Side::Right => shell.toggle_panel(cx),
                    })),
            )
    }

    /// A folded side: 14 pixels, a dot per thing waiting, a click unfolds.
    fn folded(&self, side: Side, dots: Vec<Hsla>, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        div()
            .id(match side {
                Side::Left => "sidebar-folded",
                Side::Right => "panel-folded",
            })
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .pt_2()
            .w(px(FOLDED_WIDTH))
            .flex_none()
            .h_full()
            .bg(p().surface)
            .map(|d| match side {
                Side::Left => d.border_r_1(),
                Side::Right => d.border_l_1(),
            })
            .border_color(p().border)
            .cursor_pointer()
            .hover(|d| d.bg(p().active))
            .children(
                dots.into_iter()
                    .map(|color| div().flex_none().size(px(6.)).rounded_full().bg(color)),
            )
            .on_click(cx.listener(move |shell, _, _, cx| match side {
                Side::Left => shell.toggle_sidebar(cx),
                Side::Right => shell.toggle_panel(cx),
            }))
    }

    /// Starts a shell the daemon's host holds, in the home directory, and
    /// opens it once it is listed.
    fn new_terminal(&mut self, cx: &mut Context<Self>) {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
        self.open_shell("term", home, cx);
    }

    /// Starts a shell in a project's folder: listed with the project, after
    /// its agents.
    fn new_project_terminal(&mut self, project: &str, cx: &mut Context<Self>) {
        let Some(folder) = self.project_folder(project) else {
            activity::publish(cx, Activity::failed(None, "new terminal", format!("no folder known for {project}")));
            return;
        };
        // A session's name takes letters, digits, `.`, `_` and `-`.
        let prefix: String = project
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') { c } else { '-' })
            .collect();
        self.open_shell(&prefix, folder, cx);
    }

    /// Starts a shell on the daemon's host, named `<prefix>-<n>`, in `cwd`,
    /// and opens it once it is listed.
    fn open_shell(&mut self, prefix: &str, cwd: String, cx: &mut Context<Self>) {
        // Every name the host knows, a stopped session's too (it keeps its name
        // until it is removed).
        let taken: std::collections::HashSet<String> = self.live.session_names().into_iter().collect();
        let name = (1..).map(|n| format!("{prefix}-{n}")).find(|n| !taken.contains(n)).expect("a free name");
        let shell = std::env::var("SHELL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "bash".into());
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let started = {
                let name = name.clone();
                cx.background_executor().spawn(async move { aiball.start_terminal(&name, &[shell], &cwd) }).await
            };
            let _ = this.update(cx, |shell, cx| match started {
                Ok(()) => shell.open_when_running = Some(format!("{}{name}", sessions::HOSTED_PREFIX)),
                Err(error) => activity::publish(
                    cx,
                    Activity::failed(None, "new terminal", format!("{error:#}")),
                ),
            });
        })
        .detach();
    }

    /// The projects' list: its tabs on the left, then the tab's list.
    fn sidebar(&self, width: f32, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let content = self.sessions_list(cx);
        // Its header lines up with the tickets panel's: the name, and the
        // chevron that folds the list towards the window's edge.
        let header = div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(36.))
            .px_3()
            .border_b_1()
            .border_color(p().border)
            .child(div().font_weight(FontWeight::BOLD).child("Sessions"))
            .child(div().flex_1())
            // A shell the daemon holds: it outlives tvty.
            .child(
                div()
                    .id("new-terminal")
                    .mr_2()
                    .px_1p5()
                    .rounded_sm()
                    .text_xs()
                    .text_color(p().accent)
                    .cursor_pointer()
                    .hover(|d| d.bg(p().hover))
                    .child("+ terminal")
                    .on_click(cx.listener(|shell, _, _, cx| shell.new_terminal(cx))),
            )
            .child(
                div()
                    .id("sidebar-collapse")
                    .px_1()
                    .cursor_pointer()
                    .text_color(p().accent)
                    .child("‹")
                    .on_click(cx.listener(|shell, _, _, cx| shell.toggle_sidebar(cx))),
            );
        div()
            .flex()
            .flex_none()
            .w(px(width))
            .h_full()
            .bg(p().surface)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(width))
                    .h_full()
                    .child(header)
                    .child(div().flex().flex_col().flex_1().min_h_0().text_sm().child(content)),
            )
    }

    /// The sessions that run, by project; "+ session" opens one.
    pub(super) fn live_list(&self, cx: &mut Context<Self>) -> Div {
        let mut list = div().flex().flex_col();
        if self.board.projects.is_empty() {
            list = list.child(div().px_3().text_color(p().muted).child("No session"));
        }
        for project in &self.board.projects {
            let tickets = self.board.tickets.get(&project.name);
            let critical = self.board.critical.get(&project.name).copied();
            // A project shows the sum of what it asks, collapsed or not.
            let alerts = Alerts::of(tickets.into_iter().flatten(), critical);
            // Fixed heights: the list is laid out on every frame of the
            // shell, and rows the layout must measure cost dearly.
            list = list.child(
                div()
                    .h(px(SESSION_HEADING))
                    .flex_none()
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .pt_2()
                    .pb_1()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p().muted)
                            .child(project.name.to_uppercase()),
                    )
                    .child(alerts.badges(format!("project-{}", project.name)))
                    .when(project.on_board, |d| {
                        let name = project.name.clone();
                        d.child(
                            div()
                                .id(SharedString::from(format!("new-session-{name}")))
                                .px_1()
                                .rounded_sm()
                                .text_xs()
                                .text_color(p().accent)
                                .cursor_pointer()
                                .hover(|d| d.bg(p().hover))
                                .child("+ session")
                                .on_click(cx.listener(move |shell, _, window, cx| {
                                    shell.ask_new_session(name.clone(), window, cx)
                                })),
                        )
                    })
                    // A plain shell in the project's folder, listed with it.
                    .when(project.on_board, |d| {
                        let name = project.name.clone();
                        d.child(
                            div()
                                .id(SharedString::from(format!("new-shell-{name}")))
                                .px_1()
                                .rounded_sm()
                                .text_xs()
                                .text_color(p().accent)
                                .cursor_pointer()
                                .hover(|d| d.bg(p().hover))
                                .child(">_")
                                .tip("a terminal in the project's folder, listed with it")
                                .on_click(cx.listener(move |shell, _, _, cx| shell.new_project_terminal(&name, cx))),
                        )
                    }),
            )
            .children(self.new_session_form(&project.name, cx));
            for terminal in &project.terminals {
                let session = terminal.session.clone();
                let selected = self.selected.as_deref() == Some(session.as_str());
                let open = self.terminals.contains_key(&session);
                let alerts = self.alerts_of(&project.name, terminal.agent.as_deref());
                let state = terminal.status.as_ref().and_then(|s| s.colour());
                let restart = terminal
                    .agent
                    .as_ref()
                    .and_then(|a| self.board.bars.get(a))
                    .is_some_and(|b| !b.stale && b.bar.alerts.restart_needed);
                let height = if terminal.status.is_some() { SESSION_ROW } else { SESSION_ROW_BARE };
                list = list.child(
                    div()
                        .id(SharedString::from(format!("terminal-{session}")))
                        .h(px(height))
                        .flex_none()
                        .overflow_hidden()
                        .flex()
                        .gap_2()
                        .pl_1()
                        .pr_3()
                        .py_1()
                        .cursor_pointer()
                        .when(selected, |d| d.bg(p().active).text_color(p().text))
                        .when(!selected, |d| d.hover(|d| d.bg(p().hover)))
                        // The loop's state colour: working, idle, starting.
                        .child(
                            div()
                                .w(px(3.))
                                .flex_none()
                                .rounded_sm()
                                .when_some(state, |d, colour| d.bg(colour)),
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
                                        .items_center()
                                        .gap_2()
                                        // Green: the terminal already runs in tvty.
                                        .child(
                                            div()
                                                .id("open")
                                                .w(px(7.))
                                                .when(open, |d| d.child(dot(p().success)).tip("open in tvty: its terminal runs here")),
                                        )
                                        .child(div().flex_1().min_w_0().truncate().child(terminal.label.clone()))
                                        // Its Claude runs in claude-loop (through tmux), not on aiball's host.
                                        .when(terminal.agent.is_some() && terminal.attach.is_none(), |d| {
                                            d.child(
                                                div()
                                                    .id("looped")
                                                    .text_xs()
                                                    .text_color(p().muted)
                                                    .child("⇄")
                                                    .tip("its Claude runs in claude-loop, opened through tmux (not on aiball's host)"),
                                            )
                                        })
                                        // A shell the host holds, without Claude.
                                        .when(terminal.agent.is_none() && terminal.attach.is_some(), |d| {
                                            d.child(
                                                div()
                                                    .id("shell")
                                                    .text_xs()
                                                    .text_color(p().muted)
                                                    .child(">_")
                                                    .tip("a terminal on aiball's host, without Claude"),
                                            )
                                        })
                                        // Its Claude Code waits for a restart (the button is on its bar).
                                        .when(restart, |d| {
                                            d.child(
                                                div()
                                                    .id("restart")
                                                    .text_color(p().warning)
                                                    .child("⟳")
                                                    .tip("its Claude Code installed an update: restart it from its bar"),
                                            )
                                        })
                                        .child(alerts.badges(format!("row-{}", terminal.session))),
                                )
                                .when_some(terminal.status.as_ref(), |d, status| d.child(status.line())),
                        )
                        .on_click(cx.listener(move |shell, _, window, cx| {
                            shell.select(session.clone(), window, cx)
                        })),
                );
            }
        }
        list
    }

    /// A terminal as a card: its name and alerts over a thumbnail.
    fn card(
        &self,
        project: &str,
        terminal: &Terminal,
        chosen: bool,
        live: bool,
        width: f32,
        height: f32,
        cx: &App,
    ) -> Stateful<Div> {
        let session = &terminal.session;
        let alerts = self.alerts_of(project, terminal.agent.as_deref());
        let centres = self.card_centres.clone();
        let key = session.clone();
        // Where the card lands, for up and down; the slider's enlarged card
        // is centred on its slot, so both give the same place.
        let spot = canvas(
            move |bounds, _, _| {
                centres.borrow_mut().insert(key, bounds.center());
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();
        div()
            .id(SharedString::from(format!("card-{session}{}", if live { "" } else { "-ghost" })))
            .relative()
            .bg(p().bg)
            .child(spot)
            .flex()
            .flex_col()
            .w(px(width))
            .rounded_md()
            .overflow_hidden()
            .border_2()
            .border_color(if chosen { p().accent } else { p().border })
            .when(chosen, |d| d.shadow_lg())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .bg(if chosen { p().active } else { p().surface })
                    .text_sm()
                    .child(div().flex_1().min_w_0().truncate().child(terminal.label.clone()))
                    .child(alerts.badges(format!("card-{}", terminal.session))),
            )
            .when_some(terminal.status.as_ref(), |d, status| {
                d.child(div().px_2().pb_1().bg(p().surface).child(status.line()))
            })
            .child(
                div()
                    .h(px(height))
                    .overflow_hidden()
                    .bg(p().bg)
                    .when_some(self.screen(session, cx).filter(|_| live), |d, screen| {
                        d.child(screen.viewport(CARD_LINES, CARD_COLUMNS))
                    }),
            )
    }

    fn gallery_view(&self, gallery: &Gallery, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let chosen = self.gallery_chosen();
        // Projects side by side, each with its terminals: groups flow.
        let mut body = div()
            .id("gallery")
            .flex()
            .flex_wrap()
            .content_start()
            .gap_x_6()
            .gap_y_4()
            .p_4()
            .flex_1()
            .min_h_0();
        for (project, terminals) in self.card_groups(&self.gallery_filter()) {
            let mut row = div().flex().flex_wrap().gap_3();
            for terminal in terminals {
                let session = terminal.session.clone();
                let is_chosen = chosen.as_deref() == Some(session.as_str());
                row = row.child(
                    self.card(project, terminal, is_chosen, true, 320., 190., cx)
                        .cursor_pointer()
                        .hover(|d| d.border_color(p().accent))
                        .on_click(cx.listener(move |shell, _, window, cx| {
                            shell.gallery = None;
                            shell.select(session.clone(), window, cx);
                        })),
                );
            }
            body = body.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p().muted)
                            .child(project.to_uppercase()),
                    )
                    .child(row),
            );
        }
        div()
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .flex_col()
            .bg(p().bg)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(p().border)
                    .child(div().font_weight(FontWeight::BOLD).child("All terminals"))
                    .child(
                        div()
                            .flex_1()
                            .text_color(if gallery.filter.is_empty() { p().muted } else { p().text })
                            .child(if gallery.filter.is_empty() {
                                "type to filter · arrows move · enter opens · esc closes".to_string()
                            } else {
                                format!("{}▏", gallery.filter)
                            }),
                    ),
            )
            .child(body.overflow_y_scrollbar())
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _timing = crate::stats::Timing::new("shell");
        // The workspace as it was left, once aiball and tmux have answered.
        if self.restoring.as_ref().is_some_and(|r| r.due(|s| self.terminal_of(s).is_some())) && self.live.ready() && self.local_seen {
            cx.defer_in(window, |shell, window, cx| shell.restore(window, cx));
        }
        // A loop just started runs now: open it.
        if let Some(name) = self.open_when_running.clone().filter(|n| self.terminal_of(n).is_some()) {
            self.open_when_running = None;
            cx.defer_in(window, move |shell, window, cx| shell.select(name, window, cx));
        }
        // A drag released anywhere, even over the title bar: the edge's
        // drag is gone, the width is kept.
        if self.dragged && !cx.has_active_drag() {
            self.end_resize(window, cx);
        }
        let bar = self.agent_bar(cx);
        let tabs = self.tab_bar(cx);
        let ended = self.ended_shown().map(|e| self.ended_view(e, cx));
        let tabbed = tabs.is_some() && ended.is_none();
        let center = match self.selected.as_ref().and_then(|s| self.terminals.get(s)).filter(|_| ended.is_none()) {
            // The terminal slides in on every switch. A relative offset, not a
            // margin: the terminal keeps its size, so tmux is not resized.
            // Under it, its agent's bar, which does not slide.
            Some(terminal) => div()
                .size_full()
                .flex()
                .flex_col()
                .children(tabs)
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .min_h_0()
                        // Its own drawing, reused until it changes: the rest
                        // of the window does not move with its output.
                        .child(terminal.clone().cached(StyleRefinement::default().size_full()))
                        .with_animation(
                            SharedString::from(format!("switch-{}", self.switches)),
                            Animation::new(Duration::from_millis(240)).with_easing(|t| 1. - (1. - t).powi(3)),
                            |d, t| d.left(px(60. * (1. - t))).opacity(0.25 + 0.75 * t),
                        ),
                )
                .children(bar)
                .into_any_element(),
            None if ended.is_some() => ended.unwrap_or_else(|| div().into_any_element()),
            None => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(p().muted)
                .child("Pick a terminal on the left · ctrl+shift+space shows them all")
                .into_any_element(),
        };
        // The strip stays in the layout, open or folded, so the terminal
        // keeps its width; open, the projects' list lies over the terminal.
        let overlay = self.settings.layout.sidebar_open.then(|| {
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left_0()
                .occlude()
                .flex()
                .shadow_lg()
                .child(self.sidebar(self.sidebar_width(window), cx))
                .child(self.edge(Side::Left, cx))
        });
        let left = {
            let dots = self
                .board
                .projects
                .iter()
                .filter_map(|p| {
                    let tickets = self.board.tickets.get(&p.name).into_iter().flatten();
                    let alerts = Alerts::of(tickets, self.board.critical.get(&p.name).copied());
                    alerts.color()
                })
                .collect();
            self.folded(Side::Left, dots, cx)
        };
        // Full screen, the panel lies over the window; its place shows the
        // folded strip meanwhile.
        let panel_full = self.panel.read(cx).is_full();
        let right = if self.settings.layout.panel_open && !panel_full {
            div()
                .flex()
                .flex_none()
                .w(px(self.panel_width(window) + EDGE_WIDTH))
                .h_full()
                .child(self.edge(Side::Right, cx))
                .child(div().flex_1().min_w_0().h_full().child(self.panel.clone().cached(StyleRefinement::default().size_full())))
                .into_any_element()
        } else {
            let alerts = self
                .selected
                .as_deref()
                .and_then(|s| self.terminal_of(s))
                .map(|(project, _)| {
                    let tickets = self.board.tickets.get(project).into_iter().flatten();
                    Alerts::of(tickets, self.board.critical.get(project).copied())
                })
                .unwrap_or_default();
            self.folded(Side::Right, alerts.colors(), cx).into_any_element()
        };
        // The notices sit in the terminal's top right corner: left of the
        // panel, or of its folded edge; a full screen covers the terminal,
        // then the window's corner.
        let paddings = window_paddings(window);
        let covered = panel_full || self.options.is_some() || self.full_list_shown || self.new_ticket_shown;
        // Below the tabs, when there are.
        let notices_top = f32::from(paddings.top)
            + TITLE_BAR_HEIGHT
            + NOTICE_MARGIN
            + if tabbed && !covered { tabs::TAB_BAR } else { 0. };
        let notices_right = f32::from(paddings.right)
            + NOTICE_MARGIN
            + match () {
                _ if covered => 0.,
                _ if self.settings.layout.panel_open => self.panel_width(window) + EDGE_WIDTH,
                _ => FOLDED_WIDTH,
            };
        let slider = self.slider.clone().map(|chosen| self.portfolio(&chosen, cx));
        let options = self.options.map(|section| self.options_view(section, window, cx));
        let full_list = self.full_list.clone().filter(|_| self.full_list_shown);
        let new_ticket = self.new_ticket.clone().filter(|_| self.new_ticket_shown);
        // A click on something that takes no focus (a list, the panel)
        // leaves none: the keys would reach nothing, the shell's shortcuts
        // included. Once the click is done, the shell takes them back.
        let shell_focus = self.focus.clone();
        let keep_focus = canvas(
            |_, _, _| {},
            move |_, _, window, _| {
                crate::wheel::speed_up(window);
                window.on_mouse_event(move |_: &MouseUpEvent, phase, window, cx| {
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    let focus = shell_focus.clone();
                    window.defer(cx, move |window, cx| {
                        if window.focused(cx).is_none() {
                            window.focus(&focus, cx);
                        }
                    });
                });
            },
        )
        .absolute()
        .size_0();
        let gallery = self.gallery.as_ref().map(|g| self.gallery_view(g, cx));

        let title = match self.selected.as_deref().and_then(|s| self.terminal_of(s)) {
            Some((project, terminal)) => format!("tvty — {project} · {}", terminal.label),
            None => "tvty".to_string(),
        };
        let body = div()
            .id("shell")
            .track_focus(&self.focus)
            .key_context(crate::keymap::WINDOW)
            .capture_key_down(cx.listener(Self::on_key))
            .on_action(cx.listener(|shell, _: &keymap::OpenNotice, window, cx| match notify::newest(cx) {
                Some(notice) => {
                    notify::dismiss(cx, notice.id);
                    shell.open_notice(notice, window, cx);
                }
                // No notification: the key goes on (a terminal's program, a field).
                None => cx.propagate(),
            }))
            .on_action(cx.listener(|shell, _: &keymap::SliderNext, _, cx| shell.step_slider(1, cx)))
            .on_action(cx.listener(|shell, _: &keymap::SliderBack, _, cx| shell.step_slider(-1, cx)))
            .on_action(cx.listener(|shell, _: &keymap::TabNext, window, cx| shell.step_tab(1, window, cx)))
            .on_action(cx.listener(|shell, _: &keymap::TabBack, window, cx| shell.step_tab(-1, window, cx)))
            .on_action(cx.listener(|shell, _: &keymap::Gallery, window, cx| shell.toggle_gallery(window, cx)))
            .on_action(cx.listener(|shell, _: &keymap::TogglePanel, _, cx| shell.toggle_panel(cx)))
            .on_action(cx.listener(|shell, _: &keymap::ToggleSidebar, _, cx| shell.toggle_sidebar(cx)))
            .on_action(cx.listener(|shell, _: &keymap::ToggleOptions, window, cx| shell.toggle_options(window, cx)))
            .on_action(cx.listener(|shell, _: &keymap::FontBigger, _, cx| shell.step_pref(TERMINAL_FONT, 1, cx)))
            .on_action(cx.listener(|shell, _: &keymap::FontSmaller, _, cx| shell.step_pref(TERMINAL_FONT, -1, cx)))
            .on_action(cx.listener(|shell, _: &keymap::FontReset, _, cx| shell.reset_pref(TERMINAL_FONT, cx)))
            .on_action(cx.listener(|shell, _: &keymap::NewTicket, window, cx| shell.open_new_ticket(None, None, window, cx)))
            .on_action(cx.listener(|shell, _: &keymap::FullList, window, cx| shell.go_full(window, cx)))
            .on_action(cx.listener(|shell, _: &keymap::NextTheme, _, cx| shell.next_theme(cx)))
            .on_modifiers_changed(cx.listener(Self::on_modifiers))
            .on_drag_move(cx.listener(Self::on_drag_move))
            // A section's title dragged, in the sessions list or the panel.
            .on_drag_move(cx.listener(|shell, event: &DragMoveEvent<crate::accordion::SectionDrag>, _, cx| {
                if crate::accordion::drag_to(event.drag(cx), event.event.position.y) {
                    shell.panel.update(cx, |_, cx| cx.notify());
                    cx.notify();
                    // Kept once the drag rests, wherever it is let go.
                    shell.heights_moved += 1;
                    let moved = shell.heights_moved;
                    cx.spawn(async move |this, cx| {
                        cx.background_executor().timer(Duration::from_millis(400)).await;
                        let _ = this.update(cx, |shell, cx| {
                            if shell.heights_moved == moved {
                                crate::accordion::commit(cx);
                            }
                        });
                    })
                    .detach();
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .relative()
            .flex()
            .w_full()
            .flex_1()
            .min_h_0()
            .bg(p().bg)
            .text_color(p().text)
            .when(self.resizing.is_some(), |d| d.cursor(CursorStyle::ResizeColumn))
            .child(left)
            .child(
                div()
                    .relative()
                    .flex_1()
                    .h_full()
                    .min_w_0()
                    .child(center)
                    .children(overlay),
            )
            .child(right)
            .children(slider)
            .children(gallery)
            .child(keep_focus)
            .children(full_list)
            .when(panel_full, |d| {
                d.child(div().absolute().inset_0().occlude().bg(p().bg).child(self.panel.clone().cached(StyleRefinement::default().size_full())))
            })
            .children(new_ticket)
            .children(options);

        // The window draws its own title bar: GNOME leaves decorations to the
        // application (as with VS Code or Zed). It moves the window and
        // carries its buttons; the frame and its resize edges come from Root.
        let theme_name = theme::current(cx);
        let menu = self.theme_menu.then(|| self.theme_menu_view(cx));
        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(p().bg)
            .child(
                // A fifth taller than the kit's (34 px): easier to grab.
                TitleBar::new()
                    .h(px(TITLE_BAR_HEIGHT))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_sm()
                            .text_color(p().muted)
                            .truncate()
                            .child(title),
                    )
                    .child(
                        div()
                            .id("theme-button")
                            .flex_none()
                            .mr_2()
                            .px_2()
                            .rounded_sm()
                            .text_xs()
                            .text_color(p().muted)
                            .cursor_pointer()
                            .hover(|d| d.bg(p().hover).text_color(p().text))
                            .child(format!("◐ {theme_name}"))
                            .on_click(cx.listener(|shell, _, _, cx| shell.toggle_theme_menu(cx))),
                    )
                    .child(
                        div()
                            .id("options-button")
                            .flex_none()
                            .mr_2()
                            .px_2()
                            .rounded_sm()
                            .text_sm()
                            .text_color(p().muted)
                            .cursor_pointer()
                            .hover(|d| d.bg(p().hover).text_color(p().text))
                            .child("⚙")
                            .on_click(cx.listener(|shell, _, window, cx| shell.toggle_options(window, cx))),
                    ),
            )
            .child(body)
            .children(menu)
            .children(self.viewer_view(window, cx))
            // Above everything, the full screens and the gallery included.
            .children(notify::stack(notices_top, notices_right, cx))
            // Above even the notices: the window's edges resize it.
            .children(frame::resize_band(window))
    }
}

/// What a side's edge carries while dragged.
pub struct ResizeDrag;

/// A drag that shows nothing under the pointer.
struct NoPreview;

impl Render for NoPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

fn option_group(title: &'static str) -> impl IntoElement {
    div()
        .pt_4()
        .pb_1()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(p().muted)
        .child(title.to_uppercase())
}

fn option_note(text: &'static str) -> impl IntoElement {
    div().py_2().text_sm().text_color(p().muted).child(text)
}

/// A setting: its name and what it does, its state, one button.
fn option_row(
    name: &'static str,
    about: &'static str,
    state: String,
    action: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_4()
        .p_3()
        .rounded_md()
        .bg(p().surface)
        .border_1()
        .border_color(p().border)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .flex_1()
                .min_w_0()
                .child(div().font_weight(FontWeight::BOLD).child(name))
                .child(div().text_sm().text_color(p().muted).child(about)),
        )
        .child(div().flex_none().text_sm().text_color(p().muted).child(state))
        .child(
            div()
                .id(SharedString::from(format!("options-action-{name}")))
                .flex_none()
                .px_3()
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(p().border)
                .cursor_pointer()
                .hover(|d| d.bg(p().hover))
                .child(action)
                .on_click(on_click),
        )
}

/// A size: its name and what it does, − the value +, and back to default.
fn option_stepper(
    name: &'static str,
    about: &'static str,
    value: String,
    minus: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    plus: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    reset: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let button = |id: &str, label: &'static str| {
        div()
            .id(SharedString::from(format!("options-{id}-{name}")))
            .flex_none()
            .px_3()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(p().border)
            .cursor_pointer()
            .hover(|d| d.bg(p().hover))
            .child(label)
    };
    div()
        .flex()
        .items_center()
        .gap_2()
        .p_3()
        .rounded_md()
        .bg(p().surface)
        .border_1()
        .border_color(p().border)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .flex_1()
                .min_w_0()
                .child(div().font_weight(FontWeight::BOLD).child(name))
                .child(div().text_sm().text_color(p().muted).child(about)),
        )
        .child(button("minus", "−").on_click(minus))
        .child(div().w(px(56.)).flex_none().text_center().child(value))
        .child(button("plus", "+").on_click(plus))
        .child(button("reset", "Default").on_click(reset))
}

/// A toggle: its name and what it does, its state, a switch.
fn option_switch(name: &'static str, about: &'static str, state: &'static str, switch: Switch) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_4()
        .p_3()
        .rounded_md()
        .bg(p().surface)
        .border_1()
        .border_color(p().border)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .flex_1()
                .min_w_0()
                .child(div().font_weight(FontWeight::BOLD).child(name))
                .child(div().text_sm().text_color(p().muted).child(about)),
        )
        .child(div().flex_none().text_sm().text_color(p().muted).child(state))
        .child(switch)
}

/// A key as a chip.
fn key_chip(key: String) -> impl IntoElement {
    div().px_1p5().rounded_sm().border_1().border_color(p().border).bg(p().surface).text_sm().child(key)
}

/// The legend of the ticket list: its bands, its glyphs, its stripe.
fn options_ticket_list() -> impl IntoElement {
    use crate::rowstate::{Band, Glyph};
    let line = |mark: gpui_kit::AnyElement, what: String| {
        div()
            .flex()
            .items_center()
            .gap_4()
            .py_1()
            .border_b_1()
            .border_color(p().border)
            .child(div().w(px(90.)).flex_none().flex().justify_center().child(mark))
            .child(div().flex_1().min_w_0().text_sm().child(what))
    };
    let glyph = |g: Glyph, colour: Hsla| crate::icons::icon(crate::icons::of_glyph(g), colour, 16.).into_any_element();
    let stripe = |colour: Hsla, dashed: bool| {
        div()
            .h(px(18.))
            .border_l_3()
            .border_color(colour)
            .when(dashed, |d| d.border_dashed())
            .into_any_element()
    };
    let mut page = div()
        .flex()
        .flex_col()
        .max_w(px(760.))
        .child(option_note(
            "The ticket list as aiball computes it for you: the band, whose turn and the state glyph are aiball's; the stripe is tvty's.",
        ))
        .child(option_group("Order"));
    for band in [Band::Moderate, Band::Decide, Band::AgentOnIt, Band::Open] {
        let what = match band {
            Band::Moderate => "the ticket, or comments on it, wait for moderation",
            Band::Decide => "a plan, a resolution, a wontfix or an escalation waits for your decision",
            Band::AgentOnIt => "an agent holds it or is on a step",
            _ => "open, nothing pressing",
        };
        page = page.child(line(
            div().text_xs().text_color(p().muted).child(band.title().to_uppercase()).into_any_element(),
            format!("{what}; the most recent first"),
        ));
    }
    page = page
        .child(option_group("State glyph — coloured when it waits on you, muted otherwise"))
        .child(line(glyph(Glyph::Plan, p().warning), Glyph::Plan.meaning().into()))
        .child(line(glyph(Glyph::Resolution, p().success), Glyph::Resolution.meaning().into()))
        .child(line(glyph(Glyph::Wontfix, p().muted), Glyph::Wontfix.meaning().into()))
        .child(line(glyph(Glyph::Escalation, p().danger), Glyph::Escalation.meaning().into()))
        .child(line(glyph(Glyph::Step, p().accent), format!("{} — always blue", Glyph::Step.meaning())))
        .child(line(glyph(Glyph::StalledStep, p().warning), format!("{} — always amber", Glyph::StalledStep.meaning())))
        .child(line(glyph(Glyph::Rejected, p().danger), format!("{} — always red", Glyph::Rejected.meaning())))
        .child(line(glyph(Glyph::ClosedResolved, p().muted), Glyph::ClosedResolved.meaning().into()))
        .child(line(glyph(Glyph::Closed, p().muted), Glyph::Closed.meaning().into()))
        .child(option_group("Stripe — whose turn"))
        .child(line(stripe(p().warning, false), "a decision waits on you, and it is the last message".into()))
        .child(line(stripe(p().warning, true), "a decision waits on you, but the talk went on after it".into()))
        .child(line(stripe(p().border, false), "your turn: an agent answered you".into()))
        .child(line(
            div().h(px(18.)).border_l_1().border_dashed().border_color(p().muted.opacity(0.6)).into_any_element(),
            "thin and dotted: your word is the last, you wait".into(),
        ))
        .child(line(div().into_any_element(), "no stripe: the ball is with the agent".into()))
        .child(option_group("The rest"))
        .child(line(
            crate::icons::labelled(crate::icons::Icon::Comments, p().accent, 12., "3").text_xs().text_color(p().accent).into_any_element(),
            "comments, blue: something new on it for you, unread (its title in bold too)".into(),
        ))
        .child(line(
            crate::icons::labelled(crate::icons::Icon::Comments, p().muted, 12., "3").text_xs().text_color(p().muted).into_any_element(),
            "grey: you spoke last".into(),
        ))
        .child(line(
            crate::icons::labelled(crate::icons::Icon::Comments, p().muted.opacity(0.55), 12., "3")
                .text_xs()
                .text_color(p().muted.opacity(0.55))
                .into_any_element(),
            "lighter grey: someone else spoke last, and you read it".into(),
        ))
        .child(line(
            crate::icons::labelled(crate::icons::Icon::PendingComments, p().warning, 12., "1").text_xs().text_color(p().warning).into_any_element(),
            "comments waiting for moderation".into(),
        ))
        .child(line(
            crate::icons::labelled(crate::icons::Icon::Hot, p().warning, 12., "agent")
                .text_xs()
                .text_color(p().muted)
                .into_any_element(),
            "the agent holding it; the flame when it was active lately".into(),
        ))
        .child(line(
            crate::icons::pill(crate::icons::Icon::Critical, "3", p().danger).into_any_element(),
            "the project's critical ticket: it holds 3 open tickets".into(),
        ))
        .child(line(
            div()
                .flex()
                .gap_1()
                .child(crate::icons::icon(crate::icons::Icon::PriorityUrgent, crate::icons::priority_colour("urgent"), 14.))
                .child(crate::icons::icon(crate::icons::Icon::PriorityHigh, crate::icons::priority_colour("high"), 14.))
                .child(crate::icons::icon(crate::icons::Icon::PriorityLow, crate::icons::priority_colour("low"), 14.))
                .into_any_element(),
            "priority: urgent, high, low (normal shows nothing)".into(),
        ));
    page
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use gpui_kit::{point, px};

    use super::{Move, moved};

    fn at(cards: &[(&str, f32, f32)]) -> HashMap<String, gpui_kit::Point<gpui_kit::Pixels>> {
        cards.iter().map(|(s, x, y)| (s.to_string(), point(px(*x), px(*y)))).collect()
    }

    fn groups() -> Vec<Vec<String>> {
        [&["a1", "a2", "a3"][..], &["b1"], &["c1", "c2"]]
            .iter()
            .map(|g| g.iter().map(|s| s.to_string()).collect())
            .collect()
    }

    #[test]
    fn arrows_follow_the_cards_and_wrap() {
        let g = groups();
        assert_eq!(moved(&g, Some("a3"), Move::Forth, &HashMap::new()).as_deref(), Some("b1"));
        assert_eq!(moved(&g, Some("b1"), Move::Back, &HashMap::new()).as_deref(), Some("a3"));
        assert_eq!(moved(&g, Some("c2"), Move::Forth, &HashMap::new()).as_deref(), Some("a1"));
        assert_eq!(moved(&g, Some("a1"), Move::Back, &HashMap::new()).as_deref(), Some("c2"));
    }

    #[test]
    fn up_and_down_keep_the_place_in_the_group() {
        let g = groups();
        assert_eq!(moved(&g, Some("a2"), Move::Down, &HashMap::new()).as_deref(), Some("b1"));
        assert_eq!(moved(&g, Some("c2"), Move::Down, &HashMap::new()).as_deref(), Some("a2"));
        assert_eq!(moved(&g, Some("a3"), Move::Up, &HashMap::new()).as_deref(), Some("c2"));
    }

    #[test]
    fn up_and_down_go_by_what_is_seen() {
        // a1 a2 a3 b1 on the first row, c1 c2 under a2 and a3.
        let g = groups();
        let seen = at(&[
            ("a1", 100., 100.), ("a2", 300., 100.), ("a3", 500., 100.), ("b1", 800., 100.),
            ("c1", 300., 400.), ("c2", 500., 400.),
        ]);
        assert_eq!(moved(&g, Some("a3"), Move::Down, &seen).as_deref(), Some("c2"));
        assert_eq!(moved(&g, Some("b1"), Move::Down, &seen).as_deref(), Some("c2"));
        assert_eq!(moved(&g, Some("c1"), Move::Up, &seen).as_deref(), Some("a2"));
        // Nothing below: the next group, as without positions.
        assert_eq!(moved(&g, Some("c1"), Move::Down, &seen).as_deref(), Some("a1"));
    }

    #[test]
    fn from_nowhere_the_first_card() {
        assert_eq!(moved(&groups(), None, Move::Down, &HashMap::new()).as_deref(), Some("a1"));
        assert_eq!(moved(&groups(), Some("gone"), Move::Forth, &HashMap::new()).as_deref(), Some("a1"));
        assert_eq!(moved(&[], None, Move::Forth, &HashMap::new()), None);
    }
}
