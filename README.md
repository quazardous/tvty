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

![A tour: the terminal and its ticket, the ticket full screen, the slider, the gallery, the menu, a new project, a notification](docs/images/tour.gif)

| | |
|---|---|
| ![Projects and agents, with what waits for you](docs/images/sessions.png) | ![Every terminal, live, in one gallery](docs/images/gallery.png) |
| **Projects and agents**, each with what waits for you — working or idle, since when; ▶ runs on its own, ‖ held. | **The gallery** (ctrl+shift+space): every terminal live; type to filter. |
| ![The slider, a stack per project](docs/images/slider.png) | ![A plan waiting for a go, beside the agent that wrote it](docs/images/plan.png) |
| **The slider** (ctrl+tab): a stack per project, most recent first. | **A plan** waiting for your go, beside the agent that wrote it. |
| ![An agent's answer, quoted in a notification](docs/images/notice.png) | ![The menu, under the app's icon](docs/images/menu.png) |
| **A notification** quotes what the agent said; a click opens its project. | **The menu**, under the app's icon (F1): a new project, full screen (F11), the docs, restart. |

![A ticket full screen, its blueprint inline](docs/images/ticket.png)

![New project: a folder, who works in it, set up, what next](docs/images/newproject.png)

*The station, the sphere and the time machine are fictional: a demo world
replayed by [`demo/run`](demo/run), agents included.*

## Install

> **Beta.** tvty is for people who work with
> [aiball](https://github.com/quazardous/aiball) and its claude-loop agents:
> it is aiball's desktop client, not a standalone terminal. Linux only for
> now (Wayland or X11, GNOME and KDE; x86_64). Expect rough edges, and tell
> us (the menu under the app's icon > Report an issue).

**The easy way: Terminal Velocity Updater.** One small program installs
aiball when it is missing (through aiball's own installer), then Terminal
Velocity, puts both in your desktop's applications, and keeps them up to
date — what each has, the latest release, one button, and what it does,
line by line:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/quazardous/tvty/releases/latest/download/tvty-updater-installer.sh | sh
~/.local/bin/tvty-updater
```

![Terminal Velocity Updater: aiball and Terminal Velocity, where each stands, one button](docs/images/updater.png)

Later, **Updates…** in Terminal Velocity's menu opens it again; tvty says
itself when a newer release is out (Options > Layout > Updates turns that
off).

**You need** tmux 3.x, and the libraries every Wayland or X11 desktop has
(xkbcommon, Vulkan, fontconfig, freetype). aiball needs Node.js and git; the
updater says what is missing.

**Terminal Velocity alone**, without the updater, from the same release:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/quazardous/tvty/releases/latest/download/tvty-installer.sh | sh
```

**From source** — Rust (stable, 1.85 or later) and GPUI's system libraries:

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

git clone https://github.com/quazardous/tvty && cd tvty
cargo build --release
make install-desktop BIN=$PWD/target/release/tvty
```

**First start.** `tvty` finds aiball over its local socket (`$AIBALL_SOCK`,
else aiball's default) and acts as its human user (`$TVTY_USER`, else the
human aiball saw last); Options > About says which socket and which user,
and whether the bus answered. `tvty SESSION` opens that tmux session at
start. aiball's web UI stays for everything else — remote, phone, another
OS.

**A new project.** The menu's **New project** (under the app's icon, top left), or **+ project** atop the
sessions list, walks through it: pick a folder, name the project and its
agent, and aiball sets it up (its `project.init`), then says what comes next
(accepting aiball's MCP server in Claude) and starts its first session.

One tvty runs per state directory (`~/.local/state/tvty`): launching it again
brings the running one forward, on the session asked for, and the second
launch ends there. A tvty with another state directory (`XDG_STATE_HOME`, as
the test environment sets it) stays apart; `TVTY_NEW_INSTANCE=1` opens one
more anyway. The menu's **Restart tvty** starts it afresh — after an update,
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
| Sessions | each terminal attaches to the agent's session — on aiball's session host, or in tmux: the session keeps the agent alive, `tvty` only shows it |
| Settings, shortcuts | [`crates/tvty-config`](crates/tvty-config) and [`crates/tvty-keys`](crates/tvty-keys): settings declared once, shortcuts as data — both free of the UI |

Linux for now; Windows and macOS are planned. The interface, decided, is in
[`docs/UX.md`](docs/UX.md).

## Files

`tvty` follows the XDG directories (`$XDG_CONFIG_HOME` and the others, with
their usual defaults):

| File | What |
|---|---|
| `~/.config/tvty/settings.toml` | your preferences: theme, sizes, the terminals' transparency, notifications, scroll speed… — Options writes it, you may edit it; read again once saved |
| `~/.config/tvty/keymap.toml` | your shortcuts, over the defaults — or edit them in Options > Keyboard shortcuts |
| `~/.config/tvty/themes/` | colour themes of your own |
| `~/.local/state/tvty/layout.json` | the window's layout: its state (windowed and its size, maximized, full screen), panel and list widths, folded sections |
| `~/.local/state/tvty/workspace.json` | the terminals left open, opened again at start |
| `~/.local/state/tvty/tvty.log` | the log (the previous run's in `tvty.log.1`) |
| `~/.local/share/tvty/fonts/` | fonts of your own (`make emoji-font` fetches a colour emoji font) |

A file that does not read is said in a notification, and left as it is.

## License

[MIT](LICENSE).
