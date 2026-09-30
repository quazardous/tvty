#!/usr/bin/env python3
"""The README's pictures, taken: tvty on the demo world (demo/run up), in a
wbox of its own, driven step by step; the pictures land in docs/images/.

Run through `demo/run shots`. tvty is driven by name (scripts/tvty-ctl: a
ticket by its title, a tab by its session, a button by its id), so a change
of layout does not move the clicks; the keys, the window's size and the
captures go through it too, each capture taken once the screen holds still.
wbox only runs the compositor.
"""

import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEMO = ROOT / "dev" / "readme-demo"
CONFIG = DEMO / "wbox.yaml"
SHOTS = DEMO / "screenshots"
OUT = ROOT / "docs" / "images"
PYTHON = os.environ.get("WBOX_PYTHON", str(ROOT.parent / "wbox-mcp" / ".venv" / "bin" / "python3"))

SCREEN = (1920, 1200)
# What is clicked, by name: tickets by their title (demo/seed.py), the rest
# by id.
HERO_TICKET = "Cover the thermal exhaust port"
PLAN_TICKET = "Superlaser: charge in under 24 h"
SUPERLASER_TAB = "tab-cl-superlaser"
MORE = "thread-full"
BACK_TO_LIST = "back"
MENU = "help-button"
NEW_PROJECT = "New project…"
WIZARD_NEXT = "new-project-next"
# A folder the wizard is shown making a project of (left as it is: the
# wizard stops before setting it up).
FOLDER = "/tmp/tvty-demo/escape-pod"
# The id of "Negotiate with the star" in demo/seed.py.
STAR_TICKET = 11

def wbox(*args, env=None):
    subprocess.run([PYTHON, str(ROOT / "scripts" / "wbox_ctl.py"), args[0], str(CONFIG), *map(str, args[1:])],
                   check=True, capture_output=True, env={**os.environ, **(env or {})})


# The demo's tvty, as scripts/tvty-ctl names it (`wbox_config` writes its file).
INSTANCE = "readme"


def ctl(*args):
    """scripts/tvty-ctl on the demo's tvty: its answer."""
    done = subprocess.run([sys.executable, str(ROOT / "scripts" / "tvty-ctl"), "--instance", INSTANCE, *map(str, args)], capture_output=True, text=True)
    if done.returncode:
        sys.exit(f"tvty-ctl {' '.join(map(str, args))}: {(done.stderr or done.stdout).strip()}")
    return done.stdout


def shot(name):
    """The whole screen, once it holds still."""
    return Path(json.loads(ctl("shot", name))["path"])


def part(name, regions, margin=12):
    """The picture of those elements alone (the box around them), into
    docs/images/."""
    OUT.mkdir(parents=True, exist_ok=True)
    ctl("shot", "--region", "+".join(regions), "--margin", margin, "--out", OUT / f"{name}.png", f"{name}-part")


def click(name):
    """A click on the element of that id, once it is on screen."""
    ctl("wait", name, 8000)
    ctl("click", name)


def click_said(prefix, text):
    """A click on the element under `prefix` that says `text`: a ticket by
    its title, a menu's entry by its label."""
    for _ in range(40):
        if any(line.partition("\t")[2] == text for line in ctl("tree", "--text", prefix).splitlines()):
            break
        time.sleep(0.2)
    ctl("click", "--text", prefix, text)


def key(keys):
    ctl("key", keys)


def wait_for(prefix, seconds=10):
    """Until an element whose id starts with `prefix` is on screen."""
    for _ in range(seconds * 5):
        if json.loads(ctl("tree", prefix))["elements"]:
            return
        time.sleep(0.2)
    sys.exit(f"nothing named {prefix}… after {seconds} s")


def settings():
    """tvty as the pictures want it: a dark theme, the list folded, the
    critical agent's terminal shown."""
    config, state = DEMO / "tvty" / "config" / "tvty", DEMO / "tvty" / "state" / "tvty"
    shutil.rmtree(DEMO / "tvty", ignore_errors=True)
    config.mkdir(parents=True)
    state.mkdir(parents=True)
    # No "Did you know?" card over the pictures.
    (config / "settings.toml").write_text('[appearance]\ntheme = "Tokyo Night"\n\n[tips]\nshow = false\n')
    (state / "layout.json").write_text(json.dumps({"sidebar_open": False, "panel_open": True, "sessions_folded": ["idle", "shut"]}))
    (state / "workspace.json").write_text(json.dumps({
        "open_terminals": ["cl-exhaust-port", "cl-superlaser", "cl-panel-crew"], "shown_terminal": "cl-exhaust-port"}))


def wbox_config():
    """The tests' wbox, on a larger screen, running tvty on the demo."""
    text = (ROOT / "dev" / "tvty-wbox" / "config.yaml").read_text()
    lines = []
    for line in text.splitlines():
        if line.startswith("name:"):
            line = "name: tvty-readme"
        elif line.startswith("screen:"):
            line = f"screen: {SCREEN[0]}x{SCREEN[1]}"
        elif line.strip().startswith("command:"):
            line = "  command: sh -c 'r=$(git rev-parse --show-toplevel) && exec $r/demo/run tvty'"
        lines.append(line)
    CONFIG.write_text("\n".join(lines) + "\n")
    # What scripts/tvty-ctl --instance readme drives.
    instance = ROOT / "dev" / "tvty-ctl" / f"{INSTANCE}.json"
    instance.parent.mkdir(parents=True, exist_ok=True)
    instance.write_text(json.dumps({"state_home": str(DEMO / "tvty" / "state"), "wbox_config": str(CONFIG), "screenshots": str(SHOTS)}))


def gif(frames, out, seconds=2.2):
    """A slideshow of `frames`, each shown `seconds`, as a GIF with its own
    palette."""
    listing = DEMO / "frames.txt"
    listing.write_text("".join(f"file '{f}'\nduration {seconds}\n" for f in frames) + f"file '{frames[-1]}'\n")
    scale = "scale=1000:-1:flags=lanczos"
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", str(listing),
                    "-vf", f"{scale},split[a][b];[a]palettegen=stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=4",
                    str(out)], check=True)


def main():
    if not (DEMO / "home" / "sock").exists():
        sys.exit("the demo is not up: demo/run up")
    wbox_config()
    settings()
    wbox("down")
    wbox("up")
    ctl("ready", 30000)
    ctl("window", "maximize")
    frames = [shot("01-start")]

    click_said("ticket-", HERO_TICKET)
    frames.append(shot("hero"))
    click(MORE)
    frames.append(shot("ticket"))
    key("escape")

    # The slider lives while Ctrl is held.
    ctl("hold", "ctrl")
    key("ctrl-tab")
    frames.append(shot("slider"))
    key("escape")
    ctl("release")
    key("ctrl-shift-space")
    frames.append(shot("gallery"))
    key("escape")

    key("ctrl-shift-b")
    frames.append(shot("sessions"))
    # The list with the title bar above it, down to the last section's title.
    part("sessions", ["help-button", "sidebar-collapse", "sessions-shut-title"])
    key("ctrl-shift-b")

    click(SUPERLASER_TAB)
    click(BACK_TO_LIST)
    click_said("ticket-", PLAN_TICKET)
    frames.append(shot("plan"))

    click(MENU)
    # The pointer off the button: its tooltip would lie over the menu.
    ctl("park")
    frames.append(shot("menu"))
    part("menu", ["help-button", "help-menu"])
    click_said("help-", NEW_PROJECT)
    Path(FOLDER).mkdir(parents=True, exist_ok=True)
    ctl("wait", WIZARD_NEXT, 8000)
    wbox("type", FOLDER)
    click(WIZARD_NEXT)
    ctl("park")
    frames.append(shot("newproject"))
    part("newproject", ["new-project-card"], margin=20)
    click("new-project-close")

    # An agent answers: its words come as a notification.
    sys.path.insert(0, str(ROOT / "scripts"))
    from aiballbus import Board
    board = Board(str(DEMO / "home" / "sock"))
    board.call("star-diplomat", "message.post", {
        "project": "dyson-sphere", "kind": "comment_added", "ticket_id": STAR_TICKET, "parent_id": STAR_TICKET,
        "body": "The star accepts the window, on one condition: it faces the galaxy's good side. "
                "It also asks who will clean it.",
        "summary_until": "The star agrees to the window if it faces the galaxy's good side; asks who cleans it.",
        "commits": None, "handback": True})
    board.close()
    # Its notice comes once aiball has pushed the comment.
    wait_for("notice-")
    frames.append(shot("notice"))
    part("notice", ["notices"], margin=40)

    OUT.mkdir(parents=True, exist_ok=True)
    for name in ("hero", "ticket", "slider", "gallery", "plan"):
        shutil.copy(SHOTS / f"{name}.png", OUT / f"{name}.png")
    gif(frames, OUT / "tour.gif")
    wbox("down")
    print("\n".join(sorted(str(p.relative_to(ROOT)) for p in OUT.iterdir())))


if __name__ == "__main__":
    main()
