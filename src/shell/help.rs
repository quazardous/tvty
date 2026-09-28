//! The title bar's ? (F1): where to learn about tvty and to reach its
//! people — the version and the build, the documentation, the shortcuts,
//! what changed, the repository, a bug to report — and a restart.

use gpui_kit::*;

use super::Shell;
use crate::options::Section;
use crate::theme::p;

/// The repository, as Cargo.toml declares it.
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// What the menu offers.
#[derive(Clone, Copy)]
enum Entry {
    About,
    Documentation,
    Shortcuts,
    WhatsNew,
    Repository,
    Issue,
    Restart,
}

impl Entry {
    const ALL: [Entry; 7] = [
        Entry::About,
        Entry::Documentation,
        Entry::Shortcuts,
        Entry::WhatsNew,
        Entry::Repository,
        Entry::Issue,
        Entry::Restart,
    ];

    fn label(self) -> &'static str {
        match self {
            Entry::About => "About tvty",
            Entry::Documentation => "Documentation",
            Entry::Shortcuts => "Keyboard shortcuts",
            Entry::WhatsNew => "What's new",
            Entry::Repository => "GitHub",
            Entry::Issue => "Report an issue",
            Entry::Restart => "Restart tvty",
        }
    }

    /// What it says beside its label.
    fn note(self) -> String {
        match self {
            Entry::About => version(),
            Entry::Documentation | Entry::WhatsNew | Entry::Repository | Entry::Issue => "↗".into(),
            Entry::Shortcuts => String::new(),
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
            if let Err(error) = std::process::Command::new("xdg-open").arg(&url).spawn() {
                crate::activity::publish(cx, crate::activity::Activity::failed(None, &format!("open {url}"), error));
            }
            return;
        }
        match entry {
            Entry::About => self.open_options_page(Section::About, None, window, cx),
            Entry::Shortcuts => self.open_options_page(Section::Shortcuts, None, window, cx),
            Entry::Restart => self.restart_tvty(cx),
            _ => {}
        }
    }

    /// The menu, under the title bar's ?; a click outside closes it.
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
                    .child(div().flex_1().child(entry.label()))
                    .child(div().text_xs().text_color(p().muted).child(entry.note()))
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
            .top(px(36.))
            .right(px(96.))
            .w(px(300.))
            .rounded_md()
            .bg(p().surface)
            .border_1()
            .border_color(p().border)
            .shadow_lg()
            .child(list);
        Some(div().absolute().inset_0().child(backdrop).child(card).into_any_element())
    }
}
