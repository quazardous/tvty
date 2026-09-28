# Terminal Velocity

**Twenty AI agents. One window. No tab hunting.**

*Beta — for [aiball](https://github.com/quazardous/aiball) users; Linux.*

![An agent's terminal and the critical ticket it escalated, side by side](docs/images/hero.png)

`tvty` is a native terminal for working with many AI coding agents at once:
their terminals grouped by project, and each project's tickets right beside —
the agent asks, you decide, it carries on.

## Why

- **One window instead of twenty tabs.** Every project, every agent, and who
  is waiting for you: a decision, an unread answer, the ticket that holds the
  others back.
- **Each agent's terminal, live.** Attached to its session, full speed, no
  tmux screen in the way. Switch in one keystroke, or pick from live
  thumbnails of all of them.
- **Its tickets beside it.** Plans, resolutions, escalations, with their
  pictures: accept, reject or reply without leaving the keyboard.

## See it

![A tour: the terminal and its ticket, the ticket full screen, the slider, the gallery](docs/images/tour.gif)

| | |
|---|---|
| ![Projects and agents, with what waits for you](docs/images/sessions.png) | ![Every terminal, live, in one gallery](docs/images/gallery.png) |
| **Projects and agents**, each with what waits for you — working or idle, since when. | **The gallery** (ctrl+shift+space): every terminal live; type to filter. |
| ![The slider, a stack per project](docs/images/slider.png) | ![A plan waiting for a go, beside the agent that wrote it](docs/images/plan.png) |
| **The slider** (ctrl+tab): a stack per project, most recent first. | **A plan** waiting for your go, beside the agent that wrote it. |

![A ticket full screen, its blueprint inline](docs/images/ticket.png)

*The station, the sphere and the time machine are fictional: a demo world
replayed by [`demo/run`](demo/run), agents included.*

## Install

> **Beta.** tvty is for people who already run
> [aiball](https://github.com/quazardous/aiball) and its claude-loop agents:
> it is aiball's desktop client, not a standalone terminal. Linux only for
> now (Wayland or X11, GNOME and KDE). Expect rough edges, and tell us
> (the ? menu > Report an issue).

**You need** aiball 0.49 or later, running (its daemon, `claude-loop`
with it), tmux 3.x, and Rust (stable, 1.85 or later) to build tvty.

**GPUI's system libraries**, to build:

```bash
# Fedora
sudo dnf install gcc pkgconf-pkg-config libxkbcommon-devel libxkbcommon-x11-devel \
  libxcb-devel wayland-devel vulkan-loader-devel fontconfig-devel freetype-devel alsa-lib-devel
# Debian, Ubuntu
sudo apt install build-essential pkg-config libxkbcommon-dev libxkbcommon-x11-dev \
  libxcb1-dev libwayland-dev libvulkan-dev libfontconfig-dev libfreetype-dev libasound2-dev
# Arch
sudo pacman -S base-devel libxkbcommon libxkbcommon-x11 libxcb wayland \
  vulkan-icd-loader fontconfig freetype2 alsa-lib
```

**Build and install** — a launcher in `~/.local/bin/tvty`, Terminal Velocity
in your desktop's applications:

```bash
git clone https://github.com/quazardous/tvty && cd tvty
cargo build --release
make install-desktop BIN=$PWD/target/release/tvty
```

Each [release](https://github.com/quazardous/tvty/releases) also has a
built binary for x86_64 Linux (the libraries above still needed, without
`-dev`): put it anywhere and point `make install-desktop BIN=…` at it, or run
it as it is.

**First start.** `tvty` finds aiball over its local socket (`$AIBALL_SOCK`,
else aiball's default) and acts as its human user (`$TVTY_USER`, else the
human aiball saw last); Options > About says which socket and which user,
and whether the bus answered. `tvty SESSION` opens that tmux session at
start. aiball's web UI stays for everything else — remote, phone, another
OS.

One tvty runs per state directory (`~/.local/state/tvty`): launching it again
brings the running one forward, on the session asked for, and the second
launch ends there. A tvty with another state directory (`XDG_STATE_HOME`, as
the test environment sets it) stays apart; `TVTY_NEW_INSTANCE=1` opens one
more anyway. The ? menu's **Restart tvty** starts it afresh — after an update,
say — and the agents' sessions keep running.

## Tested headless

No human clicks through `tvty` to test it: an agent does, in
[**wbox**](https://github.com/quazardous/wbox-mcp) — a nested Wayland
compositor that runs without a screen. It builds, launches, types, drags,
screenshots and reads the pictures back, on a throwaway aiball whose agents
are [fake-claude](https://github.com/quazardous/aiball) and
[simai-cli](https://github.com/quazardous/simai-cli) replays: no model
token spent, nothing touched on the real board. The pictures above come out
of the same loop — `demo/run up && demo/run shots`.
See [`docs/TESTING.md`](docs/TESTING.md).

## Built on

| | |
|---|---|
| UI | [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) (Zed's toolkit) and gpui-kit's widgets — the UI draws itself, GNOME and KDE alike |
| Terminals | [`alacritty_terminal`](https://crates.io/crates/alacritty_terminal), the pairing of Zed's terminal, with a view of its own |
| Sessions | each terminal runs `tmux attach` on the agent's claude-loop session: tmux keeps the agent alive, `tvty` only shows it |
| Settings, shortcuts | [`crates/tvty-config`](crates/tvty-config) and [`crates/tvty-keys`](crates/tvty-keys): settings declared once, shortcuts as data — both free of the UI |

Linux for now; Windows and macOS are planned. The interface, decided, is in
[`docs/UX.md`](docs/UX.md).

## Files

`tvty` follows the XDG directories (`$XDG_CONFIG_HOME` and the others, with
their usual defaults):

| File | What |
|---|---|
| `~/.config/tvty/settings.toml` | your preferences: theme, sizes, notifications, scroll speed… — Options writes it, you may edit it; read again once saved |
| `~/.config/tvty/keymap.toml` | your shortcuts, over the defaults — or edit them in Options > Keyboard shortcuts |
| `~/.config/tvty/themes/` | colour themes of your own |
| `~/.local/state/tvty/layout.json` | the window's layout: panel and list widths, folded sections |
| `~/.local/state/tvty/workspace.json` | the terminals left open, opened again at start |
| `~/.local/state/tvty/tvty.log` | the log (the previous run's in `tvty.log.1`) |
| `~/.local/share/tvty/fonts/` | fonts of your own (`make emoji-font` fetches a colour emoji font) |

A file that does not read is said in a notification, and left as it is.

## License

[MIT](LICENSE).
