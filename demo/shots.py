#!/usr/bin/env python3
"""The README's pictures, taken: tvty on the demo world (demo/run up), in a
wbox of its own, driven step by step; the pictures land in docs/images/.

Run through `demo/run shots`. The clicks are where tvty draws things on a
1920×1200 screen: a change of layout may move them — look at the pictures.
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
# Where things are, on that screen: the window first as it opens, then
# maximized.
TITLE_BAR = (800, 72)
ROW = {1: (1450, 375), 4: (1450, 318)}
SUPERLASER_TAB = (300, 56)
MORE = (1884, 100)
BACK_TO_LIST = (1358, 58)
MENU = (28, 20)
NEW_PROJECT = (100, 99)
WIZARD_NEXT = (1237, 919)
# A folder the wizard is shown making a project of (left as it is: the
# wizard stops before setting it up).
FOLDER = "/tmp/tvty-demo/escape-pod"
# The id of "Negotiate with the star" in demo/seed.py.
STAR_TICKET = 11

def wbox(*args, env=None):
    subprocess.run([PYTHON, str(ROOT / "scripts" / "wbox_ctl.py"), args[0], str(CONFIG), *map(str, args[1:])],
                   check=True, capture_output=True, env={**os.environ, **(env or {})})


def shot(name, wait=1.5):
    time.sleep(wait)
    wbox("shot", "--name", name)
    return SHOTS / f"{name}.png"


def click(at, wait=1.2):
    wbox("click", *at)
    time.sleep(wait)


def key(shortcut, wait=1.2):
    wbox("key", shortcut)
    time.sleep(wait)


def settings():
    """tvty as the pictures want it: a dark theme, the list folded, the
    critical agent's terminal shown."""
    config, state = DEMO / "tvty" / "config" / "tvty", DEMO / "tvty" / "state" / "tvty"
    shutil.rmtree(DEMO / "tvty", ignore_errors=True)
    config.mkdir(parents=True)
    state.mkdir(parents=True)
    (config / "settings.toml").write_text('[appearance]\ntheme = "Tokyo Night"\n')
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
    time.sleep(7)
    wbox("dblclick", *TITLE_BAR)
    frames = [shot("01-start")]

    click(ROW[1])
    frames.append(shot("hero"))
    click(MORE, wait=1.5)
    frames.append(shot("ticket"))
    key("Escape")

    wbox("hold", "ctrl", "Tab", 1, "--name", "slider", env={"WBOX_HOLD_CANCEL": "1", "WBOX_HOLD_WAIT": "1.2"})
    frames.append(SHOTS / "slider.png")
    key("ctrl+shift+space", wait=2)
    frames.append(shot("gallery"))
    key("Escape")

    key("ctrl+shift+b")
    sessions = shot("sessions")
    frames.append(sessions)
    key("ctrl+shift+b")

    click(SUPERLASER_TAB)
    click(BACK_TO_LIST)
    click(ROW[4])
    frames.append(shot("plan"))

    click(MENU)
    menu = shot("menu")
    frames.append(menu)
    click(NEW_PROJECT)
    Path(FOLDER).mkdir(parents=True, exist_ok=True)
    wbox("type", FOLDER)
    click(WIZARD_NEXT)
    wizard = shot("newproject")
    frames.append(wizard)
    # Once out of the field, once out of the wizard.
    key("Escape")
    key("Escape")

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
    notice = shot("notice", wait=2.5)
    frames.append(notice)

    OUT.mkdir(parents=True, exist_ok=True)
    for name in ("hero", "ticket", "slider", "gallery", "plan"):
        shutil.copy(SHOTS / f"{name}.png", OUT / f"{name}.png")
    # The parts that tell: the projects' list, the menu, the wizard, the notice.
    from PIL import Image
    Image.open(sessions).crop((0, 0, 700, 640)).save(OUT / "sessions.png")
    Image.open(menu).crop((0, 0, 720, 440)).save(OUT / "menu.png")
    Image.open(wizard).crop((600, 265, 1320, 935)).save(OUT / "newproject.png")
    Image.open(notice).crop((560, 40, 1440, 330)).save(OUT / "notice.png")
    gif(frames, OUT / "tour.gif")
    wbox("down")
    print("\n".join(sorted(str(p.relative_to(ROOT)) for p in OUT.iterdir())))


if __name__ == "__main__":
    main()
