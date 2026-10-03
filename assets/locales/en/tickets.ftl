# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# The ticket panel and what it says of a ticket (src/panel.rs,
# src/thread.rs, src/rowstate.rs).

## The panel

tickets-title = Tickets
tickets-no-project = No aiball project on this terminal.
tickets-none-open = No open ticket.
tickets-loading = Loading…
tickets-back = ← Tickets
tickets-fold-panel = Fold the ticket panel
tickets-pinned = Pinned: the panel stays beside the terminal. Not pinned: it lies over it
tickets-no-session = no session open
tickets-megaphone = The project's standing instruction and wake focus
tickets-new = + New
tickets-new-tip = A new ticket
tickets-full-list = The ticket list, full screen
tickets-sunk = sunk in {$agent}'s backlog until {$until} (in {$left}): its loop won't bring it up before, unless the thread moves

## The bands of the list

tickets-band-moderate = To moderate
tickets-band-decide = Waiting on you
tickets-band-working = Agents on it
tickets-band-open = Open
tickets-band-closed = Closed

## A row's marks

tickets-pending-comments = { $count ->
    [one] {$count} comment waiting for moderation
   *[other] {$count} comments waiting for moderation
}
tickets-unread = unread: something new on it for you
tickets-you-spoke-last = you spoke last
tickets-someone-spoke-last = someone else spoke last
tickets-comments-tip = { $count ->
    [one] {$count} comment — {$why}
   *[other] {$count} comments — {$why}
}
tickets-held-by = held by {$holder}
tickets-held-hot = held by {$holder}, active on it lately
tickets-critical = { $holds ->
    [one] the project's critical ticket: it holds {$holds} open ticket
   *[other] the project's critical ticket: it holds {$holds} open tickets
}
tickets-critical-quiet = , quiet for {$quiet}
tickets-step-resumes = step — the agent resumes at {$at}
tickets-critical-said = holds {$holds}
tickets-critical-said-quiet = holds {$holds} · quiet {$quiet}
tickets-stage-rejected = rejected
tickets-stage-closed-resolved = closed, resolved
tickets-stage-closed = closed
tickets-stage-resolved = resolved
tickets-stage-blocked = blocked
tickets-stage-snoozed = snoozed
tickets-stage-pending = pending
tickets-stage-open = open
tickets-to-moderate = to moderate

## What a glyph means

tickets-glyph-escalation = an agent escalates: it needs you to act
tickets-glyph-plan = a plan is proposed
tickets-glyph-resolution = a resolution is proposed
tickets-glyph-wontfix = closing without a fix is proposed
tickets-glyph-stalled = an agent's step went quiet
tickets-glyph-step = an agent is on a step (then: continue)
tickets-glyph-rejected = the last plan or resolution was rejected
tickets-glyph-closed-resolved = closed, resolved
tickets-glyph-closed = closed without a resolution

## Decisions

tickets-kind-plan = plan
tickets-kind-resolution = resolution
tickets-kind-wontfix = closing without a fix
tickets-kind-escalation = escalation
tickets-proposes-plan = {$who} proposes a plan
tickets-proposes-resolution = {$who} proposes a resolution
tickets-proposes-wontfix = {$who} proposes closing without a fix
tickets-proposes-escalation = {$who} proposes an escalation
# $kind (plan, resolution, wontfix, escalation): for the languages whose
# words agree with the noun.
tickets-decision-pending = {$noun} · pending
tickets-decision-accepted = {$noun} accepted
tickets-decision-rejected = {$noun} rejected
tickets-decision-superseded = {$noun} · superseded
tickets-accept-close = Accept → close
tickets-accept-wontfix = Accept → close, no fix
tickets-accept-escalation = Done → accept
tickets-accept-plan = Accept → go
tickets-reject = Reject
tickets-approve = Approve
tickets-decided-once-approved = decided once the ticket is approved
tickets-reject-say-why = To reject, say why below first.
tickets-waits-moderation = This ticket waits for moderation

## Where a ticket stands (the sentence under its title)

tickets-you = you
tickets-an-agent = an agent
tickets-closed-resolved-by = Closed, resolved by {$who}
tickets-closed-resolved = Closed, resolved
tickets-closed-unresolved = Closed without a resolution
tickets-yours-moderation = Yours: this ticket waits for moderation
tickets-your-proposal-plan = Your plan waits for a decision
tickets-your-proposal-resolution = Your resolution waits for a decision
tickets-your-proposal-wontfix = Your closing without a fix waits for a decision
tickets-your-proposal-escalation = Your escalation waits for a decision
tickets-yours-escalates = Yours: {$who} escalates — act, then accept
tickets-yours-decide-plan = Yours: accept or reject {$who}'s plan
tickets-yours-decide-resolution = Yours: accept or reject {$who}'s resolution
tickets-yours-decide-wontfix = Yours: accept or reject {$who}'s closing without a fix
tickets-step-quiet = {$agent}'s step went quiet
tickets-on-step = {$agent} is on a step
tickets-step-resumes-in = {" · "}resumes in {$span}
tickets-step-waits-on = {" · "}waits on #{$ticket}
tickets-yours-answer = Yours: answer {$who}
tickets-holds-you-spoke = {$holder} holds it: you spoke last
tickets-yours-nobody = Yours: nobody else is on it
tickets-their-turn-of = {$holder}'s turn: you spoke last
tickets-their-turn = Their turn: you spoke last
tickets-span-under-minute = under a minute
tickets-span-minutes = {$n} min
tickets-span-hours = {$n} h

## The thread

tickets-images = { $count ->
    [one] 🖼 image
   *[other] 🖼 {$count} images
}

tickets-where-it-stands = Where it stands · {$who}
tickets-edit = ✎ edit
tickets-edit-tip = Edit the title and the body
tickets-newest-first = ⇅ newest first
tickets-newest-last = ⇅ newest last
tickets-close-full = ✕  Esc
tickets-more = ⤢ more
tickets-comments-spoke = { $count ->
    [one] {$count} comment · {$who} spoke last
   *[other] {$count} comments · {$who} spoke last
}
tickets-unfold-all = unfold all
tickets-fold-before-summary = fold before the summary
tickets-fold = fold
tickets-unfold = unfold
tickets-snoozed-until = snoozed until {$until}
tickets-tokens = {$count} tok
tickets-priority-tip = the priority: a click changes it
tickets-priority-label = Priority
tickets-rel-depends = depends on
tickets-rel-blocks = blocks
tickets-rel-relates = relates to
tickets-rel-duplicates = duplicates
tickets-rel-duplicated = duplicated by
tickets-rel-parent = parent of
tickets-rel-child = child of
tickets-answer = Answer
tickets-in-reply = ✓ in the reply
tickets-step = step
tickets-resumes = resumes
tickets-resumes-on = on
tickets-resumes-or-on = or on
tickets-resumes-at = the agent resumes at {$at}
tickets-resumes-when = the agent resumes when #{$ticket} moves (a reply, a decision, a close)
tickets-or-when = or when #{$ticket} moves (a reply, a decision, a close)

## A comment's actions

tickets-comment-actions = the comment's actions
tickets-edit-comment = Edit
tickets-delete = Delete
tickets-really-delete = Really delete?
tickets-classify-as = as {$noun}
tickets-no-decision = no decision
tickets-not-a-step = not a step
tickets-a-step = a step
tickets-resurface = Resurface
tickets-cancel = Cancel
tickets-save = Save

## Writing

tickets-reply-placeholder = Reply… (ctrl+enter sends)
tickets-editing = Editing #{$id}
tickets-edit-keys = ctrl+enter saves · esc puts it back
tickets-title-first = A title first.
tickets-nothing-to-preview = Nothing to preview yet.
tickets-sending-answers = { $count ->
    [one] Sending answers {$count} question.
   *[other] Sending answers {$count} questions.
}
tickets-close = Close
tickets-reopen = Reopen
tickets-wake = Wake
tickets-snooze = Snooze ▾
tickets-snooze-for = Snooze for
tickets-snooze-hour = 1 hour
tickets-snooze-day = a day
tickets-snooze-week = a week
tickets-assign = Assign ▾
tickets-assigned-menu = → {$who} ▾
tickets-assign-to = Assign to
tickets-no-agent = the project has no agent
tickets-claim-assigns = {$agent} holds it by a claim: a click assigns it to them
tickets-unassign-tip = unassign: takes it back from {$holder}; nobody holds it, it waits for whoever takes it
tickets-without-notifying = without notifying
tickets-reply = Reply

## The column of fields

tickets-group-state = State
tickets-group-fields = Fields
tickets-group-people = People
tickets-group-links = Links
tickets-group-project = Project
tickets-group-tokens = Tokens
tickets-group-payload = Payload
tickets-lifecycle = lifecycle
tickets-lifecycle-closed-resolved = closed, resolved
tickets-lifecycle-closed = closed
tickets-lifecycle-moderation = waits for moderation
tickets-lifecycle-open = open
tickets-snoozed = snoozed
tickets-until = until {$date}
tickets-field-intent = intent
tickets-field-priority = priority
tickets-field-level = level
tickets-field-scope = scope
tickets-field-tags = tags
tickets-field-milestone = milestone
tickets-field-reporter = reporter
tickets-field-claimed = claimed by
tickets-field-assigned = assigned to
tickets-field-followers = followers
tickets-field-project = project
tickets-filed = filed {$date}
tickets-lapsed = {$who} (lapsed)
tickets-claim-until = {$who}, until {$date}
tickets-muted = muted: not notified of this ticket, even by its role — ✕ puts it back to what its role says
tickets-followers-tip = Who follows this ticket by their own choice (or muted it). The project's owners are notified by their role: they are not listed here.
tickets-sub-ticket = sub-ticket
tickets-new-sub = + a new one
tickets-sub-ticket-of = sub-ticket of
tickets-sub-tickets = sub-tickets
tickets-remove-relation = remove this relation
tickets-relate = relate
tickets-relate-to = to…
tickets-relate-add = + a ticket
tickets-move-to = Move to {$project}
tickets-in-out = in · out
tickets-cache = cache
tickets-cache-said = {$written} written · {$read} read
tickets-payload = this ticket carries a payload (see the web UI)
tickets-pick-project = a project
tickets-pick-project-search = a project…
tickets-pick-tag = add a tag
tickets-pick-tag-search = a tag…
tickets-pick-milestone = none
tickets-pick-milestone-search = a milestone…
tickets-pick-reporter = the reporter
tickets-pick-agent-or-you = an agent, or you…
tickets-pick-follower = add one
tickets-pick-intent-search = an intent…
tickets-pick-priority-search = a priority…
tickets-pick-level-search = a level…
tickets-pick-scope-search = a scope…
tickets-pick-nobody = nobody
tickets-pick-agent-search = an agent…

## What a gesture did (said in a notification)

tickets-did-reply = reply posted
tickets-did-with-reply = {$what}, with a reply
tickets-did-on = {$title} — {$what}
tickets-did-comment-edited = comment edited
tickets-did-tagged = tagged {$name}
tickets-did-tag-removed = tag {$name} removed
tickets-did-milestone-set = milestone set
tickets-did-milestone-removed = milestone removed
tickets-did-reporter = reporter now {$name}
tickets-did-assigned = assigned to {$name}
tickets-did-unassigned = unassigned
tickets-did-follows = {$name} now follows it
tickets-did-unfollows = {$name} no longer follows it
tickets-did-intent = intent {$value}
tickets-did-priority = priority {$value}
tickets-did-level = level {$value}
tickets-did-scope = scope {$value}
tickets-did-edited = title and body edited
tickets-did-related = related to #{$target} ({$kind})
tickets-did-unrelated = relation to #{$target} removed
tickets-did-moved = moved to {$project}
tickets-did-snoozed = snoozed
tickets-did-woken = woken
tickets-did-accepted = decision accepted
tickets-did-rejected = decision rejected
tickets-did-approved = approved
tickets-did-rejected-moderation = rejected in moderation
tickets-did-closed = closed
tickets-did-reopened = reopened
tickets-did-deleted = comment deleted
tickets-did-classified = comment made a {$noun}
tickets-did-plain = comment made plain
tickets-did-step-marked = step marked
tickets-did-step-unmarked = step unmarked
tickets-did-voted = voted
tickets-did-resurfaced = comment resurfaced

## The thread's events

tickets-event-closed = closed the ticket
tickets-event-reopened = reopened the ticket
tickets-event-resolved = marked it resolved
tickets-event-blocked = flagged it to be decided
tickets-event-taken-over = took over the claim
tickets-event-sub-added = added a sub-ticket{$source}
tickets-event-referenced = referenced it from{$source}
tickets-event-dependency-closed = a dependency closed{$source}
tickets-event-dependency-rejected = a dependency was rejected{$source}
tickets-event-related-closed = a related ticket closed{$source}
tickets-event-linked = linked{$source}
