#!/usr/bin/env python3
"""Find what looks clickable and does nothing, in tvty-egui, in wbox.

tvty-egui, started with TVTY_EGUI_AUDIT=<file> (src/audit.rs), writes at
each pass the widgets that take a click and the texts painted in the link
colour. This script starts it in wbox (headless), then:

  - a link no click widget covers is reported at once: painted, never
    wired (no click can reach it);
  - with --click, it clicks each click widget it can see, the pointer then
    parked away, and compares the window before and after: a click that
    changes nothing is a dead button. After each click, Escape; if the
    window is not back as it started, tvty-egui starts again, on a fresh
    copy of the state.

Steps before the audit (--open X,Y and --key KEY, in order) reach a view:
`--open 1257,60` audits the full list.

    wbox-python scripts/audit.py [--click] [--open X,Y]... [--key K]...

Run from egui/, the binary built (TVTY_EGUI_PROFILE=release for
target/release). Reports on stdout; screenshots in wbox/screenshots/audit-*.
"""

import argparse
import json
import os
import shutil
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE.parent.parent / "tvty" / "scripts"))
if not (HERE.parent.parent / "tvty" / "scripts" / "wbox_ctl.py").exists():
    # tvty-egui inside tvty's own checkout.
    sys.path.insert(0, str(HERE.parent / "scripts"))
import wbox_ctl  # noqa: E402
from PIL import Image, ImageChops  # noqa: E402
from wbox.config import resolve_dir  # noqa: E402
from wbox.server import _build_app_cmd, _build_app_env, build_compositor  # noqa: E402

PARK = (640, 795)


def inside(r, x, y):
    return r[0] <= x <= r[2] and r[1] <= y <= r[3]


def centre(r):
    return ((r[0] + r[2]) / 2, (r[1] + r[3]) / 2)


def area(r):
    return max(0, r[2] - r[0]) * max(0, r[3] - r[1])


class App:
    def __init__(self, config, steps):
        cfg, _, _ = wbox_ctl._load([config])
        self.cfg = cfg
        self.comp = build_compositor(cfg)
        self.comp.state.screenshot_dir = resolve_dir(cfg, "screenshot_dir", "./screenshots")
        self.steps = steps
        self.tmp = Path(tempfile.mkdtemp(prefix="tvty-egui-audit-"))
        self.report = self.tmp / "audit.json"
        self.shots = 0

    def start(self):
        state = self.tmp / "state"
        shutil.rmtree(state, ignore_errors=True)
        state.mkdir()
        os.environ["TVTY_EGUI_STATE"] = str(state)
        os.environ["TVTY_EGUI_AUDIT"] = str(self.report)
        self.report.unlink(missing_ok=True)
        self.comp.launch(_build_app_cmd(self.cfg), _build_app_env(self.cfg))
        for _ in range(100):
            if self.report.exists():
                break
            time.sleep(0.1)
        time.sleep(1.0)
        wbox_ctl._warm_up(self.comp, *PARK)
        for kind, value in self.steps:
            if kind == "open":
                x, y = value
                self.comp.mouse_move(x, y)
                time.sleep(0.3)
                self.comp.click(x, y, 1)
            else:
                self.comp.key(value)
            time.sleep(0.6)
        self.park()

    def stop(self):
        self.comp.kill()

    def park(self):
        self.comp.mouse_move(*PARK)
        time.sleep(0.6)

    def shot(self, tag):
        self.shots += 1
        out = self.comp.screenshot("audit-%03d-%s" % (self.shots, tag))
        path = out["path"] if isinstance(out, dict) else out
        return Image.open(path).convert("RGB")

    def read(self):
        return json.loads(self.report.read_text())


def changed(a, b, tolerance=24):
    """Pixels that differ beyond a tolerance, and where."""
    diff = ImageChops.difference(a, b).convert("L").point(lambda v: 255 if v > tolerance else 0)
    box = diff.getbbox()
    return (diff.histogram()[255] if box else 0), box


def analyse(report):
    screen = area(report["screen"])
    clicks = report["clicks"]
    # A widget as large as a quarter of the window is a backdrop (a view
    # taking the pointer): it hides what is under it, it wires nothing.
    backdrops = [c for c in clicks if area(c["rect"]) > screen / 4]
    buttons = [c for c in clicks if area(c["rect"]) <= screen / 4]

    def hidden(layer, x, y):
        return any(b["layer"] > layer and inside(b["rect"], x, y) for b in backdrops)

    dead_links = []
    for link in report["links"]:
        x, y = centre(link["rect"])
        if hidden(link["layer"], x, y):
            continue
        if not any(c["layer"] >= link["layer"] and inside(c["rect"], x, y) for c in buttons):
            dead_links.append(link)
    seen, visible = set(), []
    # What is dragged (a sash, a scroll bar, the title bar, a text field)
    # takes a click too, and changes nothing when clicked: left out.
    for c in buttons:
        if c.get("drag"):
            continue
        x, y = centre(c["rect"])
        key = tuple(c["rect"])
        if key in seen or hidden(c["layer"], x, y):
            continue
        seen.add(key)
        visible.append(c)
    return dead_links, visible


def main():
    sys.stdout.reconfigure(line_buffering=True)
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--config", default="wbox/config.yaml")
    ap.add_argument("--click", action="store_true", help="click each widget to find the dead ones")
    ap.add_argument("--open", action="append", default=[], help="X,Y clicked before the audit")
    ap.add_argument("--key", action="append", default=[], help="a key pressed before the audit")
    args = ap.parse_args()
    steps = [("open", tuple(int(v) for v in o.split(","))) for o in args.open] + [("key", k) for k in args.key]

    app = App(args.config, steps)
    app.start()
    try:
        dead_links, buttons = analyse(app.read())
        print("links painted, no click wired: %d" % len(dead_links))
        for link in dead_links:
            print("  %-28s at %s" % (json.dumps(link["text"], ensure_ascii=False), link["rect"]))
        if not args.click:
            return
        print("click widgets to try: %d" % len(buttons))
        baseline = app.shot("base")
        dead = []
        for i, b in enumerate(buttons):
            x, y = (int(v) for v in centre(b["rect"]))
            before = app.shot("before")
            app.comp.mouse_move(x, y)
            time.sleep(0.3)
            app.comp.click(x, y, 1)
            time.sleep(0.2)
            app.park()
            after = app.shot("after")
            n, box = changed(before, after)
            verdict = "ok" if n > 30 else "DEAD"
            print("  [%2d] %-5s %s at (%d,%d) changed %d px %s" % (i, verdict, b["rect"], x, y, n, box or ""))
            if verdict == "DEAD":
                dead.append(b)
            app.comp.key("Escape")
            app.park()
            if changed(baseline, app.shot("back"))[0] > 30:
                app.stop()
                app.start()
        print("dead click widgets: %d" % len(dead))
        for b in dead:
            print("  %s" % b["rect"])
    finally:
        app.stop()


if __name__ == "__main__":
    main()
