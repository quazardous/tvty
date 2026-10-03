# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# Workspaces, and the list that chooses sessions (quitting, starting
# again, a workspace kept, opened or shut) (src/shell/workspaces.rs,
# src/workspaces.rs).

## The list of workspaces

workspaces-new = + workspace
workspaces-new-tip = keeps the groups that run now, chosen in a list, under a name
workspaces-none = No workspace yet: a workspace keeps some groups, and how each of their sessions runs.
workspaces-default-name = Workspace
workspaces-its-name = its name
workspaces-act-open = Open
workspaces-act-open-tip = starts what is stopped and sets each session as kept — asked first
workspaces-act-shut = Shut
workspaces-act-shut-tip = stops its sessions — asked first, which; a group another workspace has too is left
workspaces-act-save = save
workspaces-act-save-tip = keeps it again as things are now, or adds a group — chosen in a list
workspaces-again-tip = the group shown now, kept again as it runs now
workspaces-add-tip = adds the group shown now, its sessions as they run
workspaces-act-rename = rename
workspaces-act-rename-tip = another name; Enter confirms
workspaces-act-delete = delete
workspaces-act-delete-sure = delete: sure?
workspaces-act-delete-tip = forgets the workspace; its sessions are not touched
workspaces-no-project = no project
workspaces-drop-tip = takes this group out of the workspace (its sessions are not touched)
workspaces-now-stopped = stopped
workspaces-now-runs = runs
workspaces-now-held = held
workspaces-kept-own = kept on its own; now {$now}
workspaces-kept-held = kept held; now {$now}
workspaces-open-sets = {" — "}Open sets it as kept

## What a session will become

workspaces-start-own = stopped: to start, on its own
workspaces-start-held = stopped: to start, held
workspaces-let-go = held: to let go
workspaces-to-hold = runs on its own: to hold
workspaces-as-kept = as kept

## The list that chooses

workspaces-quit-title = Stop the Claude Code sessions too?
workspaces-quit-summary = On: stopped as tvty quits (they stay restartable). Off: it runs on.
workspaces-new-title = A new workspace
workspaces-new-summary = On: kept in it, each as it runs now (on its own, or held).
workspaces-save-title = Keep {$name} as things are now
workspaces-save-summary = On: in it, each as it runs now. Off: taken out of it.
workspaces-shut-title = Shut {$name}: stop its sessions?
workspaces-shut-summary = On: stopped. Off: it runs on. A group another workspace has too is left off.
workspaces-open-title = Open {$name}
workspaces-open-summary = On: done. Off: left as it is.
workspaces-restart-title = Restart the sessions stopped when tvty quit?
workspaces-restart-summary = On: restarted, resuming its conversation. As they were: a held session is held again; fresh: each boots, then runs on its own.
workspaces-host = host
workspaces-held-for-good = {" · "}held until let go
workspaces-held-while = {" · "}held for a while
workspaces-ran-on = {$place} · ran on: its stop did not take
workspaces-not-running = does not run: kept as it was
workspaces-new-in-group = new in this group
workspaces-not-in-yet = not in it yet
workspaces-also-in = also in {$others}
workspaces-some = {$ticked} of {$of}
workspaces-whole-group = the whole group
workspaces-name = Name
workspaces-cancel = Cancel
workspaces-keep = Keep
workspaces-quit-keep = Quit, keep them all running
workspaces-quit-stop = Quit, stop the {$count} on
workspaces-shut-keep = Shut, keep them all running
workspaces-shut-stop = Shut, stop the {$count} on
workspaces-open-do = Open, do the {$count} on
workspaces-not-now = Not now
workspaces-restart-fresh = Restart the {$count} fresh
workspaces-restart-as-were = Restart the {$count} as they were
workspaces-remember-quit = Remember this choice (stop them all, or keep them all)
workspaces-remember-restart = Remember this choice (every time, all of them)
workspaces-remember-tip = Settings > Layout > Sessions changes it

## What was done

workspaces-none-runs = {$name}: none of its sessions runs
workspaces-all-as-kept = {$name}: every session is as kept
workspaces-kept = { $count ->
    [one] workspace {$name} kept: {$count} group
   *[other] workspace {$name} kept: {$count} groups
}
workspaces-shut-run-on = {$name} shut: its sessions run on
workspaces-shut-stopped = { $count ->
    [one] {$name} shut: {$count} session stopped
   *[other] {$name} shut: {$count} sessions stopped
}
workspaces-shut = shut
workspaces-no-loop = { $count ->
    [one] {$count} session on no loop of this machine: not stopped
   *[other] {$count} sessions on no loop of this machine: not stopped
}
workspaces-nothing-to-start = {$agent}: no loop nor folder known to start it
workspaces-astray = {$agent}: {$cwd} is {$other}'s folder, not started
workspaces-mode-failed = {$agent}: its mode: {$error}
workspaces-opened = { $count ->
    [one] {$name} opened: {$count} session set
   *[other] {$name} opened: {$count} sessions set
}
workspaces-opening = opening {$name}
workspaces-added = {$project} is in {$name}
