//! The window: projects and their terminals on the left, the selected
//! terminal in the middle, the ticket panel on the right (resizable,
//! collapsible), and two ways to switch — the slider (ctrl+tab, most recent
//! first) and the gallery (ctrl+shift+space, every terminal as a thumbnail).
//! See docs/UX.md.

use std::collections::HashMap;
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
const COLLAPSED_WIDTH: f32 = 28.;

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
    /// The slider is up, on this index of `recent`.
    slider: Option<usize>,
    gallery: Option<Gallery>,
    focus: FocusHandle,
    /// Wakes the refresh loop before its next tick.
    refresh_now: futures::channel::mpsc::UnboundedSender<()>,
}

struct Gallery {
    filter: String,
    thumbnails: HashMap<String, Snapshot>,
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
            alerts.decisions += ticket.pending_decision as usize;
            alerts.unread += ticket.unread as usize;
            alerts.critical |= critical == Some(ticket.id);
        }
        alerts
    }

    fn badges(&self) -> impl IntoElement + use<> {
        div()
            .flex()
            .items_center()
            .gap_1()
            .when(self.critical, |d| d.child(pill("!", 0xc72e0f)))
            .when(self.decisions > 0, |d| d.child(pill(self.decisions.to_string(), 0xcc6d00)))
            .when(self.unread > 0, |d| d.child(pill(self.unread.to_string(), 0x3b8eea)))
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
            gallery: None,
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
            cx.notify();
        }
        self.sync_panel(cx);
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

    fn toggle_panel(&mut self, cx: &mut Context<Self>) {
        self.settings.panel_open = !self.settings.panel_open;
        self.settings.save();
        cx.notify();
    }

    fn panel_width(&self, window: &Window) -> f32 {
        let total = f32::from(window.viewport_size().width);
        let max = (total - SIDEBAR_WIDTH - CENTER_MIN).max(PANEL_MIN);
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
        if m.control && key == "tab" {
            self.step_slider(if m.shift { -1 } else { 1 }, cx);
        } else if m.control && m.shift && key == "space" {
            self.toggle_gallery(window, cx);
        } else if m.control && m.shift && key == "t" {
            self.toggle_panel(cx);
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
        if let Some(index) = self.slider
            && !event.modifiers.control
        {
            self.slider = None;
            if let Some(session) = self.recent.get(index).cloned() {
                self.select(session, window, cx);
            }
            cx.notify();
        }
    }

    fn step_slider(&mut self, step: isize, cx: &mut Context<Self>) {
        if self.recent.is_empty() {
            return;
        }
        let len = self.recent.len() as isize;
        // From the current terminal (index 0), the first tap goes to the
        // previous one: the quick hop between two.
        let index = self.slider.map_or(0, |i| i as isize);
        self.slider = Some((index + step).rem_euclid(len) as usize);
        cx.notify();
    }

    fn toggle_gallery(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.gallery.is_some() {
            self.close_gallery(window, cx);
            return;
        }
        self.gallery = Some(Gallery {
            filter: String::new(),
            thumbnails: HashMap::new(),
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
    /// again while the gallery is up. Read-only: unlike attaching, it does
    /// not resize the session.
    fn capture_thumbnails(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                let Ok(sessions) = this.update(cx, |shell, _| {
                    shell.gallery.as_ref().map(|_| {
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
                    let Some(gallery) = shell.gallery.as_mut() else {
                        return false;
                    };
                    for (session, capture) in captures {
                        gallery
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

    fn slider_strip(&self, index: usize) -> impl IntoElement + use<> {
        let mut strip = div()
            .absolute()
            .top_2()
            .left_2()
            .right_2()
            .flex()
            .gap_2()
            .p_2()
            .rounded_md()
            .bg(rgba(0x1b1b1cf0))
            .border_1()
            .border_color(rgb(0x3c3c3c))
            .shadow_lg();
        for (i, session) in self.recent.iter().enumerate() {
            let (project, label, alerts) = match self.terminal_of(session) {
                Some((project, terminal)) => (
                    project.to_string(),
                    terminal.label.clone(),
                    self.alerts_of(project, terminal.agent.as_deref()),
                ),
                None => (String::new(), session.clone(), Alerts::default()),
            };
            strip = strip.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .w(px(170.))
                    .flex_none()
                    .p_2()
                    .rounded_md()
                    .border_2()
                    .border_color(if i == index { rgb(0x3b8eea) } else { rgb(0x2d2d2d) })
                    .bg(if i == index { rgb(0x2a3a4d) } else { rgb(0x252526) })
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x9d9d9d))
                            .truncate()
                            .child(project.to_uppercase()),
                    )
                    .child(div().truncate().child(label))
                    .child(alerts.badges()),
            );
        }
        strip
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
                let alerts = self.alerts_of(&project.name, terminal.agent.as_deref());
                let selected = self.selected.as_deref() == Some(session.as_str());
                let thumbnail = gallery.thumbnails.get(&session);
                row = row.child(
                    div()
                        .id(SharedString::from(format!("thumb-{session}")))
                        .flex()
                        .flex_col()
                        .w(px(320.))
                        .rounded_md()
                        .overflow_hidden()
                        .border_2()
                        .border_color(if selected { rgb(0x3b8eea) } else { rgb(0x3c3c3c) })
                        .cursor_pointer()
                        .hover(|d| d.border_color(rgb(0x6b9fd6)))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_2()
                                .py_1()
                                .bg(rgb(0x2d2d2d))
                                .text_sm()
                                .child(div().flex_1().min_w_0().truncate().child(terminal.label.clone()))
                                .child(alerts.badges()),
                        )
                        .child(
                            div()
                                .h(px(190.))
                                .overflow_hidden()
                                .bg(rgb(0x1e1e1e))
                                .when_some(thumbnail, |d, snapshot| d.child(snapshot.element())),
                        )
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
            Some(terminal) => div().size_full().child(terminal.clone()),
            None => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x808080))
                .child("Pick a terminal on the left · ctrl+shift+space shows them all"),
        };
        let open = self.settings.panel_open;
        let panel_width = self.panel_width(window);
        let panel = if open {
            div()
                .flex()
                .flex_none()
                .w(px(panel_width))
                .h_full()
                // The edge to drag.
                .child(
                    div()
                        .id("panel-edge")
                        .w(px(5.))
                        .h_full()
                        .flex_none()
                        .cursor(CursorStyle::ResizeColumn)
                        .bg(if self.resizing { rgb(0x3b8eea) } else { rgb(0x2b2b2b) })
                        .hover(|d| d.bg(rgb(0x3b8eea)))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|shell, _, _, cx| {
                                shell.resizing = true;
                                cx.notify();
                            }),
                        ),
                )
                .child(div().flex_1().min_w_0().h_full().child(self.panel.clone()))
                .into_any_element()
        } else {
            // Collapsed: a strip that still says what waits.
            let alerts = self
                .selected
                .as_deref()
                .and_then(|s| self.terminal_of(s))
                .map(|(project, terminal)| self.alerts_of(project, terminal.agent.as_deref()))
                .unwrap_or_default();
            div()
                .id("panel-collapsed")
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .pt_2()
                .w(px(COLLAPSED_WIDTH))
                .flex_none()
                .h_full()
                .bg(rgb(0x252526))
                .border_l_1()
                .border_color(rgb(0x3c3c3c))
                .cursor_pointer()
                .hover(|d| d.bg(rgb(0x2a2d2e)))
                .child(div().text_color(rgb(0x3b8eea)).child("‹"))
                .when(alerts.decisions > 0, |d| d.child(pill(alerts.decisions.to_string(), 0xcc6d00)))
                .when(alerts.unread > 0, |d| d.child(pill(alerts.unread.to_string(), 0x3b8eea)))
                .on_click(cx.listener(|shell, _, _, cx| shell.toggle_panel(cx)))
                .into_any_element()
        };
        let slider = self.slider.map(|index| self.slider_strip(index));
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
            .child(self.sidebar(cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .h_full()
                    .min_w_0()
                    .child(center)
                    .children(slider),
            )
            .child(panel)
            .children(gallery)
    }
}
