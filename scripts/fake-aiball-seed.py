#!/usr/bin/env python3
"""Fill a throwaway aiball daemon with a demo project tvty can show.

Called by scripts/fake-aiball; talks to the daemon over its socket.
Usage: fake-aiball-seed.py SOCKET DEMO_DIR

- consumers: `david` (human), `demo-claude` and `demo-crew` (agents of the
  `demo` project, whose loops run in DEMO_DIR/lead and DEMO_DIR/crew);
- tickets: one claimed by each agent, a pending plan, a pending resolution,
  a queue, a closed one.
"""

import http.client
import json
import socket
import sys


class UnixHTTPConnection(http.client.HTTPConnection):
    def __init__(self, path):
        super().__init__("aiball")
        self.path = path

    def connect(self):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.path)


SOCK, DEMO = sys.argv[1], sys.argv[2]
PROJECT = "demo"


def call(method, path, body=None, who="david"):
    conn = UnixHTTPConnection(SOCK)
    headers = {"content-type": "application/json", "x-aiball-consumer": who}
    conn.request(method, "/api" + path, json.dumps(body) if body is not None else None, headers)
    response = conn.getresponse()
    data = response.read().decode()
    if response.status >= 400:
        sys.exit(f"{method} {path} as {who}: {response.status} {data}")
    return json.loads(data) if data else None


def ticket(title, body, who, **extra):
    return call("POST", "/messages", {
        "project": PROJECT, "kind": "ticket_created", "title": title, "body": body, **extra,
    }, who)["id"]


def comment(ticket_id, body, who, decision=None):
    payload = {
        "project": PROJECT, "kind": "comment_added", "ticket_id": ticket_id,
        "parent_id": ticket_id, "body": body,
        "summary_until": body.split("\n")[0][:200], "commits": None,
    }
    if decision:
        payload["decision_kind"] = decision
    else:
        payload["handback"] = True
    return call("POST", "/messages", payload, who)["id"]


call("POST", "/consumers", {"consumer_id": "david", "kind": "human"})
call("POST", "/projects", {"name": PROJECT, "created_by": "david"})
for agent, sub in (("demo-claude", "lead"), ("demo-crew", "crew")):
    call("POST", "/consumers", {"consumer_id": agent, "kind": "agent"})
    call("PUT", f"/consumers/{agent}/state",
         {"state": "idle", "cwd": f"{DEMO}/{sub}", "project": PROJECT}, agent)

t1 = ticket("Render box-drawing characters as quads",
            "The `│` of a frame does not meet the next row at a line height of 1.3.",
            "david", intent="request", priority="high")
call("POST", f"/tickets/{t1}/assign", {}, "demo-claude")
comment(t1, "Plan: draw U+2500–U+257F as rectangles sized to the cell.\n\n"
            "1. a table of the light and heavy lines\n2. the rounded corners as arcs",
        "demo-claude", decision="plan")

t2 = ticket("Mouse wheel scrolls the history", "Wheel up in the terminal: scroll back.",
            "david", intent="feature")
t3 = ticket("Copy and paste", "Select with the mouse, copy with ctrl+shift+c.",
            "david", intent="feature", priority="low")

t4 = ticket("Colour emoji", "Emoji show in grey.", "david", intent="request")
call("POST", f"/tickets/{t4}/assign", {}, "demo-crew")
comment(t4, "The fallback now asks for Noto Color Emoji first. Fixed in abc1234.",
        "demo-crew", decision="resolution")

t5 = ticket("Startup is slow", "Measure the first frame.", "david", intent="question")
comment(t5, "First frame at 180 ms in release, most of it in font loading.", "demo-claude")
call("POST", "/messages", {"project": PROJECT, "kind": "ticket_closed", "ticket_id": t5,
                           "parent_id": t5}, "david")

print(json.dumps({"tickets": [t1, t2, t3, t4, t5]}))
