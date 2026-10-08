# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# What is left: the terminal's menu and marks, the image viewer, a few
# notices and errors the user reads (src/shell.rs, src/terminal.rs,
# src/aiball.rs, src/images.rs, src/shell/viewer.rs, src/loops.rs,
# src/daemon.rs).

## Notices

misc-aiball-started = aiball was not running: started it
misc-aiball-silent = aiball does not answer at {$at}
misc-service-failed = aiball's service did not start: {$error}
misc-no-service = {$why}; there is no aiball service to start
misc-filed = filed — {$title}
misc-filed-moderate = {$title} — a new ticket to moderate
misc-filed-other = {$title} — a new ticket
misc-something-new = something new
misc-held = {$said}: its loop is held until you let it go
misc-go-to = go to
misc-go-to-what = go to {$what}
misc-not-a-ticket = {$typed} is not a ticket: a number, or a comment's #C. link
misc-copied = Copied to clipboard
misc-new-terminal = new terminal
misc-no-folder = no folder known for {$project}
misc-not-a-directory = {$folder} is not a directory
misc-not-idle = its Claude is working: once it is idle
misc-stop-not-received = {$agent}: no loop of it received the stop (it does not run, or not where aiball can reach it)

## The terminal

misc-open-link = Open link
misc-copy-link = Copy link
misc-copy = Copy
misc-paste = Paste
misc-frozen = held still · selection
misc-frozen-tip = The screen is held still while text is selected; the session goes on. A click here, Esc or a key lets it go.
misc-size-taken = size taken by another client · click to take it back
misc-session-ended = The session ended.
misc-hub-session = This session runs on aiball's hub, another machine: it cannot be opened from this one. Its tickets are in the panel.
misc-all-terminals = All terminals
misc-gallery-hint = type to filter · arrows move · enter opens · esc closes

## The agent's model, its limits, its refusals

misc-price = {$input} / {$output} per M tokens
misc-model-out = {$name} is out
misc-prices-from = prices from {$catalog}
misc-limit = usage limit reached
misc-limit-resets = usage limit reached · resets {$resets}
misc-denials = { $count ->
    [one] {$count} tool call refused by Claude Code in the last hour{$ago}{$why} — a refused agent stops there
   *[other] {$count} tool calls refused by Claude Code in the last hour{$ago}{$why} — a refused agent stops there
}
misc-denials-last = , the last {$ago} ago

## Images

misc-image-too-large = image too large to show here
misc-image-unavailable = image unavailable
misc-fitted = {" "}(fitted)
misc-viewer-hint = wheel or + − zoom · 1 real size · 0 fit · drag to move · Esc

## The usage arrow in the top bar

usage-ratio = ×{$ratio}
usage-points = {$points} pts
usage-wall = wall {$left}
usage-no-wall = no wall
usage-window-five_hour = 5-hour window
usage-window-seven_day = Week
usage-tip-window = {$window}: {$used} % used, {$expected} % at a steady pace · resets {$resets} · {$end}
usage-tip-wall = at this pace, the quota runs out in {$left}
usage-tip-lasts = at this pace, it lasts until the reset
usage-tip-click = A click: the gap said another way.
