//! The options page: its sections, and the keyboard shortcuts — the one
//! list of them, which the page shows.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Appearance,
    Layout,
    TicketList,
    Shortcuts,
    About,
}

impl Section {
    pub const ALL: [Section; 5] = [
        Section::Appearance,
        Section::Layout,
        Section::TicketList,
        Section::Shortcuts,
        Section::About,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Section::Appearance => "Appearance",
            Section::Layout => "Layout",
            Section::TicketList => "Ticket list",
            Section::Shortcuts => "Keyboard shortcuts",
            Section::About => "About",
        }
    }
}

/// `(keys, what they do)`, grouped: `(group, shortcuts)`.
pub const SHORTCUTS: &[(&str, &[(&str, &str)])] = &[
    (
        "Switching",
        &[
            ("Ctrl+Tab", "Slider: the terminals, most recent first; arrows move; release Ctrl to open"),
            ("Ctrl+Shift+Tab", "Slider, backwards"),
            ("Ctrl+Shift+Space", "Gallery of every terminal; type to filter, arrows move, Enter opens"),
            ("Ctrl+Enter", "Go to the newest notification: the agent's terminal, its ticket"),
            ("Ctrl+Shift+N", "A new ticket (also + in the panel and the full list); Ctrl+Enter files it"),
        ],
    ),
    (
        "Window",
        &[
            ("Ctrl+Shift+B", "Fold or unfold the projects' list"),
            ("Ctrl+Shift+T", "Fold or unfold the ticket panel"),
            ("Ctrl+Shift+L", "The ticket list, full screen (also ⤢ in the panel)"),
            ("Ctrl+Shift+K", "Next colour theme"),
            ("Ctrl+Shift+= · Ctrl+Shift+− · Ctrl+Shift+0", "Terminal font bigger · smaller · default"),
            ("Ctrl+,", "Options"),
            ("Esc", "Close the gallery, the slider, the theme list, the options, the full list"),
        ],
    ),
    (
        "Terminal",
        &[
            ("Ctrl+Shift+C", "Copy the selection"),
            ("Ctrl+Shift+V", "Paste"),
            ("Drag · double click · triple click", "Select text · a word · a line"),
            ("Shift+drag", "Select even when the program takes the mouse"),
            ("Mouse wheel", "Scroll the history"),
        ],
    ),
];
