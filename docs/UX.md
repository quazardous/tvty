# Interface

What the window looks like and how you move in it. Decisions are marked as
such; open points are listed at the end.

## Navigation: projects, then their terminals

- **Tabs are grouped by project.** A project can have **several terminals**
  (one per agent: a lead, crew agents, a cto…), so the grouping is two-level:
  the project, then its terminals.
- Each terminal carries its agent's name, its current ticket, and its alerts
  (unread, a decision waiting for you, the critical ticket, a human awaited).
- A project shows the sum of its terminals' alerts, so a collapsed project
  still says it needs you.
- Each agent shows its **Claude's state**, as aiball centralises it (not
  read from tmux: the day loops run without tmux, nothing changes): a colour
  beside its name — blue working, grey idle, violet starting, none offline —
  and a line under it: who drives (▶ the loop on its own, ‖ held, ✎ a human
  typing), the state, for how long. The same line is on its cards.

## Two ways to switch

- **Gallery**: every terminal as a live thumbnail, grouped by project, alerts
  on each. Click one, or type to filter; the arrows move the blue border
  (it starts on the current terminal), enter opens it. The overview.
- **Slider** (alt-tab, as a portfolio): holding ctrl+tab dims the window and
  brings the terminals forward as live thumbnails, grouped by project, the
  chosen card enlarged; keep ctrl held and tap to move, release to switch.
  Tab goes most recent first: the quick hop between the few terminals you
  juggle. The arrows move to the card seen left, right, above, below.
- **Cards never move**: both lay them out as the projects' list does. Only
  the choice moves — in the slider, the chosen card is enlarged over its
  own slot, above its neighbours, so nothing shifts under the eye.
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
the reader (`/api/inbox?v=tvty`); tvty applies the same rules itself
(`src/rowstate.rs`) for an aiball that does not:

- **Bands**: agents on it first — the work under way —, then to moderate,
  waiting on you (a decision), unread, open. The most recent activity
  first within a band. In the panel each band is a section: a click on its
  title folds it, and each scrolls on its own — a long band never pushes
  the others out of sight.
- **Weight says unread**, and nothing else does; read titles step back.
- **One state icon**, always in the same place — Google's Material Symbols,
  as aiball's web UI draws them (`src/icons.rs`): `edit_note` plan,
  `check_circle` resolution, `block` wontfix, `priority_high` escalation,
  `play_circle` step (`pause_circle` when it went quiet), `undo` rejected,
  `task_alt` / `lock` closed. Coloured when it waits on you, muted
  otherwise; a decision wins over a later step. The flame (an agent active
  lately), the critical warning and the priority arrows are icons too: an
  emoji would show grey.
- **The stripe says whose turn**: coloured and solid when a decision waits
  on you and is the last message, dashed when the talk went on after it,
  neutral when an agent answered you, none when the ball is with the agent.
- Then who spoke last and how many messages, the agent holding it (🔥 when
  active), ⚠N on the critical ticket, a high priority, the time.

The legend is in the options page, under "Ticket list".

## The ticket list, full screen

The panel is the compact list, beside the terminal: enough to steer. ⤢ in
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
- Opening a ticket shows it in the panel; the list, hidden, keeps its scope
  and filters for the next time. Esc closes it.

## Filing a ticket

One gesture, three ways in: **+** in the panel's header or the full list's,
ctrl+shift+n anywhere, and "+ a new one" under a ticket's links (full
screen) for a sub-ticket. The form fills the window, laid out as a ticket's
full detail: what the ticket is on the left third — project (the panel's,
else the terminal's, else the last used; any project of the board), intent,
priority, scope, tags, milestone, assignee, parent — and its words on the
rest: title, a one-line summary, the body with @-mentions and pasted images.

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
  stands.
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

## The agent's bar

Under an agent's terminal, one line: what claude-loop's tmux status line
says, drawn by tvty from aiball (never read from tmux), about **the agent**
— the panel beside it is about the project. Left to right: who drives the
loop (▶ auto, ‖ held, ✎ you type) — a click offers auto, hold 10 min, hold,
through aiball (`POST /api/agents/:id/afk`) —; its Claude's state and for
how long, or offline; its events not seen yet; the tickets it holds; its
wait credit; its name and where it works. A gesture aiball refuses is said
in the bar. Its own backlog and the rest of claude-loop's bar (alerts, next
wake, prompt, typing) come with aiball's API for them.

## The terminal comes to you

When something newly waits on the user from an agent — a decision, or
something new on a ticket it holds — a banner rises over the terminal
("demo-crew — a decision waits on #3 …"). ctrl+enter, or a click, brings
that agent's terminal (it slides in) and opens the ticket in the panel. No
switch happens on its own: the user may be typing elsewhere.

## Folding the sides

The **projects' list lies over the terminal** (it does not push it aside),
at a width set by dragging its edge; the ticket panel sits beside the
terminal, its width set the same way. Both widths are remembered.

Both sides — the projects' list and the ticket panel — fold with a **grip**
on their edge (or their key, below) into a **10-pixel strip** that still
shows what waits, as dots: red for the critical ticket, orange for a
decision, blue for unread. On the left, one dot per project with something
waiting; on the right, the selected terminal's project. A click on the
strip unfolds it. Both states are remembered.

## Colour themes

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

Two sizes, in Options > Appearance, kept in `state.json`: the terminals'
font (8 to 32 px, 14 by default; ctrl+shift+= / ctrl+shift+− / ctrl+shift+0
too — shift so that no key is taken from the programs in the terminal) and
the window's text (12 to 22 px, 16 by default: the kit's size, which sets
everything else). A new terminal size applies at once: each grid is laid
out again, and its PTY — the tmux session — resized once it settles. The
cards keep their size: they fit the screen to themselves.

## Options

A full page (⚙ in the title bar, or ctrl+,) with its sections on the left:
Appearance (the window's and the terminals' colour themes), Layout (the sides: folded or not, their
widths, reset), Keyboard shortcuts (all of them, from the one list in
`src/options.rs`), About (version, aiball's socket, who tvty acts as, the
live feed, where settings and themes live). Esc closes it.

## The wheel

A notch moves about three ticket rows in the lists and the threads (GPUI's
own step, three lines of text, was barely one row), and five lines of a
terminal's history. In a terminal on tmux, the notches go to tmux in order,
one call at a time, those that come meanwhile added up into the next: a
fast wheel no longer lags behind. A touchpad scrolls as it did, by pixels.
`scroll_speed` in `state.json` multiplies both (1 by default).

## Keys

| Keys | |
|---|---|
| ctrl+tab (shift: backwards) | the slider, most recent first; arrows move too; release ctrl to switch |
| ctrl+shift+space | the gallery; type to filter, arrows move, enter opens, esc closes |
| ctrl+shift+t | fold / unfold the ticket panel |
| ctrl+shift+n | a new ticket; ctrl+enter files it, esc keeps the draft |
| ctrl+shift+= / − / 0 | terminal font bigger / smaller / default |
| ctrl+shift+b | fold / unfold the projects' list |
| ctrl+shift+k | next colour theme |
| ctrl+, | options |
| ctrl+enter | go to what the banner shows: the agent's terminal, its ticket open |
| ctrl+shift+c / ctrl+shift+v | copy the selection / paste (bracketed when the program asks) |

Everything else goes to the terminal.

## Open points

- **Where the ticket detail opens.** Proposed, and what tvty does for now: in
  the panel itself, replacing the list, with a back arrow. Not done: widening
  the panel when the thread needs room. Alternative: a floating pane over the
  terminal.
- **Which tickets the panel lists** for a project with several agents: those of
  the terminal in focus (proposed), or the whole project with a filter.
