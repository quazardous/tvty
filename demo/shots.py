#!/usr/bin/env python3
"""The README's pictures, taken: tvty on the demo world (demo/run up), in a
wbox of its own, driven step by step; the pictures land in docs/images/.

Run through `demo/run shots`. The clicks are where tvty draws things on a
1600×1000 screen: a change of layout may move them — look at the pictures.
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

# Where things are, on a 1600×1000 screen (the window maximized).
TITLE_BAR = (800, 20)
ROW = {1: (1230, 375), 4: (1230, 318)}
SUPERLASER_TAB = (275, 56)
MORE = (1562, 100)
BACK_TO_LIST = (1135, 58)


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
            line = "screen: 1600x1000"
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

    OUT.mkdir(parents=True, exist_ok=True)
    for name in ("hero", "ticket", "slider", "gallery", "plan"):
        shutil.copy(SHOTS / f"{name}.png", OUT / f"{name}.png")
    # The projects' list alone: the left of the window.
    from PIL import Image
    Image.open(sessions).crop((0, 0, 700, 640)).save(OUT / "sessions.png")
    gif(frames, OUT / "tour.gif")
    wbox("down")
    print("\n".join(sorted(str(p.relative_to(ROOT)) for p in OUT.iterdir())))


if __name__ == "__main__":
    main()
