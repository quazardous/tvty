#!/usr/bin/env python3
"""Keeps the demo's agents present: one bus connection each, subscribed to
its own events, as a loop's would be — so tvty shows them connected rather
than offline. Runs until killed.

Usage: presence.py SOCKET AGENT...
"""

import sys
import threading
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "scripts"))
from aiballbus import Bus  # noqa: E402


def hold(sock_path, agent):
    """One agent present, connected again whenever the bus lets it go."""
    while True:
        try:
            bus = Bus(sock_path, agent, timeout=None)
            bus.call("bus.subscribe", {"subject": f"agent.{agent}.events"})
            bus.serve()
        except OSError:
            pass
        time.sleep(1)


if __name__ == "__main__":
    path, agents = sys.argv[1], sys.argv[2:]
    threads = [threading.Thread(target=hold, args=(path, agent), daemon=True) for agent in agents]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
