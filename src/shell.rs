//! The window: projects and their terminals on the left, the selected
//! terminal in the middle, the ticket panel on the right (resizable,
//! collapsible), and two ways to switch — the slider (ctrl+tab, most recent
//! first) and the gallery (ctrl+shift+space, every terminal as a thumbnail).
//! See docs/UX.md.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use gpui_kit::component::{TitleBar, window_paddings};

use crate::aiball::{Aiball, TicketRow};
use crate::theme::{self, p};
use gpui_kit::component::scroll::ScrollableElement as _;
use crate::events;
use crate::options::{SHORTCUTS, Section};
use crate::panel::{BoardChanged, CollapsePanel, Scope, TicketPanel, dot, pill};
use crate::sessions::{self, Board, Terminal};
use crate::settings::Settings;
use crate::terminal::{Snapshot, TerminalView};

/// How often the tmux sessions are listed (cheap: one `tmux ls`).
const SESSIONS_EVERY: Duration = Duration::from_secs(3);
/// How often the board is read when aiball's live feed is down.
const POLL_EVERY: Duration = Duration::from_secs(5);
/// The shortest time between two reads of the board: bursts of events are
/// read at once, and aiball — one request at a time — is not flooded.
const READ_GAP: Duration = Duration::from_secs(1);
/// How often the board is read anyway, feed or not.
const REREAD_EVERY: Duration = Duration::from_secs(60);
/// How often the gallery's thumbnails are captured while it is open.
const THUMBNAILS_EVERY: Duration = Duration::from_millis(1500);

const SIDEBAR_WIDTH: f32 = 240.;
const SIDEBAR_MIN: f32 = 180.;
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
    /// A side's edge is being dragged.
    resizing: Option<Side>,
    /// The edge has moved since it was pressed.
    dragged: bool,
    /// The options page is open, on this section.
    options: Option<Section>,
    feed: events::Feed,
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
    /// The theme list is open, under the title bar.
    theme_menu: bool,

    focus: FocusHandle,
    /// Wakes the refresh loop before its next tick.
    refresh_now: futures::channel::mpsc::UnboundedSender<events::Change>,
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
struct Alerts {
    decisions: usize,
    unread: usize,
    critical: bool,
}

impl Alerts {
    fn of<'a>(tickets: impl Iterator<Item = &'a TicketRow>, critical: Option<u64>) -> Self {
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

    fn badges(&self) -> impl IntoElement + use<> {
        div()
            .flex()
            .items_center()
            .gap_1()
            .when(self.critical, |d| d.child(pill("!", critical())))
            .when(self.decisions > 0, |d| d.child(pill(self.decisions.to_string(), decision())))
            .when(self.unread > 0, |d| d.child(pill(self.unread.to_string(), unread())))
    }
}

impl Shell {
    pub fn new(selected: Option<String>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let aiball = Aiball::from_env();
        let panel = cx.new(|cx| TicketPanel::new(aiball.clone(), window, cx));
        cx.subscribe(&panel, |shell, _, _: &BoardChanged, _| {
            // A gesture moved the selected terminal's project.
            let change = shell
                .selected
                .as_deref()
                .and_then(|s| shell.terminal_of(s))
                .map(|(project, _)| events::Change::Project(project.to_string()))
                .unwrap_or(events::Change::All);
            let _ = shell.refresh_now.unbounded_send(change);
        })
        .detach();
        cx.subscribe(&panel, |shell, _, _: &CollapsePanel, cx| shell.toggle_panel(cx))
            .detach();

        let (refresh_now, wake) = futures::channel::mpsc::unbounded::<events::Change>();
        let feed = events::Feed::start(refresh_now.clone());
        Self::refresh_loop(aiball.clone(), feed.clone(), wake, cx);

        let mut shell = Self {
            aiball,
            board: Board::default(),
            terminals: HashMap::new(),
            selected: None,
            recent: Vec::new(),
            panel,
            settings: Settings::load(),
            resizing: None,
            dragged: false,
            options: None,
            feed: feed.clone(),
            slider: None,
            slider_shown: 0,
            gallery: None,
            thumbnails: HashMap::new(),
            capturing: false,
            needs: Vec::new(),
            waiting: None,
            switches: 0,
            theme_menu: false,

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
        mut wake: futures::channel::mpsc::UnboundedReceiver<events::Change>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |this, cx| {
            use futures::{FutureExt as _, StreamExt as _};
            let mut known = Vec::new();
            let mut board = Board::default();
            let mut read_at = Instant::now();
            let mut full_at = Instant::now();
            let mut changes = sessions::Changes {
                all: true,
                ..Default::default()
            };
            loop {
                if !changes.is_empty() {
                    // Bursts come in one read, at most one read a second.
                    let wait = READ_GAP.saturating_sub(read_at.elapsed());
                    if !changes.all && !wait.is_zero() {
                        cx.background_executor().timer(wait).await;
                        while let Ok(change) = wake.try_recv() {
                            add(&mut changes, change);
                        }
                    }
                    let all = changes.all;
                    let taken = std::mem::take(&mut changes);
                    let previous = board.clone();
                    let (next, read, now) = cx
                        .background_executor()
                        .spawn(async move {
                            let now = sessions::tmux_sessions();
                            let read = sessions::update(&mut reader, &previous, &taken);
                            (reader, read, now)
                        })
                        .await;
                    reader = next;
                    board = read;
                    known = now;
                    read_at = Instant::now();
                    if all {
                        full_at = read_at;
                    }
                    let (aiball, shown) = (reader.clone(), board.clone());
                    if this
                        .update(cx, |shell, cx| shell.set_board(aiball, shown, cx))
                        .is_err()
                    {
                        break;
                    }
                }
                // Redraw now and then: the states say for how long.
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
                let timer = cx.background_executor().timer(SESSIONS_EVERY).fuse();
                futures::pin_mut!(timer);
                futures::select! {
                    _ = timer => {}
                    change = wake.next() => {
                        if let Some(change) = change {
                            add(&mut changes, change);
                        }
                    }
                };
                while let Ok(change) = wake.try_recv() {
                    add(&mut changes, change);
                }
                let since = full_at.elapsed();
                if since >= REREAD_EVERY || (!feed.connected() && since >= POLL_EVERY) {
                    changes.all = true;
                }
                if changes.is_empty() {
                    let now = cx
                        .background_executor()
                        .spawn(async { sessions::tmux_sessions() })
                        .await;
                    // New or gone sessions: group the terminals again.
                    changes.consumers = now != known;
                }
            }
        })
        .detach();

        fn add(changes: &mut sessions::Changes, change: events::Change) {
            match change {
                events::Change::All => changes.all = true,
                events::Change::Consumers => changes.consumers = true,
                events::Change::Project(project) => {
                    changes.projects.insert(project);
                }
            }
        }
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
                    let wait = if ticket.pending_decision() {
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

    fn inner_width(window: &Window) -> f32 {
        let paddings = window_paddings(window);
        f32::from(window.viewport_size().width - paddings.left - paddings.right)
    }

    /// The projects' list lies over the terminal: it only needs to leave
    /// some of it visible.
    fn sidebar_width(&self, window: &Window) -> f32 {
        let max = (Self::inner_width(window) * 0.6).max(SIDEBAR_MIN);
        self.settings
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
                self.settings.sidebar_width =
                    Some(x - f32::from(paddings.left) - FOLDED_WIDTH - EDGE_WIDTH / 2.)
            }
            // The panel ends where the frame's content does.
            Some(Side::Right) => {
                let right = f32::from(window.viewport_size().width - paddings.right);
                self.settings.panel_width = Some(right - x);
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
            self.settings.panel_width = Some(self.panel_width(window));
            self.settings.sidebar_width = Some(self.sidebar_width(window));
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
        } else if m.control && !m.shift && key == "," {
            self.toggle_options(window, cx);
        } else if key == "escape" && self.options.is_some() {
            self.toggle_options(window, cx);
        } else if m.control && m.shift && key == "k" {
            self.next_theme(window, cx);
        } else if key == "escape" && self.theme_menu {
            self.theme_menu = false;
            cx.notify();
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

    // ── Options ─────────────────────────────────────────────────────────

    fn toggle_options(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
                        shell.options = Some(item);
                        cx.notify();
                    })),
            );
        }
        let content = match section {
            Section::Appearance => self.options_appearance(cx).into_any_element(),
            Section::Layout => self.options_layout(window, cx).into_any_element(),
            Section::TicketList => options_ticket_list().into_any_element(),
            Section::Shortcuts => options_shortcuts().into_any_element(),
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
                    .id("options-content")
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

    fn options_appearance(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let window_theme = theme::current(cx);
        let terminal_theme = theme::current_terminal();
        let column = || div().flex().flex_col().gap_1().flex_1().min_w_0();
        let window_list = self.theme_choices(
            column().child(option_group("Window")),
            "window",
            Some(window_theme),
            None,
            cx,
            |shell, name, window, cx| {
                if let Some(name) = name {
                    shell.set_theme(name, window, cx);
                }
            },
        );
        let terminal_list = self.theme_choices(
            column().child(option_group("Terminal")),
            "terminal",
            terminal_theme,
            Some("Same as the window"),
            cx,
            |shell, name, _, cx| shell.set_terminal_theme(name, cx),
        );
        div()
            .flex()
            .flex_col()
            .gap_1()
            .max_w(px(720.))
            .child(option_note(
                "The window's colour theme, and the terminals': the window's, or one of their own — a dark terminal in a light window. Ctrl+Shift+K steps through the window's.",
            ))
            .child(div().flex().gap_6().child(window_list).child(terminal_list))
            .child(option_note(
            "Your own themes (gpui-component's theme format) go in ~/.config/tvty/themes/: they show here the next time this page opens.",
        ))
    }

    /// The themes to pick from, dark ones first, appended to `page`; with a
    /// `default` entry first when there may be no theme (`None`).
    fn theme_choices(
        &self,
        mut page: Div,
        id: &'static str,
        chosen: Option<SharedString>,
        default: Option<&'static str>,
        cx: &mut Context<Self>,
        pick: fn(&mut Self, Option<SharedString>, &mut Window, &mut Context<Self>),
    ) -> Div {
        let choice = |key: SharedString, label: SharedString, value: Option<SharedString>, on: bool, cx: &mut Context<Self>| {
            div()
                .id(SharedString::from(format!("options-{id}-theme-{key}")))
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
                .on_click(cx.listener(move |shell, _, window, cx| {
                    pick(shell, value.clone(), window, cx);
                    shell.options = Some(Section::Appearance);
                }))
        };
        if let Some(label) = default {
            page = page.child(choice("default".into(), label.into(), None, chosen.is_none(), cx));
        }
        let mut last_dark = None;
        for (name, dark) in theme::names(cx) {
            if last_dark != Some(dark) {
                last_dark = Some(dark);
                page = page.child(
                    div()
                        .px_3()
                        .pt_2()
                        .text_xs()
                        .text_color(p().muted)
                        .child(if dark { "Dark" } else { "Light" }),
                );
            }
            let on = chosen.as_ref() == Some(&name);
            page = page.child(choice(name.clone(), name.clone(), Some(name), on, cx));
        }
        page
    }

    fn options_layout(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sidebar = format!(
            "{} · {} px",
            if self.settings.sidebar_open { "open" } else { "folded" },
            self.sidebar_width(window).round()
        );
        let panel = format!(
            "{} · {} px",
            if self.settings.panel_open { "open" } else { "folded" },
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
                "Back to the defaults: a list of 240 px, a panel a third of the window.",
                String::new(),
                "Reset",
                cx.listener(|shell, _, _, cx| {
                    shell.settings.sidebar_width = None;
                    shell.settings.panel_width = None;
                    shell.settings.save();
                    cx.notify();
                }),
            ))
    }

    fn options_about(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))
            .map(|d| d.join("tvty").display().to_string())
            .unwrap_or_default();
        let rows = [
            ("Version", env!("CARGO_PKG_VERSION").to_string()),
            ("aiball socket", crate::aiball::socket_path().display().to_string()),
            ("Acting as", self.aiball.user.clone()),
            (
                "Live feed",
                if self.feed.connected() { "connected" } else { "down — the board is polled" }.to_string(),
            ),
            ("Theme", theme::current(cx).to_string()),
            (
                "Terminal theme",
                theme::current_terminal().map_or("the window's".to_string(), |name| name.to_string()),
            ),
            ("Settings and themes", config),
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

    // ── Themes ──────────────────────────────────────────────────────────

    fn set_theme(&mut self, name: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        theme::apply(&name, Some(window), cx);
        self.settings.theme = Some(name.to_string());
        self.settings.save();
        self.theme_menu = false;
        cx.notify();
    }

    fn set_terminal_theme(&mut self, name: Option<SharedString>, cx: &mut Context<Self>) {
        theme::apply_terminal(name.as_deref(), cx);
        self.settings.terminal_theme = name.map(|n| n.to_string());
        self.settings.save();
        cx.notify();
    }

    fn next_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let names = theme::names(cx);
        let current = theme::current(cx);
        let at = names.iter().position(|(n, _)| *n == current).map_or(0, |i| i + 1);
        if let Some((name, _)) = names.get(at % names.len().max(1)).cloned() {
            self.set_theme(name, window, cx);
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
                    .on_click(cx.listener(move |shell, _, window, cx| {
                        shell.set_theme(pick.clone(), window, cx)
                    })),
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

    /// A folded side: 10 pixels, a dot per thing waiting, a click unfolds.
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

    fn sidebar(&self, width: f32, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let mut list = div()
            .id("projects")
            .flex()
            .flex_col()
            .w(px(width))
            .flex_none()
            .h_full()
            .py_2()
            .bg(p().surface)
            .text_sm();
        if self.board.projects.is_empty() {
            list = list.child(div().px_3().text_color(p().muted).child("No session"));
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
                            .text_color(p().muted)
                            .child(project.name.to_uppercase()),
                    )
                    .child(alerts.badges()),
            );
            for terminal in &project.terminals {
                let session = terminal.session.clone();
                let selected = self.selected.as_deref() == Some(session.as_str());
                let open = self.terminals.contains_key(&session);
                let alerts = self.alerts_of(&project.name, terminal.agent.as_deref());
                let state = terminal.status.as_ref().and_then(|s| s.colour());
                list = list.child(
                    div()
                        .id(SharedString::from(format!("terminal-{session}")))
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
                                        .child(div().w(px(7.)).when(open, |d| d.child(dot(p().success))))
                                        .child(div().flex_1().min_w_0().truncate().child(terminal.label.clone()))
                                        .child(alerts.badges()),
                                )
                                .when_some(terminal.status.as_ref(), |d, status| d.child(status.line())),
                        )
                        .on_click(cx.listener(move |shell, _, window, cx| {
                            shell.select(session.clone(), window, cx)
                        })),
                );
            }
        }
        list.overflow_y_scrollbar()
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
                        .text_color(if is_chosen_group { p().text } else { p().muted })
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
            Wait::Decision => decision(),
            Wait::Unread => unread(),
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
                .bg(p().surface)
                .border_1()
                .border_color(color)
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
                .when(more > 0, |d| d.child(div().text_color(p().muted).child(format!("+{more}"))))
                .child(div().text_xs().text_color(p().muted).child("ctrl+enter"))
                .child(
                    div()
                        .id("banner-close")
                        .px_1()
                        .text_color(p().muted)
                        .hover(|d| d.text_color(p().text))
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
                    .child(alerts.badges()),
            )
            .when_some(terminal.status.as_ref(), |d, status| {
                d.child(div().px_2().pb_1().bg(p().surface).child(status.line()))
            })
            .child(
                div()
                    .h(px(height))
                    .overflow_hidden()
                    .bg(p().bg)
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
            .min_h_0();
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
                            .child(project.name.to_uppercase()),
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
                                "type to filter · enter opens the first · esc closes".to_string()
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
        // A drag released anywhere, even over the title bar: the edge's
        // drag is gone, the width is kept.
        if self.dragged && !cx.has_active_drag() {
            self.end_resize(window, cx);
        }
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
                .text_color(p().muted)
                .child("Pick a terminal on the left · ctrl+shift+space shows them all")
                .into_any_element(),
        };
        let banner = self.banner(cx);
        // The strip stays in the layout, open or folded, so the terminal
        // keeps its width; open, the projects' list lies over the terminal.
        let overlay = self.settings.sidebar_open.then(|| {
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
        let options = self.options.map(|section| self.options_view(section, window, cx));
        let gallery = self.gallery.as_ref().map(|g| self.gallery_view(g, cx));

        let title = match self.selected.as_deref().and_then(|s| self.terminal_of(s)) {
            Some((project, terminal)) => format!("tvty — {project} · {}", terminal.label),
            None => "tvty".to_string(),
        };
        let body = div()
            .id("shell")
            .track_focus(&self.focus)
            .capture_key_down(cx.listener(Self::on_key))
            .on_modifiers_changed(cx.listener(Self::on_modifiers))
            .on_drag_move(cx.listener(Self::on_drag_move))
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
                    .children(banner)
                    .children(overlay),
            )
            .child(right)
            .children(slider)
            .children(gallery)
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
                TitleBar::new()
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

fn options_shortcuts() -> impl IntoElement {
    let mut page = div().flex().flex_col().max_w(px(760.));
    for (group, shortcuts) in SHORTCUTS {
        page = page.child(option_group(group));
        for (keys, what) in *shortcuts {
            page = page.child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .py_1p5()
                    .border_b_1()
                    .border_color(p().border)
                    .child(
                        div().w(px(260.)).flex_none().child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap_1()
                                .children(keys.split(" · ").map(|k| {
                                    div()
                                        .px_1p5()
                                        .rounded_sm()
                                        .border_1()
                                        .border_color(p().border)
                                        .bg(p().surface)
                                        .text_sm()
                                        .child(k.to_string())
                                })),
                        ),
                    )
                    .child(div().flex_1().min_w_0().text_sm().child(*what)),
            );
        }
    }
    page
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
    let glyph = |g: Glyph, colour: Hsla| {
        div()
            .font_weight(FontWeight::BOLD)
            .text_color(colour)
            .child(g.symbol())
            .into_any_element()
    };
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
            "A prototype of the ticket list proposed on the board: computed in tvty from the same rows as aiball's web UI.",
        ))
        .child(option_group("Order"));
    for band in [Band::Moderate, Band::Decide, Band::Unread, Band::AgentOnIt, Band::Open] {
        let what = match band {
            Band::Moderate => "the ticket, or comments on it, wait for moderation",
            Band::Decide => "a plan, a resolution, a wontfix or an escalation waits for your decision",
            Band::Unread => "something new on it",
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
        .child(line(glyph(Glyph::Step, p().muted), Glyph::Step.meaning().into()))
        .child(line(glyph(Glyph::StalledStep, p().muted), Glyph::StalledStep.meaning().into()))
        .child(line(glyph(Glyph::Rejected, p().muted), Glyph::Rejected.meaning().into()))
        .child(line(glyph(Glyph::ClosedResolved, p().muted), Glyph::ClosedResolved.meaning().into()))
        .child(line(glyph(Glyph::Closed, p().muted), Glyph::Closed.meaning().into()))
        .child(option_group("Stripe — whose turn"))
        .child(line(stripe(p().warning, false), "a decision waits on you, and it is the last message".into()))
        .child(line(stripe(p().warning, true), "a decision waits on you, but the talk went on after it".into()))
        .child(line(stripe(p().border, false), "your turn: an agent answered you".into()))
        .child(line(div().into_any_element(), "no stripe: the ball is with the agent".into()))
        .child(option_group("The rest"))
        .child(line(
            div().font_weight(FontWeight::BOLD).child("Title").into_any_element(),
            "bold: unread — and nothing else says it".into(),
        ))
        .child(line(
            div().text_xs().text_color(p().muted).child("you · 3 msg").into_any_element(),
            "who spoke last, and how many messages".into(),
        ))
        .child(line(
            div().text_xs().text_color(p().muted).child("🔥 agent").into_any_element(),
            "the agent holding it; 🔥 when it was active lately".into(),
        ))
        .child(line(pill("⚠ 3", p().danger).into_any_element(), "the project's critical ticket: it holds 3 open tickets".into()));
    page
}
