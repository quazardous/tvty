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
- **its own settings** (`XDG_CONFIG_HOME=dev/tvty-wbox/config`, the
  projects' list folded to start with): the user's `state.json` is never
  read nor written;
- **the throwaway aiball** below (`AIBALL_SOCK`, `AIBALL_URL`), never the
  user's board.
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
`make aiball-down` drops it all.

The test tvty points at it already (`AIBALL_SOCK` for the API, `AIBALL_URL`
for its live feed, a fallback when the socket does not serve it). To open a session at
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

## Measuring

With `TVTY_STATS=<file>` (set in the wbox config, to
`dev/tvty-wbox/log/tvty-stats.log`), tvty appends:

- `echo <ms>` — keystroke sent to the PTY → first frame painted after the
  PTY answered;
- `stream <n> fps <m> wakeups, prepaint max <ms>` — each second the PTY
  talked. The second is a full second: a burst shorter than that reads low,
  so measure on a sustained stream (`tvty-flood`).

To start tvty on one session, without clicking:
`scripts/wbox_ctl.py up dev/tvty-wbox/config.yaml -s "app.command=$PWD/scripts/test-tvty tvty-flood"`.

First numbers (24/09, labwc headless, Radeon 680M):

| | debug | release |
|---|---|---|
| keystroke → echo | 3–15 ms | 1–4 ms |
| sustained flood | ~37 fps | ~60 fps (the output's refresh) |
| grid preparation, worst per second | 35–60 ms | 6–29 ms |

## What the first slice measures

- keystroke → echo latency, and smoothness while an agent streams output;
- the same on GNOME and KDE (Wayland and X11), and on Windows (psmux).
