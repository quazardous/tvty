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

/// The keys that are not commands (see [`crate::keymap`]): those of what
/// is up, and the mouse's. `(keys, what they do)`.
pub const FIXED_KEYS: &[(&str, &str)] = &[
    ("Esc", "Close the gallery, the slider, the theme list, the options, the full list"),
    ("Arrows", "In the slider and the gallery: move to the card seen there"),
    ("Drag · double click · triple click", "In a terminal: select text · a word · a line — copied to the primary selection"),
    ("Middle click", "In a terminal: paste the primary selection"),
    ("Right click", "In a terminal: Copy, Paste"),
    ("Mouse wheel", "Scroll the history"),
];
