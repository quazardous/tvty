//! tvty — Terminal Velocity: a native terminal for working with many AI
//! agents, their terminals grouped by project and their tickets beside.

mod aiball;
mod events;
mod options;
mod panel;
mod sessions;
mod settings;
mod shell;
mod stats;
mod status;
mod terminal;
mod theme;

use gpui_kit::*;

fn main() {
    // Quiet by default: Vulkan's loader warns about every driver it probes
    // and skips (other vendors' GPUs), which is not tvty's business.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
        "warn,wgpu_hal=error,gpui_component::theme::mono_font=error",
    ))
    .init();
    // tvty may be started from inside tmux; its terminals attach to tmux
    // sessions of their own, which tmux refuses while $TMUX is set.
    // SAFETY: no other thread exists yet.
    unsafe { std::env::remove_var("TMUX") };
    // `tvty [SESSION]`: open that tmux session at start.
    let selected = std::env::args().nth(1);

    // The kit's icons (the title bar's buttons among them) come from its assets.
    gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(move |cx| {
        gpui_kit::init(cx);
        theme::init(settings::Settings::load().theme.as_deref(), cx);
        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("tvty".into()),
                        ..gpui_kit::component::TitleBar::title_bar_options()
                    }),
                    // Draw our own frame everywhere: GNOME would draw none, and
                    // the same frame on GNOME, KDE and the rest looks the same.
                    window_decorations: Some(WindowDecorations::Client),
                    ..gpui_kit::component::TitleBar::window_options()
                },
                |window, cx| {
                    let view = cx.new(|cx| shell::Shell::new(selected, window, cx));
                    // Root draws the window's frame; no shadow around it: where
                    // the compositor does not show it as transparent, it reads
                    // as a wide dark border.
                    cx.new(|cx| gpui_kit::component::Root::new(view, window, cx).window_shadow_size(px(0.)))
                },
            )
            .expect("failed to open the window");
        })
        .detach();
    });
}
