"""aiballbus — aiball's bus from a script, with the standard library only.

The bus is JSON-RPC over a WebSocket on `/bus`, through aiball's local
socket. The identity is the connection's: it is opened as a consumer
(`x-aiball-consumer`), and every call on it acts as that consumer — so a
script that acts as several agents opens one connection each (`Board`).

    bus = Bus(sock, "david")
    bus.call("session.list")

Used by scripts/aiball-call, the seeds of the fake aiball and of the demo,
and the demo's presence keeper. Aimed at a throwaway aiball: its gestures
act as whoever the connection names.
"""

import base64
import json
import os
import socket
import struct


class BusError(Exception):
    """A call the bus refused: its JSON-RPC error."""

    def __init__(self, method, error):
        super().__init__(f"{method}: {error.get('message', error)}")
        self.method = method
        self.error = error


def _frame(opcode, payload):
    """A client's frame: final, masked."""
    mask = os.urandom(4)
    head = bytes([0x80 | opcode])
    n = len(payload)
    if n < 126:
        head += bytes([0x80 | n])
    elif n < 65536:
        head += bytes([0x80 | 126]) + struct.pack(">H", n)
    else:
        head += bytes([0x80 | 127]) + struct.pack(">Q", n)
    return head + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(payload))


class Bus:
    """One connection to the bus, as one consumer."""

    def __init__(self, sock_path, consumer, timeout=30):
        self.consumer = consumer
        self.sock = socket.socket(socket.AF_UNIX)
        self.sock.settimeout(timeout)
        self.sock.connect(sock_path)
        key = base64.b64encode(os.urandom(16)).decode()
        self.sock.sendall(
            (
                "GET /bus HTTP/1.1\r\nHost: aiball\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n"
                f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n"
                f"x-aiball-consumer: {consumer}\r\n\r\n"
            ).encode()
        )
        head = b""
        while b"\r\n\r\n" not in head:
            head += self._read(1)
        if b" 101 " not in head.split(b"\r\n")[0]:
            raise ConnectionError(head.decode(errors="replace").split("\r\n")[0])
        self.next_id = 0

    def _read(self, n):
        data = b""
        while len(data) < n:
            chunk = self.sock.recv(n - len(data))
            if not chunk:
                raise ConnectionError("the bus closed")
            data += chunk
        return data

    def messages(self):
        """The server's text messages, one by one; its pings answered, as
        the bus closes a connection that does not."""
        parts = b""
        while True:
            b0, b1 = self._read(2)
            n = b1 & 0x7F
            if n == 126:
                n = struct.unpack(">H", self._read(2))[0]
            elif n == 127:
                n = struct.unpack(">Q", self._read(8))[0]
            payload = self._read(n)
            opcode = b0 & 0x0F
            if opcode == 0x8:
                raise ConnectionError("the bus closed")
            if opcode == 0x9:
                self.sock.sendall(_frame(0xA, payload))
            elif opcode in (0x1, 0x0):
                parts += payload
                if b0 & 0x80:
                    yield json.loads(parts.decode())
                    parts = b""

    def call(self, method, params=None):
        """A method called, its result answered (or BusError raised)."""
        self.next_id += 1
        ask = self.next_id
        request = {"jsonrpc": "2.0", "id": ask, "method": method, "params": params if params is not None else {}}
        self.sock.sendall(_frame(0x1, json.dumps(request).encode()))
        for message in self.messages():
            if message.get("id") != ask:
                continue  # a notice, an event
            if "error" in message:
                raise BusError(method, message["error"])
            return message.get("result")
        raise ConnectionError("the bus closed")

    def serve(self):
        """Reads (and drops) what comes, for as long as the connection
        lives: a subscription kept open."""
        for _ in self.messages():
            pass

    def close(self):
        self.sock.close()


class Board:
    """Several consumers on one aiball: a connection each, opened the
    first time one acts."""

    def __init__(self, sock_path):
        self.sock_path = sock_path
        self.buses = {}

    def as_(self, consumer):
        if consumer not in self.buses:
            self.buses[consumer] = Bus(self.sock_path, consumer)
        return self.buses[consumer]

    def call(self, who, method, params=None):
        return self.as_(who).call(method, params)

    def close(self):
        for bus in self.buses.values():
            bus.close()
