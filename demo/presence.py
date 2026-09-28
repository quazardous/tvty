#!/usr/bin/env python3
"""Keeps the demo's agents present, as loops would be: one bus connection
each, subscribed to its own events, pushing its loop's bar — so tvty shows
them connected and driven (on their own, held for a while, held until let
go) rather than offline. Runs until killed.

Usage: presence.py SOCKET AGENT...
"""

import sys
import threading
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "scripts"))
from aiballbus import Bus  # noqa: E402

# Who is held, and how: the others run on their own.
HELD = {"tractor-beam": "wait_10m", "paradox-police": "wait_inf"}
# Who works, as demo/seed.py has them.
BUSY = {"superlaser", "panel-crew", "flux"}


def bar(agent):
    """The bar a loop in tmux would push."""
    afk = HELD.get(agent, "off")
    return {
        "phase": "busy" if agent in BUSY else "idle",
        "presence": "wait" if afk != "off" else "loop",
        "afk": {"mode": afk, "expires_at": None},
        "prompt": {"visible": agent not in BUSY, "has_input": False},
        "human_typing": False,
        "marker": {"info": None, "health_prompt": False, "resume_picker": False, "resume_mode_picker": False},
        "alerts": {"link_down": False, "daemon_down": False, "not_logged_in": False, "trust_dialog": False,
                   "api_unreachable": False},
        "proxy_alive": True,
        "zen": False,
        "counters": None,
        "next_wake_at": None,
        "boot": None,
        "host": "tmux",
        "attach": {"socket": None, "reason": "no_socket"},
    }


def hold(sock_path, agent):
    """One agent present, connected again whenever the bus lets it go."""
    while True:
        try:
            bus = Bus(sock_path, agent, timeout=None)
            bus.call("bus.subscribe", {"subject": f"agent.{agent}.events"})
            # Pushed once: it stays live while the agent is present.
            bus.call("consumer.push_bar", {"consumer_id": agent, "bar": bar(agent)})
            bus.serve()
        except Exception as e:  # the bus gone, or a bar refused: said, then again
            print(f"{agent}: {e}", file=sys.stderr, flush=True)
        time.sleep(1)


if __name__ == "__main__":
    path, agents = sys.argv[1], sys.argv[2:]
    threads = [threading.Thread(target=hold, args=(path, agent), daemon=True) for agent in agents]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
