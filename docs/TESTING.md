# Testing

tvty is a GUI in front of live agent sessions. Three tools let us test it
without touching the desktop and without spending model tokens.

| Tool | What it is | What it tests here |
|---|---|---|
| [wbox](https://github.com/quazardous/wbox-mcp) | runs tvty in its own nested compositor, headless if wanted, and lets an agent click, type and screenshot inside it | the interface: tabs, gallery, slider, the ticket panel |
| `fake-claude` ([aiball](https://github.com/quazardous/aiball/blob/main/docs/FAKE-CLAUDE.md)) | a scripted (YAML) stand-in for Claude Code's screen: boot, prompt, `esc to interrupt`, streamed output | the whole chain with no tokens: a real claude-loop, in a real tmux session, through the real PTY proxy, shown by tvty — attach, tabs per agent, alerts |
| [simai-cli](https://github.com/quazardous/simai-cli) | replays an agent CLI session with Claude's chrome from a script, and records it (`.cast`) | rendering and speed: dense, fast output replayed identically to measure smoothness; demo recordings |

## wbox

Configured for this repo in `dev/tvty-wbox/` (labwc, headless, 1280x800).
Its MCP tools (`tvty-wbox`) reach a Claude session started **in this
directory**; `scripts/wbox_ctl.py` drives the same instance from anywhere,
and `make check` chains build → launch → screenshot → stop.

The MCP wiring is each machine's own (`.mcp.json` is not in the repo). For
an agent working here, next to aiball's own server:

```json
{
  "mcpServers": {
    "aiball": { "command": "aiball-mcp" },
    "tvty-wbox": { "type": "stdio", "command": "wbox-mcp", "args": ["serve", "dev/tvty-wbox/config.yaml"] }
  }
}
```

### On Windows

`dev/tvty-wbox-win/` is the same for Windows. wbox has no compositor
there; it isolates the test tvty in **Windows Sandbox** instead (wbox 0.9
and later, its `docs/windows.md`): a throwaway Windows whose pointer,
keyboard and clipboard are its own, its window minimized (`headless`).
Nothing of it touches the desktop, and it cannot see the user's tvty or
aiball. It needs Windows Sandbox turned on — wbox's `setup.ps1` does it,
then a reboot. No `make`:

```bash
W=path/to/wbox-mcp/.venv/Scripts/python.exe
$W scripts/wbox_ctl.py up   dev/tvty-wbox-win/config.yaml
$W scripts/wbox_ctl.py shot dev/tvty-wbox-win/config.yaml --name check
$W scripts/wbox_ctl.py down dev/tvty-wbox-win/config.yaml
```

The first `up` boots the sandbox (about 15 seconds); it stays up between
launches (`keep`), so `down`, `cargo build`, `up` costs a relaunch only.
`down` ends tvty, not the sandbox: closing its window ends it. The sandbox
sees `target/debug` read-only and `dev/tvty-wbox-win/home` read-write — the
test tvty's settings and its log (`home/state/tvty/tvty.log`).

A fresh Windows has what tvty needs only because tvty brings it: the C
runtime linked in (`.cargo/config.toml`) and, in a debug build, GPUI's
shaders compiled at build time (`Cargo.toml`), which needs the Windows
SDK's `fxc.exe` at build time.

`scripts/test-tvty.ps1` runs a test tvty on the desktop instead, apart from
the user's (its own settings, never their aiball, its own psmux server), for
when the sandbox is not there.

For now tvty there has no aiball (its socket is Unix only), no session
host and no multiplexer: the window, its settings and its panels.

**The Windows setup, whole.** `scripts/sandbox_install_test.py` (under
wbox's interpreter, as `wbox_ctl.py`) boots a fresh sandbox *with* a network
(`dev/tvty-wbox-win/install.yaml`), installs winget in it (a sandbox has
none), and runs `packaging/windows/tvty-setup.ps1` as a user does, through
`iex`: the prerequisites, tvty and its updater, aiball. `--from
target/distrib` makes it install a local build (`dist build --artifacts=all
--target x86_64-pc-windows-msvc`) instead of the published release;
`--aiball-ref REF` makes it install aiball at a tag or a branch rather than
its latest release (the updater's `TVTY_AIBALL_REF`), to try a fix of
aiball before it is released. It
prints the steps as they come and leaves in
`dev/tvty-wbox-win/home/install-test`: `out.txt` (everything said),
`setup.log` (the script's own log), and `desk.png`, **the sandbox's whole
desktop** — a window's screenshot does not show an error dialog another
program raised; this one does. About ten minutes.

## Something to attach to

`scripts/fake-loop` starts plain tmux sessions, no tokens spent:

- `make fake-up` — `tvty-fake`, aiball's fake-claude;
- `make sim-up` — `tvty-sim`, simai-cli replaying `dev/sim/dense.txt`
  (300 long lines with wide glyphs and emoji) as fast as it can;
- `make flood-up` — `tvty-flood`, which pours `dense.txt` at full speed for
  5 s each time Enter is pressed in it: the stress test;
- `make fake-down` — kills them all.

tvty lists them under "tmux", beside the agents' loops. They are not
claude-loops on purpose: a loop registered for this directory
would clash with the agent's own. For the whole chain, start one elsewhere
with `claude-loop start --command "$(realpath ../aiball/bin/fake-claude)"`.

## A test tvty lives apart

Everything that tests tvty goes through `scripts/test-env`, so that a test
never shows in — nor touches — the user's own tvty:

- **its own tmux server** (`TMUX_TMPDIR=dev/tmux`): the fake sessions and
  the demo loops run there, so the user's tvty does not list them, and a
  test tvty cannot even see a real agent's loop — let alone attach it,
  which would resize it for every client, the user's own included. psmux
  (Windows) ignores `TMUX_TMPDIR`: there `scripts/test-tvty.ps1` sets
  `TVTY_MUX_SERVER=tvty-test`, and tvty then passes `-L tvty-test` to every
  psmux call (`psmux -L tvty-test ls` to look at it);
- **its own settings** (`XDG_CONFIG_HOME=dev/tvty-wbox/config`) **and
  memory** (`XDG_STATE_HOME=dev/tvty-wbox/state`, the projects' list
  folded to start with): the user's `settings.toml`, `keymap.toml`, layout
  and workspace are never read nor written;
- **the throwaway aiball** below (`AIBALL_SOCK`), never the
  user's board.
- **its own fonts** (`XDG_DATA_HOME=dev/tvty-wbox/data`): for colour emoji,
  `XDG_DATA_HOME=dev/tvty-wbox/data make emoji-font` once;
- **its own loops**: `CLAUDE_LOOP_STATE_ROOT=dev/claude-loop` and
  `CL_CLAUDE_CMD` set to fake-claude, so a loop a test starts (the idle
  tab, "+ session") keeps its state apart and costs no token; start them
  in a directory under `dev/loops/`.

`scripts/test-tvty [SESSION]` runs tvty so (starting the throwaway aiball
if it is not up); it is wbox's command. To look at the test tmux server
from a shell: `TMUX_TMPDIR=dev/tmux tmux ls` (with `TMUX` unset, which
wins otherwise).

To show the user something, post a wbox screenshot on the ticket: never a
test session on their tmux server.

## A board to click on

The ticket panel reads **and writes** aiball: accepting, rejecting and
replying act as the user. Never test them on the real board. `make aiball-up`
starts a throwaway daemon instead (`scripts/fake-aiball`): its own data dir,
config dir, port (7797) and socket under `dev/fake-aiball/`, seeded with a
`demo` project — two agents, `demo-claude` and `demo-crew`, whose loops are
the tmux sessions `cl-demo-lead` / `cl-demo-crew`, and tickets with a claim,
a pending plan, a pending resolution, a queue and a closed one.
`make aiball-down` drops it all: the sessions its host runs, the test loops
(`dev/claude-loop`) and what runs in them go too, and `up` first stops what a
daemon that fell left behind. `make fake-down` stops the test tmux server once
it is empty. Leftovers are found by command line or environment and stopped
by pid, never by a kill by pattern.

To act on it as aiball's clients do, `scripts/aiball-call METHOD [PARAMS]`
makes one call on its bus (standard library only), as the user or, with
`--as`, as an agent. The seeds (`scripts/fake-aiball-seed.py`,
`scripts/fake-aiball-load.py`, `demo/seed.py`) speak the bus too, through
`scripts/aiballbus.py`: the identity is the connection's, so a seed opens
one connection per consumer it acts as; only uploads stay HTTP. A loop on the host, running fake-claude, then held ten
minutes, as the AFK chip does:

```bash
export AIBALL_SOCK=$PWD/dev/fake-aiball/home/sock
scripts/aiball-call session.start "{\"cwd\":\"$PWD/dev/fake-aiball/demo/crew\",\"agent\":\"demo-crew\"}"
scripts/aiball-call consumer.afk '{"name":"demo-crew","action":"arm_10m"}'
```

The test tvty points at it already (`AIBALL_SOCK`: the API and its live
feed). To open a session at
start:

```bash
scripts/wbox_ctl.py up dev/tvty-wbox/config.yaml -s "app.command=$PWD/scripts/test-tvty cl-demo-lead"
```

## Behind a proxy node

aiball spans machines: a node in proxy mode relays `/api` and the bus to
its hub. `scripts/proxy-stack up` (`make proxy-up`) runs two of them in
containers (`dev/proxy-stack/compose.yaml`), built from the aiball
checkout beside tvty with aiball's own test images:

- **hub**, machine A: the core, with the demo board seeded (the same as
  the throwaway aiball's), on 127.0.0.1:17797;
- **node**, machine B: in proxy mode, its `config.yaml` naming the hub and
  a node token the script issued on it, on 127.0.0.1:17798. Its socket is
  `dev/proxy-stack/run/node/sock/sock`.

The two talk only over the compose network, with the token, as two
machines would. `TVTY_AIBALL=proxy make wbox-up` runs the test tvty on the
node: what a tvty on machine B sees. `up --loop` adds a real claude-loop on
fake-claude behind the node (the image of aiball's full-stack tests).
`make proxy-logs` follows the daemons; `make proxy-down` stops it all and
removes its data (`dev/proxy-stack/run`). The containers run as the user's
uid, the source read-only.

## Driving tvty by name

A test tvty — a development build (`debug_assertions`; a release build has
no such control), with `TVTY_DEBUG_CONTROL=1` as `scripts/test-env` sets it
(the user's tvty never has it) — listens in its state directory, through
`tvty-ipc` as the instance's rendez-vous does: on `tvty-control.sock` where
there are Unix sockets, on Windows on a loopback port written with a secret
in `tvty-control.addr` (the client says the secret first;
`TVTY_CONTROL_TCP=1`, for tvty and the client, takes that way on Linux too,
to test it). `scripts/tvty-ctl` speaks to it, on either — no coordinates to guess, no sleep to
hope for:

- `tvty-ctl tree [PREFIX]` — what is on screen, by id, with where it is
  painted: **every element that has an id**. The buttons, chips and links,
  the sessions' rows (`terminal-NAME`), the tabs (`tab-NAME`,
  `sidebar-tab-workspaces`, `edit-body-preview`), the projects' headings
  (`heading-NAME`), the tickets' rows (`ticket-N`), a section's title
  (`band-Open-title`), a thread's entries (`entry-N`), the combos
  (`new-intent`, `field-combo`, `options-scope`). In the code an element
  takes its id with `.named(id)` (`ui::Named`) where GPUI's `.id(id)`
  would be: that is all it takes to be seen and clicked by name. A kit
  widget with an id of its own (a `Select`) gets
  `.children(crate::inspect::mark_if(id))` on the element around it. GPUI
  keeps no list of its elements a program can read (its inspector's is
  private, and filled only while picking): an element with no id is not
  known;
- `tvty-ctl click ID`, `hover ID`, `key KEYS` (`ctrl-enter`, `escape`),
  `type TEXT` — put in through the window, as the user's would: the same on
  Windows, where wbox is not; `where ID` gives the middle, for wbox's real
  pointer when the platform must see it (a tooltip);
- `tvty-ctl wait ID [MS]` — until ID is painted;
- `tvty-ctl press ID` — a press never released (let go outside the
  window), and `tvty-ctl selection` — the text selected in the window: a
  stuck selection is provoked and read without a screenshot;
- `tvty-ctl state` — the page, the terminal, the panel (project, ticket,
  field being edited, menu open), the bus and its subscriptions;
- `tvty-ctl inspector` — GPUI's inspector (debug builds; its key is
  ctrl+alt+shift+i): pick an element, read its id and where it is made;
- failure paths, provoked: `tvty-ctl bus-reconnect` drops the bus and
  connects again; `tvty-ctl fault Tickets` makes that subscription's next
  asking fail once — `state` then shows it failed, its retry, and live
  again (a failed subscription is read whole, never resumed on another's
  cursor).

The screenshot stays the proof of what is seen; this is how to get there,
and how to read what a screenshot does not say.

## Gestures wbox has no tool for

`scripts/wbox_ctl.py` adds them (`make help` for the make side):

- `scroll X Y N` — mouse wheel, N notches, negative = up;
- `drag X1 Y1 X2 Y2` — press, move, release (a side's edge — not its grip,
  which folds it); `WBOX_DRAG_STEPS=60` moves in 60 short steps, closer to
  a hand;
- `hold MODIFIER KEY TIMES --name NAME` — hold the modifier, tap the key,
  screenshot, then release: how the slider (ctrl+tab) is seen while up.

Reading the real board is fine (a screenshot of the alerts); a click on a
real ticket marks it read for the user, so don't.

## What the desktop decides

`make desktop-check` (`scripts/desktop-check`) looks at what the compositor
decides about tvty's window, under labwc in wbox — labwc announces the
protocols KDE's KWin uses for these (xdg-decoration, KDE server decoration,
xdg-activation):

1. tvty draws its own frame: the compositor granted client decorations
   (tvty logs `window: decorations client|server`);
2. its window is the active one (`wbox_ctl windows`, the compositor's own
   list);
3. with the terminals see-through and "Blur behind" on, it asks for a
   blurred background (`window: background Blurred` in its log).

A second launch does not bring the window forward under Wayland — GPUI
cannot use the launcher's activation token: said as `KNOWN`, not failed.
The check restores the test settings it changes. CI runs it (the `desktop`
job) with `TVTY_NO_AIBALL=1`: the test tvty then starts without the fake
aiball, which a runner does not have. KWin itself (the blur drawn) is left
to a KWin backend of wbox, when one exists.

## The README's pictures

`demo/run up` builds a small fictional world apart from everything else —
its own aiball (port 7798, `dev/readme-demo`), tmux server and tvty
settings: three projects (a battle station, a Dyson sphere, a time
machine), their agents present on the bus (`demo/presence.py`), tickets in
every state with pictures (`demo/seed.py`, `demo/images.py`), and each
agent's terminal replaying `demo/scenes/<agent>.txt` with simai-cli in
`/tmp/tvty-demo`. `demo/run shots` (`make readme-shots`) runs tvty on it in
a wbox of its own (1920×1200), drives it and writes `docs/images/`;
`demo/run film` drives it again while grim films the wbox's display, about
ten frames a second, into `docs/images/tvty.gif` — the README's film, one
agent answering live. `demo/run down` removes it all. Their clicks are
positions: after a change of layout, look at the pictures.

## Measuring

With `TVTY_STATS=<file>` (set in the wbox config, to
`dev/tvty-wbox/log/tvty-stats.log`), tvty appends:

- `echo <ms>` — keystroke sent to the PTY → first frame painted after the
  PTY answered;
- `reflow <ms> (sent after <ms>)` — the terminal's view took a new size
  (a side opened or closed) → first frame painted after the program
  answered the new size; and how much of it went before the size was sent
  (none for a size alone; the settle while a side is dragged);
- `stream <n> fps <m> wakeups, prepaint max <ms>` — each second the PTY
  talked. The second is a full second: a burst shorter than that reads low,
  so measure on a sustained stream (`tvty-flood`).

- `window <n> frames, max <ms>, total <ms>, <view> <n>× max <ms>…` — each
  second the window was drawn: every frame, from the frame view's render to
  its last paint, then how often each view rendered and the phases (`tree`,
  `layout+prepaint`, `paint`).

A board the size of a real one: `scripts/fake-aiball-load.py
$PWD/dev/fake-aiball/home/sock 300` files 300 tickets on the throwaway
aiball (it refuses any other socket). Then `scripts/perf-frames` launches
the test tvty, selects a live loop and reports three moments — at rest,
the pointer gliding over the ticket list, typing in the terminal — as
frames, the slowest and the mean, and the CPU tvty used.

To start tvty on one session, without clicking:
`scripts/wbox_ctl.py up dev/tvty-wbox/config.yaml -s "app.command=$PWD/scripts/test-tvty tvty-flood"`.

First numbers (24/09, labwc headless, Radeon 680M):

| | debug | release |
|---|---|---|
| keystroke → echo | 3–15 ms | 1–4 ms |
| sustained flood | ~37 fps | ~60 fps (the output's refresh) |
| grid preparation, worst per second | 35–60 ms | 6–29 ms |

On 312 tickets (26/09, debug, labwc headless), `scripts/perf-frames`:

| | before | ticket rows: fixed height, only those in view | + panel and terminal reused, sessions rows fixed |
|---|---|---|---|
| at rest (8 s) | 34 frames, mean 108 ms, 3.9 s of CPU | 34 frames, mean 18 ms, 0.7 s | 32 frames, mean 7.8 ms, 0.32 s |
| hover (4 s) | 49 frames, mean 108 ms, 5.7 s | 58 frames, mean 23 ms, 1.5 s | 59 frames, mean 9.5 ms, 0.68 s |
| typing (20 keys) | 32 frames, mean 122 ms, 3.6 s | 43 frames, mean 22 ms, 0.9 s | 44 frames, mean 9.9 ms, 0.48 s |

The frames "at rest" are the test loop's own: its spinner and tmux's clock.
What is left of a frame is mostly the terminal's paint (about 5 ms).

Two things GPUI does that shape tvty's views: a view's notify marks all
its ancestors dirty, and a *cached* view that draws again makes every view
inside it draw again. So the shell is drawn plainly (its render is cheap),
the terminal and the ticket panel are cached inside it — reused whenever
they did not change — and anything the shell lays out itself on every
frame (the sessions list, the bars) keeps fixed heights.

## What the first slice measures

- keystroke → echo latency, and smoothness while an agent streams output;
- the same on GNOME and KDE (Wayland and X11), and on Windows (psmux).
