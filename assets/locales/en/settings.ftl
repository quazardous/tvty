# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# Settings (src/settings.rs, src/options.rs, src/shell.rs's options pages).
# A setting's words go by its key: appearance.terminal_font_size is
# setting-appearance-terminal_font_size, its description .about, a
# switch's two states .on and .off.

## The settings

setting-appearance-language = Language
    .about = The interface's language. What people write (tickets, comments, names) stays as written.
setting-appearance-terminal_font_size = Terminal font
    .about = The terminals' text. Ctrl+Shift+= / Ctrl+Shift+− / Ctrl+Shift+0 too.
setting-appearance-window_font_size = Window text
    .about = Everything around the terminals — lists, tickets, menus.
setting-notifications-max = Shown at most
    .about = In the terminal's top right corner (bottom left over a full screen), the newest in the corner; older ones make room.
setting-notifications-seconds = Seconds shown
    .about = Then it goes, unless the pointer is on it.
setting-notifications-own = Your own gestures
    .about = A ticket closed, a reply posted, a plan accepted: said once aiball has it. A refusal is always said.
    .on = shown
    .off = hidden
setting-notifications-copied = Copied
    .about = A brief "Copied to clipboard" at the bottom when text goes to the clipboard (not for a mere selection).
    .on = shown
    .off = hidden
setting-terminal-scrollback = Lines of history
    .about = What each terminal keeps to scroll back through. More costs memory: about 4 KB a line at 160 columns, so 5000 lines is about 19 MB a terminal at most. A tmux session's history is tmux's own (its history-limit).
setting-scroll-speed = Wheel speed
    .about = Times tvty's own: at 1, a notch is about three ticket rows, five lines of a terminal's history.
setting-mouse-focus = Focus
    .about = What gives the keyboard to the terminal or to a box to write in: a click, or the pointer moving over it (as a window manager's focus follows the mouse). By default, as the system does.
setting-appearance-theme = Window
    .about = The window's colour theme. Ctrl+Shift+K steps through them.
setting-appearance-terminal_theme = Terminal
    .about = The terminals' own, or the window's — a dark terminal in a light window.
setting-appearance-terminal_opacity = Terminal opacity
    .about = Below 100 %, the desktop shows through the terminals' background, live. The lists, the tickets and the colours a program sets stay opaque.
setting-appearance-terminal_blur = Blur behind
    .about = Behind a see-through terminal, the desktop blurred. Only where the compositor offers it (KDE); elsewhere it shows sharp.
    .on = blurred
    .off = sharp
setting-updates-check = Check for updates
    .about = At start and once a day, asks GitHub whether a newer Terminal Velocity is out, and says so once (nothing else is sent). The updater (the menu's Updates…) installs it.
    .on = check
    .off = never
setting-tips-show = Show tips
    .about = "Did you know?": a short tip once after start, and the first time a page is opened; never one for what you already use. The menu's Tips… shows them all.
    .on = show
    .off = never
setting-sessions-order = Order
    .about = The projects in the list, the slider and the gallery: the one used last first, as ctrl+tab goes; alphabetical; or yours, as you drag them in the list. Also the ⇅ in the list's header.
setting-sessions-show_hub = The hub's sessions
    .about = When this machine reaches aiball through a proxy node: list the sessions that run on aiball's hub too, apart, under "on hub". They are read from here (state, tickets), never opened: a session is attached from its own machine. The sessions of other nodes are never listed.
    .on = listed
    .off = not listed
setting-sessions-on_quit = On quit
    .about = When tvty quits with Claude Code loops of this machine running: ask, stop them (through aiball: they stay restartable), or keep them running.
setting-sessions-on_start = On start
    .about = The loops tvty stopped when it quit: ask, restart them as they were (resuming their conversation, a held one held again), restart them fresh (booting, then on their own), or leave them stopped.
setting-tickets-panel_overlay = Place
    .about = Beside the terminal, which is then narrower — or over it, as the sessions' list on the left: the terminal keeps its whole width and the panel covers its right side while open.
    .on = over the terminal
    .off = beside the terminal
setting-tickets-newest_first = Newest first, full screen
    .about = A ticket full screen shows its thread's newest word first (also ⇅ in the ticket's header).
    .on = newest first
    .off = newest last
setting-tickets-panel_newest_first = Newest first, in the panel
    .about = A ticket in the panel shows its thread's newest word first; off, last, by the reply (also ⇅ there). The panel has an order of its own: the full screen's is not its.
    .on = newest first
    .off = newest last
setting-tickets-summary_open = Where it stands, open
    .about = In the ticket panel, a ticket's "Where it stands" shows its text at once; off, it is folded to its title, a click away. Full screen it always shows.
    .on = open
    .off = folded
setting-tickets-ctrl_enter_opens = Ctrl+Enter
    .about = In a new ticket, Ctrl+Enter files it and goes back to where you were ("File and exit"), or files it and opens it ("File the ticket"). The buttons do each, whatever this says.
    .on = files and opens it
    .off = files and exits

## The pages and groups, (settings-title-<name>)

settings-title-project = Project
settings-title-appearance = Appearance
settings-title-layout = Layout
settings-title-ticket-list = Ticket list
settings-title-keyboard-shortcuts = Keyboard shortcuts
settings-title-aiball = aiball
settings-title-about = About
settings-title-language = Language
settings-title-sizes = Sizes
settings-title-notifications = Notifications
settings-title-mouse = Mouse
settings-title-colours = Colours
settings-title-updates = Updates
settings-title-tips = Tips
settings-title-sessions = Sessions
settings-title-ticket-panel = Ticket panel
settings-title-thread = Thread
settings-title-new-ticket = New ticket
settings-title-sides = Sides
settings-title-legend = Legend
settings-title-window = Window
settings-title-workspace = Workspace
settings-title-terminal = Terminal
settings-title-fixed-keys = Fixed keys
settings-title-folder = Folder
settings-title-board = Board
settings-title-board-wide-only = Board-wide only
settings-title-per-project-only = Per project only
settings-title-in-aiball-yaml = In .aiball.yaml

## The page

settings-heading = Settings
settings-search-placeholder = Search…
settings-modified-hint = @modified: what you changed
settings-search = Search: {$text}
settings-close = ✕  Esc
settings-nothing-found = Nothing found. Search a name, a word it says, a key (ctrl+shift+b), a value — or @modified.
settings-default = default
settings-default-back = Default
settings-back-to = back to {$value}
settings-own-themes = Your own themes (gpui-component's theme format) go in ~/.config/tvty/themes/: they show here the next time this page opens.

## Choices

setting-language-auto = Automatic ({$lang})
settings-ask = Ask
settings-order-recent = Most recent first
settings-order-alpha = Alphabetical
settings-order-yours = Yours (dragged in the list)
settings-quit-stop = Stop them
settings-quit-keep = Keep them running
settings-start-restart = Restart them as they were
settings-start-fresh = Restart them fresh
settings-start-leave = Leave them stopped
settings-focus-click = Click
settings-focus-hover = Hover
settings-same-as-window = Same as the window
settings-dark = Dark
settings-light = Light
settings-theme-dark = DARK
settings-theme-light = LIGHT
settings-theme-window = Window
settings-theme-terminal = Terminal
settings-theme-opacity = Terminal opacity

## Sides

settings-side-open = open
settings-side-folded = folded
settings-side-said = {$state} · {$width} px
settings-side-list = Projects' list
settings-side-list-about = Over the terminal, on the left. Drag its edge to resize it; its grip folds it.
settings-side-panel = Ticket panel
settings-side-panel-about = On the right of the terminal. Drag its edge to resize it; its grip folds it.
settings-widths = Widths
settings-widths-about = Back to the defaults: a list of 290 px, a panel a third of the window.
settings-reset = Reset

## About

settings-about-version = Version
settings-about-aiball-at = aiball at
settings-about-acting-as = Acting as
settings-about-bus = aiball bus
settings-about-bus-said = version {$version}, as {$who}
settings-about-bus-said-kind = version {$version}, as {$who} ({$kind})
settings-about-not-connected = not connected
settings-about-live = Live board
settings-about-not-subscribed = not subscribed
settings-about-subscriptions = { $count ->
    [one] {$count} subscription on the bus
   *[other] {$count} subscriptions on the bus
}
settings-about-theme = Theme
settings-about-terminal-theme = Terminal theme
settings-about-the-windows = the window's
settings-about-config = Settings, shortcuts, themes
settings-about-state = Layout and workspace
settings-about-fonts = Fonts
settings-about-what = A native terminal for working with many AI coding agents at once: their terminals grouped by project, each project's tickets beside — the agent asks, you decide, it carries on. Built on aiball, which runs the agents' loops and their board.
settings-about-github = GitHub ↗
settings-about-aiball = aiball on GitHub ↗
settings-about-license = MIT licence ↗
settings-about-footer = Terminal Velocity (tvty). MIT licence. Bundled themes: see themes/README.md.

## aiball's config

settings-aiball-failed = aiball config
settings-aiball-unreadable = aiball's config could not be read: {$error}. Choose the scope again, top left, to try again.
settings-aiball-reading = Reading aiball's config…
settings-aiball-board = The board's own config: what every project gets unless it says otherwise. Choose a project, top left, for its own.
settings-aiball-project = What {$project} says over the board's config; ↺ gives a key back to the board's value.
settings-aiball-in-file = aiball reads these from each project's .aiball.yaml, not from its config store: change them in that file.
settings-aiball-global-only = aiball declares these for the whole board: one value for every project, set in Global.
settings-aiball-project-only = aiball declares these per project only: no board-wide value; choose a project above to set them.
settings-aiball-open-global = Open Global
settings-aiball-protected = 🔒 protected
settings-aiball-set-in-file = set in .aiball.yaml
settings-aiball-board-wide = board-wide: set in Global
settings-aiball-per-project = per project
settings-aiball-from-board = {" "}From the board's config.
settings-aiball-board-back = Board
settings-on = on
settings-off = off

## The ticket list's legend

settings-legend-intro = The ticket list as aiball computes it for you: the band, whose turn and the state glyph are aiball's; the stripe is tvty's.
settings-legend-order = Order
settings-legend-band-moderate = the ticket, or comments on it, wait for moderation
settings-legend-band-decide = a plan, a resolution, a wontfix or an escalation waits for your decision
settings-legend-band-working = an agent holds it or is on a step
settings-legend-band-open = open, nothing pressing
settings-legend-band = {$what}; the most recent first
settings-legend-glyph = State glyph — coloured when it waits on you, muted otherwise
settings-legend-always-blue = {$what} — always blue
settings-legend-always-amber = {$what} — always amber
settings-legend-always-red = {$what} — always red
settings-legend-stripe = Stripe — whose turn
settings-legend-stripe-solid = a decision waits on you, and it is the last message
settings-legend-stripe-dashed = a decision waits on you, but the talk went on after it
settings-legend-stripe-yours = your turn: an agent answered you
settings-legend-stripe-waiting = thin and dotted: your word is the last, you wait
settings-legend-stripe-none = no stripe: the ball is with the agent
settings-legend-stripe-tape = construction tape: the ticket waits for moderation, nothing goes on until you let it through
settings-legend-rest = The rest
settings-legend-unread = comments, blue: something new on it for you, unread (its title in bold too)
settings-legend-others-spoke = bright, its point on the left: someone else spoke last
settings-legend-you-spoke = discreet, its point on the right: you spoke last
settings-legend-pending = comments waiting for moderation
settings-legend-agent = agent
settings-legend-holder = the agent holding it; the flame when it was active lately
settings-legend-critical = the project's critical ticket: it holds 3 open tickets
settings-legend-priority = priority: urgent, high, low (normal shows nothing)
