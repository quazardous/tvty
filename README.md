# Terminal Velocity (`tvty`)

A native terminal built for working with many AI agents at once: one window,
the agents' terminals grouped by project, and each project's tickets beside
its terminal.

`tvty` is a client. [aiball](https://github.com/quazardous/aiball) is the engine behind it (tickets,
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
| Sessions | each terminal runs `tmux attach` (`psmux attach` on Windows) on the agent's claude-loop session — tmux keeps the agent alive, tvty only shows it. Replacing tmux by a terminal host of aiball's own stays an option for later |

See [`docs/UX.md`](docs/UX.md) for the interface: navigation, switching, the ticket panel.
[`docs/TESTING.md`](docs/TESTING.md): how it is tested without a desktop or model tokens (wbox, fake-claude, simai-cli).

## Status

Early. What works today:

- the agents' terminals, grouped by project, attached to their tmux sessions;
- alerts per agent and per project: decisions waiting for you, unread
  tickets, the critical ticket;
- the ticket panel: the agent's tickets and the project's queue, a ticket's
  thread, accept / reject / reply.
- switching: a slider (ctrl+tab) and a gallery of live thumbnails
  (ctrl+shift+space); the panel resizes and folds away;
- live updates from aiball's event feed (`$AIBALL_URL`, default
  `http://127.0.0.1:7777`).

Linux only for now; Windows and macOS are planned.

## Build and run

```bash
cargo run --release              # needs a running aiball daemon
cargo run --release -- SESSION   # open that tmux session at start
```

On Linux, GPUI needs the development packages of xcb, xkbcommon, vulkan,
fontconfig, freetype and alsa. tvty reaches aiball over its local socket
(`$AIBALL_SOCK`, else aiball's default) and acts as its human user
(`$TVTY_USER`, else the human aiball saw last).

## License

[MIT](LICENSE).
