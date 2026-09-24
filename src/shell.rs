//! The window: projects and their terminals on the left, the selected
//! terminal in the middle, the ticket panel on the right (resizable,
//! collapsible), and two ways to switch — the slider (ctrl+tab, most recent
//! first) and the gallery (ctrl+shift+space, every terminal as a thumbnail).
//! See docs/UX.md.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::aiball::{Aiball, TicketRow};
use crate::events;
use crate::panel::{BoardChanged, CollapsePanel, Scope, TicketPanel, dot, pill};
use crate::sessions::{self, Board, Terminal};
use crate::settings::Settings;
use crate::terminal::{Snapshot, TerminalView};

/// How often the tmux sessions are listed (cheap: one `tmux ls`).
const SESSIONS_EVERY: Duration = Duration::from_secs(3);
/// How often the board is read when aiball's live feed is down.
const POLL_EVERY: Duration = Duration::from_secs(5);
/// How often the board is read anyway, feed or not.
const REREAD_EVERY: Duration = Duration::from_secs(60);
/// How often the gallery's thumbnails are captured while it is open.
const THUMBNAILS_EVERY: Duration = Duration::from_millis(1500);

const SIDEBAR_WIDTH: f32 = 220.;
const PANEL_MIN: f32 = 260.;
const CENTER_MIN: f32 = 320.;
/// A folded side: just enough for a few dots saying what waits.
const FOLDED_WIDTH: f32 = 10.;
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
    /// The panel's edge is being dragged.
    resizing: bool,
    /// The slider is up, on this index of [`Shell::slider_order`].
    slider: Option<usize>,
    /// Counts the slider's openings: each one replays its entrance.
    slider_shown: usize,
    gallery: Option<Gallery>,
    /// Screens of every terminal, for the gallery and the slider; kept
    /// between openings so they open already filled.
    thumbnails: HashMap<String, Snapshot>,
    /// A capture loop is running.
    capturing: bool,
    /// What agents wait on from the user, newest last: the banner.
    needs: Vec<Need>,
    /// What was already waiting at the last read, so that only what is new
    /// raises the banner. `None` before the first read.
    waiting: Option<HashSet<(u64, Wait)>>,
    /// Counts terminal switches: each one replays the slide.
    switches: usize,
    focus: FocusHandle,
    /// Wakes the refresh loop before its next tick.
    refresh_now: futures::channel::mpsc::UnboundedSender<()>,
}

struct Gallery {
    filter: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Wait {
    Decision,
    Unread,
}

/// Something new an agent waits on from the user.
#[derive(Clone)]
struct Need {
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

const CRITICAL: u32 = 0xc72e0f;
const DECISION: u32 = 0xcc6d00;
const UNREAD: u32 = 0x3b8eea;

/// What a set of tickets asks of the user.
#[derive(Default)]
struct Alerts {
    decisions: usize,
    unread: usize,
    critical: bool,
}

impl Alerts {
    fn of<'a>(tickets: impl Iterator<Item = &'a TicketRow>, critical: Option<u64>) -> Self {
        let mut alerts = Self::default();
        for ticket in tickets {
            alerts.decisions += ticket.pending_decision as usize;
            alerts.unread += ticket.unread as usize;
            alerts.critical |= critical == Some(ticket.id);
        }
        alerts
    }

    /// One dot per kind of thing waiting, most pressing first.
    fn colors(&self) -> Vec<u32> {
        let mut colors = Vec::new();
        if self.critical {
            colors.push(CRITICAL);
        }
        if self.decisions > 0 {
            colors.push(DECISION);
        }
        if self.unread > 0 {
            colors.push(UNREAD);
        }
        colors
    }

    /// The most pressing thing waiting, if any.
    fn color(&self) -> Option<u32> {
        self.colors().first().copied()
    }

    fn badges(&self) -> impl IntoElement + use<> {
        div()
            .flex()
            .items_center()
            .gap_1()
            .when(self.critical, |d| d.child(pill("!", CRITICAL)))
            .when(self.decisions > 0, |d| d.child(pill(self.decisions.to_string(), DECISION)))
            .when(self.unread > 0, |d| d.child(pill(self.unread.to_string(), UNREAD)))
    }
}

impl Shell {
    pub fn new(selected: Option<String>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let aiball = Aiball::from_env();
        let panel = cx.new(|cx| TicketPanel::new(aiball.clone(), window, cx));
        cx.subscribe(&panel, |shell, _, _: &BoardChanged, _| {
            let _ = shell.refresh_now.unbounded_send(());
        })
        .detach();
        cx.subscribe(&panel, |shell, _, _: &CollapsePanel, cx| shell.toggle_panel(cx))
            .detach();

        let (refresh_now, wake) = futures::channel::mpsc::unbounded::<()>();
        let feed = events::Feed::start(refresh_now.clone());
        Self::refresh_loop(aiball.clone(), feed, wake, cx);

        let mut shell = Self {
            aiball,
            board: Board::default(),
            terminals: HashMap::new(),
            selected: None,
            recent: Vec::new(),
            panel,
            settings: Settings::load(),
            resizing: false,
            slider: None,
            slider_shown: 0,
            gallery: None,
            thumbnails: HashMap::new(),
            capturing: false,
            needs: Vec::new(),
            waiting: None,
            switches: 0,
            focus: cx.focus_handle(),
            refresh_now,
        };
        match selected {
            Some(session) => shell.select(session, window, cx),
            None => window.focus(&shell.focus.clone(), cx),
        }
        shell
    }

    /// Reads the board when aiball's feed says it moved (or every few
    /// seconds while the feed is down), and when the tmux sessions change.
    fn refresh_loop(
        mut reader: Aiball,
        feed: events::Feed,
        mut wake: futures::channel::mpsc::UnboundedReceiver<()>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |this, cx| {
            use futures::{FutureExt as _, StreamExt as _};
            let mut known = Vec::new();
            let mut read_at = Instant::now();
            let mut full = true;
            loop {
                if full {
                    let (next, board, now) = cx
                        .background_executor()
                        .spawn(async move {
                            let now = sessions::tmux_sessions();
                            let board = sessions::discover(&mut reader);
                            (reader, board, now)
                        })
                        .await;
                    reader = next;
                    known = now;
                    read_at = Instant::now();
                    let aiball = reader.clone();
                    if this
                        .update(cx, |shell, cx| shell.set_board(aiball, board, cx))
                        .is_err()
                    {
                        break;
                    }
                }
                let timer = cx.background_executor().timer(SESSIONS_EVERY).fuse();
                futures::pin_mut!(timer);
                let woken = futures::select! {
                    _ = timer => false,
                    _ = wake.next() => true,
                };
                // A burst of events is one read.
                while wake.try_recv().is_ok() {}
                let since = read_at.elapsed();
                full = woken || since >= REREAD_EVERY || (!feed.connected() && since >= POLL_EVERY);
                if !full {
                    let now = cx
                        .background_executor()
                        .spawn(async { sessions::tmux_sessions() })
                        .await;
                    full = now != known;
                }
            }
        })
        .detach();
    }

    fn set_board(&mut self, aiball: Aiball, board: Board, cx: &mut Context<Self>) {
        self.aiball = aiball;
        if self.board != board {
            self.board = board;
            self.update_needs();
            cx.notify();
        }
        self.sync_panel(cx);
    }

    /// Raises the banner for what newly waits on the user: a decision or
    /// something unread on a ticket an agent with a terminal holds.
    fn update_needs(&mut self) {
        let mut now = HashSet::new();
        let mut found = Vec::new();
        for project in &self.board.projects {
            for terminal in &project.terminals {
                let Some(agent) = &terminal.agent else { continue };
                let tickets = self.board.tickets.get(&project.name).into_iter().flatten();
                for ticket in tickets.filter(|t| t.holder() == Some(agent.as_str())) {
                    // One entry per ticket: a decision says more than "new".
                    let wait = if ticket.pending_decision {
                        Wait::Decision
                    } else if ticket.unread {
                        Wait::Unread
                    } else {
                        continue;
                    };
                    now.insert((ticket.id, wait));
                    found.push(Need {
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
                    self.needs.retain(|n| n.ticket != need.ticket);
                    self.needs.push(need);
                }
            }
        }
        // What no longer waits (decided, read) leaves the banner.
        self.needs.retain(|n| now.contains(&(n.ticket, n.wait)));
        self.waiting = Some(now);
    }

    /// Goes to what the banner shows: the agent's terminal, the panel open
    /// on the ticket.
    fn answer_need(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(need) = self.needs.pop() else { return };
        self.select(need.session.clone(), window, cx);
        if !self.settings.panel_open {
            self.toggle_panel(cx);
        }
        self.panel.update(cx, |panel, cx| panel.open(need.ticket, cx));
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
        let terminal = self
            .terminals
            .entry(session.clone())
            .or_insert_with(|| {
                cx.new(|cx| TerminalView::tmux(&session, cx).expect("failed to spawn the terminal"))
            })
            .clone();
        let focus = terminal.read(cx).focus_handle().clone();
        window.focus(&focus, cx);
        if self.selected.as_deref() != Some(session.as_str()) {
            self.switches += 1;
        }
        self.needs.retain(|n| n.session != session);
        self.recent.retain(|s| *s != session);
        self.recent.insert(0, session.clone());
        self.selected = Some(session);
        self.sync_panel(cx);
        cx.notify();
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
        self.settings.sidebar_open = !self.settings.sidebar_open;
        self.settings.save();
        cx.notify();
    }

    fn toggle_panel(&mut self, cx: &mut Context<Self>) {
        self.settings.panel_open = !self.settings.panel_open;
        self.settings.save();
        cx.notify();
    }

    fn panel_width(&self, window: &Window) -> f32 {
        let total = f32::from(window.viewport_size().width);
        let left = if self.settings.sidebar_open { SIDEBAR_WIDTH } else { FOLDED_WIDTH };
        let max = (total - left - 2. * EDGE_WIDTH - CENTER_MIN).max(PANEL_MIN);
        self.settings
            .panel_width
            .unwrap_or(total / 3.)
            .clamp(PANEL_MIN, max)
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.resizing {
            return;
        }
        let total = f32::from(window.viewport_size().width);
        self.settings.panel_width = Some(total - f32::from(event.position.x));
        cx.notify();
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.resizing {
            self.resizing = false;
            // Store what is shown, not an out-of-range drag.
            self.settings.panel_width = Some(self.panel_width(window));
            self.settings.save();
            cx.notify();
        }
    }

    // ── Keys: the slider and the gallery ────────────────────────────────

    /// Before the terminal sees a key: the window's own shortcuts.
    fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        let m = &keystroke.modifiers;
        let key = keystroke.key.as_str();
        if m.control && key == "enter" && !self.needs.is_empty() {
            self.answer_need(window, cx);
        } else if m.control && key == "tab" {
            self.step_slider(if m.shift { -1 } else { 1 }, cx);
        } else if m.control && m.shift && key == "space" {
            self.toggle_gallery(window, cx);
        } else if m.control && m.shift && key == "t" {
            self.toggle_panel(cx);
        } else if m.control && m.shift && key == "b" {
            self.toggle_sidebar(cx);
        } else if key == "escape" && (self.slider.is_some() || self.gallery.is_some()) {
            self.slider = None;
            self.close_gallery(window, cx);
        } else if let Some(gallery) = self.gallery.as_mut() {
            // The gallery is up: keys filter it.
            match key {
                "backspace" => {
                    gallery.filter.pop();
                }
                "enter" => {
                    if let Some(session) = self.gallery_terminals().first().map(|t| t.session.clone()) {
                        self.gallery = None;
                        self.select(session, window, cx);
                    }
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
        if let Some(index) = self.slider
            && !event.modifiers.control
        {
            self.slider = None;
            if let Some(session) = self.slider_order().get(index).cloned() {
                self.select(session, window, cx);
            }
            cx.notify();
        }
    }

    /// The slider's order: the terminals used lately, most recent first,
    /// then the others as the list shows them.
    fn slider_order(&self) -> Vec<String> {
        let mut order = self.recent.clone();
        for project in &self.board.projects {
            for terminal in &project.terminals {
                if !order.contains(&terminal.session) {
                    order.push(terminal.session.clone());
                }
            }
        }
        order
    }

    fn step_slider(&mut self, step: isize, cx: &mut Context<Self>) {
        log::debug!("slider step {step} from {:?}", self.slider);
        let len = self.slider_order().len() as isize;
        if len == 0 {
            return;
        }
        if self.slider.is_none() {
            self.slider_shown += 1;
        }
        // From the current terminal (index 0), the first tap goes to the
        // previous one: the quick hop between two.
        let index = self.slider.map_or(0, |i| i as isize);
        self.slider = Some((index + step).rem_euclid(len) as usize);
        self.capture_thumbnails(cx);
        cx.notify();
    }

    fn toggle_gallery(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.gallery.is_some() {
            self.close_gallery(window, cx);
            return;
        }
        self.gallery = Some(Gallery {
            filter: String::new(),
        });
        // Keys go to the window while the gallery is up.
        window.focus(&self.focus.clone(), cx);
        self.capture_thumbnails(cx);
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

    /// Captures every terminal's screen with `tmux capture-pane`, again and
    /// again while the gallery or the slider is up. Read-only: unlike
    /// attaching, it does not resize the session.
    fn capture_thumbnails(&mut self, cx: &mut Context<Self>) {
        if self.capturing {
            return;
        }
        self.capturing = true;
        cx.spawn(async move |this, cx| {
            loop {
                let Ok(sessions) = this.update(cx, |shell, _| {
                    let shown = shell.gallery.is_some() || shell.slider.is_some();
                    if !shown {
                        shell.capturing = false;
                    }
                    shown.then(|| {
                        shell
                            .board
                            .projects
                            .iter()
                            .flat_map(|p| p.terminals.iter().map(|t| t.session.clone()))
                            .collect::<Vec<_>>()
                    })
                }) else {
                    break;
                };
                let Some(sessions) = sessions else { break };
                let captures = cx
                    .background_executor()
                    .spawn(async move {
                        sessions
                            .into_iter()
                            .filter_map(|s| sessions::capture(&s).map(|c| (s, c)))
                            .collect::<Vec<_>>()
                    })
                    .await;
                let alive = this.update(cx, |shell, cx| {
                    for (session, capture) in captures {
                        shell
                            .thumbnails
                            .entry(session)
                            .or_insert_with(Snapshot::new)
                            .load(capture.columns, capture.lines, &capture.text);
                    }
                    cx.notify();
                    true
                });
                if !matches!(alive, Ok(true)) {
                    break;
                }
                cx.background_executor().timer(THUMBNAILS_EVERY).await;
            }
        })
        .detach();
    }

    /// The terminals the gallery shows, by its filter (label or project).
    fn gallery_terminals(&self) -> Vec<&Terminal> {
        let filter = self
            .gallery
            .as_ref()
            .map(|g| g.filter.to_lowercase())
            .unwrap_or_default();
        self.board
            .projects
            .iter()
            .flat_map(|p| p.terminals.iter().map(move |t| (p, t)))
            .filter(|(p, t)| {
                filter.is_empty()
                    || t.label.to_lowercase().contains(&filter)
                    || p.name.to_lowercase().contains(&filter)
            })
            .map(|(_, t)| t)
            .collect()
    }

    // ── Rendering ───────────────────────────────────────────────────────

    /// The strip between a side and the terminal: a grip that folds the
    /// side away; on the right, the rest of it drags the panel's width.
    fn edge(&self, side: Side, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let resizing = self.resizing;
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
            .bg(rgb(0x1e1e1e))
            .when(side == Side::Right, |d| {
                d.cursor(CursorStyle::ResizeColumn)
                    .when(resizing, |d| d.bg(rgb(0x2a3a4d)))
                    .hover(|d| d.bg(rgb(0x2a3a4d)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|shell, _, _, cx| {
                            shell.resizing = true;
                            cx.notify();
                        }),
                    )
            })
            .child(
                div()
                    .id(match side {
                        Side::Left => "sidebar-grip",
                        Side::Right => "panel-grip",
                    })
                    .w(px(6.))
                    .h(px(44.))
                    .rounded_full()
                    .bg(rgb(0x4a4a4a))
                    .cursor_pointer()
                    .hover(|d| d.bg(rgb(0x3b8eea)))
                    // A press on the grip is a toggle, not the start of a drag.
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(cx.listener(move |shell, _, _, cx| match side {
                        Side::Left => shell.toggle_sidebar(cx),
                        Side::Right => shell.toggle_panel(cx),
                    })),
            )
    }

    /// A folded side: 10 pixels, a dot per thing waiting, a click unfolds.
    fn folded(&self, side: Side, dots: Vec<u32>, cx: &mut Context<Self>) -> impl IntoElement + use<> {
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
            .bg(rgb(0x252526))
            .map(|d| match side {
                Side::Left => d.border_r_1(),
                Side::Right => d.border_l_1(),
            })
            .border_color(rgb(0x3c3c3c))
            .cursor_pointer()
            .hover(|d| d.bg(rgb(0x2a3a4d)))
            .children(
                dots.into_iter()
                    .map(|color| div().flex_none().size(px(6.)).rounded_full().bg(rgb(color))),
            )
            .on_click(cx.listener(move |shell, _, _, cx| match side {
                Side::Left => shell.toggle_sidebar(cx),
                Side::Right => shell.toggle_panel(cx),
            }))
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let mut list = div()
            .id("projects")
            .flex()
            .flex_col()
            .w(px(SIDEBAR_WIDTH))
            .flex_none()
            .h_full()
            .py_2()
            .overflow_y_scroll()
            .bg(rgb(0x252526))
            .text_sm();
        if self.board.projects.is_empty() {
            list = list.child(div().px_3().text_color(rgb(0x808080)).child("No session"));
        }
        for project in &self.board.projects {
            let tickets = self.board.tickets.get(&project.name);
            let critical = self.board.critical.get(&project.name).copied();
            // A project shows the sum of what it asks, collapsed or not.
            let alerts = Alerts::of(tickets.into_iter().flatten(), critical);
            list = list.child(
                div()
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
                            .text_color(rgb(0x9d9d9d))
                            .child(project.name.to_uppercase()),
                    )
                    .child(alerts.badges()),
            );
            for terminal in &project.terminals {
                let session = terminal.session.clone();
                let selected = self.selected.as_deref() == Some(session.as_str());
                let open = self.terminals.contains_key(&session);
                let alerts = self.alerts_of(&project.name, terminal.agent.as_deref());
                list = list.child(
                    div()
                        .id(SharedString::from(format!("terminal-{session}")))
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_1()
                        .cursor_pointer()
                        .when(selected, |d| d.bg(rgb(0x37373d)).text_color(rgb(0xffffff)))
                        .when(!selected, |d| d.hover(|d| d.bg(rgb(0x2a2d2e))))
                        // Green: the terminal already runs in tvty.
                        .child(div().w(px(7.)).when(open, |d| d.child(dot(0x23d18b))))
                        .child(div().flex_1().min_w_0().truncate().child(terminal.label.clone()))
                        .child(alerts.badges())
                        .on_click(cx.listener(move |shell, _, window, cx| {
                            shell.select(session.clone(), window, cx)
                        })),
                );
            }
        }
        list
    }

    /// The slider as a portfolio: the window fades behind, and the groups of
    /// terminals come forward one after another, the chosen card enlarged.
    fn portfolio(&self, index: usize) -> impl IntoElement + use<> {
        let order = self.slider_order();
        let chosen = order.get(index).cloned();
        let chosen_project = chosen
            .as_deref()
            .and_then(|s| self.terminal_of(s))
            .map(|(p, _)| p.to_string());
        // The groups, the chosen one's first; within a group, the slider's order.
        let mut groups: Vec<(&str, Vec<&Terminal>)> = Vec::new();
        for session in &order {
            let Some((project, terminal)) = self.terminal_of(session) else {
                continue;
            };
            match groups.iter_mut().find(|(p, _)| *p == project) {
                Some((_, terminals)) => terminals.push(terminal),
                None => groups.push((project, vec![terminal])),
            }
        }
        if let Some(chosen) = &chosen_project {
            if let Some(at) = groups.iter().position(|(p, _)| p == chosen) {
                let group = groups.remove(at);
                groups.insert(0, group);
            }
        }

        let mut body = div()
            .flex()
            .flex_wrap()
            .content_start()
            .justify_center()
            .gap_x_8()
            .gap_y_5()
            .px_6()
            .pt_6()
            .flex_1()
            .min_h_0()
            .overflow_hidden();
        let count = groups.len().max(1) as f32;
        for (i, (project, terminals)) in groups.into_iter().enumerate() {
            let is_chosen_group = chosen_project.as_deref() == Some(project);
            let mut cards = div().flex().flex_wrap().gap_3();
            for terminal in terminals {
                let chosen = chosen.as_deref() == Some(terminal.session.as_str());
                let (width, height) = if chosen { (340., 205.) } else { (230., 138.) };
                cards = cards.child(self.card(project, terminal, chosen, width, height));
            }
            // Staggered entrance: each group starts a little after the one before.
            let delay = 0.35 * i as f32 / count;
            let group = div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_chosen_group { rgb(0xffffff) } else { rgb(0x8a8a8a) })
                        .child(project.to_uppercase()),
                )
                .child(cards)
                .with_animation(
                    SharedString::from(format!("portfolio-{}-{project}", self.slider_shown)),
                    Animation::new(Duration::from_millis(420)).with_easing(move |t| {
                        let t = ((t - delay) / (1. - delay)).clamp(0., 1.);
                        1. - (1. - t).powi(3)
                    }),
                    |group, t| group.opacity(t).mt(px(40. * (1. - t))),
                );
            body = body.child(group);
        }

        div()
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            // The frosted glass, for now without the frost: GPUI cannot blur
            // what lies under an element.
            .bg(rgba(0x0b0b0ce8))
            .child(body)
            .child(
                div()
                    .flex()
                    .justify_center()
                    .py_3()
                    .text_sm()
                    .text_color(rgb(0x8a8a8a))
                    .child("tab: next · shift+tab: back · release ctrl to open · esc cancels"),
            )
            .with_animation(
                SharedString::from(format!("portfolio-veil-{}", self.slider_shown)),
                Animation::new(Duration::from_millis(180)),
                |veil, t| veil.opacity(t),
            )
    }

    /// The newest thing an agent waits on, over the top of the terminal.
    fn banner(&self, cx: &mut Context<Self>) -> Option<impl IntoElement + use<>> {
        let need = self.needs.last()?;
        let more = self.needs.len() - 1;
        let what = match need.wait {
            Wait::Decision => "a decision waits on",
            Wait::Unread => "something new on",
        };
        let color = match need.wait {
            Wait::Decision => DECISION,
            Wait::Unread => UNREAD,
        };
        Some(
            div()
                .id(SharedString::from(format!("banner-{}-{:?}", need.ticket, need.wait as u8)))
                .absolute()
                .top_2()
                .left_4()
                .right_4()
                .flex()
                .items_center()
                .gap_3()
                .px_3()
                .py_2()
                .rounded_md()
                .bg(rgb(0x1f2a36))
                .border_1()
                .border_color(rgb(color))
                .shadow_lg()
                .text_sm()
                .cursor_pointer()
                .child(dot(color))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .child(format!("{} — {what} #{} {}", need.agent, need.ticket, need.title)),
                )
                .when(more > 0, |d| d.child(div().text_color(rgb(0x8a8a8a)).child(format!("+{more}"))))
                .child(div().text_xs().text_color(rgb(0x8a8a8a)).child("ctrl+enter"))
                .child(
                    div()
                        .id("banner-close")
                        .px_1()
                        .text_color(rgb(0x8a8a8a))
                        .hover(|d| d.text_color(rgb(0xffffff)))
                        .child("×")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            cx.stop_propagation();
                            shell.needs.pop();
                            cx.notify();
                        })),
                )
                .on_click(cx.listener(|shell, _, window, cx| shell.answer_need(window, cx)))
                .with_animation(
                    SharedString::from(format!("banner-in-{}-{}", need.ticket, need.wait as u8)),
                    Animation::new(Duration::from_millis(220)).with_easing(|t| 1. - (1. - t).powi(3)),
                    |banner, t| banner.opacity(t).top(px(8. - 16. * (1. - t))),
                ),
        )
    }

    /// A terminal as a card: its name and alerts over a thumbnail.
    fn card(
        &self,
        project: &str,
        terminal: &Terminal,
        chosen: bool,
        width: f32,
        height: f32,
    ) -> Stateful<Div> {
        let session = &terminal.session;
        let alerts = self.alerts_of(project, terminal.agent.as_deref());
        div()
            .id(SharedString::from(format!("card-{session}")))
            .flex()
            .flex_col()
            .w(px(width))
            .rounded_md()
            .overflow_hidden()
            .border_2()
            .border_color(if chosen { rgb(0x3b8eea) } else { rgb(0x3c3c3c) })
            .when(chosen, |d| d.shadow_lg())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .bg(if chosen { rgb(0x2a3a4d) } else { rgb(0x2d2d2d) })
                    .text_sm()
                    .child(div().flex_1().min_w_0().truncate().child(terminal.label.clone()))
                    .child(alerts.badges()),
            )
            .child(
                div()
                    .h(px(height))
                    .overflow_hidden()
                    .bg(rgb(0x1e1e1e))
                    .when_some(self.thumbnails.get(session), |d, snapshot| d.child(snapshot.element())),
            )
    }

    fn gallery_view(&self, gallery: &Gallery, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let shown: Vec<&Terminal> = self.gallery_terminals();
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
            .min_h_0()
            .overflow_y_scroll();
        for project in &self.board.projects {
            let terminals: Vec<&Terminal> = project
                .terminals
                .iter()
                .filter(|t| shown.iter().any(|s| s.session == t.session))
                .collect();
            if terminals.is_empty() {
                continue;
            }
            let mut row = div().flex().flex_wrap().gap_3();
            for terminal in terminals {
                let session = terminal.session.clone();
                let selected = self.selected.as_deref() == Some(session.as_str());
                row = row.child(
                    self.card(&project.name, terminal, selected, 320., 190.)
                        .cursor_pointer()
                        .hover(|d| d.border_color(rgb(0x6b9fd6)))
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
                            .text_color(rgb(0x9d9d9d))
                            .child(project.name.to_uppercase()),
                    )
                    .child(row),
            );
        }
        div()
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            .bg(rgb(0x161617))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(rgb(0x3c3c3c))
                    .child(div().font_weight(FontWeight::BOLD).child("All terminals"))
                    .child(
                        div()
                            .flex_1()
                            .text_color(if gallery.filter.is_empty() { rgb(0x6b6b6b) } else { rgb(0xffffff) })
                            .child(if gallery.filter.is_empty() {
                                "type to filter · enter opens the first · esc closes".to_string()
                            } else {
                                format!("{}▏", gallery.filter)
                            }),
                    ),
            )
            .child(body)
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let center = match self.selected.as_ref().and_then(|s| self.terminals.get(s)) {
            // The terminal slides in on every switch. A relative offset, not a
            // margin: the terminal keeps its size, so tmux is not resized.
            Some(terminal) => div()
                .relative()
                .size_full()
                .child(terminal.clone())
                .with_animation(
                    SharedString::from(format!("switch-{}", self.switches)),
                    Animation::new(Duration::from_millis(240)).with_easing(|t| 1. - (1. - t).powi(3)),
                    |d, t| d.left(px(60. * (1. - t))).opacity(0.25 + 0.75 * t),
                )
                .into_any_element(),
            None => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x808080))
                .child("Pick a terminal on the left · ctrl+shift+space shows them all")
                .into_any_element(),
        };
        let banner = self.banner(cx);
        let left = if self.settings.sidebar_open {
            div()
                .flex()
                .flex_none()
                .h_full()
                .child(self.sidebar(cx))
                .child(self.edge(Side::Left, cx))
                .into_any_element()
        } else {
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
            self.folded(Side::Left, dots, cx).into_any_element()
        };
        let right = if self.settings.panel_open {
            div()
                .flex()
                .flex_none()
                .w(px(self.panel_width(window) + EDGE_WIDTH))
                .h_full()
                .child(self.edge(Side::Right, cx))
                .child(div().flex_1().min_w_0().h_full().child(self.panel.clone()))
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
        let slider = self.slider.map(|index| self.portfolio(index));
        let gallery = self.gallery.as_ref().map(|g| self.gallery_view(g, cx));

        div()
            .id("shell")
            .track_focus(&self.focus)
            .capture_key_down(cx.listener(Self::on_key))
            .on_modifiers_changed(cx.listener(Self::on_modifiers))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .relative()
            .flex()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xd4d4d4))
            .when(self.resizing, |d| d.cursor(CursorStyle::ResizeColumn))
            .child(left)
            .child(
                div()
                    .relative()
                    .flex_1()
                    .h_full()
                    .min_w_0()
                    .child(center)
                    .children(banner),
            )
            .child(right)
            .children(slider)
            .children(gallery)
    }
}
