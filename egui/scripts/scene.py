#!/usr/bin/env python3
"""Dump what tvty's window shows from an aiball daemon, as tvty-egui's scene.

Usage: scene.py SOCKET [USER] > scene.json

It asks what tvty asks (consumer.list, inbox.list per project) and keeps
the answers whole: tvty-egui reads the fields it draws. With them, what
the sessions list shows besides aiball: the tmux sessions of the tests'
server (dev/tmux) and the terminals the tests' tvty keeps open.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from aiballbus import Board  # noqa: E402

sock = sys.argv[1]
user = sys.argv[2] if len(sys.argv) > 2 else "david"
board = Board(sock)
consumers = board.call(user, "consumer.list", {})
projects = sorted({c["project"] for c in consumers if c.get("project")})
scene = {"user": user, "consumers": consumers, "projects": []}
for name in projects:
    inbox = board.call(user, "inbox.list", {"project": name, "view": "turn", "sort": "band", "limit": 200})
    scene["projects"].append({"name": name, "tickets": inbox["rows"]})
root = Path(__file__).resolve().parents[2]
env = {k: v for k, v in os.environ.items() if k != "TMUX"}
env["TMUX_TMPDIR"] = str(root / "dev" / "tmux")
listed = subprocess.run(["tmux", "ls", "-F", "#{session_name}\t#{session_path}"], env=env, capture_output=True, text=True)
scene["sessions"] = [line.split("\t", 1) for line in listed.stdout.splitlines() if "\t" in line]
workspace = root / "dev" / "tvty-wbox" / "state" / "tvty" / "workspace.json"
kept = json.loads(workspace.read_text()) if workspace.exists() else {}
scene["open"] = kept.get("open_terminals", [])
scene["order"] = kept.get("terminal_order", {})
json.dump(scene, sys.stdout, indent=1)
