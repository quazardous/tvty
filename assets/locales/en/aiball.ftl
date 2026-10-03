# aiball's config, as Settings > aiball shows it: by its key, stable (a key
# tvty does not know shows aiball's own English). Dots in a key are dashes.

aiball-group-autopoll = Autopoll
aiball-group-backlog = Backlog
aiball-group-claude-loop = Claude loop
aiball-group-defaults = Defaults
aiball-group-earn = Earn
aiball-group-on-repetitive-denied = On repetitive denied
aiball-group-rules = Rules
aiball-group-steps = Steps
aiball-group-tickets = Tickets
aiball-group-updates = Updates
aiball-group-wait-credit = Wait credit

aiball-config-updates-check = Check for updates
    .about = true (default) = when the daemon starts, and when someone asks, it reads the latest aiball release on GitHub so the tray, the GNOME extension and `aiball version` can say an update is out. false = no outbound call.
aiball-config-tickets-defaults-priority = Default ticket priority
    .about = Priority applied to a new ticket created without an explicit one. Set a global default; a project may override it.
aiball-config-tickets-defaults-broadcast-new = Broadcast new tickets by default
    .about = When on, a new ticket with no explicit scope is flagged broadcast (project followers are pinged). Declared; enforcement lands with its consumer.
aiball-config-tickets-rules-summary-max = Summary budget (characters)
    .about = Longest summary_until an agent may write on a comment. A longer one is refused with an explanation and nothing is posted; it is never truncated. Humans are exempt. 0 = no limit.
aiball-config-tickets-rules-require-then = Agent comments need then: or handback
    .about = When on, an agent's comment with no then: must set handback (true: it hands the ticket back, false: it keeps it), or it is refused with an explanation and nothing is posted. Humans are exempt.
aiball-config-tickets-rules-require-commits = Agent comments need commits
    .about = When on, an agent's comment must say which commits it delivers (commits: ["<sha>"]) or that it delivers none (commits: null or "none"), or it is refused with an explanation. A client from before the field is warned instead of refused until it reconnects. Humans, close and reopen are exempt.
aiball-config-tickets-steps-stale = Stalled step after
    .about = A step (then: continue) with nothing after it for this long is flagged in the inbox: the work it announced went quiet. 0 = never flag.
aiball-config-tickets-steps-hot = Minutes a step keeps its ticket at the top of its author's backlog
    .about = After an agent posts a step (then: continue), its ticket leads that agent's backlog — right after the events, ahead of every other ticket — for this long, counted from the step. Past it, the ticket ranks like any other. It only changes the order; the visible 'hot' mark keeps its own rule. 0 = a step gets no priority.
aiball-config-tickets-steps-max-wait = Longest wait a step may declare
    .about = The most an agent may put in resume_on.timer on a step (then: continue). A longer wait is refused with this limit in the reason — past it the work is not one step waiting on a job any more: hand the ticket back, or propose a plan.
aiball-config-tickets-wait-credit-enabled = Wait credit
    .about = true (default) = a step's resume_on.timer spends an agent's wait credit, earned by proof of work. false = waits are free and uncapped (still at most tickets.step_after_max_minutes), nothing is earned, spent or refunded, and replies and wakes say nothing about credit.
aiball-config-tickets-wait-credit-refund = Wait credit: refund an early return
    .about = true (default) = an agent speaking on a ticket again before its step's wait ends gets the rest of that wait back. false = a wait is spent in full once declared.
aiball-config-tickets-wait-credit-earn-commit-max-age = Wait credit: oldest commit that still earns
    .about = A cited commit whose commit date is older than this earns nothing: the credit rewards fresh work.
aiball-config-tickets-wait-credit-earn-commits-per-comment = Wait credit: commits counted per comment
    .about = The most commits one comment can cite for credit; the ones past it earn nothing and the answer says so.
aiball-config-tickets-wait-credit-start = Wait credit an agent starts with
    .about = Every agent starts each project with this much wait credit, so a new agent can wait on a first build. The credit is spent by step timers (resume_on.timer) and earned by proof of work.
aiball-config-tickets-wait-credit-max = Most wait credit an agent holds
    .about = A balance never goes over this: what would take it over is not credited, and a balance already over it is cut back. 0 = no cap.
aiball-config-tickets-wait-credit-floor = Wait a step always gets, even without credit
    .about = Short of credit, a step's wait is capped to the balance but never below this, so an agent with no credit does not come back in a loop. It never takes the balance below zero. A step asking 0 (carry on at once) is always granted.
aiball-config-tickets-wait-credit-earn-resolved = Wait credit earned by a ticket closed resolved, with a commit
    .about = Earned once per ticket by the agent whose resolution was accepted, when it cited a commit on that ticket (commits: [...]) before it closed.
aiball-config-tickets-wait-credit-earn-resolved-no-commit = Wait credit earned by a ticket closed resolved, without a commit
    .about = Earned once per ticket by the agent whose resolution was accepted when it cited no commit on that ticket: a resolution without code is worth less.
aiball-config-tickets-wait-credit-earn-wontfix = Wait credit earned by a ticket closed wontfix
    .about = Earned once per ticket by the agent whose wontfix was accepted.
aiball-config-tickets-wait-credit-earn-lines-per-minute = Changed lines per minute of wait credit from a commit
    .about = A commit an agent cites on a reply (commits: [...]) earns one minute per this many changed lines, read in the agent's checkout. Once per commit.
aiball-config-tickets-wait-credit-earn-commit-max = Most wait credit one commit earns
    .about = The cap on what a single commit earns, however large its diff.
aiball-config-tickets-wait-credit-earn-commit-min = Least wait credit one commit earns
    .about = What a cited commit with at least one changed line earns, however small its diff: a short fix is work too. 0 = only the per-line rate counts.
aiball-config-tickets-backlog-claim-protect = Minutes a working agent's claim is protected
    .about = How long a claim holds against another agent's claim, counted from its holder's last action on the ticket — working on it keeps the protection alive. Another agent's claim inside that window is refused; past it the ticket can be taken over, and the thread records it. An assignment always wins over a claim. 0 = no protection.
aiball-config-tickets-backlog-rest = How long a ticket rests after a backlog wake
    .about = After a backlog wake named a ticket, how long it stays out of the agent's next backlog wakes while nobody else moves on it. A blocked ticket rests `blocked_multiplier` times longer, a ticket whose last action is a step only `after_step`. A loop started with CL_BACKLOG_COOLDOWN_SEC applies that instead. 0 = no rest.
aiball-config-tickets-backlog-depth = How deep the backlog wakes an agent
    .about = The deepest backlog tier a wake may name an agent for, when it has no event to read. Critical, hot and actionable tickets always wake it. followup: also a ticket where someone answered but the agent's own pending decision holds it. waiting: also a ticket where the agent spoke last and nothing moved since. blocked: also a ticket held by an open dependency (it comes back anyway when its blocker closes). A ticket under the setting stays in the backlog, shown; it just wakes no one.
aiball-config-tickets-backlog-blocked-multiplier = Backlog cooldown multiplier for blocked tickets
    .about = How much longer a backlog wake keeps a BLOCKED ticket (gated by an open depends_on) out of the wake pool, compared with any other ticket. It must keep surfacing so it is not forgotten, but nothing moves on it between two wakes. 1 = same cooldown as the rest.
aiball-config-tickets-backlog-after-step = Backlog cooldown after a step
    .about = How long a backlog wake keeps a ticket out of the wake pool when its last action is a step (then: continue), instead of the whole cooldown. Short on purpose: the step says there is work to do now, and the pause only lets the queue turn over. 0 = never sink it.
aiball-config-claude-loop-on-repetitive-denied-threshold = Denials before the loop answers
    .about = How many tool calls Claude Code's permission system must deny an agent in the last hour before its loop sends the prompt below (or the command's output). Nothing is sent while both are empty.
aiball-config-claude-loop-on-repetitive-denied-max-per-hour = Prompts sent per hour, at most
    .about = At most this many prompts an hour for repeated denials, so a denial, prompt, denial cycle stops. 0 = never send.
aiball-config-claude-loop-on-repetitive-denied-prompt = Prompt sent on repeated denials
    .about = Typed into Claude's prompt as it is, like a wake and only when a wake could be (no AFK hold, zen, typing, usage limit, boot or busy). Empty = nothing (the default).
aiball-config-claude-loop-on-repetitive-denied-command = Command that writes the prompt
    .about = A shell command run on the loop's machine, in its folder, for 10 s at most. It gets the denials as JSON on stdin (agent, project, cwd, tool, reason, last_hour, recent); what it prints is the prompt. It wins over the prompt above. Empty = none.
aiball-config-autopoll-volatile = Autopoll: one-shot reminders
    .about = true = notify only when a strictly newer ping arrives (no time-based reminders). false (default) = persistent reminder re-fires after throttle_seconds.
aiball-config-autopoll-throttle = Autopoll: reminder cadence
    .about = Reminder cadence in seconds, ignored when volatile=true. 0 = every Stop (spammy). New pings / new open tickets bypass the throttle.
aiball-config-autopoll-recent-tickets = Autopoll: recent ticket titles to include
    .about = Up to N recent unread ticket titles in the hook's reason so the agent knows what's waiting before draining. 0 = count only.
aiball-config-autopoll-backlog = Autopoll: backlog as a trigger
    .about = true (default) = open tickets in scope trigger notifications even without unread pings. false = context-only (display in reason, never the trigger).
aiball-config-autopoll-tone = Autopoll: tone
    .about = hint = polite, easy to ignore. directive (default) = names the action explicitly. imperative = last resort if the agent persists in asking permission patterns.
aiball-config-assign-window-sec = Assignment window
    .about = How long an assignment or a claim stays live before the ticket returns to the shared pool.
aiball-config-hot-window-sec = Hot window
    .about = How recent a move keeps a ticket hot in the inbox.
aiball-config-upstream-transport = Upstream: transport
    .about = How upstream (GitHub / GitLab) calls go out: gh (the CLI's credential), http, or auto (gh when it works). A coupling's own transport wins.
aiball-config-upstream-sync = Upstream: watch
    .about = pull (default) = a coupled repo is watched and its changes announced; off = linked only. A coupling's own sync wins.
