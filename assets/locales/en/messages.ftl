# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# Notifications and the small dialogs: a session ended, quitting, the
# conversation to resume, the tips' card (src/pings.rs, src/updates.rs,
# src/shell/ended.rs, src/shell/quit.rs, src/shell/resume.rs,
# src/shell/tips.rs).

## What pinged

messages-proposes-plan = proposes a plan
messages-proposes-close = proposes to close
messages-proposes-wontfix = proposes to close without a fix
messages-escalates = escalates
messages-new-ticket = a new ticket
messages-new-comment = a new comment
messages-new-ticket-moderate = a new ticket to moderate
messages-missed = { $more ->
    [0] {$what} · while tvty was closed
    [one] {$what} · and {$more} more while tvty was closed
   *[other] {$what} · and {$more} more while tvty was closed
}
messages-dismiss = dismiss

## Updates

messages-update-out = Terminal Velocity {$latest} is out (you run {$running}): {$how}
messages-update-how = the menu's Updates… installs it
messages-update-dev = this one is a development build: update its checkout (the menu's Updates… says how)
messages-aiball-old = aiball {$version} is older than Terminal Velocity needs ({$needs}): the menu's Updates… updates it

## A session that ended

messages-detached = — detached by another client
messages-ended = — the session ended
messages-attach-again = Attach again
messages-starting = Starting…
messages-restart = Restart
messages-close = Close
messages-copies-failed = make the other terminals copies

## Quitting and starting again

messages-stopping = Stopping the Claude Code sessions, then quitting…
messages-quit-stop-failed = stop when tvty quit
messages-ran-on = {$agents} ran on: the stop did not take
messages-restarted-as-were = { $count ->
    [one] {$count} session restarted as it was, resuming its conversation
   *[other] {$count} sessions restarted as they were, resuming their conversation
}
messages-restarted-fresh = { $count ->
    [one] {$count} session restarted fresh, resuming its conversation
   *[other] {$count} sessions restarted fresh, resuming their conversation
}
messages-restart-failed = restart of the stopped sessions
messages-restart-tvty-failed = restart tvty

## Which conversation to take up

messages-its-agent = its agent
messages-ago = {$time} ago
messages-some-time-ago = some time ago
messages-last-one = The last one, {$when}:
messages-says-nothing = (it says nothing yet)
messages-quoted = “{$said}”
messages-older = { $count ->
    [one] {$count} older one too: /resume in Claude Code picks among them.
   *[other] {$count} older ones too: /resume in Claude Code picks among them.
}
messages-new-conversation = New conversation
messages-resume-last = Resume the last conversation
messages-start = Start {$who}
messages-resume-question = Claude Code already has conversations in {$cwd}, none of them a loop's: which one does {$who} take up?

## The tips' card

messages-tip-title = Did you know?
messages-tips-browsing = Tips · {$at} / {$of}
messages-not-now = Not now
messages-previous = ‹ Previous
messages-next = Next ›
messages-show-all-again = Show them all again
messages-got-it = Got it
messages-next-tip = Next tip
messages-turn-off = Turn tips off
messages-tips-off = Tips are off: Settings > Layout > Tips turns them back on
messages-tips-again = Every tip will show again, one at a time
