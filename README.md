# Terminal Velocity (`tvty`)

A native terminal built for working with many AI agents at once: one window,
the agents' terminals grouped by project, and each project's tickets beside
its terminal.

`tvty` is a client. [aiball](../aiball) is the engine behind it (tickets,
agents, wakes, the API) and evolves at its own pace; aiball's web UI keeps
existing for everything else (remote, phone, another OS).

## What it aims at

- **One window instead of twenty tabs.** Projects and their agents on the
  left, each with its alerts (unread, a decision waiting for you, the critical
  ticket). Switch agents in one keystroke, or from an overview of live
  thumbnails.
- **The agent's terminal, live**, attached to its session — no tmux screen in
  the way.
- **A collapsible ticket panel on the right** of a project's terminal: the
  agent's current ticket and its queue, with the usual gestures (accept,
  reject, reply).
- Later: the interface follows the work (a ticket created for an agent brings
  its terminal forward), then the dispatcher and cto levels.

## Choices made

| | |
|---|---|
| Platforms | Linux (GNOME and KDE alike), Windows, macOS — the UI draws itself, no desktop theme involved |
| UI toolkit | [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) (Zed's toolkit, Apache-2.0), widgets from gpui-kit |
| Terminal emulation | [`alacritty_terminal`](https://crates.io/crates/alacritty_terminal) — the same pairing as Zed's terminal, with a view of our own |
| Sessions | attached through aiball's terminal host rather than tmux |

## Status

Empty scaffold. Nothing to run yet.
