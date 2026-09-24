//! tvty — Terminal Velocity. The window: projects and their terminals on the
//! left, the selected terminal in the middle, the ticket panel on the right
//! (see docs/UX.md).

mod sessions;
mod stats;
mod terminal;

use std::collections::HashMap;
use std::time::Duration;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sessions::Project;
use terminal::TerminalView;

/// How often the list of terminals is read again.
const DISCOVER_EVERY: Duration = Duration::from_secs(5);

struct Shell {
    projects: Vec<Project>,
    /// Terminals opened so far, by tmux session: they stay alive when hidden.
    terminals: HashMap<String, Entity<TerminalView>>,
    selected: Option<String>,
}

impl Shell {
    fn new(selected: Option<String>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                let projects = cx.background_executor().spawn(async { sessions::discover() }).await;
                let alive = this
                    .update(cx, |shell, cx| {
                        if shell.projects != projects {
                            shell.projects = projects;
                            cx.notify();
                        }
                    })
                    .is_ok();
                if !alive {
                    break;
                }
                cx.background_executor().timer(DISCOVER_EVERY).await;
            }
        })
        .detach();

        let mut shell = Self {
            projects: Vec::new(),
            terminals: HashMap::new(),
            selected: None,
        };
        if let Some(session) = selected {
            shell.select(session, window, cx);
        }
        shell
    }

    fn select(&mut self, session: String, window: &mut Window, cx: &mut Context<Self>) {
        let terminal = self
            .terminals
            .entry(session.clone())
            .or_insert_with(|| {
                cx.new(|cx| {
                    TerminalView::new("tmux", &["attach", "-t", &format!("={session}")], cx)
                        .expect("failed to spawn the terminal")
                })
            })
            .clone();
        let focus = terminal.read(cx).focus_handle().clone();
        window.focus(&focus, cx);
        self.selected = Some(session);
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
        if self.projects.is_empty() {
            list = list.child(div().px_3().text_color(rgb(0x808080)).child("No session"));
        }
        for project in &self.projects {
            list = list.child(
                div()
                    .px_3()
                    .pt_2()
                    .pb_1()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0x9d9d9d))
                    .child(project.name.to_uppercase()),
            );
            for terminal in &project.terminals {
                let session = terminal.session.clone();
                let selected = self.selected.as_deref() == Some(session.as_str());
                let open = self.terminals.contains_key(&session);
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
                        // A dot on the terminals already running in tvty.
                        .child(
                            div()
                                .size(px(6.))
                                .rounded_full()
                                .when(open, |d| d.bg(rgb(0x23d18b))),
                        )
                        .child(terminal.label.clone())
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
                    .flex()
                    .flex_col()
                    .w(relative(1. / 3.))
                    .h_full()
                    .p_2()
                    .bg(rgb(0x252526))
                    .child("Tickets"),
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
