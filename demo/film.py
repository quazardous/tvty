#!/usr/bin/env python3
"""The README's film: tvty on the demo world (demo/run up), filmed while it
is driven — an agent answering live, the slider, the gallery, a
notification, a ticket full screen — as docs/images/tvty.gif.

Run through `demo/run film`, after `demo/run shots` has set the scene up
once (the same wbox, the same clicks). wbox's own screenshot is too slow to
film: grim reads its compositor directly, about ten times a second.
"""

import os
import shutil
import subprocess
import sys
import threading
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import shots  # noqa: E402
from shots import DEMO, OUT, ROOT, click, ctl, key, wbox  # noqa: E402

FRAMES = DEMO / "film"
FPS = 10
WIDTH = 1280
SIMCLI = ROOT.parent / "simai-cli" / "dist" / "main.js"


def display():
    """The wbox's Wayland display, as it saved it."""
    import json
    import tempfile
    state = Path(tempfile.gettempdir()) / "wbox_tvty-readme_state.json"
    return json.loads(state.read_text()).get("wayland_display") if state.exists() else None


class Camera(threading.Thread):
    """grim on the wbox's display, a frame every 1/FPS s, each with the
    time it was taken."""

    def __init__(self, display):
        super().__init__(daemon=True)
        self.display, self.frames, self.rolling = display, [], True

    def run(self):
        env = {**os.environ, "WAYLAND_DISPLAY": self.display}
        while self.rolling:
            at = time.monotonic()
            out = FRAMES / f"{len(self.frames):05}.png"
            if subprocess.run(["grim", "-l", "0", str(out)], env=env, capture_output=True).returncode == 0:
                self.frames.append((out, at))
            time.sleep(max(0, 1 / FPS - (time.monotonic() - at)))

    def stop(self):
        self.rolling = False
        self.join()


def replay(agent, project):
    """The agent's scene played again, typed, in its terminal: the film
    opens on it answering."""
    noise = ["Claude Code, played", "Type anything to talk", "agent does —", "left the session",
             "● You said", "Cogitating…", "Running…"]
    grep = " ".join(f"-e '{n}'" for n in noise)
    command = (f"clear; node '{SIMCLI}' --as claude --script '{ROOT}/demo/scenes/{agent}.txt' --pace 0.02 "
               f"--think 0.6 --inline --no-banner 2>&1 | grep --line-buffered -v {grep}; printf '\\n❯ '; exec cat")
    env = {**os.environ, "TMUX_TMPDIR": str(DEMO / "tmux")}
    env.pop("TMUX", None)
    subprocess.run(["tmux", "respawn-pane", "-k", "-t", f"cl-{agent}", "-c", f"/tmp/tvty-demo/{project}/{agent}", command],
                   check=True, env=env)


def notice():
    sys.path.insert(0, str(ROOT / "scripts"))
    from aiballbus import Board
    board = Board(str(DEMO / "home" / "sock"))
    board.call("star-diplomat", "message.post", {
        "project": "dyson-sphere", "kind": "comment_added", "ticket_id": shots.STAR_TICKET, "parent_id": shots.STAR_TICKET,
        "body": "The star accepts the window, on one condition: it faces the galaxy's good side. "
                "It also asks who will clean it.",
        "summary_until": "The star agrees to the window if it faces the galaxy's good side; asks who cleans it.",
        "commits": None, "handback": True})
    board.close()


def gif(frames, out):
    """The frames at the pace they were taken, as a GIF: one palette, and
    only what changed from a frame to the next."""
    listing = DEMO / "film.txt"
    lines = []
    for (frame, at), (_, then) in zip(frames, frames[1:] + [(None, frames[-1][1] + 2.5)]):
        lines.append(f"file '{frame}'\nduration {then - at:.3f}\n")
    listing.write_text("".join(lines) + f"file '{frames[-1][0]}'\n")
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", str(listing),
                    "-vf", f"fps={FPS},scale={WIDTH}:-1:flags=lanczos,split[a][b];"
                           "[a]palettegen=max_colors=192:stats_mode=full[p];"
                           "[b][p]paletteuse=dither=none:diff_mode=rectangle",
                    str(out)], check=True)


def main():
    if not (DEMO / "home" / "sock").exists():
        sys.exit("the demo is not up: demo/run up")
    shots.wbox_config()
    shots.settings()
    wbox("down")
    wbox("up")
    shots.ctl("ready", 30000)
    time.sleep(3)
    if not display():
        sys.exit("no wayland display for the wbox")
    ctl("window", "maximize")
    shots.click_said("ticket-", shots.HERO_TICKET)
    ctl("settle")

    shutil.rmtree(FRAMES, ignore_errors=True)
    FRAMES.mkdir(parents=True)
    camera = Camera(display())
    replay("exhaust-port", "battle-station")
    time.sleep(0.5)
    camera.start()
    time.sleep(9)
    # The slider: a stack per project, one step, then the next.
    ctl("hold", "ctrl")
    for _ in range(3):
        key("ctrl-tab")
        time.sleep(1.5)
    key("escape")
    ctl("release")
    time.sleep(0.8)
    key("ctrl-shift-space")
    time.sleep(3)
    wbox("type", "sphere")
    time.sleep(2)
    key("escape")
    time.sleep(1)
    notice()
    time.sleep(3.5)
    click(shots.MORE)
    time.sleep(3.5)
    key("escape")
    time.sleep(1.5)
    camera.stop()
    wbox("down")

    OUT.mkdir(parents=True, exist_ok=True)
    gif(camera.frames, OUT / "tvty.gif")
    size = (OUT / "tvty.gif").stat().st_size
    print(f"{len(camera.frames)} frames, {size / 1e6:.1f} MB: docs/images/tvty.gif")


if __name__ == "__main__":
    main()
