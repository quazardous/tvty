# Testing

tvty is a GUI in front of live agent sessions. Three tools let us test it
without touching the desktop and without spending model tokens.

| Tool | What it is | What it tests here |
|---|---|---|
| [wbox](../../wbox-mcp) | runs tvty in its own nested compositor, headless if wanted, and lets an agent click, type and screenshot inside it | the interface: tabs, gallery, slider, the ticket panel |
| `fake-claude` ([aiball](../../aiball/docs/FAKE-CLAUDE.md)) | a scripted (YAML) stand-in for Claude Code's screen: boot, prompt, `esc to interrupt`, streamed output | the whole chain with no tokens: a real claude-loop, in a real tmux session, through the real PTY proxy, shown by tvty — attach, tabs per agent, alerts |
| [simai-cli](../../simai-cli) | replays an agent CLI session with Claude's chrome from a script, and records it (`.cast`) | rendering and speed: dense, fast output replayed identically to measure smoothness; demo recordings |

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
- `make fake-down` — kills both.

They are not claude-loops on purpose: a loop registered for this directory
would clash with the agent's own. For the whole chain, start one elsewhere
with `claude-loop start --command "$(realpath ../aiball/bin/fake-claude)"`.

## What the first slice measures

- keystroke → echo latency, and smoothness while an agent streams output;
- the same on GNOME and KDE (Wayland and X11), and on Windows (psmux).
