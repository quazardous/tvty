# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# A project's 📢 (standing instruction, wake focus, a message to every
# agent) and its options (src/shell/megaphone.rs, src/shell/projectopts.rs).

## The 📢

megaphone-no-project = A standing instruction and a wake focus are a project's: show one first
megaphone-standing = Standing instruction — {$project}
megaphone-standing-hint = Put at the head of every wake of its agents, event and backlog alike. Leave one before stepping away; clear it when you are back.
megaphone-prompt-placeholder = e.g. light debugging first, no big changes
megaphone-focus = Wake focus
megaphone-focus-hint = Only these tickets wake the project's agents, backlog and events. 123, 456 keeps just those; !789 keeps all but it. 123+ brings its children, 123++ all its descendants, +123 / ++123 its parents, 123~ its linked tickets. Events outside stay unread until you clear it.
megaphone-tickets-placeholder = e.g. 2518, 2523++   or   !2180
megaphone-until-placeholder = until (optional): 2026-09-30 18:00
megaphone-not-a-date = until: {$typed} is not a date (2026-09-30 18:00)
megaphone-not-a-time = until: {$typed} is not a time here
megaphone-clear = Clear
megaphone-save = Save
megaphone-said-nothing = {$project}: nothing steers its agents now
megaphone-said-prompt = {$project}: its agents read “{$prompt}” at every wake
megaphone-said-focus = {$project}: only its focus wakes its agents
megaphone-tip-standing = Standing instruction: {$prompt}
megaphone-tip-focus = Wake focus: {$focus}

## A message to every agent

megaphone-message = Message to every agent
megaphone-message-hint = Typed into each running agent session now, whatever it is doing. Send & hold also holds every loop (not AFK ∞): no wake starts new work until you release them. Left empty, the text shown is sent.
megaphone-no-loop = No agent loop is running.
megaphone-running = {$count} running: {$names}
megaphone-release = Release holds
megaphone-send = Send
megaphone-send-hold = Send & hold
megaphone-typed-into = typed into {$names}
megaphone-queued-for = queued for {$names}
megaphone-held = held {$names}
megaphone-released = released {$names}
megaphone-hold-not-applied = hold not applied

## A project's options

projectopts-global = Global
projectopts-a-project = a project…
projectopts-failed = project settings
projectopts-choose = Choose a project above.
projectopts-no-folder = aiball knows no folder of this project on this machine: none of its agents works here.
projectopts-asking = Asking aiball…
projectopts-could-not-say = aiball could not say: {$error}
projectopts-no-file = {$folder} has no .aiball.yaml, nor any folder above it: aiball's defaults apply. New project… sets it up.
projectopts-written-in = Written in {$file}.
projectopts-also-serves = It also serves {$others}: a change here is theirs too.
projectopts-on-named = on: {$name}
projectopts-none-set = {$project} sets none of the board's config: it has the board's values.
projectopts-all-keys = All its keys
