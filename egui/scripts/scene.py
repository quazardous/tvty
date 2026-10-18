#!/usr/bin/env python3
"""Dump what tvty's window shows from an aiball daemon, as tvty-egui's scene.

Usage: scene.py SOCKET [USER] > scene.json

It asks what tvty asks (consumer.list, inbox.list per project) and keeps
the answers whole: tvty-egui reads the fields it draws.
"""

import json
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
json.dump(scene, sys.stdout, indent=1)
