//! The title bar's menu, under the app's icon (F1): where to learn about tvty and to reach its
//! people — the version and the build, the documentation, the shortcuts,
//! what changed, the repository, a bug to report — and a restart.

use gpui_kit::*;

use super::Shell;
use crate::options::Section;
use crate::theme::p;

/// The repository, as Cargo.toml declares it.
pub(super) const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
/// aiball's, the engine tvty runs on.
pub(super) const AIBALL_REPOSITORY: &str = "https://github.com/quazardous/aiball";

/// What the menu offers.
#[derive(Clone, Copy)]
enum Entry {
    About,
    NewProject,
    FullScreen,
    Documentation,
    Shortcuts,
    WhatsNew,
    Repository,
    Issue,
    Aiball,
    Updates,
    Restart,
}

impl Entry {
    const ALL: [Entry; 11] = [
        Entry::About,
        Entry::NewProject,
        Entry::FullScreen,
        Entry::Documentation,
        Entry::Shortcuts,
        Entry::WhatsNew,
        Entry::Repository,
        Entry::Issue,
        Entry::Aiball,
        Entry::Updates,
        Entry::Restart,
    ];

    fn label(self) -> &'static str {
        match self {
            Entry::About => "About Terminal Velocity",
            Entry::NewProject => "New project…",
            Entry::FullScreen => "Full screen",
            Entry::Documentation => "Documentation",
            Entry::Shortcuts => "Keyboard shortcuts",
            Entry::WhatsNew => "What's new",
            Entry::Repository => "GitHub",
            Entry::Issue => "Report an issue",
            Entry::Aiball => "aiball",
            Entry::Updates => "Updates…",
            Entry::Restart => "Restart tvty",
        }
    }

    /// What it says beside its label.
    fn note(self, cx: &App) -> String {
        match self {
            // Its key, as the keymap in force has it.
            Entry::FullScreen => crate::keymap::current(cx).keys_of("window.fullscreen").first().map(|k| k.pretty()).unwrap_or_default(),
            Entry::About => version(),
            Entry::Documentation | Entry::WhatsNew | Entry::Repository | Entry::Issue => "↗".into(),
            Entry::Aiball => "the board ↗".into(),
            // A newer release, when one is known.
            Entry::Updates => crate::updates::newer(cx).map(|v| format!("{v} is out ↑")).unwrap_or_default(),
            Entry::Shortcuts | Entry::NewProject => String::new(),
            Entry::Restart => "sessions kept".into(),
        }
    }

    /// The page it opens in the browser, if it is one.
    fn url(self) -> Option<String> {
        match self {
            Entry::Documentation => Some(format!("{REPOSITORY}#readme")),
            Entry::WhatsNew => Some(format!("{REPOSITORY}/blob/main/CHANGELOG.md")),
            Entry::Repository => Some(REPOSITORY.to_string()),
            Entry::Issue => Some(format!("{REPOSITORY}/issues/new")),
            _ => None,
        }
    }
}

/// The version, and the commit it was built from.
pub(super) fn version() -> String {
    match env!("TVTY_COMMIT") {
        "" => env!("CARGO_PKG_VERSION").to_string(),
        commit => format!("{} · {commit}", env!("CARGO_PKG_VERSION")),
    }
}

impl Shell {
    pub(super) fn toggle_help_menu(&mut self, cx: &mut Context<Self>) {
        self.help_menu = !self.help_menu;
        cx.notify();
    }

    fn help_entry(&mut self, entry: Entry, window: &mut Window, cx: &mut Context<Self>) {
        self.help_menu = false;
        cx.notify();
        if let Some(url) = entry.url() {
            // The browser, whatever the OS.
            log::info!("help: opens {url}");
            cx.open_url(&url);
            return;
        }
        match entry {
            // aiball's web UI, where aiball says it serves it.
            Entry::Aiball => {
                let aiball = self.aiball.clone();
                cx.spawn(async move |_, cx| {
                    let url = cx.background_executor().spawn(async move { aiball.web_ui() }).await;
                    let _ = cx.update(|cx| match url {
                        Ok(url) => {
                            log::info!("help: opens {url}");
                            cx.open_url(&url);
                        }
                        Err(error) => crate::activity::publish(cx, crate::activity::Activity::failed(None, "open aiball's board", format!("{error:#}"))),
                    });
                })
                .detach();
            }
            Entry::About => self.open_options_page(Section::About, None, window, cx),
            Entry::Shortcuts => self.open_options_page(Section::Shortcuts, None, window, cx),
            Entry::Restart => self.restart_tvty(window, cx),
            Entry::NewProject => self.open_new_project(window, cx),
            Entry::FullScreen => window.toggle_fullscreen(),
            Entry::Updates => {
                if !crate::updates::open_updater() {
                    crate::activity::publish(
                        cx,
                        crate::activity::Activity::failed(None, "open the updater", "tvty-updater is not installed: see the README's Quick start"),
                    );
                }
            }
            _ => {}
        }
    }

    /// The menu, under the app's icon; a click outside closes it.
    pub(super) fn help_menu_view(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.help_menu {
            return None;
        }
        let mut list = div().id("help-menu").flex().flex_col().py_1().text_sm();
        for (i, entry) in Entry::ALL.into_iter().enumerate() {
            if matches!(entry, Entry::Repository | Entry::Restart) {
                list = list.child(div().my_1().h(px(1.)).bg(p().border));
            }
            list = list.child(
                div()
                    .id(("help", i))
                    .flex()
                    .items_center()
                    .gap_4()
                    .px_3()
                    .py_1()
                    .cursor_pointer()
                    .hover(|d| d.bg(p().hover))
                    .child(div().flex_1().whitespace_nowrap().child(entry.label()))
                    .child(div().flex_none().whitespace_nowrap().text_xs().text_color(p().muted).child(entry.note(cx)))
                    .on_click(cx.listener(move |shell, _, window, cx| shell.help_entry(entry, window, cx))),
            );
        }
        let backdrop = div().id("help-menu-backdrop").absolute().inset_0().occlude().on_mouse_down(
            MouseButton::Left,
            cx.listener(|shell, _, _, cx| {
                shell.help_menu = false;
                cx.notify();
            }),
        );
        let card = div()
            .absolute()
            .occlude()
            // Under the app's icon, at the title bar's left.
            .top(px(36.))
            .left(px(6.))
            // As wide as its longest line (the version and its commit), 300 px at least.
            .min_w(px(300.))
            .rounded_md()
            .bg(p().surface)
            .border_1()
            .border_color(p().border)
            .shadow_lg()
            .child(list);
        Some(div().absolute().inset_0().child(backdrop).child(card).into_any_element())
    }
}
