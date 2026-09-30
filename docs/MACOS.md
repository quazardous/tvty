# macOS — help wanted

Terminal Velocity does not run on macOS yet, and nobody is working on it:
the project is developed and tested on Linux (in a nested compositor) and
on Windows, and **has no Mac to build on, run on, or look at**. The
milestones below are what a macOS version takes; none is started.

If you have a Mac and want it there, this is where to help:
[open an issue](https://github.com/quazardous/tvty/issues) saying which
milestone you take, or just try the first one and say what breaks.

## What already fits

- The UI toolkit, [GPUI](https://www.gpui.rs), is native on macOS — it is
  where it comes from. The terminal emulation (`alacritty_terminal`) runs
  there, and tmux installs with Homebrew.
- tvty's code has few branches by system, nearly all "Windows or Unix":
  macOS takes the Unix one.
- [aiball](https://github.com/quazardous/aiball), the engine, runs on macOS
  as best effort: its daemon, its command and its loops work; it ships no
  launchd unit, so the daemon is started by hand.

## Milestones

1. **It builds.** `cargo build` on `aarch64-apple-darwin` (and
   `x86_64-apple-darwin` for Intel Macs), the unit tests green, a macOS
   runner in CI. The patched GPUI crate under `vendor/` is the Linux
   platform's alone and is not in the way.
2. **It runs, by hand.** tvty started from a terminal against a running
   aiball: the sessions' list, a terminal through tmux, the ticket panel.
   What to look at first: the window's buttons and title bar, the fonts'
   fallbacks, what the clipboard does (there is no primary selection).
3. **It feels like a Mac application.** The shortcuts where macOS expects
   Cmd rather than Ctrl, the menu bar, the application's icon, its state
   and settings in the folders macOS uses.
4. **The updater.** What is missing named with its Homebrew command (git,
   Node.js, tmux; Claude Code by its own installer), and tvty put in place
   as an application (`.app`) rather than a desktop launcher.
5. **A release.** The macOS targets in the release workflow, with the
   shell installer. Signing and notarization need an Apple developer
   account: without them macOS refuses to open a downloaded application
   unless the user overrides it, which the install guide would then say.
6. **aiball at login.** A launchd unit on aiball's side, so that the
   daemon is there after a restart — a ticket for aiball, not for tvty.

## How to test one

[`docs/TESTING.md`](./TESTING.md) describes the loop tvty is developed
with. Its nested compositor is Linux's; on a Mac the window is simply the
desktop's. The debug control (`scripts/tvty-ctl`) is not tied to Linux: a
development build started with `TVTY_DEBUG_CONTROL=1` is read and driven
by name there too.
