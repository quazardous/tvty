# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# The new project's wizard (src/shell/newproject.rs). What aiball says it
# did shows as aiball says it.

newproject-title = New project
newproject-step-folder = Folder
newproject-step-identity = Who works in it
newproject-step-done = Set up
newproject-step-next = What next

## The folder

newproject-which-folder = Which folder becomes the project?
newproject-folder-placeholder = the project's folder, e.g. ~/dev/app
newproject-choose-folder = Choose the project's folder
newproject-browse = Browse…
newproject-folder-empty = The folder the agent will work in: the project's root.
newproject-folder-missing = No such folder.
newproject-folder-file = This is a file, not a folder.
newproject-folder-checking = Asking aiball…
newproject-folder-project = Already an aiball project: {$name} ({$file}). Next shows what it says, to set it up again.
newproject-folder-configured = Already set up for aiball ({$file}). Next shows what it says, to set it up again.
newproject-folder-git = A git repository: ready.
newproject-folder-ready = Ready (not a git repository).
newproject-open-running = Open {$label}, running
newproject-open = Open {$name}
newproject-next = Next →
newproject-back = ← Back

## Who works in it

newproject-project-placeholder = project
newproject-agent-placeholder = agent
newproject-field-project = project
newproject-field-agent = agent
newproject-bad-project = The project's name: letters, digits, -, _ and .
newproject-bad-agent = The agent's name: letters, digits, -, _ and .
newproject-filled-from = Filled from {$file}: change anything, what is here is what gets set up.
newproject-crew = a crew agent
newproject-crew-about = Beside the project's lead, on the tickets it is given; off: the lead.
newproject-host = its loops on aiball's host
newproject-host-about = Where the loops started in this folder run; off: in {$mux}.
newproject-rc = Remote Control
newproject-rc-about = Its Claude can be reached from claude.ai and the Claude app.
newproject-private = a private project
newproject-private-about = aiball serves it its private kit (no public tickets, no followers).
newproject-noclaim = no claiming
newproject-noclaim-about = The agent works only on the tickets assigned to it, never takes one from the pool.
newproject-in = In {$folder}, aiball:
newproject-will-file = {$will} {$file}
newproject-will-created = creates
newproject-will-added = adds its entry to
newproject-will-rewritten = rewrites its entry in
newproject-will-patched = updates
newproject-will-overwrote = overwrites
newproject-will-kept = keeps
newproject-joins = {$name} is on the board already: this folder joins it (another folder, or a crew agent).
newproject-then-sets = then sets {$what} in .aiball.yaml
newproject-files = .mcp.json: aiball's MCP server for Claude Code · .aiball.yaml: project, agent, role, where its loops run.
newproject-set-up = Set it up
newproject-setting-up = Setting it up…
newproject-unsaved = set up, but where its loops run and its Remote Control were not saved: {$error}
newproject-set-in = set {$what} in {$file}

## Set up

newproject-done = {$name} is set up. aiball said:
newproject-failed = aiball could not set it up:
newproject-nothing-said = (nothing said)

## What next

newproject-next-start = Start the agent
newproject-next-start-host = "Start its first session" starts {$agent}'s Claude Code on aiball's host, in the project's folder; its terminal opens here, its tickets beside.
newproject-next-start-mux = "Start its first session" starts {$agent}'s Claude Code in {$mux}, in the project's folder; its terminal opens here, its tickets beside.
newproject-next-mcp = Accept aiball's MCP server
newproject-next-mcp-about = At its first start in this folder, Claude Code asks whether to use the MCP server that .mcp.json declares (aiball): accept it. Without it the agent can neither read the board nor answer its tickets. Refused by mistake? /mcp in Claude Code turns it on.
newproject-next-skill = Install aiball's skill
newproject-next-skill-about = Claude Code has no aiball skill on this machine yet: `aiball init skill`, once, installs it — the agent then knows the board's good gestures.
newproject-next-work = Give it work
newproject-next-work-about = "+ New" in the ticket panel files a ticket on {$name}; the agent picks it up at its next wake, and its plans and questions come back as notifications.
newproject-close = Close
newproject-start-first = Start its first session
