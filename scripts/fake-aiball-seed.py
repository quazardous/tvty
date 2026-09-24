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

# ── Every case of the ticket list, by the API's own gestures ────────────
def decide(comment_id, status):
    call("POST", f"/messages/{comment_id}/decide", {"status": status, "decided_by": "david"}, "david")

def step(ticket_id, body, who):
    return call("POST", "/messages", {
        "project": PROJECT, "kind": "comment_added", "ticket_id": ticket_id, "parent_id": ticket_id,
        "body": body, "summary_until": body[:200], "commits": None,
        "step": True, "step_after_minutes": 0,
    }, who)["id"]

# A plan the talk went on after: the stripe turns dashed.
t6 = ticket("Slide the terminal in on switch", "An animation when switching.", "david", intent="feature")
call("POST", f"/tickets/{t6}/assign", {}, "demo-claude")
comment(t6, "Plan: a relative offset, so tmux is not resized.", "demo-claude", decision="plan")
comment(t6, "Also: keep it under 250 ms.", "demo-claude")

# Closing without a fix, proposed.
t7 = ticket("Blink the cursor", "Make the cursor blink.", "david", intent="request")
call("POST", f"/tickets/{t7}/assign", {}, "demo-crew")
comment(t7, "Claude Code draws its own cursor: nothing to do here.", "demo-crew", decision="wontfix")

# An escalation: the agent needs a human.
t8 = ticket("Install the GPUI system libraries", "xcb, xkbcommon, vulkan…", "david", intent="request")
call("POST", f"/tickets/{t8}/assign", {}, "demo-crew")
comment(t8, "Needs sudo: please run the dnf install.", "demo-crew", decision="escalation")

# An agent on a step.
t9 = ticket("Measure the frame rate", "Under a flood of output.", "david", intent="request")
call("POST", f"/tickets/{t9}/assign", {}, "demo-claude")
step(t9, "Debug build measured; release next.", "demo-claude")

# A resolution david rejected: the ball is back with the agent.
t10 = ticket("Truncate long agent names", "They wrap in the sidebar.", "david", intent="request")
call("POST", f"/tickets/{t10}/assign", {}, "demo-crew")
proposal = comment(t10, "Truncated with an ellipsis.", "demo-crew", decision="resolution")
decide(proposal, "rejected")
call("POST", "/messages", {"project": PROJECT, "kind": "comment_added", "ticket_id": t10,
                           "parent_id": t10, "body": "Still wraps on narrow windows.", "by_agent": "david"}, "david")

# An agent answered david, no decision: his turn, plain.
t11 = ticket("Which font for the title bar?", "The kit's or the terminal's?", "david", intent="question")
comment(t11, "The kit's: it follows the theme.", "demo-claude")

# David spoke last: the agent's turn.
t12 = ticket("Remember the theme", "Across restarts.", "david", intent="feature")
call("POST", "/messages", {"project": PROJECT, "kind": "comment_added", "ticket_id": t12,
                           "parent_id": t12, "body": "Store it with the panel widths.", "by_agent": "david"}, "david")

# A ticket an agent filed: it waits for moderation.
t13 = ticket("Add a status bar", "Filed by an agent.", "demo-crew", intent="feature")

# The critical ticket: two others depend on it.
for blocked in (t2, t3):
    call("POST", f"/tickets/{blocked}/relations", {"target_ticket_id": t8, "kind": "depends_on"}, "david")

print(json.dumps({"tickets": [t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12, t13]}))
