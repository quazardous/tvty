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

## Two ways to switch

- **Gallery**: every terminal as a live thumbnail, grouped by project, alerts
  on each. Click one, or type to filter. The overview.
- **Slider** (alt-tab): a horizontal strip over the current terminal; keep the
  modifier held and tap to move along it, release to switch. The quick hop
  between the few terminals you are juggling, most recent first.

## The ticket panel

- On a project's terminal, a **panel on the right** with that project's tickets
  (the agent's current one first, then its queue), interactive: the usual
  gestures — accept, reject, reply.
- **Resizable, one third of the window by default**, and collapsible. Its
  width and whether it is open are remembered.
- **Clicking a ticket opens its detail**: the thread, its decisions, the reply
  box.

## Folding the sides

Both sides — the projects' list and the ticket panel — fold with a **grip**
on their edge (or their key, below) into a **10-pixel strip** that still
shows what waits, as dots: red for the critical ticket, orange for a
decision, blue for unread. On the left, one dot per project with something
waiting; on the right, the selected terminal's project. A click on the
strip unfolds it. Both states are remembered.

## Keys

| Keys | |
|---|---|
| ctrl+tab (shift: backwards) | the slider, most recent first; release ctrl to switch |
| ctrl+shift+space | the gallery; type to filter, enter opens the first, esc closes |
| ctrl+shift+t | fold / unfold the ticket panel |
| ctrl+shift+b | fold / unfold the projects' list |

Everything else goes to the terminal.

## Open points

- **Where the ticket detail opens.** Proposed, and what tvty does for now: in
  the panel itself, replacing the list, with a back arrow. Not done: widening
  the panel when the thread needs room. Alternative: a floating pane over the
  terminal.
- **Which tickets the panel lists** for a project with several agents: those of
  the terminal in focus (proposed), or the whole project with a filter.
