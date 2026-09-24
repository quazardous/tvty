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
