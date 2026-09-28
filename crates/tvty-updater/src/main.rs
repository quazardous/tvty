//! Terminal Velocity Updater: one small window whose only job is to install
//! and update Terminal Velocity and aiball — what each has, what the latest
//! release is, a button, and what is being done, line by line.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Disableable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use tvty_updater::{State, Status};

const NAME: &str = "Terminal Velocity Updater";

/// A program the updater looks after.
#[derive(Clone, Copy, PartialEq)]
enum Program {
    Tvty,
    Aiball,
}

impl Program {
    fn title(self) -> &'static str {
        match self {
            Program::Tvty => "Terminal Velocity",
            Program::Aiball => "aiball",
        }
    }

    fn about(self) -> &'static str {
        match self {
            Program::Tvty => "the terminal for your AI agents",
            Program::Aiball => "the engine: its daemon, the board, the loops",
        }
    }

    /// The oldest version that will do.
    fn min(self) -> Option<&'static str> {
        match self {
            Program::Tvty => None,
            Program::Aiball => Some(tvty_updater::MIN_AIBALL),
        }
    }
}

/// A gesture of the updater.
#[derive(Clone, Copy, PartialEq)]
enum Gesture {
    Install(Program),
    Update(Program),
    Rollback,
    All,
}

struct Updater {
    tvty: Option<Status>,
    aiball: Option<Status>,
    /// What is being done, and what was said.
    busy: Option<&'static str>,
    log: Arc<Mutex<Vec<String>>>,
    failed: bool,
    scroll: ScrollHandle,
}

impl Updater {
    fn new(cx: &mut Context<Self>) -> Self {
        let mut this = Self { tvty: None, aiball: None, busy: None, log: Arc::default(), failed: false, scroll: ScrollHandle::new() };
        this.check(cx);
        this
    }

    /// Asks both programs where they stand, off the window's thread.
    fn check(&mut self, cx: &mut Context<Self>) {
        self.tvty = None;
        self.aiball = None;
        cx.spawn(async move |this, cx| {
            let (tvty, aiball) = cx.background_executor().spawn(async { (tvty_updater::tvty_status(), tvty_updater::aiball_status()) }).await;
            let _ = this.update(cx, |updater, cx| {
                updater.tvty = Some(tvty);
                updater.aiball = Some(aiball);
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn status(&self, program: Program) -> Option<&Status> {
        match program {
            Program::Tvty => self.tvty.as_ref(),
            Program::Aiball => self.aiball.as_ref(),
        }
    }

    /// Runs a gesture off the window's thread, its lines shown as they come;
    /// both programs asked again once it is done.
    fn run(&mut self, gesture: Gesture, cx: &mut Context<Self>) {
        if self.busy.is_some() {
            return;
        }
        self.busy = Some(match gesture {
            Gesture::Install(_) => "Installing…",
            Gesture::Update(_) | Gesture::All => "Updating…",
            Gesture::Rollback => "Going back…",
        });
        self.failed = false;
        let (aiball_state, tvty_state) = (self.state(Program::Aiball), self.state(Program::Tvty));
        let log = self.log.clone();
        // How it ended, once it has: read by the loop below.
        let outcome: Arc<Mutex<Option<bool>>> = Arc::default();
        let ended = outcome.clone();
        cx.background_executor().spawn(async move {
            let mut say = |line: String| log.lock().expect("the log").push(line);
            let aiball = |say: &mut dyn FnMut(String)| match aiball_state {
                Some(State::Missing) => tvty_updater::install_aiball(say),
                _ => tvty_updater::update_aiball(say),
            };
            let result = match gesture {
                Gesture::Install(Program::Aiball) | Gesture::Update(Program::Aiball) => aiball(&mut say),
                Gesture::Install(Program::Tvty) | Gesture::Update(Program::Tvty) => tvty_updater::update_tvty(&mut say),
                Gesture::Rollback => tvty_updater::rollback_tvty(&mut say),
                Gesture::All => {
                    let first = if matches!(aiball_state, Some(State::UpToDate)) { Ok(()) } else { aiball(&mut say) };
                    first.and_then(|()| {
                        if matches!(tvty_state, Some(State::UpToDate)) { Ok(()) } else { tvty_updater::update_tvty(&mut say) }
                    })
                }
            };
            if let Err(error) = &result {
                say(format!("✗ {error:#}"));
            }
            *ended.lock().expect("the outcome") = Some(result.is_ok());
        })
        .detach();
        // The lines as they come, while it runs.
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(150)).await;
                let finished = *outcome.lock().expect("the outcome");
                let alive = this.update(cx, |updater, cx| {
                    updater.scroll.scroll_to_bottom();
                    if let Some(ok) = finished {
                        updater.busy = None;
                        updater.failed = !ok;
                        updater.log.lock().expect("the log").push(if ok { "✓ done".into() } else { "stopped".into() });
                        updater.check(cx);
                    }
                    cx.notify();
                });
                if alive.is_err() || finished.is_some() {
                    break;
                }
            }
        })
        .detach();
        cx.notify();
    }

    fn state(&self, program: Program) -> Option<State> {
        self.status(program).map(|s| s.state(program.min()))
    }

    fn row(&self, program: Program, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let status = self.status(program).cloned();
        let state = self.state(program);
        let (said, colour) = match state {
            None => ("checking…", theme.muted_foreground),
            Some(State::Missing) => ("not installed", theme.warning),
            Some(State::TooOld) => ("too old for Terminal Velocity", theme.danger),
            Some(State::UpdateAvailable) => ("update available", theme.warning),
            Some(State::UpToDate) => ("up to date", theme.success),
            Some(State::Unknown) => ("latest release not known", theme.muted_foreground),
        };
        let versions = status.as_ref().map(|s| {
            let mut parts = Vec::new();
            if let Some(v) = &s.installed {
                parts.push(format!("installed {v}"));
            }
            if let Some(v) = s.running.as_ref().filter(|r| Some(*r) != s.installed.as_ref()) {
                parts.push(format!("running {v}"));
            }
            if let Some(v) = &s.latest {
                parts.push(format!("latest {v}"));
            }
            parts.join(" · ")
        });
        let note = status.as_ref().and_then(|s| s.note.clone());
        let busy = self.busy.is_some();
        let action = match state {
            Some(State::Missing) => Some(("Install", Gesture::Install(program))),
            Some(State::TooOld | State::UpdateAvailable) => Some(("Update", Gesture::Update(program))),
            // Without a latest release known, the update may still be asked.
            Some(State::Unknown) => Some(("Update", Gesture::Update(program))),
            _ => None,
        };
        let rollback = program == Program::Tvty && tvty_updater::previous_dir().join("tvty").exists();
        div()
            .flex()
            .items_center()
            .gap_4()
            .p_4()
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().flex().items_baseline().gap_2().child(div().text_lg().font_weight(FontWeight::BOLD).child(program.title())).child(div().text_sm().text_color(theme.muted_foreground).child(program.about())))
                    .child(div().text_sm().text_color(colour).child(said))
                    .children(versions.filter(|v| !v.is_empty()).map(|v| div().text_sm().text_color(theme.muted_foreground).child(v)))
                    .children(note.map(|n| div().text_xs().text_color(theme.muted_foreground).child(n))),
            )
            .when(rollback, |d| {
                d.child(
                    Button::new(SharedString::from(format!("{}-back", program.title())))
                        .label("Go back")
                        .disabled(busy)
                        .on_click(cx.listener(|updater, _, _, cx| updater.run(Gesture::Rollback, cx))),
                )
            })
            .children(action.map(|(label, gesture)| {
                Button::new(SharedString::from(format!("{}-go", program.title())))
                    .primary()
                    .label(label)
                    .disabled(busy)
                    .on_click(cx.listener(move |updater, _, _, cx| updater.run(gesture, cx)))
            }))
    }
}

impl Render for Updater {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let lines = self.log.lock().expect("the log").clone();
        let busy = self.busy;
        let anything = [Program::Aiball, Program::Tvty]
            .iter()
            .any(|p| matches!(self.state(*p), Some(State::Missing | State::TooOld | State::UpdateAvailable)));
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(gpui_kit::component::TitleBar::new().child(div().text_sm().child(NAME)))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .child(self.row(Program::Aiball, cx))
                    .child(self.row(Program::Tvty, cx))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(div().flex_1().text_sm().text_color(if self.failed { theme.danger } else { theme.muted_foreground }).child(match (busy, self.failed) {
                                (Some(doing), _) => doing.to_string(),
                                (None, true) => "It did not go through: what was said is below.".to_string(),
                                (None, false) => "aiball first, then Terminal Velocity: each updates its own way.".to_string(),
                            }))
                            .child(
                                Button::new("check-again")
                                    .label("Check again")
                                    .disabled(busy.is_some())
                                    .on_click(cx.listener(|updater, _, _, cx| updater.check(cx))),
                            )
                            .child(
                                Button::new("all")
                                    .primary()
                                    .label("Update all")
                                    .disabled(busy.is_some() || !anything)
                                    .on_click(cx.listener(|updater, _, _, cx| updater.run(Gesture::All, cx))),
                            ),
                    )
                    .child(
                        div()
                            .id("log")
                            .flex_1()
                            .min_h(px(120.))
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            .p_2()
                            .rounded_md()
                            .border_1()
                            .border_color(theme.border)
                            .font_family("monospace")
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .children(if lines.is_empty() {
                                vec![div().child("What is done shows here.")]
                            } else {
                                lines.into_iter().map(|l| div().child(l)).collect()
                            }),
                    ),
            )
    }
}

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        let bounds = Bounds::centered(None, size(px(760.), px(560.)), cx);
        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions { title: Some(NAME.into()), ..gpui_kit::component::TitleBar::title_bar_options() }),
                    window_decorations: Some(WindowDecorations::Client),
                    app_id: Some("tvty-updater".into()),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..gpui_kit::component::TitleBar::window_options()
                },
                |window, cx| {
                    let updater = cx.new(Updater::new);
                    cx.new(|cx| gpui_kit::component::Root::new(updater, window, cx))
                },
            )
            .expect("failed to open the window");
        })
        .detach();
    });
}
