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

- A ticket list that says at a glance what is yours: bands from "to
  moderate" to "open", bold for unread, one state glyph, a stripe for whose
  turn it is; its legend is in the options.
- A ticket full screen (⤢ more, or from the full list): its invariants on
  the left third, the whole talk on the rest.
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
