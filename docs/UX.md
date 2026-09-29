# Interface

What the window looks like and how you move in it. Decisions are marked as
such; open points are listed at the end.

**Everything that means something says what, under the pointer**: a glyph,
a colour, a count, a badge carries a tooltip (`src/tip.rs`, `.tip("…")`).
A new one gets its tooltip with it.

## Navigation: projects, then their terminals

- **Tabs are grouped by project.** A project can have **several terminals**
  (one per agent: a lead, crew agents, a cto…), so the grouping is two-level:
  the project, then its terminals.
- A project's **`>_`** opens a plain shell (no Claude) on aiball's host, in
  the project's folder, named `<project>-1`, `<project>-2`… It is listed
  with the project, after its agents, marked `>_`: a shell the host holds
  goes with the project whose folder it **started** in (a `cd` later does
  not move it). "+ terminal" opens one in the home directory, listed under
  "terminals".
- The projects carry the counters: the critical ticket, the decisions
  waiting for you, the unread — so a collapsed project still says it needs
  you. The critical ticket's red **!** opens it (the project's first, when
  another one is shown): the ticket that holds the most open tickets back.
- A Claude session carries its agent's own, as claude-loop's line has them,
  in light badges with their letter: **b** its backlog (tickets for it to
  look at; `-` until its loop says), **e** its events not seen yet — on
  their colour when not zero, faint otherwise — and the red `!` when it
  holds the critical ticket. The
  list, the tabs, the slider's and the gallery's cards show the same.
- Each agent shows its **Claude's state**, as aiball centralises it (not
  read from tmux: the day loops run without tmux, nothing changes): a colour
  beside its name — blue working, grey idle, violet starting, none offline —
  and a line under it: who drives (▶ the loop on its own, ‖ held, ✎ a human
  typing), the state, for how long. The same line is on its cards.

## A new project

**+ project** in the sessions list's header (also the menu under the app's icon, and "Set up
your first project" while aiball has none) opens a wizard over the window:
one frame of one size, the steps on top (the kit's stepper; a click goes
back to a step done), the page in the middle, its buttons at the bottom
right (`src/shell/newproject.rs`):

1. **The folder**: typed, or chosen with the system's picker (Browse…). It
   says what it is: missing, a file, a git repository, or already an aiball
   project (its `.aiball.yaml`) — then "Open it" shows that project.
2. **Who works in it**: the project's and the agent's names, proposed from
   the folder (`app`, `app-claude`), a crew agent or the lead, a private
   project, no claiming — each with what it means. aiball says, as they are
   chosen, what it will do (a dry run of its `project.init`: creates, keeps
   or updates `.mcp.json` and `.aiball.yaml`), why it would not (a folder it
   cannot write, a name it refuses), and whether the project is on the board
   already — then the folder joins it.
3. **Set it up**: aiball does it, through its API (`project.init`, what
   `aiball init` does; no command run, whatever the OS), and tvty shows what
   it said, or its error.
4. **What next**: start the agent ("Start its first session", on aiball's
   host: the project shows in the list, its tickets beside); accept
   aiball's MCP server when Claude Code asks at its first start in the
   folder (`/mcp` if refused); install aiball's skill when this machine has
   none (`aiball init skill`); give it work with "+ New".

Nothing is written before "Set it up"; Esc leaves a box, then closes it.

## Opening a session

The sessions list ("Sessions", ‹ folds it) has three foldable sections,
each with its count, like the ticket list's bands (one component,
`src/accordion.rs`: ▾/▸ and a click, each section scrolls on its own and
they share the height — **dragging a title** moves the border with the
section above, its least height kept; the heights are kept with the layout,
and a double click on a title gives the list its shares back): **live** (the sessions that run, by project),
**idle** (the loops this machine knows, as aiball lists them
(`loop.list`), that are stopped: ▶ start runs one again where it worked, for
its agent), **shut** (the agents aiball knows with no loop here: ▶ start
opens one where the agent works). A click on a project's name, in any of
the three, shows its tickets in the panel with no session opened ("no
session open" when none of its sessions is here); choosing a terminal
brings the panel back to that terminal's project. "+ session" on a project opens a small
form: the working directory — proposed from the project's loops or agents,
checked as it is typed —, the agent, whether it is a crew agent, and
where it runs: **on aiball's host** (the default, no tmux: aiball's
`session.start`, then tvty attaches over the session's socket) or, box
unticked, in claude-loop's tmux (the same `session.start`, in tmux). Either
way it starts **in that directory**, and tvty never starts Claude itself:
it opens the session once it runs. An agent whose loop runs already (on
the host, or claude-loop's in tmux) is never started again: tvty opens it
**as a copy**, as claude-loop joins a loop that runs.

A terminal has **the controls** or is **a copy** (the agent bar's chip).
The controls are shared, as tmux shares them: claude-loop's own terminal
and tvty both type, and the session's size follows the last one that
typed. A copy only watches: nothing typed reaches Claude, and it never
resizes the session — it is drawn at the session's size. Either leaves
without stopping Claude.

**Quitting tvty** with Claude Code loops of this machine running asks
whether to stop them too (through aiball: they stay restartable) or
keep them running — or cancel; "Remember this choice" keeps the answer
(Options > Layout > Sessions > On quit). The dialog lists the sessions
by project, each with its mark (▶ on its own, ‖ held for now, ■ held
until let go), in a box that scrolls when they are many. The loops tvty
stopped are kept in its workspace with their hold: at the next start it
asks whether to restart them (aiball's `loop.restart`, each where it
ran, its conversation resumed) **as they were** — a held one held again
(`consumer.afk`) — or **fresh**, each booting then running on its own,
or not now; with its own remembered choice (On start). tvty quits once
they stopped, 15 s at most. Loops of another machine are never touched.

When the session shown ends (Claude quit, the loop stopped), its terminal
gives way to an end screen: the agent, its project and directory, and
**Restart** (Enter: its loop starts again there, and opens once it runs)
or **Close** (Esc: back to the terminal used before). The panel keeps the
project's tickets meanwhile; the loop shows under idle.


- **Gallery**: every terminal as a live thumbnail, grouped by project, alerts
  on each. Click one, or type to filter; the arrows move the blue border
  (it starts on the current terminal), enter opens it. The overview.
- **Tabs**: over the terminal, the terminals of its group (a project,
  "terminals", "tmux") open in tvty, as in a browser: its agents, then its
  shells, each with its state and its alerts, the one shown underlined. A
  click or ctrl+pgup / ctrl+pgdn moves along them; "+" opens a shell in the
  project's folder (in the home directory for "terminals"). **×** (on the
  tab shown and the one under the pointer, or a middle click) closes a tab:
  for an agent or a tmux session, only tvty's view goes — Claude runs on;
  a shell of aiball's host stops, as a terminal's tab does. Closing the
  tab shown goes back to the terminal used before it, of any group.
- **The workspace is kept**: closed and started again, tvty opens the
  terminals that were open (those still there), the tab shown last on top. The list on the
  left keeps every terminal, with its state; a click there opens a tab again.
- **Slider** (alt-tab, as a portfolio), by group: holding ctrl+tab dims the
  window and brings each group forward as a **stack** of cards, offset down
  and to the right (four at most, then "+n"), under a single header — the
  group, its terminal on top, the group's counters. The card on top is the
  group's terminal used last, live; the chosen stack is enlarged. Keep ctrl
  held and tap to move from group to group, most recent first; release to
  open the group on that terminal, the tabs then move within it. The
  arrows move to the stack seen left, right, above, below.
- **One order everywhere**: the projects' list, the slider and the gallery
  lay the groups out alike — by default the group used last first, as
  ctrl+tab goes (the groups never used after, alphabetical), or
  alphabetical: **⇅** in the list's header, or Options > Layout. The
  projects' list takes that order once, when the work left last time is
  back, and keeps it: a click never moves a project under the pointer
  (projects that come later go last; ⇅ takes it afresh); the slider and
  the gallery follow the order of use as it goes. A group's
  terminals keep their place, so that its tabs never move. While the slider
  is up, nothing moves: the chosen card is enlarged over its own slot.
- **A filter atop the list** (ctrl+shift+f unfolds the list and goes to
  it): every word typed must be found in a row's project, agent, name or
  folder; the three sections keep what it finds, a folded one opens while
  it holds something, the words are marked. Enter opens the live session
  the arrows are on (the first found), Esc empties the filter, then gives
  the keys back to the terminal.
- **The cards are live**: a terminal open in tvty draws its own screen; any
  other session gets a read-only client (`tmux attach -r`: read-only and
  ignore-size, so it never resizes the session) while the slider or the
  gallery shows it, closed with them. Each card shows a viewport — the
  bottom 30 lines, first 100 columns, where Claude Code writes — which
  keeps a frame light (measured: at most 6 ms for nine cards, against 51
  for their full grids).

## The ticket panel

- On a project's terminal, a **panel on the right** with that project's tickets
  (the agent's current one first, then its queue), interactive: the usual
  gestures — accept, reject, reply.
- **Resizable, one third of the window by default**, and collapsible. Its
  width and whether it is open are remembered.
- **Clicking a ticket opens its detail**: the thread, its decisions, the reply
  box.

## The ticket list

The band, whose turn it is and the state glyph are computed by aiball for
the reader (`/api/inbox?view=turn`); tvty draws them (`src/rowstate.rs`):

- **Bands**: agents on it first — the work under way —, then to moderate,
  waiting on you (a decision), open. The most recent activity
  first within a band. In the panel each band is a section: a click on its
  title folds it, and each scrolls on its own — a long band never pushes
  the others out of sight.
- **Unread is a mark, not a band**: the comments' bubble in blue, and the
  title in bold; read titles step back. The ticket keeps its band. Read,
  the bubble points to who spoke last: its point on the left and bright
  when someone else did, on the right and discreet when you did.
- **One state icon**, always in the same place — Google's Material Symbols,
  as aiball's web UI draws them (`src/icons.rs`): `edit_note` plan,
  `check_circle` resolution, `block` wontfix, `priority_high` escalation,
  `play_circle` step (`pause_circle` when it went quiet), `cancel` rejected (always red),
  `task_alt` / `lock` closed. Coloured when it waits on you, muted
  otherwise; a decision wins over a later step. The flame (an agent active
  lately), the critical warning and the priority arrows are icons too: an
  emoji would show grey.
- **The stripe says whose turn**: coloured and solid when a decision waits
  on you and is the last message, dashed when the talk went on after it,
  neutral when an agent answered you, a thin dotted line when your word is
  the last (you wait), none otherwise. A ticket waiting for moderation
  wears a construction tape instead — yellow and black bands: nothing goes
  on until it is let through.
- Then who spoke last and aiball's **comment count** (a bubble and the
  number; a clock while comments wait for
  moderation), the agent holding it (🔥 when active), ⚠N on the critical
  ticket (and how long it went quiet: "⚠2 · 9 d"; its header says "holds 2
  · quiet 9 d"), a high priority, the time. A thread full screen is headed by the
  same count and who spoke last; its fields column (a third of the window,
  460 px at most, by default) is resized by dragging its border, and a
  double click on the border gives the default back. The full list's side
  and the new ticket's fields are the same column, with the same width
  (`src/sidecol.rs`).

The legend is in the options page, under "Ticket list".

## The ticket list, full screen

The panel is the compact list, beside the terminal: enough to steer. The
full list is headed, when nothing filters it, by **Critical**: each
project's critical ticket, what holds the most back first, then what went
quiet the longest — the tickets to move first, whatever their project. ⤢ in
its header, or ctrl+shift+l, goes full screen on what the panel shows: its
list opens the list full screen, on the same project; its open ticket opens
that ticket full screen. Coming back (Esc, ✕, ctrl+shift+l again) finds the
panel as it was, whatever was browsed full screen. The list full screen,
over the window, is to look over the board (`src/fulllist.rs`):

- **The left third, the scope and the counters**: all projects or one, each
  with its badges; the bands with their counts — a click narrows to one, a
  second click widens again; open tickets or all of them (closed ones are
  read on demand); a search in the titles; the tags present, to narrow to
  those carrying them all.
- **The rest, the list**: the same bands, stripe, glyph and weight as the
  panel, and all the row has to say — the start of its body, intent, level,
  tags, milestone, priority, who holds it (claimed or assigned), who spoke
  last, tokens, critical, blocked, snoozed, its scope, who filed it and when.
- **One list, sorted**: by last activity (the default), whose turn (the
  bands' order), priority, creation or number; a second click on the sort
  reverses it. The bands are a filter here, not sections.
- **Several tickets at once**: ctrl+click adds a row to the selection (or
  takes it out), shift+click takes the rows between the last one and this
  one, ctrl+a every row shown (outside the search box). With rows chosen,
  the left third becomes their actions, each with the number of chosen
  tickets it applies to (greyed at none): approve, reject, close, reopen,
  mark read or unread, snooze three days, unsnooze, mark as step, link
  (the newest relates to the others). Close and reject ask first. Esc
  clears the selection, then closes the list (`src/bulk.rs`).
- Opening a ticket shows it in the panel; the list, hidden, keeps its scope
  and filters for the next time. Esc closes it.

## Filing a ticket

One gesture, three ways in: **+** in the panel's header or the full list's,
ctrl+shift+n anywhere, and "+ a new one" under a ticket's links (full
screen) for a sub-ticket. The form fills the window, laid out as a ticket's
full detail: what the ticket is on the left third — project (the panel's,
else the terminal's, else the last used; any project of the board, in a
list that opens), then every other field with all its choices in sight,
the chosen one lit: intent, priority, level, scope (and who it notifies),
tags, milestone, assignee (the project's agents first), and a parent to
type as `#ticket` — and its words on the
rest: title, and the body with @-mentions and pasted images, under
**Write / Preview** tabs — Preview shows it as it will read (its images
said). No summary: aiball's agents write one, a person's title says
enough. Full screen, a ticket's reply has the same tabs.

Ctrl+enter files it. One call creates it (with the machine's platform tag,
as every aiball client does); tags, assignee and milestone follow, as
aiball's web UI does them. A ticket that exists is worth more than one
perfectly labelled: what does not follow is said on the new ticket, which
opens full screen. Esc puts the form away and keeps the draft.

## A ticket's thread

The detail says where the ticket stands once, then the talk, then the
gestures (`src/thread.rs` reads the thread, the panel draws it):

- **On top**: the list's glyph and the title; one sentence for whose turn it
  is ("Yours: accept or reject demo-claude's plan", "demo-claude is on a step
  · waits on #9", "Closed, resolved by you"), with the list's stripe; short
  chips only when they say something (who holds it — a lapsed claim does
  not count —, a high priority, the tickets it holds, tokens, what it
  depends on or blocks); and the latest `summary_until`, pinned: where it
  stands. A step's comment says when its agent resumes, as it said
  (`resumes 08:16 or on #8`: a time to come, local, and the ticket it waits
  on, a click away); the list's step glyph says the time in its tip.
- **The talk**, oldest first, so the latest word sits by the reply box, where
  the thread opens. Comments before the latest snapshot fold to one line —
  their own snapshot, else their first line — as aiball's `brief` mode cuts
  a thread; a click on a comment's head unfolds it. Decisions carry the
  list's glyphs, and **"superseded"** when a newer decision replaced them:
  only the latest can be taken. Only the latest step shows as running.
  Events are one grey line, merged when one author repeats them.
- **⇅ sets the order**: newest last, the reply box under the talk (the
  default), or newest first, the reply box at the top; it is remembered.
- **The frequent gestures stay at hand** in the panel — a dev's round is
  read, decide or answer, next: a click on the priority chip changes it;
  Snooze gives an hour, a day or a week (Wake when snoozed); "without
  notifying" sends the next reply to nobody; `@` at the end of the reply
  offers who to mention; an image pasted (Ctrl+V) goes to aiball's uploads
  and its link into the reply; an open question (`- [ ]`) of a comment has
  an Answer chip that quotes it into the reply, and sending ticks it.
- **Full screen** (⤢ more in the detail, or opening a ticket from the full
  list): the title across the top; on the left third what holds for the
  ticket as a whole — its state and chips, lifecycle and snooze, its fields
  (intent, priority, level, milestone, scope, tags), who is on it (reporter,
  a claim and until when — a lapsed one says so —, an assignment), what it
  is linked to, its tokens — each changeable in place: a click on a row
  offers its values (intent, priority, level, scope, milestone, tags, the
  reporter, the assignee or release, a relation to add or undo, the
  project to move to), the title and body open an editor; no reply goes
  with these; on the rest the talk at a readable width, whole — each
  comment with a ⋯ menu: edit, delete (a second click confirms), classify
  as a plan, a resolution, closing without a fix or an escalation (one way,
  the four kinds; accepting stays with the decision card), no decision, a
  step (an agent's comment), vote, resurface, copy its `#C.` reference —
  unless folded on demand, with the same gestures. Esc returns where it was
  opened from: the panel, or the full list.
- **The gestures, in one place**: the pending decision (not your own) with
  Accept — "→ go", "→ close" — and Reject, which wants a reason typed first;
  moderation; the reply box; Close or Reopen. What is typed is posted first,
  so every gesture carries its why. The rest (snooze, editing, relations,
  votes) stays in aiball's web UI.
- **Every ticket reference is a link**, says its ticket under the pointer
  ("#12 — its title", its project, open or closed: from the board when the
  ticket is on it, else asked of aiball at the first hover and kept ten
  minutes), and follows the same way wherever it is painted — a thread's words (`#12`, a comment's `#C.` link; not in code
  or an address), its events, the relations and sub-tickets, the step it
  waits on, the agent's backlog, a new ticket's parent. Full screen, the
  ticket opens full screen; in the panel beside a terminal, another
  project's ticket first moves the panel there (its terminal used last, or
  its tickets alone), then opens.

## The agent's bar

Under an agent's terminal, one line: what claude-loop's tmux status line
says, drawn by tvty from aiball (never read from tmux), about **the agent**
— the panel beside it is about the project. Once the loop pushes its bar
to aiball (`/api/consumers/:id/bar`, and `agent_bar` on the live feed), all
of it; before (a loop started earlier), what the agent's state says.

Left to right: who drives the loop, as claude-loop's bar says it — ▶ it runs
on its own or ‖ held (the mode in force), ⌨ while you type, then the little
man for the AFK mode armed: grey (you are away: auto), `웃348s` the seconds
of a ten-minute hold, `웃∞` held. **F9**, anywhere in the window, moves it
one step (auto → 10 min → ∞ → auto); the mode takes effect 3 s after the
last press, and until then the little man is dotted, with "…". A click
offers auto, hold 10 min, hold. Then its Claude's
phase, for how long, and the passing word (`retry 2`, `compacting`); when
its Claude Code installed an update, **⟳ update** — a click has its loop
restart it as soon as it is idle ("⟳ restart pending" meanwhile, an
offer turned into a state), and the terminal comes back
on its own; "waits for an answer" when a
dialog is up; the alerts in red (usage limit reached · resets …, trust
this folder?, not logged in, API unreachable, loop link down, aiball
unreachable) — a usage limit also comes as a notification, the loop held
until let go (■); the prompt ❯ (bright when
it holds text), ⌨ while a human types, ⇄ the proxy; its counters in
claude-loop's terms and full words — `all:` the project's open tickets,
`backlog:`, `events:` (claude-loop's a: b: e:) — then the tickets it holds;
a click on `backlog:` lists **its own backlog** by tier
(`/api/consumers/:id/backlog`), a click on one opens it —; the next wake;
where its loop runs (`host`: aiball's session host, `tmux`: claude-loop in
tmux) — for a loop of this machine, a click offers to **move** it to the
other (a confirmation over the bar, which says when Claude works; aiball
restarts it there — `loop.restart` —, resuming its conversation, and the terminal comes back
on the new session, "moving…" meanwhile); **RC**, lit while its Claude is in
Remote Control, whatever turned it on — its folder's setting or `/rc` typed —
as its loop reads it on the screen (said, not set: none from a loop too old
to say it); whether this terminal has the **controls** or is a **copy** (a
click switches: the terminal leaves and comes back in the other mode, its
Claude goes on); its name and where it works. A gesture aiball refuses
comes as a notification. The countdowns move every second.

## Images in a thread

An image alone on its line, or ending it (`Here: ![shot](…)`) — a pasted
capture — is drawn by tvty: a
thumbnail in the compact panel (about 160 × 100, its proportions kept, ⤢ in
a corner), large full screen. A click opens the **viewer**, over the whole
window: the image fitted first; the wheel zooms about the pointer, + and −
too, 1 shows the real size, 0 or a double click fits again, a drag moves
it, ← → go through the thread's images, Esc closes. Its bar names it (the
text's alt, else its file), gives its size and zoom, and opens it in the
default application when aiball keeps it on this machine. An image inside
a sentence stays in the text.

## The terminal comes to you

When something newly waits on the user from an agent — a decision it
proposes, or something new on a ticket it holds — a **notification** comes: in the
terminal's top right corner — in the window's bottom left one while a full
screen (the options, the full list, a ticket or a new ticket full screen)
covers the terminal, out of its way — above everything (full screens and
gallery included), a little translucent. They stack from their corner, the
newest in it, at most five; each goes after six seconds, unless the pointer is on it (both
set in Options > Appearance). While one is up, its ticket **shines** in
every list shown, in the notification's colour. A click, or ctrl+enter for the newest, brings that
agent's terminal (it slides in when the project changes — a tab of the same project shows at once) and opens the ticket in the panel; × lets
it go. No switch happens on its own: the user may be typing elsewhere.
What came while tvty was closed comes at start as one notification — the
last ping said, the others counted ("· and 2 more while tvty was
closed"); a dropped connection replays what it missed, one by one.
The user's own gestures come the same way, once aiball has them — "#12
closed", "decision accepted", "filed", in green (Options: hide them) — and
a gesture aiball refuses always does, in red. One place makes them all
(`src/activity.rs`): the views publish what happened on the internal bus,
never a notification themselves.

**aiball's pings to the user** come too: tvty subscribes to them on aiball's
bus (`user.<me>.pings`, over the socket), with the board itself. Each ping
— an agent answered, mentioned you, proposes a plan or to close, a new
ticket — is a notification (red for a panic, yellow for a decision to
take), quoting the start of what was written; a click opens its ticket
and goes to its project — the agent's terminal when it has one here, else
the project's terminal used last, else its tickets alone —, which marks it read in
aiball, so its web UI agrees. At start, one notification sums up the pings
waiting unread; a click opens the full list filtered to the unread. A ticket
has one notification at a time, the newest.
A **proposal to decide** — a plan, a resolution, a wontfix, an escalation,
by someone else on a project of the board — comes even when its ping does
not: tvty also follows the board's messages (`board.events`), as aiball's
web UI does. Whichever comes first is said; the other is not said again.
tvty's log (`~/.local/state/tvty/tvty.log`) keeps the bus's connections,
the pings, the proposals and the notifications, to tell a missed one apart.

Views talk through an internal bus (`src/bus.rs`): one publishes a signal
(a notification, a ticket to open, a new ticket asked for, the board
changed), whoever cares listens — a list does not know who opens its
tickets, a notification does not know which lists light its ticket up.

## Folding the sides

The **projects' list lies over the terminal** (it does not push it aside),
at a width set by dragging its edge; the ticket panel sits beside the
terminal, its width set the same way. Both widths are remembered.

Both sides — the projects' list and the ticket panel — fold with a **grip**
on their edge (or their key, below) into a **10-pixel strip** that still
says something. On the left, one mark per running Claude, in the list's
order: its colour its state (grey idle, blue working, yellow booting, faint
offline), its shape who drives it (▶ on its own, ‖ held for a while, ■ held
until let go, a dot otherwise), its name and state under the pointer. On the
right, the selected terminal's project, as dots: red for the critical
ticket, orange for a decision, blue for unread. A click on the strip
unfolds it. Both states are remembered.

## Colour themes

**Terminal opacity** (a slider under the title bar's theme menu, and in
Options > Appearance > Colours; 100 % by default):
below 100 %, the desktop shows through the terminals' background, live —
the compositor blends it at every frame, on Wayland and X11. What a program
colours itself, the lists, the tickets and the full pages stay opaque; the
window turns see-through only then. "Blur behind" blurs it where the
compositor can (KDE); elsewhere it shows sharp.

Everything takes its colours from one theme — the window, the panels, the
alerts, the terminals' palette (the theme's ANSI colours, text, background,
cursor, selection). Pick it from the title bar (◐ and its name), or step
through them with ctrl+shift+k; it is remembered. Six theme sets are bundled
(light and dark variants, see `themes/`), and any gpui-component theme file
dropped in `~/.config/tvty/themes/` joins the list.

The terminals can have a theme of their own — a dark terminal in a light
window, as many like it: in the options' Appearance, next to the window's
theme. By default they follow the window's.

## Sizes

Two sizes, in Options > Appearance, kept in `settings.toml`: the terminals'
font (8 to 32 px, 14 by default; ctrl+shift+= / ctrl+shift+− / ctrl+shift+0
too — shift so that no key is taken from the programs in the terminal) and
the window's text (12 to 22 px, 16 by default: the kit's size, which sets
everything else). A new terminal size applies at once: each grid is laid
out again, and its PTY — the tmux session — resized once it settles. The
cards keep their size: they fit the screen to themselves.

## Options

A full page (⚙ in the title bar, or ctrl+,): on the left a search box
(focused at once: ctrl+, then type) and a tree, each page unfolded into its
groups — a click on a group brings it up, the one in view is lit. A search
finds settings and shortcuts together, by name, text, key
(`terminal_font_size`, `ctrl+shift+b`) or value, the words marked, the tree
cut down to what matched; Esc clears it, then closes the page. A number is
stepped with − and +, or slid when it is a range the eye reads (an
opacity, in %). A setting or
a shortcut away from its default stands out (lighter, a bar on its left),
says its default (`Default: 16 px`) and has a "↺ Default" button that puts
it back — in an aiball project, "↺ Board", the board's value; `@modified` lists every one; a setting shows its key as settings.toml
spells it. The pages:
Appearance (sizes, notifications, the wheel's speed, the window's and the
terminals' colour themes, the terminals' opacity), Layout (the sides: folded or not, their widths,
reset), Ticket list (the thread's order, the list's legend), Keyboard
shortcuts (all of them, edited in place — see Keys), aiball (the board's
config, read and written through aiball, in the scope chosen: Global or a
project; a project's value overrides the board's, ↺ clears it
there; 🔒 on the keys only a human may change; a change made elsewhere
shows at once), About (version,
aiball's socket, who tvty acts as, the live feed, where settings and
themes live).

**Scope.** Atop the tree, Global or a project of the board. Global: the
pages above. A project (or its ⚙ in the sessions list): a **Project** page
comes first, and the aiball page shows that project's layer. The Project
page has two groups. *Folder*: what the folder's `.aiball.yaml` sets, as
aiball resolves and writes it (tvty never opens that file) — where the
loops started there run (session host or tmux) and its Claude's Remote
Control, each saying where its value comes from (the file, the machine's
global config, the default), ↺ removing it from the file; a project with
several folders (a lead's, a crew's) shows one at a time, and says which
other folders the same file serves. *Board*: the board's keys the project
sets over the board's values, and a link to all of them.

The settings' pages are built from the settings themselves: each is
declared once in `src/settings.rs` (its key in `settings.toml`, its page and
group, what it says, its kind and bounds — `crates/tvty-config`), and a
change from a page, a shortcut or a hand edit of the file goes the same way:
into the store, which writes the file and puts it in force.

## Tips

"Did you know?": a small card in the tips' own colour, a violet (not the
accent's blue nor a state's colour), never in the way. A tip about an
element of the window sits beside it, never over it — under it, or above
when there is no room, 24 px away so that the element and what is around it
stay in sight —, the element wearing a violet halo; when that element is not
on screen, when there is no room either way, and for a tip about a key, the
card keeps its corner: bottom left (bottom right over a full page, where the notifications
take the left). One
comes 5 s after start, and one the first time a page is opened (the full
list, a ticket full screen, the gallery, the options…). A tip says one
thing, with its key as the keymap in force has it; the tip of a command
already used is not shown, nor one shown in the last day.

- **Got it**: that tip never comes back; **Next tip**: another one for
  the same page; **✕**: not now; **Turn tips off**: none any more, until
  Options > Layout > Tips turns them back on.
- The menu's **Tips…** goes through them all (‹ Previous, Next ›), and
  **Show them all again** forgets which ones were understood.
- The tips are files, `tips/<id>.md`: a header (`id`, `surface` — a key
  context —, `command`, `target` — an element the code marks with
  `tips::target`) and one short text, `{key}` for the command's key.

## The wheel

A notch moves about three ticket rows in the lists and the threads (GPUI's
own step, three lines of text, was barely one row), and five lines of a
terminal's history. In a terminal on tmux, the notches go to tmux in order,
one call at a time, those that come meanwhile added up into the next: a
fast wheel no longer lags behind. A touchpad scrolls as it did, by pixels.
`speed` under `[scroll]` in `settings.toml` multiplies both (1 by default).

## Buttons

Everything that acts, and is not a row of a list (a ticket, a session, a
tab, a menu item), is one of five kinds (`src/ui/buttons.rs`):

- **link**: an action in words, in the accent colour, no border ("+
  terminal", "← Tickets", "⇅ recent", "✕ Esc");
- **icon**: a glyph alone (‹ › ⤢ ⚙ ⋯), muted; its tooltip says the action
  and its shortcut. The ✕ or × that takes something away turns red under
  the pointer;
- **chip**: a bordered label, for an option or a state to change ("+ New",
  a choice among several, the agent bar's chips); a chosen one has the
  chosen row's background and an accent border;
- **answers** of a form or a dialog, all the same size whatever their
  kind: **primary** the main one, **secondary** (outlined) the others —
  Cancel, Not now, Back —, or a colour of their own for a decision
  (Approve, Reject, Move). Never a chip nor a link beside them.

Under the pointer each gets a light background (an icon's glyph
brightens), pressed a stronger one, and the hand pointer; one with
nothing to do now is greyed, with neither. In a bar, groups of actions are
set apart by a thin vertical line; inside a group, a plain gap.

What folds leads with ▾ (open) or ▸ (folded), in the accordions' style —
the same glyphs, colour and width: the list's sections, a thread's folded
comments and the ticket's body. The whole head, and a folded line, fold or
unfold it on a click, with a background under the pointer.

## Keys

Every shortcut is a **command**, bound to keys in a **context** that
follows the focus — each surface that holds it is one (`src/keymap.rs`):
`Window` everywhere; `Workspace`, the terminals, the sessions list and the
panel beside them; `Terminal`, a terminal in it; and each full-screen page
(`FullList`, `Ticket`, `Options`, `NewTicket`, `NewProject`, `Gallery`),
beside the workspace, not in it. A workspace command (the sides folded, the
tabs, the sessions filter, F9, the font size) is not there on a full-screen
page; the page up takes the focus, and gives it back to the terminal when
it goes. A global one (`Window`) reaches every surface, fields and terminals
included — none is a key a field edits with nor one of the terminal's (a
test says so) — and one that goes somewhere (the slider, the gallery, a
notification, a ticket gone to) puts the full-screen page away first, its
draft kept. The deepest binding wins, and a key bound nowhere
goes on to the terminal's program. `keymap.toml`, beside `settings.toml`,
overrides the defaults below — a `[Window]`, `[Workspace]` or `[Terminal]` section, and in
it `ctrl-alt-t = "theme.next"`: a key bound to a command by its name, or
`ctrl-c = false` to give it back to what has the focus. A key may be
written as Options shows it (`"Ctrl+Alt+T"`), modifiers in any order. It is
read again once saved; a wrong one is said in a notification and changes
nothing.

Options > Keyboard shortcuts edits the same file: a key clicked listens for
its replacement (every shortcut is off meanwhile, so that Ctrl+Tab is heard,
not run; Esc gives up), + adds a key, × removes one (a default key is then
given back), Default puts a command's keys back, Reset all every one. A key
another command runs asks first (Replace / Cancel); a key a terminal's
program types (a letter, Enter) is refused in the Terminal context. The
page shows too what is changed, and a window key a terminal masks. The file
keeps only what differs from the defaults (`crates/tvty-keys`).

| Keys (by default) | |
|---|---|
| ctrl+tab (shift: backwards) | the slider, a stack per group, most recent first; arrows move too; release ctrl to switch |
| ctrl+pgup / ctrl+pgdn | the tab before / after, in the group shown |
| ctrl+shift+space | the gallery; type to filter, arrows move, enter opens, esc closes |
| ctrl+shift+t | fold / unfold the ticket panel |
| ctrl+shift+n | a new ticket; ctrl+enter files it, esc keeps the draft |
| ctrl+shift+= / − / 0 | terminal font bigger / smaller / default |
| ctrl+shift+b | fold / unfold the projects' list |
| ctrl+shift+f | filter the sessions |
| ctrl+shift+g | go to a ticket: the title bar's "#…" (a number or a comment's #C. link, Enter opens it full screen, whatever its project; Esc lets it go) |
| F9 | the shown agent's AFK mode, one step |
| ctrl+shift+k | next colour theme |
| ctrl+, | options |
| F11 | the window full screen, or back |
| F1 | the menu under the app's icon, at the title bar's left: about Terminal Velocity (version, build), a new project, full screen, the documentation, the shortcuts, the tips, what's new, GitHub, report an issue, aiball (its web UI at the local address aiball gives, as its GNOME extension opens it) — and **Restart tvty**: the window only, the Claude Code sessions and the terminals run on and come back |
| ctrl+enter | go to the newest notification: the agent's terminal, its ticket open |
| ctrl+shift+c / ctrl+shift+v, shift+insert | copy the selection / paste the clipboard (bracketed when the program asks) |
| drag, double click, triple click | select text, a word, a line — whatever the program, tmux with its mouse on too; copied at once to the primary selection, and kept even when the program draws over it |
| middle click | paste the primary selection |
| right click | a menu: Copy, Paste (Esc closes it) |

Esc in a text field (a search, the reply, a form's box) only leaves the
field; the next Esc closes the page it sits on. A click anywhere else — a
button, a menu, a list — leaves the field too; picking a mention gives it
back, to go on typing.

Everything else goes to the terminal.

## Open points

- **Where the ticket detail opens.** Proposed, and what tvty does for now: in
  the panel itself, replacing the list, with a back arrow. Not done: widening
  the panel when the thread needs room. Alternative: a floating pane over the
  terminal.
- **Which tickets the panel lists** for a project with several agents: those of
  the terminal in focus (proposed), or the whole project with a filter.
