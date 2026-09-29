//! tvty — Terminal Velocity: a native terminal for working with many AI
//! agents, their terminals grouped by project and their tickets beside.

mod accordion;
mod activity;
mod attach;
mod aiball;
mod bulk;
mod bus;
mod daemon;
mod emoji;
mod field;
mod focusmode;
mod fonts;
mod composer;
mod config;
mod fulllist;
mod icons;
mod images;
mod instance;
mod keymap;
mod loops;
mod newticket;
mod notify;
mod options;
mod panel;
mod live;
mod pings;
mod rowstate;
mod sessions;
mod settings;
mod shell;
mod sidecol;
mod stats;
mod status;
mod terminal;
mod thread;
mod theme;
mod tip;
mod tips;
mod ui;
mod updates;
mod wheel;
mod wire;

use gpui_kit::*;

fn main() {
    // tvty may be started from inside tmux; its terminals attach to tmux
    // sessions of their own, which tmux refuses while $TMUX is set.
    // SAFETY: no other thread exists yet.
    unsafe { std::env::remove_var("TMUX") };
    // `tvty --version`: said and done — for the updater, and for a human.
    if std::env::args().nth(1).as_deref().is_some_and(|a| a == "--version" || a == "-V") {
        println!("tvty {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    // `tvty [SESSION]`: open that tmux session at start.
    let selected = std::env::args().nth(1);
    // One tvty per state directory: a later launch brings the running one
    // forward (with the session asked for), and ends here — before its log
    // is opened, which would set the running one's aside.
    let raises = match instance::claim(selected.as_deref()) {
        instance::Claim::Handed => return,
        instance::Claim::First(raises) => Some(raises),
        instance::Claim::Alone => None,
    };
    // Quiet by default: Vulkan's loader warns about every driver it probes
    // and skips (other vendors' GPUs), which is not tvty's business. tvty's
    // own lines at info: the bus's connections, the pings, the
    // notifications — what tells a missed notification apart afterwards.
    // Written to stderr and to ~/.local/state/tvty/tvty.log (the previous
    // run's in tvty.log.1).
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
        "warn,tvty=info,wgpu_hal=error,gpui_component::theme::mono_font=error",
    ))
    .target(env_logger::Target::Pipe(Box::new(config::log())))
    .init();

    // The kit's icons (the title bar's buttons among them) come from its assets.
    gpui_kit::application().with_assets(icons::Assets).run(move |cx| {
        gpui_kit::init(cx);
        cx.set_global(instance::Raises(raises));
        // The user's own fonts (a colour emoji font), before any text is laid out.
        fonts::load(cx);
        // The sets of settings (settings.toml, the layout, the workspace).
        settings::init(cx);
        tips::init(cx);
        // Whether the pointer gives the keyboard, as the window manager does.
        focusmode::start(cx);
        // The full pages' side column, as last dragged.
        cx.set_global(sidecol::SideWidth(config::get::<settings::Layout>(cx).fields_width));
        let prefs = config::get::<settings::Preferences>(cx).clone();
        bus::init(cx);
        notify::init(
            cx,
            prefs.notifications.max.unwrap_or(notify::MAX_DEFAULT),
            prefs.notifications.seconds.unwrap_or(notify::SECONDS_DEFAULT),
        );
        activity::init(cx, prefs.notifications.own);
        // Whether a newer tvty is out, aiball recent enough: said once.
        updates::init(|cx| config::get::<settings::Preferences>(cx).updates.check, cx);
        wheel::set_speed(prefs.scroll.speed);
        theme::init(prefs.appearance.theme.as_deref(), prefs.appearance.terminal_theme.as_deref(), cx);
        theme::set_window_font(prefs.appearance.window_font_size, cx);
        keymap::init(cx);
        terminal::set_font_size(prefs.appearance.terminal_font_size.unwrap_or(terminal::FONT_SIZE_DEFAULT));
        terminal::set_opacity(prefs.appearance.terminal_opacity);
        // The window as it was left: maximized, full screen, or windowed at
        // its size (a compositor places it).
        let bounds = config::get::<settings::Layout>(cx).window.clone().map(|w| {
            let at = Bounds::centered(None, size(px(w.width.max(480.)), px(w.height.max(320.))), cx);
            match w.state.as_str() {
                "maximized" => WindowBounds::Maximized(at),
                "fullscreen" => WindowBounds::Fullscreen(at),
                _ => WindowBounds::Windowed(at),
            }
        });
        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some(shell::NAME.into()),
                        ..gpui_kit::component::TitleBar::title_bar_options()
                    }),
                    // Draw our own frame everywhere: GNOME would draw none, and
                    // the same frame on GNOME, KDE and the rest looks the same.
                    window_decorations: Some(WindowDecorations::Client),
                    // What ties the window to its launcher (tvty.desktop), and
                    // so to its icon in the dock.
                    app_id: Some("tvty".into()),
                    window_bounds: bounds,
                    ..gpui_kit::component::TitleBar::window_options()
                },
                |window, cx| {
                    let shell = cx.new(|cx| shell::Shell::new(selected, window, cx));
                    // The shell draws again only when it changes: a terminal's
                    // output or a hover in the panel redraws just that view.
                    let view = cx.new(|_| Frame(shell));
                    // Root draws the window's frame; no shadow around it: where
                    // the compositor does not show it as transparent, it reads
                    // as a wide dark border.
                    // Its background is the shell's to paint: see-through
                    // where the terminals are, when asked.
                    cx.new(|cx| {
                        gpui_kit::component::Root::new(view, window, cx).window_shadow_size(px(0.)).bg(transparent_black())
                    })
                },
            )
            .expect("failed to open the window");
            cx.update(settings::report_errors);
        })
        .detach();
    });
}

/// The window's content: the shell, and the probe that times each frame.
struct Frame(Entity<shell::Shell>);

impl Render for Frame {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // Every frame passes here: the probe times the whole frame, cached
        // parts included (TVTY_STATS).
        stats::window_started();
        div()
            .relative()
            .size_full()
            // Not cached: a cached view that draws again makes every view
            // inside it draw again (GPUI's `refreshing`). Drawn plainly, the
            // shell's own render is cheap, and the cached views inside it —
            // the terminal, the panel — are reused unless they changed.
            .child(self.0.clone())
            .child(div().absolute().child(stats::Probe))
    }
}
