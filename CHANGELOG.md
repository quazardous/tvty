# Changelog

All notable changes to this project will be documented in this file.

> This is a curated, human-readable record — **not a commit log**. Each
> entry says *what changed and why it matters to a user*, in plain
> language, not *how* it was implemented. Skip internal refactors.
> Do not add a line per commit: a fix to something new in the same version
> is part of its entry, and a series of commits on one feature is one line.
> No protocol names, signals, timings or file internals unless the user
> sees or sets them.
>
> **House style** for editors:
> - One short bullet per change. Multi-paragraph entries are only for
>   the major changes a user really needs to read in full.
> - No internal tracker IDs (`#NNN`, `PROJ-123`) unless that tracker
>   has a public link — they're noise otherwise. Mention the change,
>   not the ticket.
> - **Version bump = SemVer**: any `### Added` entry is at least
>   MINOR; `### Fixed` alone is PATCH; breaking change is MAJOR.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- A folded comment, or a ticket's folded body, leads with ▸ (▾ once open),
  as the lists' sections do; its head and its grey line both unfold it.
- Notifications are wider and quote the start of what was written: the
  comment, or the new ticket's body, under its title. A click on one goes
  to its project too, even when its agent has no terminal here.
- Esc in a text field only leaves the field; the next Esc closes the page
  it sits on (options, new ticket, full list, a ticket full screen).
- A ticket waiting for moderation shows its proposal but no Accept or
  Reject: it is decided once the ticket is approved.
- Every button shows it is clickable the same way: a background under the
  pointer, the hand, and a tooltip with its shortcut on the icons (the
  chevrons, ⤢, ⚙); links, icons, chips and main buttons each look alike.
- The terminal slides in only when the project changes; a tab of the same
  project shows at once.
- A proposal to decide (a plan, a resolution, a wontfix, an escalation)
  always comes as one yellow notification, even when its ping does not
  arrive; tvty's log keeps the pings and notifications it saw.
- The restart offer in the agent bar reads "⟳ update" until clicked, then
  "⟳ restart pending".
- The agent bar shows the AFK mode as claude-loop does: ▶ or ‖ for the mode
  in force, ⌨ while you type, and the little man for the mode F9 arms,
  dotted until it takes effect. F9 works anywhere in the window.

### Added

- The full ticket list acts on several tickets at once: ctrl+click, shift+click
  or ctrl+a chooses them, and the left side offers what applies (approve,
  close, mark read, snooze, link…).
- "#…" in the title bar goes to a ticket: its number or a comment's #C.
  link, Enter opens it full screen (ctrl+shift+g to get there).
- A brief "Copied to clipboard" confirms a copy (a terminal's selection, a
  comment's #C. link); Options > Appearance can hide it.
- Launching tvty again brings the running one forward (on the session
  asked for, `tvty SESSION`) instead of opening a second window. A test
  tvty, with a state directory of its own, stays apart;
  `TVTY_NEW_INSTANCE=1` opens one more anyway.
- Quitting asks whether to stop the Claude Code loops too; the ones
  stopped are offered to restart at the next start. Both choices can be
  remembered (Options > Layout > Sessions).
- A click on a project's name in the sessions list shows its tickets,
  without opening a session; the stopped and shut agents are grouped by
  project too.
- A step says when its agent resumes: "resumes 08:16 or on #8" on its
  comment, the time in the list's step glyph tip.
- A click on the agent bar's "host" / "tmux" badge moves the loop to the
  other, once confirmed: its Claude restarts there, resuming its
  conversation, and the terminal follows.
- A terminal has the controls or is a copy that only watches: a chip in
  the agent bar switches. "+ session" on an agent that runs already opens
  it as a copy instead of failing or starting a second Claude.
- The agent bar says where its loop runs: "host" (aiball's session host)
  or "tmux".
- The tickets' comment bubble points to who spoke last: left and bright
  when it was someone else, right and discreet when it was you.

### Fixed

- A folded body or comment says "🖼 image" instead of the image's markdown;
  an image alone in a quote is drawn with no empty quote; an image that
  failed to load is tried again at the next read, not until a restart.
- A pasted image, an @-mention and a quoted question go in where the
  cursor is, the cursor after them — no more cursor sent back to the start;
  a mention is suggested in the middle of a text too.
- A loop in tmux is shown under its own agent, also when a lead and its
  crew share one folder.

## [0.3.0] - 2026-09-27

### Added

- Options as a tree with a search, as in VS Code: pages unfold into their
  groups, and the search finds settings and shortcuts by name, text, key or
  value; `@modified` lists what you changed.
- A changed setting stands out, shows its default, and has a "↺ Default"
  button to go back to it.
- aiball's own settings in the options, for the whole board or one project
  (durations written `1h30m`); those that can only be set elsewhere are
  listed apart and say where.
- Keyboard shortcuts you can change, in Options › Keyboard shortcuts or in
  `keymap.toml`.
- Settings in `~/.config/tvty/settings.toml`, editable by hand (applied once
  saved; a broken file is reported, never overwritten); the layout, the
  open terminals and the log in `~/.local/state/tvty/`.
- A filter atop the sessions list (Ctrl+Shift+F): type to narrow the
  sessions, Enter opens one, the arrows move between them.
- Copy and paste in the terminals: drag to select, middle click or
  Shift+Insert to paste, a right click for a menu.
- Each Claude session shows its agent's backlog and events (`b`, `e`).
- Tickets waiting for moderation wear a yellow and black construction tape.
- Pings that came while tvty was closed are summed up in one notification
  at start.
- "Eclipse Dark" and "Eclipse Light" colour themes; the title bar's theme
  menu also picks the terminals' theme.
- The sections of the sessions list and of the ticket panel resize by their
  titles, and tvty opens again the terminals left open.

### Changed

- One order for the projects everywhere: the last used first, or A–Z (⇅).
  The projects' list keeps its order while you use it; the slider follows
  your use as it goes.
- Over a full-screen view, notifications come in the bottom left corner.
- A loop's boot shows in yellow, with its time elapsed and left, as in
  claude-loop.
- "⟳ restart" also works while Claude is busy: it restarts as soon as Claude
  is idle, and the terminal comes back on its own.
- Closing the tab shown goes back to the terminal used before it.
- A held loop shows claude-loop's little man (`웃`) in the agent bar.

### Fixed

- Opening or closing a side redraws the terminal about four times faster.
- A terminal closed and opened again right away opens at once, without a
  "runs already" error.
- Loops on aiball's host are listed right (not idle while they run, under
  their project), and "start" brings a stopped one back instead of failing.
- Idle loops of no project are listed last, under "No project".
- A selection in a terminal survives the program redrawing its screen.
- A full-screen ticket uses the whole width of a wide screen, and its side
  column can be resized.
- The desktop launcher survives a new session.
- Tab and Shift+Tab reach the terminal.
- The ticket lists no longer flicker at the end of a scroll.
- No more meaningless "N pings unread" at start.
- A closed terminal leaves the list at once; one that fails to stop comes
  back with an error.
- The slider's and the gallery's cards show a shell's screen.

## [0.2.0] - 2026-09-26

### Added

- Tooltips on what means something: the badges, the state glyphs, the
  comments' bubble, the priority, the holder, the critical ticket, the
  sessions' marks and state line, the agent bar's glyphs.
- `>_` on a project opens a plain terminal in the project's folder, listed
  with the project after its agents; the panel shows the project's tickets.
- Tabs over the terminal: the terminals of its group open in tvty,
  Ctrl+PgUp / Ctrl+PgDn to move along them, "+" for a shell in the
  project's folder, × to close one (an agent's Claude runs on; a shell
  stops).
- An icon, and a launcher for the desktop (`make install-desktop`): the
  window is tied to it, so the dock shows Terminal Velocity's icon.

### Changed

- "+ session" starts the agent on aiball's host by default, without
  tmux; untick the box for claude-loop's tmux.
- Ctrl+Tab goes by group: each group is a stack of its terminals under one
  header with its counters, the one used last on top.

### Fixed

- `>_` and "+ session" take a project's folder on this machine only: an
  agent working on another machine (a Windows path) gave its folder, and
  aiball's host could not start a shell there.
- "+ terminal" no longer fails with `HOST_BUSY` when a terminal of the
  same name ended on aiball's host: it takes a free name, and closing an
  ended terminal releases its name.
- "+ session" on aiball's host starts the agent: it failed with "invalid
  params", and a crew agent is now asked for by its name.

## [0.1.0] - 2026-09-26

The first version: everything tvty does as of this day.

### Added

- Options > About shows the commit tvty was built from, beside its version.

- Two ways an agent's terminal runs, side by side: an agent on aiball's
  host is listed in its project and opened over its attach socket, without
  tmux; one still in claude-loop is opened through tmux and marked ⇄.

- The terminals aiball's daemon holds (sessions without an agent) are
  listed under "terminals" and opened without tmux, over aiball's attach
  protocol; they outlive tvty.
- "+ terminal", in the sessions list's header, starts a shell the daemon
  holds (in the home directory) and opens it.

- Colour emoji in the terminals, drawn from a colour emoji font in
  `~/.local/share/tvty/fonts/` (`make emoji-font` fetches Noto Color
  Emoji); fonts put there are loaded at start.

- The ticket lists' headers (the panel's, the full screen's) recall what
  the project's tickets ask of you: the critical one, decisions, unread.

- A restart button in the agent's bar when its Claude Code installed an
  update (⟳ on its row in the sessions list too): aiball restarts it once
  idle, resuming the conversation.

- tvty starts aiball when it is not running (its systemd user service,
  detached: aiball never lives and dies with tvty), or says it is missing.
  A test's aiball, set by `AIBALL_SOCK`, is never started.

- tvty keeps a permanent connection to aiball's bus (JSON-RPC over the
  local socket), and says in Options > About who aiball sees it as. The
  board moves onto it as aiball's methods for tvty arrive.

- A ticket filed on a project of your board is notified, whoever filed it
  and wherever: an agent's in blue, one waiting for moderation in orange,
  yours (from aiball's web UI, say) in green.

- Your own gestures are notified once aiball has them ("#12 closed",
  "decision accepted", "filed"), in green; a refusal always is, in red.
  Options hide your own. A ticket a notification is about shines in its
  colour in the lists, clearly now.

- The sessions list has a header, "Sessions", in line with the tickets
  panel's, and a ‹ that folds it; the panel's › now opens its header, on
  the side it folds away from.

- aiball's pings reach tvty: an agent answering you, mentioning you or
  proposing a decision comes as a notification; opening it marks it read
  in aiball too. At start, one notification counts the pings unread.

- Images in a thread open: a thumbnail in the compact panel, the thread's
  full width full screen, and a click shows it over the whole window — fitted, zoom with
  the wheel or + / −, 1 for the real size, drag to move, ← → through the
  thread's images, Esc to close.

- Open a session from tvty: the projects' list gets three tabs — live,
  idle (stopped loops, one click starts one again where it worked) and
  shut (agents with no loop here) — and "+ session" on a project starts a
  new one, in a working directory checked as it is typed, through
  claude-loop.

- The new-ticket form shows every choice at once, the chosen one lit:
  intent, priority, a new level field, scope (with who it notifies), tags,
  milestone and assignee (the project's agents first); a parent can be
  typed as #ticket.

- The agent's bar now says all that claude-loop's bar says, once the loop
  pushes it to aiball: a hold's time left, the passing state, dialogs and
  alerts, the prompt and a human typing, the proxy, the next wake — and a
  click on its backlog lists the agent's own backlog by tier.

- aiball's comment count on every ticket row, in the panel and the full
  list: a bubble and the number, green when you spoke last (a clock while
  comments wait for moderation); a thin dotted stripe on a row where your
  word is the last; and the count, with who spoke last, heading a thread
  full screen.

- Notifications in the window's bottom left corner, above everything:
  stacked bottom up, the newest in the corner, at most five, gone after six seconds unless hovered (both set in Options),
  the ticket each is about shining in the lists meanwhile. They replace the
  banner over the terminal, and carry tvty's own messages too.

- An agent's bar under its terminal: who drives its loop (with a click to
  hold it or let it run), its Claude's state, its events, the tickets it
  holds, its wait credit and where it works — from aiball, like the rest.

- Font sizes of your own, in Options > Appearance: the terminals' (8 to
  32 px, also ctrl+shift+= / − / 0) and the window's text (12 to 22 px).
  Both are remembered.

- File a ticket from tvty: + in the ticket panel or the full list,
  ctrl+shift+n, or a sub-ticket from a ticket's full view. A full-screen
  form, laid out as a ticket's detail (project, intent, priority, scope,
  tags, milestone, assignee on the left; title, summary and body with
  mentions and pasted images on the right). The draft survives Esc; what
  aiball refuses after filing (a closed milestone…) is said on the ticket.

- A ticket list that says at a glance what is yours: bands from "to
  moderate" to "open", bold for unread, one state glyph, a stripe for whose
  turn it is; its legend is in the options.
- A ticket full screen (⤢ more, or from the full list): its invariants on
  the left third — fields, tags, milestone, reporter, assignee, relations,
  project, title and body, each changed in place — and the whole talk on
  the rest, each comment with a ⋯ menu (edit, delete, classify, step, vote,
  resurface, copy its reference).
- An open ticket's thread reads itself again when someone else writes.
- The slider and gallery cards are live — each session watched read-only,
  never resized — and show only the bottom of the screen: ctrl+tab no
  longer drags.
- In the panel, a band with many tickets no longer overlaps the next one:
  each keeps its title and two rows, and scrolls within.
- In a ticket's thread: the order (newest first or last), a click to
  change the priority, a short snooze, replies without notifying, `@`
  mentions, pasted screenshots, and questions answered and ticked from
  the reply.
- The ticket list's states, the flame, the critical warning and the
  priorities are drawn as Material Symbols icons, in the theme's colours —
  the emoji they replace showed grey.
- The ticket list, full screen (⤢ in the panel, or ctrl+shift+l): projects,
  band counters and filters on the left, and every ticket with all its row
  has to say on the right, in one list sorted by activity, whose turn,
  priority, creation or number.
- In the panel, each band of tickets folds with a click on its title and
  scrolls on its own; the tickets agents are on come first.
- A ticket's thread says where it stands once, on top: whose turn it is in
  one sentence and the latest summary pinned; older comments fold to their
  summary line, replaced decisions say "superseded", and the gestures sit in
  one place under the thread — accept, reject with a reason, moderate,
  reply, close or reopen.
- Dragging a side's edge no longer makes the program in the terminal redraw
  at every column: the terminal takes its new size once the drag settles.
  The projects' list, lying over the terminal, can be resized again.
- Visible scrollbars on the ticket list, the ticket thread, the projects'
  list, the options, the theme menu and the gallery.
- An options page (⚙ or ctrl+,): appearance, layout, every keyboard shortcut,
  and where tvty's settings live.
- The projects' list lies over the terminal, which keeps its width whether
  the list is open or folded, and can be resized by its edge.
- Colour themes: the whole window and the terminals follow the chosen theme,
  picked from the title bar or with ctrl+shift+k and remembered; light and
  dark themes bundled (Catppuccin, Everforest, Flexoki, Gruvbox, Solarized,
  Tokyo Night), and your own in `~/.config/tvty/themes/`.
- The terminals can have a colour theme of their own — a dark terminal in a
  light window — chosen in the options; by default they follow the window's.
- tvty draws its own window frame — a title bar to move it, with minimize,
  maximize and close, and edges to resize it — so it can be handled on GNOME,
  which leaves decorations to applications, and looks the same elsewhere.
- A window with your agents' terminals on the left, grouped by project, the
  selected one in the middle, and its project's tickets on the right.
- Terminals attach to the agents' tmux sessions, render colours, styles, wide
  characters and frame lines, and keep running while hidden.
- Each agent and project shows what waits for you: decisions to take, unread
  tickets, the ticket holding the most work back.
- Open a ticket to read its thread, accept or reject the pending plan or
  resolution, and reply — without leaving the terminal.
- The mouse wheel scrolls back through the agent's history.
- Each agent's Claude state at a glance, as aiball has it: working, idle or
  starting and for how long, autonomous, held or with a human typing, or
  offline — under its name, as a colour beside it, and on its cards.
- Switch terminals like alt-tab, as a portfolio: hold ctrl and tap tab, the
  window dims and the terminals come forward as live thumbnails grouped by
  project; release to switch.
- When an agent newly needs you (a decision, something new on its ticket), a
  banner says so; ctrl+enter brings its terminal and opens the ticket.
- Select text in the terminal with the mouse, copy and paste with
  ctrl+shift+c / ctrl+shift+v.
- See every terminal at once as live thumbnails (ctrl+shift+space), type to
  filter, enter to open — without resizing the agents' sessions.
- Resize the ticket panel by its edge. Fold either side away with the grip
  on its edge (ctrl+shift+b, ctrl+shift+t) into a 10-pixel strip that still
  shows what waits as coloured dots; tvty remembers it.
- Tickets and alerts update as soon as something happens on the board.

### Changed

- Unread is no longer a band of the ticket lists: an unread ticket keeps
  its band (agents on it, open…) and shows its blue mark. The full list
  has an Unread filter instead.

- The lists' section titles stand out: a band with a line above and below,
  the title in full colour, the count in a pill.

- Notifications show in the terminal's top right corner (the window's
  while a full screen covers it), the newest highest.

- The agent's bar no longer shows its wait credit.

- An unread ticket shows it more: its comments' bubble turns blue, beside
  its bold title; the bubble is grey when you spoke last, lighter grey when
  someone else did and you read it.

- The long lists of choices are searched as you type, not laid out whole:
  in a new ticket the project, tags, milestone and assignee (Enter takes
  the first match, ✕ drops a choice), in a ticket's detail the tags,
  reporter, assignee and project to move to.

- In the ticket lists, a step (`then: continue`) keeps its colour whoever's
  turn it is: blue, amber once it went quiet, as in aiball's web UI.

- The agent's bar shows claude-loop's envelope: ✉ while work waits for
  the loop, with the countdown to its next wake once armed (`✉ 54s`).
- The window is drawn again only when a time it shows changes (once a
  minute for most), not every few seconds.

- The board, the agents' states and bars, and the pings come pushed by
  aiball over its bus, as data: tvty no longer reads the board again
  when something moves, nor polls it; after a drop it resumes where it
  was. Options > About says how many subscriptions are up.

- The sessions list drops its vertical tabs for three foldable sections,
  live, idle and shut, drawn like the ticket list's bands; what is folded
  is remembered (idle and shut start folded).

- A new ticket is filed whole, its tags, assignee, level, milestone and
  parent in the same call: aiball files all of it or refuses all of it,
  and says why. No more "filed, but not all of it".

- tvty no longer names its user in what it writes to aiball: aiball takes
  the author from the connection, and refuses a body naming someone else.

- Debug builds (`make run`) draw at almost release speed: the gallery went
  from about one frame a second to eight or nine, as the dependencies are
  now optimised even there.

### Fixed

- Ctrl+Enter sends a reply in a ticket's thread, and files a new ticket
  from any of its text fields: it only put in a new line.

- A ticket list that grew or moved on its own (rows pushed by aiball, a
  section above shrinking) no longer leaves blank rows until the mouse
  moves.

- Pings raised notifications again: none came since tvty took aiball's
  bus.

- Opening tvty no longer raises a notification for every ticket already
  waiting on you: only what comes after is news.

- A new colour theme reaches the terminals at once: they waited for their
  program's next output to take it.

- tvty no longer slows down as the board grows: a ticket list lays out
  only the rows in view, all of the same height. On 300 tickets a frame
  went from about 110 ms to under 10 ms, and tvty at rest from half a
  core to a few percent: the ticket panel and the terminal are drawn
  again only when they change, and the sessions list's rows keep a fixed
  height.

- A ping from aiball is no longer lost when its ticket cannot be read
  back, and the pings sent while tvty's stream was down come once it is
  back.

- When Claude quits in the terminal shown, tvty no longer leaves a dead
  "[exited]" screen: an end screen offers to start the loop again (Enter)
  or go back to the previous terminal (Esc), and the panel keeps the
  project's tickets. Starting the loop again attaches afresh.

- The window resizes from its left edge again, and from every edge more
  easily: a 6-pixel band along each free edge, above everything, where a
  press only resizes. The folded list's strip, which took the press to
  open the list, is wider and starts after that band.

- A ticket whose claim has expired no longer shows its former claimant as
  the agent on it: tvty reads who holds a ticket from aiball, which now
  says it on every row. The lists follow aiball's renamed turn view.

- A ticket full screen no longer leaves the talk half empty with a
  scrollbar at the bottom: a long "where it stands" wraps under the count
  instead of widening the column. Images at the end of a sentence are
  drawn as tvty's thumbnails (large full screen) instead of line-high.

- Images in a ticket's thread show: they were never loaded, and each one
  wrote an error on the standard output. One that cannot be read says
  "image unavailable" (or "too large", past 5 MB) instead.

- A rejected plan or resolution shows as aiball shows it: a red ⊗ on the
  ticket, whoever's turn it is (it was a grey arrow once the ball was the
  agent's), with "rejected" in the full list.

- The title bar is a fifth taller (41 px), easier to grab; the panel's
  new-ticket button reads "+ New", in a frame, instead of a lone "+".

- Going full screen follows what the panel shows: its list opens the full
  list on the same project, its ticket opens that ticket full screen; and
  coming back finds the panel as it was left.

- The mouse wheel is no longer slow: a notch moves about three ticket rows
  in the lists and threads (it was one), five lines of a terminal's
  history, and a fast wheel in a tmux terminal no longer lags behind.
  `scroll_speed` in the settings adjusts it.

- The slider's cards stay in place: they are laid out as the projects'
  list, and only the enlarged, chosen card moves. The arrows move the
  choice, in the slider and in the gallery, whose enter now opens the
  card it shows chosen.

- Debug builds (`make run`) draw at almost release speed: the gallery went
  from about one frame a second to eight or nine, as the dependencies are
  now optimised even there.
