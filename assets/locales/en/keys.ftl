# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# Keyboard shortcuts (src/keymap.rs, src/options.rs, src/shell.rs). A
# command's words go by its name: notice.open is keys-notice-open.

## What each command does

keys-notice-open = Go to the newest notification: the agent's terminal, its ticket
keys-slider-next = Slider: the groups as stacks, most recent first; release Ctrl to open
keys-slider-back = Slider, backwards
keys-tab-next = The tab after, in the group shown
keys-tab-back = The tab before, in the group shown
keys-gallery-toggle = Gallery of every terminal; type to filter, arrows move, Enter opens
keys-panel-toggle = Fold or unfold the ticket panel
keys-sidebar-toggle = Fold or unfold the projects' list
keys-afk-cycle = The shown agent's AFK mode: auto → hold 10 min → hold, in force 3 s after the last press (a terminal without an agent gets F9)
keys-sessions-filter = Filter the sessions: type, Enter opens, arrows move, Esc clears
keys-ticket-goto = Go to a ticket: its number or a comment's #C. link, Enter opens it (the title bar's #…)
keys-options-toggle = Settings
keys-app-quit = Quit tvty, as closing its window does (running loops: stopped or kept, as chosen)
keys-help-menu = Help: about tvty, its documentation, what's new, a restart
keys-window-fullscreen = The window full screen, or back
keys-font-bigger = Terminal font bigger
keys-font-smaller = Terminal font smaller
keys-font-reset = Terminal font back to its default
keys-ticket-new = A new ticket (also + in the panel and the full list)
keys-project-megaphone = The 📢 of the shown project: its standing instruction and its wake focus
keys-list-full = The ticket list, full screen
keys-theme-next = Next colour theme
keys-debug-inspector = GPUI's inspector: pick an element, see its id and where it is made (debug builds)
keys-terminal-copy = Copy the selection
keys-terminal-paste = Paste the clipboard
keys-terminal-tab = Tab, to the program (not the focus to the next element)
keys-terminal-back-tab = Shift+Tab, to the program

## The keys that are not commands: what they do, the keys as .keys

keys-fixed-esc = Close the gallery, the slider, the theme list, the options, the full list
    .keys = Esc
keys-fixed-arrows = In the slider and the gallery: move to the card seen there
    .keys = Arrows
keys-fixed-select = In a terminal: select text · a word · a line — copied to the primary selection
    .keys = Drag · double click · triple click
keys-fixed-middle-click = In a terminal: paste the primary selection
    .keys = Middle click
keys-fixed-right-click = In a terminal: Copy, Paste — on a link, Open link, Copy link
    .keys = Right click
keys-fixed-ctrl-click = In a terminal, on a link: open it
    .keys = Ctrl+click
keys-fixed-wheel = Scroll the history
    .keys = Mouse wheel
## The page

keys-intro = Each shortcut is a command, bound in a context: Terminal when a terminal has the focus, Window anywhere else. The deepest binding wins; a key bound nowhere goes to the terminal's program.
keys-how = Click a key to change it, + to add one, × to remove it. Kept in {$file}, which you may edit too.
keys-reset-all = Reset all
keys-not-applied = Not applied: {$error}
keys-context-window = Window — anywhere in tvty, full-screen pages included
keys-context-workspace = Workspace — the terminals, the sessions list and the panel beside them
keys-context-terminal = Terminal — when a terminal has the focus
keys-press = Press a key… (Esc gives up)
keys-typing-refused = {$key} is typing: in a terminal it stays the program's
keys-remove = remove this key
keys-the-program = the program
keys-masked = {$what} — masked in a terminal by {$by}
keys-no-key = no key
keys-conflict = {$key} runs {$other}: take it for {$name}?
keys-replace = Replace
keys-cancel = Cancel
keys-freed-terminal = Given back to the terminal's program
keys-freed-focus = Given back to what has the focus
