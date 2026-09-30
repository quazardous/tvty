//! Terminal Velocity Updater: one small window whose only job is to install
//! and update Terminal Velocity and aiball — what each has, what the latest
//! release is, a button, and what is being done, line by line.

// On Windows a window program: no console window behind it; `--install`
// started from a console still writes there.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Disableable as _, Sizable as _};
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
    /// What is missing and installs by itself (Claude Code).
    Prerequisites,
}

struct Updater {
    icon: Arc<Image>,
    tvty: Option<Status>,
    aiball: Option<Status>,
    /// What the machine needs and does not have (Claude Code, tmux…).
    missing: Vec<&'static tvty_updater::prerequisites::Prerequisite>,
    /// What is being done, and what was said.
    busy: Option<&'static str>,
    log: Arc<Mutex<Vec<String>>>,
    failed: bool,
    scroll: ScrollHandle,
}

impl Updater {
    fn new(cx: &mut Context<Self>) -> Self {
        let icon = Arc::new(Image::from_bytes(ImageFormat::Svg, tvty_updater::ICON.to_vec()));
        let mut this = Self { icon, tvty: None, aiball: None, missing: Vec::new(), busy: None, log: Arc::default(), failed: false, scroll: ScrollHandle::new() };
        this.check(cx);
        this
    }

    /// Asks both programs where they stand, off the window's thread.
    fn check(&mut self, cx: &mut Context<Self>) {
        self.tvty = None;
        self.aiball = None;
        self.missing = tvty_updater::prerequisites::missing();
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
            Gesture::Install(_) | Gesture::Prerequisites => "Installing…",
            Gesture::Update(_) | Gesture::All => "Updating…",
            Gesture::Rollback => "Going back…",
        });
        self.failed = false;
        said(&self.log, format!("— {}", self.busy.unwrap_or_default()));
        let (aiball_state, tvty_state) = (self.state(Program::Aiball), self.state(Program::Tvty));
        let log = self.log.clone();
        // How it ended, once it has: read by the loop below.
        let outcome: Arc<Mutex<Option<bool>>> = Arc::default();
        let ended = outcome.clone();
        cx.background_executor().spawn(async move {
            let mut say = |line: String| said(&log, line);
            let aiball = |say: &mut dyn FnMut(String)| match aiball_state {
                // A development install: its own update refuses to run.
                Some(State::Dev) => {
                    say("aiball is a development install: left as it is, updated by hand".into());
                    Ok(())
                }
                // There but silent: its own update would not run either.
                Some(State::Missing | State::Silent) => tvty_updater::install_aiball(say),
                _ => tvty_updater::update_aiball(say),
            };
            let result = match gesture {
                Gesture::Install(Program::Aiball) | Gesture::Update(Program::Aiball) => aiball(&mut say),
                Gesture::Install(Program::Tvty) | Gesture::Update(Program::Tvty) => tvty_updater::update_tvty(&mut say),
                Gesture::Rollback => tvty_updater::rollback_tvty(&mut say),
                Gesture::Prerequisites => {
                    let still = tvty_updater::prerequisites::ensure(&mut say);
                    if still.is_empty() { Ok(()) } else { Err(anyhow::anyhow!("still missing: {}", still.join(", "))) }
                }
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
                        said(&updater.log, if ok { "✓ done".into() } else { "stopped".into() });
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

    /// What is missing on the machine, each with what it is for and the
    /// command that installs it, to copy; none missing: nothing shown.
    fn missing_view(&self, cx: &mut Context<Self>) -> Option<impl IntoElement + use<>> {
        if self.missing.is_empty() {
            return None;
        }
        let theme = cx.theme().clone();
        let mut list = div()
            .id("missing")
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .rounded_md()
            .border_1()
            .border_color(theme.warning)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().flex_1().font_weight(FontWeight::BOLD).child("Missing on this machine"))
                    .child(Button::new("missing-check").label("Check again").small().on_click(cx.listener(|updater, _, _, cx| updater.check(cx)))),
            );
        for p in &self.missing {
            let how = p.how_here();
            list = list.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap_2()
                            .child(div().font_weight(FontWeight::BOLD).child(p.command))
                            .child(div().text_sm().text_color(theme.muted_foreground).child(format!("for {}", p.purpose))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().flex_1().min_w_0().px_2().py_1().rounded_sm().bg(theme.muted).text_sm().font_family("monospace").child(how.clone()))
                            .child(Button::new(SharedString::from(format!("missing-copy-{}", p.command))).label("Copy").small().on_click(move |_, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(how.clone()));
                            }))
                            // Its own installer, no password: done from here.
                            .when(p.installable(), |d| {
                                d.child(
                                    Button::new(SharedString::from(format!("missing-install-{}", p.command)))
                                        .label("Install")
                                        .primary()
                                        .small()
                                        .disabled(self.busy.is_some())
                                        .on_click(cx.listener(|updater, _, _, cx| updater.run(Gesture::Prerequisites, cx))),
                                )
                            }),
                    ),
            );
        }
        Some(list)
    }

    fn row(&self, program: Program, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let status = self.status(program).cloned();
        let state = self.state(program);
        let (said, colour) = match state {
            None => ("checking…", theme.muted_foreground),
            Some(State::Missing) => ("not installed", theme.warning),
            Some(State::Silent) => ("installed, but it does not answer", theme.danger),
            Some(State::Dev) => ("a development install: updated by hand, not from here", theme.muted_foreground),
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
        // A development install: the command that updates it by hand, to copy.
        let by_hand = status.as_ref().filter(|s| s.dev).and_then(|s| s.update_command.clone());
        let busy = self.busy.is_some();
        let action = match state {
            Some(State::Missing) => Some(("Install", Gesture::Install(program))),
            // Its installer run again mends a half-made install.
            Some(State::Silent) => Some(("Reinstall", Gesture::Install(program))),
            Some(State::TooOld | State::UpdateAvailable) => Some(("Update", Gesture::Update(program))),
            // Without a latest release known, the update may still be asked.
            Some(State::Unknown) => Some(("Update", Gesture::Update(program))),
            _ => None,
        };
        let rollback = program == Program::Tvty && tvty_updater::has_previous();
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
                    .children(note.map(|n| div().text_xs().text_color(theme.muted_foreground).child(n)))
                    .children(by_hand.map(|command| {
                        let copied = command.clone();
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .pt_1()
                            // Where the line is to be pasted.
                            .child(div().flex_none().text_xs().text_color(theme.muted_foreground).child(if cfg!(windows) { "in PowerShell:" } else { "in a terminal:" }))
                            .child(div().flex_1().min_w_0().px_2().py_1().rounded_sm().bg(theme.muted).text_xs().font_family("monospace").child(command))
                            .child(Button::new(SharedString::from(format!("{}-by-hand", program.title()))).label("Copy").small().on_click(move |_, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                            }))
                    })),
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
            .any(|p| matches!(self.state(*p), Some(State::Missing | State::Silent | State::TooOld | State::UpdateAvailable)));
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(gpui_kit::component::TitleBar::new().child(div().text_sm().child(NAME)))
            .child(
                div()
                    // Taller than the window (something missing, a command
                    // to copy): it scrolls, the console never out of reach.
                    .id("content")
                    .overflow_y_scroll()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    // Who it is, and where the projects live.
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(img(self.icon.clone()).size(px(40.)).flex_none())
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .child(div().text_lg().font_weight(FontWeight::BOLD).child(NAME))
                                    .child(div().text_sm().text_color(theme.muted_foreground).child("Installs Terminal Velocity and aiball, and keeps them up to date."))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_wrap()
                                            .gap_4()
                                            .pt_1()
                                            .child(link("link-tvty", "Terminal Velocity on GitHub ↗", tvty_updater::TVTY_URL, &theme))
                                            .child(link("link-aiball", "aiball on GitHub ↗", tvty_updater::AIBALL_URL, &theme))
                                            .child(link("link-licence", "MIT licence ↗", &format!("{}/blob/main/LICENSE", tvty_updater::TVTY_URL), &theme)),
                                    )
                                    // Whose work it is, the years running to this one.
                                    .child(div().pt_1().text_xs().text_color(theme.muted_foreground).child(tvty_config::legal::copyright())),
                            ),
                    )
                    .children(self.missing_view(cx))
                    .child(self.row(Program::Aiball, cx))
                    .child(self.row(Program::Tvty, cx))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_3()
                            .child(div().flex_1().min_w(px(240.)).text_sm().text_color(if self.failed { theme.danger } else { theme.muted_foreground }).child(match (busy, self.failed) {
                                (Some(doing), _) => doing.to_string(),
                                (None, true) => "It did not go through: what was said is below.".to_string(),
                                (None, false) => "aiball first, then Terminal Velocity: each updates its own way.".to_string(),
                            }))
                            // Installed and done: it may start now.
                            .when(busy.is_none() && matches!(self.state(Program::Tvty), Some(State::UpToDate | State::Unknown)), |d| {
                                d.child(
                                    Button::new("launch")
                                        .label("Launch Terminal Velocity")
                                        .on_click(cx.listener(|updater, _, _, cx| {
                                            let line = match tvty_updater::launch_tvty() {
                                                Ok(()) => "Terminal Velocity started".to_string(),
                                                Err(error) => format!("✗ {error:#}"),
                                            };
                                            said(&updater.log, line);
                                            cx.notify();
                                        })),
                                )
                            })
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
                    // The console's own gestures: all of it to the clipboard
                    // (to paste in a message), the folder of the file that
                    // keeps it (to attach it whole).
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            // Cut when long: the buttons stay in the window.
                            .child(div().flex_1().min_w_0().truncate().text_xs().text_color(theme.muted_foreground).child(match tvty_updater::log_file() {
                                Some(file) => format!("kept in {}", short(&file)),
                                None => "not kept in a file here".to_string(),
                            }))
                            .child(Button::new("log-copy").label("Copy").small().disabled(lines.is_empty()).on_click({
                                // Windows' line ends: pasted in any of its programs, the lines stay lines.
                                let text = lines.join(if cfg!(windows) { "\r\n" } else { "\n" });
                                move |_, _, cx| cx.write_to_clipboard(ClipboardItem::new_string(text.clone()))
                            }))
                            .child(Button::new("log-folder").label("Open the log's folder").small().disabled(tvty_updater::log_file().is_none()).on_click(|_, _, cx| {
                                if let Some(file) = tvty_updater::log_file() {
                                    // Made now if nothing was said yet: a folder to open.
                                    if let Some(dir) = file.parent() {
                                        let _ = std::fs::create_dir_all(dir);
                                        if file.exists() { cx.reveal_path(&file) } else { cx.open_with_system(dir) }
                                    }
                                }
                            })),
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
                    )
                    .child(div().text_xs().text_color(theme.muted_foreground).child(
                        if cfg!(windows) {
                            "Beta software, provided as is, without warranty (MIT licence). Installing aiball sets its daemon up to \
                             start when you sign in (a scheduled task of your user). The updater asks GitHub for the latest releases, \
                             and nothing else leaves this machine."
                        } else {
                            "Beta software, provided as is, without warranty (MIT licence). Installing aiball sets its daemon up as a \
                             service of your user (systemd). The updater asks GitHub for the latest releases, and nothing else leaves \
                             this machine."
                        },
                    )),
            )
    }
}

/// A link to a page, opened in the browser.
/// A path as it reads: the home folder written `~`.
fn short(path: &std::path::Path) -> String {
    match tvty_config::home().and_then(|home| path.strip_prefix(home).ok().map(|rest| rest.to_path_buf())) {
        Some(rest) => format!("~{}{}", std::path::MAIN_SEPARATOR, rest.display()),
        None => path.display().to_string(),
    }
}

/// A line of the console: shown in the window, and kept in the log file.
fn said(log: &Arc<Mutex<Vec<String>>>, line: String) {
    tvty_updater::log_append(&line);
    // Shown as it is kept: what is copied from here may be pasted anywhere.
    log.lock().expect("the log").push(tvty_updater::plain(&line));
}

fn link(id: &'static str, label: &'static str, url: &str, theme: &gpui_kit::component::Theme) -> impl IntoElement {
    let url = url.to_string();
    div()
        .id(id)
        .flex_none()
        .text_sm()
        .text_color(theme.link)
        .cursor_pointer()
        .hover(|d| d.underline())
        .on_click(move |_, _, cx| cx.open_url(&url))
        .child(label)
}

fn main() {
    #[cfg(windows)]
    // SAFETY: a plain Win32 call; it fails harmlessly without a parent console.
    unsafe {
        windows_sys::Win32::System::Console::AttachConsole(windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS)
    };
    // Without a window, and installing nothing: what is missing on the
    // machine, each with how to get it; fails when something is.
    // The same, and what installs by itself is installed (Claude Code).
    if std::env::args().any(|a| a == "--prerequisites") {
        let missing = tvty_updater::prerequisites::ensure(&mut |line| println!("{line}"));
        if missing.is_empty() {
            println!("nothing is missing");
        }
        std::process::exit(if missing.is_empty() { 0 } else { 1 });
    }
    if std::env::args().any(|a| a == "--check") {
        let missing = tvty_updater::prerequisites::report(&mut |line| println!("{line}"));
        if missing.is_empty() {
            println!("nothing is missing");
        }
        std::process::exit(if missing.is_empty() { 0 } else { 1 });
    }
    // Without a window: what the Windows setup script runs, its lines on
    // standard output.
    if std::env::args().any(|a| a == "--install") {
        // Said on standard output, and kept in the updater's log as the
        // window's lines are.
        let result = tvty_updater::install_all(&mut |line| {
            println!("{line}");
            tvty_updater::log_append(&line);
        });
        if let Err(error) = &result {
            eprintln!("✗ {error:#}");
            tvty_updater::log_append(&format!("✗ {error:#}"));
        }
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }
    // The kit's icons: the title bar's window buttons are drawn with them.
    gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(|cx| {
        gpui_kit::init(cx);
        // Dark, as Terminal Velocity opens by default.
        gpui_kit::component::Theme::change(gpui_kit::component::ThemeMode::Dark, None, cx);
        let bounds = Bounds::centered(None, size(px(880.), px(700.)), cx);
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
