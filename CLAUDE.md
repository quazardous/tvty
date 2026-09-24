# tvty — agent guide

**Terminal Velocity** (`tvty`): a native terminal for working with many AI
agents — their terminals grouped by project, each project's tickets in a panel
beside. Read [`README.md`](./README.md) (what and why), [`docs/UX.md`](./docs/UX.md)
(the interface, decided) and [`docs/TESTING.md`](./docs/TESTING.md) before
touching code.

## You work alone

You develop **and test** tvty without a human in the loop: build, launch, look,
fix, again. Never ask david to "try it" — show him a screenshot you took.

- Stack (decided): Rust, **GPUI** (Zed's UI toolkit) + gpui-kit widgets,
  `alacritty_terminal` for emulation. Each terminal runs `tmux attach` (Linux)
  or `psmux attach` (Windows) on an agent's claude-loop session.
- aiball is the engine: tvty only talks to its API. aiball's source is next
  door (`../aiball`); change it through a ticket on its project, not from here.

## The test loop — wbox

tvty runs inside **wbox**, a nested compositor that is **headless by default**:
nothing appears on david's desktop. Two ways to drive it, same instance:

| MCP tools (this session) | make | |
|---|---|---|
| `mcp__tvty-wbox__kill` | `make wbox-down` | stop |
| `mcp__tvty-wbox__build` | `make build` | build — never over a running binary |
| `mcp__tvty-wbox__launch` | `make wbox-up` | start tvty in the compositor |
| `mcp__tvty-wbox__screenshot` | `make wbox-shot NAME=..` | look (then read the PNG) |
| `click`, `type_text`, `key` | `make wbox-click X= Y=`, `make wbox-key K=` | drive it |

`make check` does build → launch → screenshot → stop in one go.
`WBOX_VISIBLE=1 make wbox-up` gives a real window, only when you must *look*
at motion. Screenshots land in `dev/tvty-wbox/screenshots/`.

A reply saying "ok" proves nothing about a GUI: **the screenshot is the proof.**

## Something to attach to, without tokens

- **fake-claude** (aiball): a scripted stand-in for Claude Code's screen.
  A real loop on it:
  `claude-loop start --command "$(realpath ../aiball/bin/fake-claude)"`
  — or a bare session: `tmux new-session -d -s tvty-fake ../aiball/bin/fake-claude`.
- **simai-cli** (`simcli`, `../simai-cli`): replays a Claude-looking session
  from a script and records `.cast` — use it to stress rendering and measure.
- `make fake-up` / `sim-up` / `flood-up` / `fake-down` wrap them
  (`scripts/fake-loop`, tmux sessions `tvty-fake`, `tvty-sim`, `tvty-flood`).
  tvty lists them under "tmux". **Never click a real agent's loop in wbox**:
  attaching resizes it for every client, david's included.
- **fake aiball** (`make aiball-up`): a throwaway daemon with a `demo`
  project, for the ticket panel. Its gestures act as david: never try them
  on the real board.

## Conventions

- Code, comments, commits and docs in **English**; talk to david in
  **French** (chat and ticket threads).
- Commit locally as you go; **never push** (no remote yet) or publish without
  david's go.
- Tickets live on the `tvty` project of aiball — use the aiball skill.
