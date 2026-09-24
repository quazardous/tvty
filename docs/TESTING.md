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

**Never click a real agent's loop inside wbox**: attaching resizes its tmux
window to tvty's for every client, the user's own included.

## A board to click on

The ticket panel reads **and writes** aiball: accepting, rejecting and
replying act as the user. Never test them on the real board. `make aiball-up`
starts a throwaway daemon instead (`scripts/fake-aiball`): its own data dir,
config dir, port (7797) and socket under `dev/fake-aiball/`, seeded with a
`demo` project — two agents, `demo-claude` and `demo-crew`, whose loops are
the tmux sessions `cl-demo-lead` / `cl-demo-crew`, and tickets with a claim,
a pending plan, a pending resolution, a queue and a closed one.
`make aiball-down` drops it all.

Point tvty at it with `AIBALL_SOCK` (the API) and `AIBALL_URL` (its live
feed, served on the TCP port only); `scripts/fake-aiball env` prints both.
Give it its own `XDG_CONFIG_HOME` too, so a test run does not rewrite your
remembered panel width:

```bash
scripts/wbox_ctl.py up dev/tvty-wbox/config.yaml \
  -s "app.command=env $(scripts/fake-aiball env) XDG_CONFIG_HOME=/tmp/tvty-test $PWD/target/debug/tvty cl-demo-lead"
```

## Gestures wbox has no tool for

`scripts/wbox_ctl.py` adds them (`make help` for the make side):

- `scroll X Y N` — mouse wheel, N notches, negative = up;
- `drag X1 Y1 X2 Y2` — press, move, release (the panel's edge);
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
`scripts/wbox_ctl.py up dev/tvty-wbox/config.yaml -s "app.command=$PWD/target/release/tvty tvty-flood"`.

First numbers (24/09, labwc headless, Radeon 680M):

| | debug | release |
|---|---|---|
| keystroke → echo | 3–15 ms | 1–4 ms |
| sustained flood | ~37 fps | ~60 fps (the output's refresh) |
| grid preparation, worst per second | 35–60 ms | 6–29 ms |

## What the first slice measures

- keystroke → echo latency, and smoothness while an agent streams output;
- the same on GNOME and KDE (Wayland and X11), and on Windows (psmux).
