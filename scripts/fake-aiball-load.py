#!/usr/bin/env python3
"""Loads the throwaway aiball's demo project with many tickets, to measure
tvty on a board the size of a real one.

    scripts/fake-aiball-load.py SOCKET [COUNT]

Tickets are filed by demo-crew and demo-claude in turn, some with a comment
from the other, a few claimed: bands, glyphs and holders vary like on a
real board. Never run it against a real aiball: it files as those agents.
"""

import sys

from aiballbus import Board


def main():
    sock = sys.argv[1]
    if "fake-aiball" not in sock:
        sys.exit("refusing: this is not the throwaway aiball's socket")
    count = int(sys.argv[2]) if len(sys.argv) > 2 else 300
    board = Board(sock)
    agents = ["demo-crew", "demo-claude"]
    for i in range(count):
        by, other = agents[i % 2], agents[(i + 1) % 2]
        ticket = board.call(by, "message.post", {
            "project": "demo", "kind": "ticket_created",
            "title": f"Load ticket {i}: a title long enough to be cut in the panel's narrow column",
            "body": f"Filed to measure tvty on a large board ({i}).",
            "intent": ["request", "question", "feature", "fyi"][i % 4],
        })
        tid = ticket.get("id")
        if not tid:
            sys.exit(f"aiball refused ticket {i}: {ticket}")
        if tid and i % 3 == 0:
            board.call(other, "message.post", {
                "project": "demo", "kind": "comment_added", "ticket_id": tid, "parent_id": tid,
                "body": "A comment, so that the last speaker varies.",
                "summary_until": "load test", "commits": None, "handback": True,
            })
    board.close()
    print(f"filed {count} tickets on demo")


if __name__ == "__main__":
    main()
