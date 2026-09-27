#!/usr/bin/env python3
"""Keeps the demo's agents present: one bus connection each, subscribed to
its own events, as a loop's would be — so tvty shows them connected rather
than offline. Runs until killed.

Usage: presence.py SOCKET AGENT...
"""

import base64
import importlib.machinery
import json
import os
import socket
import sys
import threading
import time
from pathlib import Path

# The bus client of scripts/aiball-call (a script without an extension).
call = importlib.machinery.SourceFileLoader(
    "aiball_call", str(Path(__file__).resolve().parent.parent / "scripts" / "aiball-call")
).load_module()


def pong(sock, payload):
    """A client's pong: final, masked."""
    mask = os.urandom(4)
    sock.sendall(bytes([0x8A, 0x80 | len(payload)]) + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(payload)))


def serve(sock):
    """Reads frames for as long as the connection lives, answering pings:
    the bus closes a connection that does not."""
    while True:
        b0, b1 = call.read_exact(sock, 2)
        n = b1 & 0x7F
        if n == 126:
            n = int.from_bytes(call.read_exact(sock, 2), "big")
        elif n == 127:
            n = int.from_bytes(call.read_exact(sock, 8), "big")
        payload = call.read_exact(sock, n)
        opcode = b0 & 0x0F
        if opcode == 0x8:
            return
        if opcode == 0x9:
            pong(sock, payload)


def hold(sock_path, agent):
    """One agent present, connected again whenever the bus lets it go."""
    while True:
        try:
            connect(sock_path, agent)
        except OSError:
            pass
        time.sleep(1)


def connect(sock_path, agent):
    sock = socket.socket(socket.AF_UNIX)
    sock.connect(sock_path)
    key = base64.b64encode(os.urandom(16)).decode()
    sock.sendall(
        (
            "GET /bus HTTP/1.1\r\nHost: aiball\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n"
            f"x-aiball-consumer: {agent}\r\n\r\n"
        ).encode()
    )
    head = b""
    while b"\r\n\r\n" not in head:
        head += call.read_exact(sock, 1)
    subscribe = {"jsonrpc": "2.0", "id": 1, "method": "bus.subscribe", "params": {"subject": f"agent.{agent}.events"}}
    sock.sendall(call.frame(json.dumps(subscribe)))
    serve(sock)


if __name__ == "__main__":
    path, agents = sys.argv[1], sys.argv[2:]
    threads = [threading.Thread(target=hold, args=(path, agent), daemon=True) for agent in agents]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
