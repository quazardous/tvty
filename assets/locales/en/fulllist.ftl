# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# The ticket list, full screen, and its bulk actions (src/fulllist.rs,
# src/bulk.rs). A bulk action's words go by its key: its label, .about,
# and what it did (.done).

fulllist-title = Tickets — {$project}
fulllist-title-all = Tickets — all projects
fulllist-shown = {$count} shown
fulllist-new = + New ticket
fulllist-search = Search titles…
fulllist-projects = Projects
fulllist-all-projects = All projects
fulllist-bands = Bands
fulllist-sort = Sort
fulllist-sort-activity = Last activity
fulllist-sort-turn = Whose turn (bands)
fulllist-sort-priority = Priority
fulllist-sort-created = Created
fulllist-sort-number = Number
fulllist-filters = Filters
fulllist-open = Open
fulllist-all = All
fulllist-unread = Unread
fulllist-reading-small = reading…
fulllist-critical = Critical — what holds the most back
fulllist-all-group = All
fulllist-reading = Reading…
fulllist-none = No ticket here.

## A row

fulllist-priority = priority: {$priority}
fulllist-rejected = rejected
fulllist-milestone = milestone {$title}
fulllist-assigned-to = assigned to {$who}
fulllist-claimed-by = claimed by {$who}
fulllist-spoke-last = {$who} spoke last
fulllist-you-spoke-last = you spoke last
fulllist-blocked = blocked
fulllist-payload = payload
fulllist-by = by {$who} · {$date}
fulllist-hot = an agent was active on it lately

## Bulk

fulllist-selected = {$count} selected
fulllist-select-all = Select all shown
fulllist-clear = Clear · Esc
fulllist-actions = Actions
fulllist-actions-about = Each acts on the chosen tickets it fits: how many, on the right.
fulllist-working = Working…
fulllist-count-of = {$count} of {$of}
fulllist-confirm = { $count ->
    [one] {$action} {$count} ticket?
   *[other] {$action} {$count} tickets?
}
fulllist-cancel = Cancel
bulk-refused = , {$count} refused ({$first})
bulk-approve = Approve
    .about = Lets through the chosen tickets that wait for moderation
    .done = {$count} approved
bulk-reject = Reject
    .about = Turns down the chosen tickets that wait for moderation
    .done = {$count} rejected
bulk-close = Close
    .about = Closes the chosen tickets that are open (asks first)
    .done = {$count} closed
bulk-reopen = Reopen
    .about = Opens again the chosen tickets that are closed
    .done = {$count} reopened
bulk-mark-read = Mark read
    .about = Marks read the chosen tickets that have something new
    .done = {$count} marked read
bulk-mark-unread = Mark unread
    .about = Marks unread the chosen tickets already read
    .done = {$count} marked unread
bulk-snooze = Snooze 3 days
    .about = Puts the chosen open tickets away for 3 days: they come back then
    .done = {$count} snoozed
bulk-unsnooze = Unsnooze
    .about = Brings back now the chosen tickets that are snoozed
    .done = {$count} unsnoozed
bulk-step = Mark as step
    .about = Marks the last word of the chosen open tickets as a step: the agent carries on, nothing to decide
    .done = {$count} marked as step
bulk-link = Link
    .about = Links the chosen tickets: the newest relates to each of the others
    .done = {$count} linked
