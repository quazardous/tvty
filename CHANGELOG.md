# Changelog

All notable changes to this project will be documented in this file.

> This is a curated, human-readable record — **not a commit log**. Each
> entry says *what changed and why it matters to a user*, in plain
> language, not *how* it was implemented. Skip internal refactors.
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

### Added

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

- Notifications in the window's top left corner, above everything: stacked,
  at most five, gone after six seconds unless hovered (both set in Options),
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

- Debug builds (`make run`) draw at almost release speed: the gallery went
  from about one frame a second to eight or nine, as the dependencies are
  now optimised even there.

### Fixed

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
