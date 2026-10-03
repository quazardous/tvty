# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# The tips (tips/<id>.md: their English is the file's, a test checks
# it). {$key} is the key of the tip's command, {$key-font-reset} the key
# of font.reset: as they are, never translated. **bold** and `code` stay.

tip-slider = **{$key}** shows your projects as stacks, the most recent first. Release Ctrl to open the one chosen.
tip-gallery = **{$key}** shows every terminal live, side by side. Type to filter, arrows to move, Enter to open.
tip-gallery-filter = Every thumbnail is live: watch all your agents work at once, and open the one that needs you.
tip-notice = **{$key}** jumps to the newest notification: the agent's terminal, its ticket beside it.
tip-goto = **{$key}** goes to a ticket: type its number, or paste a comment's `#C.` link.
tip-full-list = **{$key}** shows every ticket full screen, the critical one at the top.
tip-bulk = **Ctrl+click** picks tickets, **Shift+click** a range: then close, snooze or mark them all at once.
tip-afk = **{$key}** steps the shown agent's AFK mode: on its own ▶, held 10 minutes ‖, held until you let go ■.
tip-remote-control = **RC** lights up in the agent bar while its Claude is in Remote Control: follow and answer it from claude.ai. `/rc` turns it on.
tip-move-loop = The agent bar's **… · controls** chip (host, tmux or psmux): a click takes or leaves the controls, or moves the loop to the other side.
tip-critical = A red **!** marks the critical ticket: the open one that holds back the most others. Moving it frees the most work.
tip-references = Every **#N** in a ticket is a link, even to a ticket of another project: a click opens it.
tip-comment-link = Click a comment's **#C.** mark to copy its link, then paste it into any ticket or the go-to box.
tip-reply = **Ctrl+Enter** sends your reply; **without notifying** posts it without pinging anyone.
tip-preview = **Write / Preview**: see your Markdown rendered, pictures included, before you file it.
tip-escape = **Esc** leaves the field you type in first; a second **Esc** closes the page.
tip-sections = Drag a section's title to resize it; a double click on a title shares the room out again.
tip-fullscreen = **{$key}** puts the window full screen, and back.
tip-filter = **{$key}** filters the sessions: type, Enter opens, Esc clears.
tip-new-project = **New project…** in the menu (the app's icon, top left) sets a folder up for aiball and starts its agent.
tip-restart = **Restart tvty**, in the menu, restarts the window only: your agents' sessions keep running.
tip-shortcuts = Every shortcut can change: here in **Keyboard shortcuts**, or in `keymap.toml` in tvty's config folder.
tip-opacity = **Terminal opacity** below 100 % lets your desktop show through the terminals.
tip-aiball-board = **aiball**, in the menu, opens aiball's board in your browser: every project's tickets.
tip-font = **{$key}** makes the terminals' font bigger, **{$key-font-smaller}** smaller, **{$key-font-reset}** as it was.
tip-new-ticket = **{$key}** files a new ticket from anywhere, to the project you filed the last one to.
tip-room = **{$key}** folds the ticket panel away, **{$key-sidebar-toggle}** the projects' list: all the room to the terminal.
tip-themes = **{$key}** steps through the colour themes; the title bar's **◐** lists them all.
tip-wizard = The wizard shows what it will write before writing anything: nothing changes until you set it up.
tip-file-and-exit = **File and exit** files the ticket without opening it: you are back at the terminal, or the full list.
tip-panel-pin = The **pin** in the ticket panel's header: pinned, the panel stays beside the terminal; unpinned, it lies over it and the terminal keeps its width.
tip-followers = **Followers** are told of every move of this ticket: ✕ ends one, the dropdown adds one. A muted follower is greyed.
tip-thread-order = **⇅** turns the thread upside down: newest first or last, in the panel and full screen each their own way.
tip-held-selection = Selecting text in a terminal **holds it still** while you copy: the agent's output waits, nothing scrolls away under the mouse.
tip-workspaces = A **workspace** keeps a set of projects and how each of their agents runs, on its own ▶ or held ■. **+ workspace** keeps what runs now under a name.
tip-workspace-open = A workspace's **Open** starts what it keeps and lets go what it runs on its own; **Shut** stops its sessions, those other workspaces share left alone.
tip-project-order = Drag a project in the list onto another to put them in your own order; **⇅** steps between recent first, alphabetical and yours.
tip-language = The two letters in the title bar speak tvty in another language: English, French, Spanish or German.
tip-stop-session = Over a running agent in the list, **⏹** stops its loop: a first click asks, a second stops. It stays there, to start again.
