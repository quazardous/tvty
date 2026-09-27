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
  which would resize it for every client, the user's own included;
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
`--as`, as an agent. A loop on the host, running fake-claude, then held ten
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

## The README's pictures

`demo/run up` builds a small fictional world apart from everything else —
its own aiball (port 7798, `dev/readme-demo`), tmux server and tvty
settings: three projects (a battle station, a Dyson sphere, a time
machine), their agents present on the bus (`demo/presence.py`), tickets in
every state with pictures (`demo/seed.py`, `demo/images.py`), and each
agent's terminal replaying `demo/scenes/<agent>.txt` with simai-cli in
`/tmp/tvty-demo`. `demo/run shots` (`make readme-shots`) runs tvty on it in
a wbox of its own (1600×1000), drives it and writes `docs/images/`;
`demo/run down` removes it all. Its clicks are positions: after a change of
layout, look at the pictures.

## Measuring

With `TVTY_STATS=<file>` (set in the wbox config, to
`dev/tvty-wbox/log/tvty-stats.log`), tvty appends:

- `echo <ms>` — keystroke sent to the PTY → first frame painted after the
  PTY answered;
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
