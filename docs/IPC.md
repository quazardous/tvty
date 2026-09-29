# How tvty talks to other processes

Everything tvty says to another process — aiball's API and bus, a session
it attaches to, a second tvty handing over to the first — goes through one
crate, `crates/tvty-ipc`. This page is its contract: what the rest of tvty
may rely on, whatever the platform.

## Two transports

| Transport | Where | Who may connect |
|---|---|---|
| **Unix socket** | Linux, macOS | the operating system decides: a socket file of mode `0600` |
| **TCP on the loopback** | everywhere; the only one on Windows | anyone on the machine: a **secret** decides |

Windows has named pipes too; tvty does not use them. aiball's daemon, a
Node program, serves TCP on Windows (Node has no Unix socket there), and one
transport per platform is simpler than two.

## An address: `Endpoint`

One string, parsed once:

| Written | Is |
|---|---|
| `/path/to/sock`, `unix:/path/to/sock` | a Unix socket |
| `tcp://127.0.0.1:7777`, `http://127.0.0.1:7777` | TCP |

TCP is to the loopback: another host is refused unless it is asked for
explicitly (`Endpoint::parse_remote`), since whatever listens there is not
this machine's.

## Who we are: `Credentials`

Kept apart from the address, and never implied by it:

- **`Peer`** — nothing to send: the operating system vouches (a Unix
  socket).
- **`Bearer(secret)`** — sent as `Authorization: Bearer <secret>`.

TCP without a secret is an error, before anything is sent: a request that
goes out unauthenticated and fails there teaches nothing.

## A connection: `Conn`

The same guarantees on every transport:

- `Read` and `Write`;
- `set_read_timeout(Some(d))`: a read that waits longer fails with
  `WouldBlock` or `TimedOut` — callers treat both alike;
- `try_clone()`: a second handle on the same connection, for one thread to
  read while another writes;
- `shutdown()`: ends the connection both ways, and unblocks a thread
  waiting in a read; calling it again does nothing.

## Listening: `Listener`

`Listener::bind(endpoint)` then `accept()`, giving `Conn`s. A Unix socket is
made `0600`. On TCP, `tcp://127.0.0.1:0` takes a free port, and `local()`
says which — the one to write down for a client. A TCP listener is reachable
by anyone on the machine: what it serves must check a secret first.

## Where aiball is: `aiball::locate`

As aiball's own clients find it (its `bin/launcher.js` and `src/client.ts`),
so that tvty and the `aiball` command never disagree:

1. `AIBALL_SOCK`, when set: that socket. Set but empty: TCP, even where a
   socket exists.
2. Otherwise, on Unix, `$AIBALL_HOME/sock` when it is a socket.
3. Otherwise TCP: `AIBALL_URL`, else `http://127.0.0.1:$AIBALL_PORT`
   (7777), with the token `AIBALL_TOKEN`, else the one in
   `$AIBALL_HOME/cli-env` (`export AIBALL_TOKEN=…`, read as aiball's
   launcher reads it).

`AIBALL_HOME` defaults to `~/.local/share/aiball` (`%USERPROFILE%` on
Windows). The token found in `cli-env` belongs to the human who set aiball
up: tvty acts as that human.

Over TCP aiball refuses some calls even to the human (it reserves them to
its socket): `loop.list`, `loop.restart`, `loop.wake`, `project.init`,
`project.settings`, `session.host`, `daemon.reload`. On Windows those
features of tvty wait for aiball to change that.

## One tvty per state directory

A second launch hands over to the first through a rendez-vous in the state
directory: on Unix the socket `tvty.sock`; on Windows a TCP listener on a
free loopback port, whose address and a fresh secret are written to
`tvty.addr` (readable by the user only, as the state directory is). A
second launch reads that file, connects, and says the secret before
anything else; a connection that does not is closed.

## Tests

`crates/tvty-ipc/tests/contract.rs` holds every promise above — echo, a
read timeout, one thread reading while another writes, `shutdown` waking a
blocked reader, a listener's `accept`, TCP refused without a secret — and
runs it against each transport the platform has: both on Linux, TCP on
Windows. A new transport passes the same tests before tvty uses it.
