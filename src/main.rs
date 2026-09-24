//! tvty — Terminal Velocity. The window: projects and their terminals on the
//! left, the selected terminal in the middle, the ticket panel on the right
//! (see docs/UX.md).

mod aiball;
mod panel;
mod sessions;
mod stats;
mod terminal;

use std::collections::HashMap;
use std::time::Duration;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use aiball::{Aiball, TicketRow};
use panel::{BoardChanged, Scope, TicketPanel, dot, pill};
use sessions::{Board, Terminal};
use terminal::TerminalView;

/// How often the terminals and the tickets are read again.
const REFRESH_EVERY: Duration = Duration::from_secs(5);

struct Shell {
    aiball: Aiball,
    board: Board,
    /// Terminals opened so far, by tmux session: they stay alive when hidden.
    terminals: HashMap<String, Entity<TerminalView>>,
    selected: Option<String>,
    panel: Entity<TicketPanel>,
    /// Wakes the refresh loop before its next tick.
    refresh_now: futures::channel::mpsc::UnboundedSender<()>,
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

    fn badges(&self) -> impl IntoElement {
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
    fn new(selected: Option<String>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let aiball = Aiball::from_env();
        let panel = cx.new(|cx| TicketPanel::new(aiball.clone(), window, cx));
        cx.subscribe(&panel, |shell, _, _: &BoardChanged, _| {
            let _ = shell.refresh_now.unbounded_send(());
        })
        .detach();

        let (refresh_now, mut wake) = futures::channel::mpsc::unbounded::<()>();
        let mut reader = aiball.clone();
        cx.spawn(async move |this, cx| {
            use futures::{FutureExt as _, StreamExt as _};
            loop {
                let (next, board) = cx
                    .background_executor()
                    .spawn(async move {
                        let board = sessions::discover(&mut reader);
                        (reader, board)
                    })
                    .await;
                reader = next;
                let aiball = reader.clone();
                if this
                    .update(cx, |shell, cx| shell.set_board(aiball, board, cx))
                    .is_err()
                {
                    break;
                }
                let timer = cx.background_executor().timer(REFRESH_EVERY).fuse();
                futures::pin_mut!(timer);
                futures::select! {
                    _ = timer => {}
                    _ = wake.next() => {}
                }
            }
        })
        .detach();

        let mut shell = Self {
            aiball,
            board: Board::default(),
            terminals: HashMap::new(),
            selected: None,
            panel,
            refresh_now,
        };
        if let Some(session) = selected {
            shell.select(session, window, cx);
        }
        shell
    }

    fn set_board(&mut self, aiball: Aiball, board: Board, cx: &mut Context<Self>) {
        self.aiball = aiball;
        if self.board != board {
            self.board = board;
            cx.notify();
        }
        self.sync_panel(cx);
    }

    fn selected_terminal(&self) -> Option<(&str, &Terminal)> {
        let selected = self.selected.as_deref()?;
        self.board.projects.iter().find_map(|p| {
            p.terminals
                .iter()
                .find(|t| t.session == selected)
                .map(|t| (p.name.as_str(), t))
        })
    }

    /// Points the panel at the selected terminal's project and agent.
    fn sync_panel(&mut self, cx: &mut Context<Self>) {
        let on_board = |name: &str| self.board.projects.iter().any(|p| p.name == name && p.on_board);
        let scope = self
            .selected_terminal()
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
        self.selected = Some(session);
        self.sync_panel(cx);
        cx.notify();
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut list = div()
            .id("projects")
            .flex()
            .flex_col()
            .w(px(220.))
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
                let alerts = match &terminal.agent {
                    Some(agent) => Alerts::of(
                        tickets
                            .into_iter()
                            .flatten()
                            .filter(|t| t.holder() == Some(agent.as_str())),
                        critical,
                    ),
                    None => Alerts::default(),
                };
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
}

impl Render for Shell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let center = match self.selected.as_ref().and_then(|s| self.terminals.get(s)) {
            Some(terminal) => div().size_full().child(terminal.clone()),
            None => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0x808080))
                .child("Pick a terminal on the left"),
        };
        div()
            .flex()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xd4d4d4))
            .child(self.sidebar(cx))
            .child(div().flex_1().h_full().min_w_0().child(center))
            .child(
                div()
                    .w(relative(1. / 3.))
                    .h_full()
                    .border_l_1()
                    .border_color(rgb(0x3c3c3c))
                    .child(self.panel.clone()),
            )
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    // tvty may be started from inside tmux; its terminals attach to tmux
    // sessions of their own, which tmux refuses while $TMUX is set.
    // SAFETY: no other thread exists yet.
    unsafe { std::env::remove_var("TMUX") };
    // `tvty [SESSION]`: open that tmux session at start.
    let selected = std::env::args().nth(1);

    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);
        gpui_kit::component::Theme::change(gpui_kit::component::ThemeMode::Dark, None, cx);
        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("tvty".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| Shell::new(selected, window, cx));
                    cx.new(|cx| gpui_kit::component::Root::new(view, window, cx))
                },
            )
            .expect("failed to open the window");
        })
        .detach();
    });
}
