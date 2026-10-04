# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# The sessions' list, a session's line, the tabs and the title bar
# (src/shell.rs, src/shell/loopstabs.rs, src/shell/tabs.rs, src/status.rs).

## The list's head

sessions-tab-sessions = Sessions
sessions-tab-workspaces = Workspaces
sessions-order-recent = the project used last first, as ctrl+tab goes; a click: alphabetical
sessions-order-alpha = alphabetical; a click: your order (drag a project)
sessions-order-yours = your order: drag a project; a click: the project used last first
sessions-new-project = + project
sessions-new-project-tip = a folder made an aiball project (aiball init), then its first session
sessions-new-terminal-tip = a terminal of its own, which outlives tvty
sessions-filter = Filter…  ctrl+shift+f
sessions-fold = Fold the sessions' list

## Its groups

sessions-group-live = live
sessions-group-idle = idle
sessions-group-shut = shut
sessions-group-on-hub = on hub
sessions-none = No session
sessions-none-found = No session found
sessions-first-project = Set up your first project
sessions-none-stopped = No stopped loop
sessions-none-stopped-found = No stopped loop found
sessions-none-shut = No agent without a loop
sessions-none-shut-found = No agent found
sessions-none-hub = No session on the hub
sessions-none-hub-found = No session found on the hub
sessions-no-project = No project

## A project's heading

sessions-heading-tip = its tickets in the panel, no session opened; drag it onto another to move it
sessions-new-session = + session
sessions-project-options = its settings: where its loops run, Remote Control, its board config
sessions-project-terminal = a terminal in the project's folder, listed with it

## A session's row

sessions-open = open in tvty: its terminal runs here
sessions-looped = {$mux}: its Claude runs in claude-loop, opened through {$mux} (not on aiball's host)
sessions-attached = { $others ->
    [one] {$others} other client attached (claude-loop's terminal, another tvty), {$typing} with the controls
   *[other] {$others} other clients attached (claude-loop's terminal, another tvty), {$typing} with the controls
}
sessions-attached-copy = : opened here as a copy
sessions-host-shell = a terminal on aiball's host, without Claude
sessions-update = its Claude Code installed an update: restart it from its bar
sessions-update-pending = its restart is asked: its Claude restarts as soon as it is idle (the badge is on its bar)
sessions-starting = starting…
sessions-start = ▶ start
sessions-astray = ⚠ {$other}'s folder
sessions-astray-tip = {$other} works in {$cwd}: started here, this agent would resume its conversation. Not started.
sessions-stop-ask = stop?
sessions-stop-tip = stop {$who}'s loop: through aiball, it stays restartable (under the stopped ones) — a second click stops it
sessions-stopped = stopped {$who}
sessions-stop-failed-loop = stop {$who}
sessions-forget-ask = forget?
sessions-forget-tip = forget {$who}: aiball no longer lists it; its folder, its .aiball.yaml and the project's tickets stay — a second click forgets
sessions-forgot = forgot {$who}
sessions-forget-failed = forget {$who}

## A session's state, in a line

sessions-offline = offline
sessions-offline-tip = its loop is not connected to aiball
sessions-working = working
sessions-starting-state = starting
sessions-idle = idle
sessions-booting = booting
sessions-claude-working = Claude is working
sessions-claude-starting = Claude is starting
sessions-claude-idle = Claude is idle
sessions-held-typing-for-good = held until let go (a human is typing in it): the loop does not wake it
sessions-typing = a human is typing in it: the loop waits
sessions-held-for-good = held until let go: the loop does not wake it
sessions-held-while = held a while: the loop does not wake it
sessions-loop-drives = the loop drives it on its own
sessions-mark-held-for-good = held until let go
sessions-mark-held-while = held for a while
sessions-mark-own = on its own

## The folded list's marks

sessions-mark-tip = {$agent}: {$said}
sessions-mark-limit = {$agent}: {$said} — {$limit}

## "+ session"

sessions-form-cwd = working directory
sessions-form-agent = agent
sessions-form-where = where its Claude works
sessions-form-host-here = aiball's host starts it here
sessions-form-loop-here = claude-loop starts here
sessions-form-no-dir = no such directory
sessions-form-crew = a crew agent, next to the main loop
sessions-form-on-host = on aiball's host, without {$mux}
sessions-form-cancel = Cancel
sessions-form-start = Start
sessions-a-loop = a loop
sessions-astray-said = {$cwd} is {$other}'s folder: {$agent} would resume its conversation
sessions-start-failed = start
sessions-start-host-failed = start on the host
sessions-started-host = started {$agent} on aiball's host in {$cwd}
sessions-started = started {$agent} in {$cwd}
sessions-runs-already = {$agent} runs already: opened as a copy

## The badges: a project's, an agent's

sessions-badge-critical = the critical ticket, #{$ticket}, is here: it holds the most open tickets — a click opens it
sessions-badge-decisions = { $count ->
    [one] {$count} ticket waiting for your decision
   *[other] {$count} tickets waiting for your decision
}
sessions-badge-unread = { $count ->
    [one] {$count} ticket with something new for you
   *[other] {$count} tickets with something new for you
}
sessions-light-backlog = { $count ->
    [one] backlog: {$count} ticket for it to look at
   *[other] backlog: {$count} tickets for it to look at
}
sessions-light-backlog-unknown = backlog: not known until its loop says
sessions-light-critical = it holds the critical ticket: the one that holds the most open tickets
sessions-light-events = events: {$count} not seen yet — pings, answers, decisions waiting for it
sessions-hub-mark = On aiball's hub, another machine: read from here (its state, its tickets), not opened — a session is attached from its own machine.

## The tabs

sessions-tab-name = its name
sessions-rename-failed = rename the terminal
sessions-stop-failed = stop the terminal
sessions-rename-tip = rename it (or a double click, F2)
sessions-close-shell = close: stops this terminal
sessions-close-tab = close the tab: its Claude goes on
sessions-tab-shell-tip = a terminal on aiball's host, without Claude; a double click (or F2) renames it
sessions-tab-tip = ctrl+pgup / ctrl+pgdn: the tab before, after
sessions-new-tab-project = a terminal in the project's folder
sessions-new-tab-home = a terminal in the home directory

## The window

sessions-pick = Pick a terminal on the left · ctrl+shift+space shows them all
sessions-on-hub = {$agent} — on hub
sessions-window-hub-project = {$name} — {$project} · {$agent} · on hub
sessions-window-hub = {$name} — {$agent} · on hub
sessions-bus-down = aiball's bus is down: tvty connects again; the lists may be behind meanwhile
sessions-bus-failing = subscribing to {$what} failed: tried again, read whole; the lists may be behind meanwhile
sessions-menu = Menu: about, help, restart, quit
sessions-goto = Go to a ticket: its number or a comment's #C. link, Enter · ctrl+shift+g
sessions-themes = The colour themes — the next one
sessions-message-all = A message to every running agent: send, send & hold, release holds
sessions-settings = Settings
sessions-user = who tvty acts as on aiball's board
