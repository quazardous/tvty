#!/usr/bin/env python3
"""Dump what tvty's window shows from an aiball daemon, as tvty-egui's scene.

Usage: scene.py SOCKET [USER] > scene.json

It asks what tvty asks (consumer.list, inbox.list per project,
session.list for the terminals the daemon's host holds, ticket.get for
each open ticket's thread) and keeps
the answers whole: tvty-egui reads the fields it draws. With them, what
the sessions list shows besides aiball: the tmux sessions of the tests'
server (dev/tmux) and the terminals the tests' tvty keeps open: those of
scripts/test-env when it is sourced (TMUX_TMPDIR, XDG_STATE_HOME), else
this checkout's.
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
    threads = {}
    for row in inbox["rows"]:
        if not row.get("closed"):
            threads[str(row["id"])] = board.call(user, "ticket.get", {"id": row["id"], "full": True, "limit": 9999})
    scene["projects"].append({"name": name, "tickets": inbox["rows"], "threads": threads})
scene["hosted"] = board.call(user, "session.list", {})
root = Path(__file__).resolve().parents[2]
env = {k: v for k, v in os.environ.items() if k != "TMUX"}
env["TMUX_TMPDIR"] = os.environ.get("TMUX_TMPDIR") or str(root / "dev" / "tmux")
listed = subprocess.run(["tmux", "ls", "-F", "#{session_name}\t#{session_path}"], env=env, capture_output=True, text=True)
scene["sessions"] = [line.split("\t", 1) for line in listed.stdout.splitlines() if "\t" in line]
state = Path(os.environ.get("XDG_STATE_HOME") or root / "dev" / "tvty-wbox" / "state")
workspace = state / "tvty" / "workspace.json"
kept = json.loads(workspace.read_text()) if workspace.exists() else {}
scene["open"] = kept.get("open_terminals", [])
scene["order"] = kept.get("terminal_order", {})
json.dump(scene, sys.stdout, indent=1)
