//! tvty — Terminal Velocity: a native terminal for working with many AI
//! agents, their terminals grouped by project and their tickets beside.

mod aiball;
mod events;
mod panel;
mod sessions;
mod settings;
mod shell;
mod stats;
mod status;
mod terminal;

use gpui_kit::*;

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
                    let view = cx.new(|cx| shell::Shell::new(selected, window, cx));
                    cx.new(|cx| gpui_kit::component::Root::new(view, window, cx))
                },
            )
            .expect("failed to open the window");
        })
        .detach();
    });
}
