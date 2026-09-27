#!/usr/bin/env python3
"""Fill the demo's aiball with a small fictional world for the README.

Usage: seed.py SOCKET WORK_DIR IMAGES_DIR

Three projects, each with its agents (who work in WORK_DIR/<project>/<agent>)
and tickets in every state tvty shows: a plan and a resolution waiting for
the director, an escalation, a critical ticket others depend on, an agent
on a step, one waiting for moderation, pictures in threads.
"""

import http.client
import json
import socket
import sys
from pathlib import Path


class UnixHTTPConnection(http.client.HTTPConnection):
    def __init__(self, path):
        super().__init__("aiball")
        self.path = path

    def connect(self):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.path)


SOCK, WORK, IMAGES = sys.argv[1], Path(sys.argv[2]), Path(sys.argv[3])
HUMAN = "director"

# project → its agents, the first one busy.
PROJECTS = {
    "battle-station": ["exhaust-port", "superlaser", "tractor-beam"],
    "dyson-sphere": ["panel-crew", "star-diplomat"],
    "time-machine": ["flux", "paradox-police"],
}
BUSY = {"superlaser", "panel-crew", "flux"}


def request(method, path, body, who, content_type="application/json", extra=None):
    conn = UnixHTTPConnection(SOCK)
    headers = {"content-type": content_type, "x-aiball-consumer": who, **(extra or {})}
    conn.request(method, "/api" + path, body, headers)
    response = conn.getresponse()
    data = response.read().decode()
    if response.status >= 400:
        sys.exit(f"{method} {path} as {who}: {response.status} {data}")
    return json.loads(data) if data else None


def call(method, path, body=None, who=HUMAN):
    return request(method, path, json.dumps(body) if body is not None else None, who)


def picture(name):
    """An image uploaded, as markdown to cite."""
    answer = request("POST", "/uploads", (IMAGES / name).read_bytes(), HUMAN, "image/png", {"x-aiball-upload-name": name})
    return f"![{name}]({answer['url']})"


def ticket(project, title, body, who=HUMAN, **extra):
    return call("POST", "/messages", {"project": project, "kind": "ticket_created", "title": title, "body": body, **extra}, who)["id"]


def comment(project, ticket_id, body, who, decision=None, step=False, summary=None):
    payload = {
        "project": project, "kind": "comment_added", "ticket_id": ticket_id, "parent_id": ticket_id,
        "body": body, "summary_until": summary or body.split("\n")[0][:200], "commits": None,
    }
    if decision:
        payload["decision_kind"] = decision
    elif step:
        payload.update(step=True, step_after_minutes=0)
    elif who != HUMAN:
        payload["handback"] = True
    return call("POST", "/messages", payload, who)["id"]


def assign(ticket_id, agent):
    call("POST", f"/tickets/{ticket_id}/assign", {}, agent)


def depends(ticket_id, on):
    call("POST", f"/tickets/{ticket_id}/relations", {"target_ticket_id": on, "kind": "depends_on"}, HUMAN)


call("POST", "/consumers", {"consumer_id": HUMAN, "kind": "human"})
for project, agents in PROJECTS.items():
    call("POST", "/projects", {"name": project, "created_by": HUMAN})
    for agent in agents:
        call("POST", "/consumers", {"consumer_id": agent, "kind": "agent"})
        call("PUT", f"/consumers/{agent}/state",
             {"state": "busy" if agent in BUSY else "idle", "cwd": str(WORK / project / agent), "project": project}, agent)

# ── battle-station ───────────────────────────────────────────────────
P = "battle-station"
port = ticket(P, "Cover the thermal exhaust port",
              "The final review found a two-metre port that leads straight to the main reactor.",
              priority="urgent", intent="request")
assign(port, "exhaust-port")
comment(P, port, "One proton torpedo down that shaft and the whole station goes. The ray shield does "
                 "not cover it and the particle shield is off.\n\nI need a human: the grate needs a "
                 "budget line, and the plans have already left the building. Sheet 7:\n\n"
                 + picture("exhaust-port.png"), "exhaust-port", decision="escalation",
        summary="Critical: the port leads straight to the reactor. A grate needs the director's budget.")
inspection = ticket(P, "Final inspection before the grand opening", "Walk every trench, sign every sheet.", intent="request")
opening = ticket(P, "Open the station to visitors", "Guided tours of the superlaser, weekends only.", intent="feature")
depends(inspection, port)
depends(opening, port)

laser = ticket(P, "Superlaser: charge in under 24 h", "Four days between two shots is not a weapon, it is a schedule.",
               intent="request", priority="high")
assign(laser, "superlaser")
comment(P, laser, "Plan:\n\n1. tap the hypermatter reactor directly instead of the secondary bus\n"
                  "2. pre-charge the eight tributary beams in parallel\n3. keep the focus lens cold between shots\n\n"
                  + picture("superlaser.png"), "superlaser", decision="plan",
        summary="Three changes should bring a charge to 23 h; waiting for the director's go.")

compactor = ticket(P, "Waste compactor: stop when something is alive inside",
                   "Detention level reports people in the compactor, again.", intent="request")
assign(compactor, "tractor-beam")
comment(P, compactor, "Added a life sensor on the floor: the walls stop as soon as it reads anything "
                      "breathing. Tested with a very patient droid. Fixed in 7a3f9c1.\n\n" + picture("compactor.png"),
        "tractor-beam", decision="resolution",
        summary="Life sensor in, fixed in 7a3f9c1; waiting for the director to accept.")

moon = ticket(P, "Visitors keep saying \"that's no moon\"", "Should we paint it grey?", intent="question")
comment(P, moon, "It is, however, the size of one. I would lean into it: a small sign, \"Not a moon.\"", "exhaust-port")

ledge = ticket(P, "Tractor beam: its power switch is on a very narrow ledge", "Over a very deep shaft.", intent="request")
assign(ledge, "tractor-beam")
comment(P, ledge, "Nobody will ever go there on their own. Closing without a fix.", "tractor-beam", decision="wontfix")

ticket(P, "Add a second exhaust port, for symmetry", "The station looks lopsided.", who="superlaser", intent="feature")

aim = ticket(P, "Turbolasers: hit something, anything", "Target practice results attached.", intent="request")
comment(P, aim, "Calibrated: they now hit what they aim at. Mostly.", "superlaser")
call("POST", "/messages", {"project": P, "kind": "ticket_closed", "ticket_id": aim, "parent_id": aim}, HUMAN)

# ── dyson-sphere ─────────────────────────────────────────────────────
P = "dyson-sphere"
panels = ticket(P, "Enclose the star", "One billion panels, give or take.", intent="request", priority="high")
assign(panels, "panel-crew")
comment(P, panels, "Panel 4 812 bolted. 999 995 188 to go; the pace holds.\n\n" + picture("dyson-sphere.png"),
        "panel-crew", step=True)
star = ticket(P, "Negotiate with the star", "It flares whenever a panel goes up.", intent="question")
assign(star, "star-diplomat")
comment(P, star, "The star asks for weekends off and a window facing the galaxy.", "star-diplomat")
comment(P, star, "Weekends yes. The window we can discuss.", HUMAN)
ticket(P, "Night side: who switches off the lights?", "The sphere has no night side any more.", intent="question")

# ── time-machine ─────────────────────────────────────────────────────
P = "time-machine"
paradox = ticket(P, "Tests fail before the bug exists", "CI went red on Monday for a regression introduced on Thursday.",
                 intent="request", priority="high")
assign(paradox, "flux")
comment(P, paradox, "Plan: pin the test run to the commit's own date, not the machine's.\n\n" + picture("timeline.png"),
        "flux", decision="plan", summary="Pin each test run to its commit's date; plan waiting for a go.")
yesterday = ticket(P, "Close this ticket yesterday", "It was due then.", intent="request")
assign(yesterday, "paradox-police")
comment(P, yesterday, "Done, yesterday. You may not remember it yet.", "paradox-police", decision="resolution")
ticket(P, "Arrive before we leave", "Two minutes early would do.", intent="feature")

print(json.dumps({"tickets": [port, laser, compactor, panels, paradox]}))
