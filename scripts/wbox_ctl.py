#!/usr/bin/env python3
# Copyright (c) David Berlioz
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Drive a wbox compositor from make, without going through MCP.

wbox-mcp only exposes its compositor as MCP tools, which a Makefile cannot
call. This script uses the same library directly, so `make` can start
tvty inside a nested compositor, stop it and screenshot it. (Taken from
nelson-mcp's scripts/wbox_ctl.py.)

It must run under wbox's own interpreter (wbox lives in its own venv), which
is why the Makefile invokes it through $(WBOX_PYTHON) rather than python3.

    wbox_ctl.py up     CONFIG [-s key=value ...]
    wbox_ctl.py down   CONFIG [-s key=value ...]
    wbox_ctl.py status CONFIG [-s key=value ...]
    wbox_ctl.py shot   CONFIG [-s key=value ...] [--name NAME]
    wbox_ctl.py click  CONFIG [-s key=value ...] X Y
    wbox_ctl.py key    CONFIG [-s key=value ...] SHORTCUT [SHORTCUT ...]
    wbox_ctl.py type   CONFIG [-s key=value ...] TEXT
    wbox_ctl.py scroll CONFIG [-s key=value ...] X Y NOTCHES   (negative = up)
    wbox_ctl.py drag   CONFIG [-s key=value ...] X1 Y1 X2 Y2   (WBOX_DRAG_STEPS=N)
    wbox_ctl.py hold   CONFIG [-s key=value ...] MODIFIER KEY TIMES [--name NAME]

Headless by default: the compositor renders offscreen and nothing appears on
the desktop. Set WBOX_VISIBLE=1 to get a window, for when the assertion is
something you have to look at. An explicit `-s headless=...` still wins.

The instance is identified by the config's `name`, and its PIDs live in a
state file, so `down` can stop an instance a previous `up` started — and the
tvty-wbox MCP tools see the same instance when the name matches.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

from wbox.config import apply_overrides, load_config, resolve_dir
from wbox.server import _build_app_cmd, _build_app_env, build_compositor

_TRUE = ("1", "true", "yes", "on")


def _headless_from_env():
    return os.environ.get("WBOX_VISIBLE", "").strip().lower() not in _TRUE


def _load(argv):
    if not argv:
        sys.exit(__doc__)
    config_path = Path(argv[0]).resolve()
    overrides, name, positional, i = [], None, [], 1
    while i < len(argv):
        if argv[i] in ("-s", "--set") and i + 1 < len(argv):
            overrides.append(argv[i + 1])
            i += 2
        elif argv[i] == "--name" and i + 1 < len(argv):
            name = argv[i + 1]
            i += 2
        else:
            positional.append(argv[i])
            i += 1

    cfg = load_config(config_path)
    cfg["headless"] = _headless_from_env()
    apply_overrides(cfg, overrides)          # explicit overrides win
    cfg["_config_dir"] = str(config_path.parent)
    # Relative paths in the config (log dir, screenshots, pre_launch scripts)
    # are relative to the config file, as they are for wbox-mcp serve.
    os.chdir(config_path.parent)
    return cfg, name, positional


def _run_pre_launch(cfg):
    for script in cfg.get("app", {}).get("pre_launch", []) or []:
        result = subprocess.run(script, shell=True, cwd=cfg["_config_dir"],
                                capture_output=True, text=True, timeout=30)
        if result.returncode != 0:
            sys.exit("pre_launch failed: %s\n%s%s"
                     % (script, result.stdout, result.stderr))


def _warm_up(comp, x=None, y=None):
    """Make the first real input event count.

    Every invocation of this script opens a fresh virtual pointer/keyboard, and
    the compositor swallows the first event sent to a device it has not finished
    propagating — wbox's own launch() warms up for exactly this reason. Without
    it a click only hovers. Moving to the target first also gives the app its
    pointer-enter before the button press.
    """
    import time
    warm = getattr(comp, "_vptr_op", None)
    if warm:
        try:
            warm("warm_up")
        except Exception:
            pass
    if x is not None:
        comp.mouse_move(x, y)
    time.sleep(0.3)


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    command, rest = sys.argv[1], sys.argv[2:]
    cfg, name, args = _load(rest)
    comp = build_compositor(cfg)
    # Where wbox-mcp serve puts screenshots, so both land in the same place.
    comp.state.screenshot_dir = resolve_dir(cfg, "screenshot_dir",
                                            "./screenshots")

    if command == "up":
        _run_pre_launch(cfg)
        result = comp.launch(_build_app_cmd(cfg), _build_app_env(cfg))
        result["headless"] = cfg["headless"]
    elif command == "down":
        result = comp.kill()
    elif command == "shot":
        # The sequence counter restarts with every process, so an unnamed shot
        # would overwrite the previous one. Name it by time instead.
        if not name:
            import time
            name = time.strftime("wbox_%Y%m%d_%H%M%S")
        result = comp.screenshot(name)
    elif command == "click":
        if len(args) != 2:
            sys.exit("click needs X Y")
        x, y = int(args[0]), int(args[1])
        _warm_up(comp, x, y)
        result = comp.click(x, y)
    elif command == "key":
        if not args:
            sys.exit("key needs at least one shortcut, e.g. alt+F12")
        _warm_up(comp)
        result = comp.keys(args) if len(args) > 1 else comp.key(args[0])
    elif command == "type":
        if len(args) != 1:
            sys.exit("type needs one TEXT argument")
        _warm_up(comp)
        result = comp.type_text(args[0])
    elif command == "scroll":
        # wbox has no wheel yet: speak zwlr_virtual_pointer_v1 directly,
        # through wbox's own connection. N notches, negative = up.
        if len(args) != 3:
            sys.exit("scroll needs X Y NOTCHES")
        x, y, notches = int(args[0]), int(args[1]), int(args[2])
        _warm_up(comp, x, y)
        import struct
        import time as clock
        client = comp._vptr_client()
        vp = client._ensure_vptr()
        client._motion(vp, x, y)
        step = -1 if notches < 0 else 1
        for _ in range(abs(notches)):
            now = struct.pack("<I", int(clock.monotonic() * 1000) & 0xFFFFFFFF)
            client._send(vp, 5, struct.pack("<I", 0))  # axis_source: wheel
            # axis_discrete: time, axis (0 = vertical), value (fixed 24.8), discrete
            client._send(vp, 7, now + struct.pack("<I", 0) +
                         struct.pack("<i", step * 15 * 256) + struct.pack("<i", step))
            client._send(vp, 4)  # frame
            client.roundtrip()
            clock.sleep(0.03)
        result = {"ok": True}
    elif command == "drag":
        # Press at X1 Y1, move to X2 Y2 in steps, release.
        if len(args) != 4:
            sys.exit("drag needs X1 Y1 X2 Y2")
        x1, y1, x2, y2 = (int(a) for a in args)
        _warm_up(comp, x1, y1)
        import time as clock
        from wbox import pointer as wp
        client = comp._vptr_client()
        vp = client._ensure_vptr()
        client._motion(vp, x1, y1)
        client._send(vp, 2, client._uint(wp._now_ms()) + client._uint(wp.BTN_LEFT) + client._uint(wp.PRESSED))
        client._send(vp, 4)
        client.roundtrip()
        # WBOX_DRAG_STEPS: more, shorter steps are closer to a hand's drag.
        import os
        steps = max(1, int(os.environ.get("WBOX_DRAG_STEPS", "10")))
        pause = 0.3 / steps if steps > 10 else 0.03
        for i in range(1, steps + 1):
            client._motion(vp, x1 + (x2 - x1) * i // steps, y1 + (y2 - y1) * i // steps)
            client.roundtrip()
            clock.sleep(pause)
        client._send(vp, 2, client._uint(wp._now_ms()) + client._uint(wp.BTN_LEFT) + client._uint(wp.RELEASED))
        client._send(vp, 4)
        client.roundtrip()
        result = {"ok": True}
    elif command == "hold":
        # Hold a modifier, tap a key N times, screenshot, then release: a
        # gesture like alt-tab, which acts on release, seen while held.
        if len(args) != 3:
            sys.exit("hold needs MODIFIER KEY TIMES (and --name for the shot)")
        modifier, key, times = args[0], args[1], int(args[2])
        _warm_up(comp)
        import time as clock
        from wbox import pointer as wp
        client = comp._vptr_client()
        client._ensure_vkbd()
        mask, mod_code = wp._MODIFIERS[modifier.lower()]
        code, _, _ = wp._parse_shortcut(key)
        client._kbd_key(mod_code, wp.PRESSED)
        client._kbd_mods(mask)
        for _ in range(times):
            client._kbd_key(code, wp.PRESSED)
            client._kbd_key(code, wp.RELEASED)
            client.roundtrip()
            clock.sleep(0.15)
        clock.sleep(float(os.environ.get("WBOX_HOLD_WAIT", "0.5")))
        result = comp.screenshot(name or "hold")
        client._kbd_mods(0)
        client._kbd_key(mod_code, wp.RELEASED)
        client.roundtrip()
    elif command == "status":
        result = {"running": comp.is_running(), "headless": cfg["headless"]}
    else:
        sys.exit("unknown command: %s\n%s" % (command, __doc__))

    print(json.dumps(result, default=str))
    return 1 if isinstance(result, dict) and result.get("error") else 0


if __name__ == "__main__":
    sys.exit(main())
