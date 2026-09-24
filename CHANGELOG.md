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

- A window with your agents' terminals on the left, grouped by project, the
  selected one in the middle, and its project's tickets on the right.
- Terminals attach to the agents' tmux sessions, render colours, styles, wide
  characters and frame lines, and keep running while hidden.
- Each agent and project shows what waits for you: decisions to take, unread
  tickets, the ticket holding the most work back.
- Open a ticket to read its thread, accept or reject the pending plan or
  resolution, and reply — without leaving the terminal.
- The mouse wheel scrolls back through the agent's history.
- Switch terminals like alt-tab: hold ctrl and tap tab to go through the
  most recent ones, release to switch.
- See every terminal at once as live thumbnails (ctrl+shift+space), type to
  filter, enter to open — without resizing the agents' sessions.
- Resize the ticket panel by its edge, fold it away (ctrl+shift+t); tvty
  remembers both.
- Tickets and alerts update as soon as something happens on the board.
