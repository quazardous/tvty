# Terminal Velocity

**Twenty AI agents. One window. No tab hunting.**

*Beta — for [aiball](https://github.com/quazardous/aiball) users; Linux.*

![Terminal Velocity at work: an agent answers live beside its ticket; the slider, the gallery, a notification, a ticket full screen](docs/images/tvty.gif)

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

## Quick start

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/quazardous/tvty/releases/latest/download/tvty-updater-installer.sh | sh
~/.local/bin/tvty-updater
```

Terminal Velocity Updater installs aiball when it is missing, then Terminal
Velocity, puts both in your desktop's applications, and keeps them up to
date. Press **Launch Terminal Velocity**: it finds aiball on its own, and the
menu under its icon (F1) makes a new project.

You need Linux (Wayland or X11, x86_64) and tmux 3.x; aiball needs Node.js
and git.

## Docs

- [Install](docs/INSTALL.md) — the updater, tvty alone, from source; first start, a new project, updates
- [Guide](docs/GUIDE.md) — a tour in pictures, what it is built on, its files, how it is tested
- [Interface](docs/UX.md) — every surface and shortcut, as decided
- [Testing](docs/TESTING.md) — headless, in wbox, on a fake aiball
- [Changelog](CHANGELOG.md)

## License

[MIT](LICENSE).
