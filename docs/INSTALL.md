# Installing Terminal Velocity

> **Beta.** Terminal Velocity runs on
> [aiball](https://github.com/quazardous/aiball), the engine that keeps your
> Claude agents going (their loops, board and tickets): it is aiball's
> desktop client, not a standalone terminal. Linux for
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

![Terminal Velocity Updater: aiball and Terminal Velocity, where each stands, one button](images/updater.png)

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


## Updates

Terminal Velocity says itself, once, when a newer release is out (at start
and once a day; Options > Layout > Updates turns that off), and at start
when aiball is older than it needs. **Updates…** in its menu (under the
app's icon) opens Terminal Velocity Updater, which updates aiball its own way
(`aiball update`) and Terminal Velocity from its release, the version in
place kept to go back to.

## First start

 `tvty` finds aiball over its local socket (`$AIBALL_SOCK`,
else aiball's default) and acts as its human user (`$TVTY_USER`, else the
human aiball saw last); Options > About says which socket and which user,
and whether the bus answered. `tvty SESSION` opens that tmux session at
start. aiball's web UI stays for everything else — remote, phone, another
OS.


## A new project

 The menu's **New project** (under the app's icon, top left), or **+ project** atop the
sessions list, walks through it: pick a folder, name the project and its
agent, and aiball sets it up (its `project.init`), then says what comes next
(accepting aiball's MCP server in Claude) and starts its first session.

One tvty runs per state directory (`~/.local/state/tvty`): launching it again
brings the running one forward, on the session asked for, and the second
launch ends there. A tvty with another state directory (`XDG_STATE_HOME`, as
the test environment sets it) stays apart; `TVTY_NEW_INSTANCE=1` opens one
more anyway. The menu's **Restart tvty** starts it afresh — after an update,
say — and the agents' sessions keep running.


## Uninstall

Remove `~/.local/bin/tvty`, `~/.local/bin/tvty-updater` and their launchers
(`~/.local/share/applications/tvty*.desktop`); your settings stay in
`~/.config/tvty` and `~/.local/state/tvty`. aiball uninstalls its own way
(`./install.sh --uninstall` in its checkout).
