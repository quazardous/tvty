# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# The agent's bar, under its terminal (src/shell/agentbar.rs).

## Who drives the loop

agentbar-afk-auto = auto
agentbar-afk-hold-10m = hold 10 min
agentbar-afk-hold = hold
agentbar-afk-boot = … boot
agentbar-afk-arming-tip = armed: the little man shows the mode chosen with F9, in force 3 s after the last press — ▶ or ‖ says the one in force until then
agentbar-afk-tip = who drives the loop: ▶ on its own, ‖ held for you (or while you type); the little man is the AFK mode — grey: you are away, the loop runs on its own; the seconds of a 10 min hold; ∞ held. F9 cycles it (auto → 10 min → ∞), in force 3 s after the last press; a click chooses

## What its Claude does

agentbar-working = working
agentbar-starting = starting
agentbar-idle = idle
agentbar-offline = offline
# After the time since the boot began: " · 12s left".
agentbar-boot-left = {" · "}{$seconds}s left
agentbar-boot-tip = its loop is starting: Claude loads, may resume its conversation or compact; the loop wakes it once the boot ends (30 s at least, longer while a resume or a compaction shows)
agentbar-state-tip = what its Claude does, and since when
agentbar-dialog = waits for an answer

## Restarting its Claude

agentbar-restarting = restarting…
agentbar-restarting-tip = its Claude restarts, resuming its conversation
agentbar-restart-pending = restart pending
agentbar-restart-pending-tip = a restart is asked: its Claude restarts as soon as it is idle, resuming its conversation
agentbar-update = update
agentbar-update-tip = its Claude Code installed an update: a click asks to restart it (at once if idle, otherwise as soon as it is idle), resuming its conversation
agentbar-restart-ask-busy = Restart its Claude once it is idle?
agentbar-restart-ask = Restart its Claude now?
agentbar-restart = Restart
agentbar-restart-resumed = its conversation is resumed
agentbar-cancel = Cancel
agentbar-restart-done = {$agent}'s Claude restarts once idle, resuming its conversation
agentbar-restart-failed = restart of {$agent}'s Claude

## Where its loop runs, whose hands are on it

agentbar-place-host = host
agentbar-hands-copy = copy
agentbar-hands-controls = controls
agentbar-moving = moving…
agentbar-runs-hosted = its loop runs on aiball's session host;
agentbar-runs-mux = its loop runs in {$mux} (claude-loop);
agentbar-copy-tip = this terminal is a copy: you watch, nothing you type reaches its Claude, and the session keeps its size.
agentbar-controls-tip = you have the controls, shared with any other client: the size follows who types last.
agentbar-others-attached = { $others ->
    [one] {$others} other client attached ({$typing} with the controls).
   *[other] {$others} other clients attached ({$typing} with the controls).
}
agentbar-proxy-alive = The terminal proxy in front of Claude is alive.
agentbar-place-click = A click: take or leave the controls, move the loop.
agentbar-others-type = claude-loop's terminal types into it too
agentbar-take-controls = Take the controls
agentbar-leave-copy = Leave for a copy
agentbar-close-others = Close the others ({$others})
agentbar-close-others-tip = { $others ->
    [one] the other client attached to this session leaves it (claude-loop's terminal, another Terminal Velocity); its Claude and this terminal go on
   *[other] the {$others} other clients attached to this session leave it (claude-loop's terminal, another Terminal Velocity); its Claude and this terminal go on
}
agentbar-move-into = Move into {$mux}
agentbar-move-to-host = Move to host
agentbar-move-tip-into = its Claude restarts into {$mux}, resuming its conversation
agentbar-move-tip-to-host = its Claude restarts on aiball's host, resuming its conversation
agentbar-move-interrupts = {" — "}it works now: the move interrupts it
agentbar-moved-into = {$agent} moved into {$mux}, its conversation resumed
agentbar-moved-to-host = {$agent} moved to aiball's host, its conversation resumed
agentbar-move-failed-into = move of {$agent} into {$mux}
agentbar-move-failed-to-host = move of {$agent} to aiball's host
agentbar-hold-failed = hold of {$agent}
agentbar-closed-others = { $count ->
    [one] closed the other client of this session
   *[other] closed the {$count} other clients of this session
}
agentbar-closed-others-all = closed the other clients of this session
agentbar-closed-others-asked = asked the session's host to close its other clients
agentbar-others-left = { $count ->
    [one] {$count} other client still attached: {$mux} cannot tell it from this one yet
   *[other] {$count} other clients still attached: {$mux} cannot tell them from this one yet
}
agentbar-close-others-failed = close the other clients

## Remote Control, alerts, the prompt

agentbar-rc-on = Remote Control is on: this Claude can be taken up from claude.ai and the mobile app
agentbar-rc-off = Remote Control is off. /rc in the session turns it on; a folder's loops get it from the project's settings
agentbar-trust = trust this folder?
agentbar-not-logged-in = not logged in
agentbar-api-unreachable = API unreachable
agentbar-link-down = loop link down
agentbar-aiball-unreachable = aiball unreachable
agentbar-prompt-input = Claude's prompt is on screen, with text not sent yet
agentbar-prompt-empty = Claude's prompt is on screen, empty
agentbar-typing = a human typed in its terminal a moment ago: the loop holds off
agentbar-zen = zen
agentbar-zen-tip = zen mode: the loop keeps quiet

## Its counters

agentbar-all = all:{$count}
agentbar-all-tip = a: all the project's open tickets
agentbar-backlog = backlog:{$count}
agentbar-backlog-tip = b: its backlog, the tickets for it to look at; a click lists them
agentbar-events = events:{$count}
agentbar-events-tip = e: its events not seen yet — pings, answers, decisions waiting for it
agentbar-holds = holds:{$count}
agentbar-holds-tip = the tickets it holds
agentbar-wake-tip = work waits for the loop: it wakes its Claude on it when the countdown ends
agentbar-pending-tip = work waits for the loop (events or backlog)

## Its backlog, over the bar

agentbar-backlog-of = {$agent}'s backlog
agentbar-reading = reading…
agentbar-backlog-error = backlog: {$error}
agentbar-backlog-empty = nothing in its backlog
agentbar-tier-critical = critical
agentbar-tier-hot = hot
agentbar-tier-yours = yours
agentbar-tier-decision = your decision pending
agentbar-tier-waiting = waiting on them
agentbar-tier-blocked = blocked
agentbar-tier-other = other

agentbar-info-resuming = resuming
agentbar-info-compacting = compacting
agentbar-info-wait = waiting
agentbar-info-interrupted = interrupted
agentbar-info-user = a human at the keys
agentbar-info-picker-session = choosing a session
agentbar-info-picker-mode = choosing a mode
agentbar-info-error-rate-limit = rate limited
agentbar-info-error-overloaded = API overloaded
agentbar-info-error-api = API error
agentbar-info-retry = retry {$attempt}
