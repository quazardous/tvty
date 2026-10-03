# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und ids wie
# Englisch (assets/locales/en/), die Referenz.
# Das Ticket-Panel und was es über ein Ticket sagt (src/panel.rs,
# src/thread.rs, src/rowstate.rs).

## Das Panel

tickets-title = Tickets
tickets-no-project = Kein aiball-Projekt auf diesem Terminal.
tickets-none-open = Kein offenes Ticket.
tickets-loading = Wird geladen…
tickets-back = ← Tickets
tickets-fold-panel = Das Ticket-Panel einklappen
tickets-pinned = Angeheftet: Das Panel bleibt neben dem Terminal. Nicht angeheftet: Es liegt darüber
tickets-no-session = keine Sitzung offen
tickets-megaphone = Die ständige Anweisung des Projekts und der Weck-Fokus
tickets-new = + Neu
tickets-new-tip = Ein neues Ticket
tickets-full-list = Die Ticketliste, im Vollbild
tickets-sunk = versenkt im Backlog von {$agent} bis {$until} (in {$left}): Sein Loop holt es nicht vorher hervor, außer der Thread bewegt sich

## Die Bänder der Liste

tickets-band-moderate = Zu moderieren
tickets-band-decide = Wartet auf dich
tickets-band-working = Agents dran
tickets-band-open = Offen
tickets-band-closed = Geschlossen

## Die Markierungen einer Zeile

tickets-pending-comments = { $count ->
    [one] {$count} Kommentar wartet auf Moderation
   *[other] {$count} Kommentare warten auf Moderation
}
tickets-unread = ungelesen: etwas Neues für dich
tickets-you-spoke-last = du hast zuletzt gesprochen
tickets-someone-spoke-last = jemand anderes hat zuletzt gesprochen
tickets-comments-tip = { $count ->
    [one] {$count} Kommentar — {$why}
   *[other] {$count} Kommentare — {$why}
}
tickets-held-by = gehalten von {$holder}
tickets-held-hot = gehalten von {$holder}, kürzlich daran aktiv
tickets-critical = { $holds ->
    [one] das kritische Ticket des Projekts: Es hält {$holds} offenes Ticket auf
   *[other] das kritische Ticket des Projekts: Es hält {$holds} offene Tickets auf
}
tickets-critical-quiet = , still seit {$quiet}
tickets-step-resumes = Schritt — der Agent macht um {$at} weiter
tickets-critical-said = hält {$holds} auf
tickets-critical-said-quiet = hält {$holds} auf · still {$quiet}
tickets-stage-rejected = abgelehnt
tickets-stage-closed-resolved = geschlossen, gelöst
tickets-stage-closed = geschlossen
tickets-stage-resolved = gelöst
tickets-stage-blocked = blockiert
tickets-stage-snoozed = schlummert
tickets-stage-pending = ausstehend
tickets-stage-open = offen
tickets-to-moderate = zu moderieren

## Was ein Glyph bedeutet

tickets-glyph-escalation = ein Agent eskaliert: Er braucht dein Handeln
tickets-glyph-plan = ein Plan wird vorgeschlagen
tickets-glyph-resolution = eine Lösung wird vorgeschlagen
tickets-glyph-wontfix = Schließen ohne Fix wird vorgeschlagen
tickets-glyph-stalled = der Schritt eines Agents ist verstummt
tickets-glyph-step = ein Agent ist an einem Schritt (then: continue)
tickets-glyph-rejected = der letzte Plan oder die letzte Lösung wurde abgelehnt
tickets-glyph-closed-resolved = geschlossen, gelöst
tickets-glyph-closed = ohne Lösung geschlossen

## Entscheidungen

tickets-kind-plan = Plan
tickets-kind-resolution = Lösung
tickets-kind-wontfix = Schließen ohne Fix
tickets-kind-escalation = Eskalation
tickets-proposes-plan = {$who} schlägt einen Plan vor
tickets-proposes-resolution = {$who} schlägt eine Lösung vor
tickets-proposes-wontfix = {$who} schlägt vor, ohne Fix zu schließen
tickets-proposes-escalation = {$who} eskaliert
# $kind (plan, resolution, wontfix, escalation): für die Sprachen, deren
# Wörter mit dem Nomen übereinstimmen.
tickets-decision-pending = {$noun} · ausstehend
tickets-decision-accepted = {$noun} angenommen
tickets-decision-rejected = {$noun} abgelehnt
tickets-decision-superseded = {$noun} · ersetzt
tickets-accept-close = Annehmen → schließen
tickets-accept-wontfix = Annehmen → schließen, ohne Fix
tickets-accept-escalation = Erledigt → annehmen
tickets-accept-plan = Annehmen → los
tickets-reject = Ablehnen
tickets-approve = Freigeben
tickets-decided-once-approved = entschieden, sobald das Ticket freigegeben ist
tickets-reject-say-why = Zum Ablehnen sag erst unten, warum.
tickets-waits-moderation = Dieses Ticket wartet auf Moderation

## Wo ein Ticket steht (der Satz unter seinem Titel)

tickets-you = du
tickets-an-agent = ein Agent
tickets-closed-resolved-by = Geschlossen, gelöst von {$who}
tickets-closed-resolved = Geschlossen, gelöst
tickets-closed-unresolved = Ohne Lösung geschlossen
tickets-yours-moderation = Du bist dran: Dieses Ticket wartet auf Moderation
tickets-your-proposal-plan = Dein Plan wartet auf eine Entscheidung
tickets-your-proposal-resolution = Deine Lösung wartet auf eine Entscheidung
tickets-your-proposal-wontfix = Dein Schließen ohne Fix wartet auf eine Entscheidung
tickets-your-proposal-escalation = Deine Eskalation wartet auf eine Entscheidung
tickets-yours-escalates = Du bist dran: {$who} eskaliert — handle, dann nimm an
tickets-yours-decide-plan = Du bist dran: Nimm den Plan von {$who} an oder lehne ihn ab
tickets-yours-decide-resolution = Du bist dran: Nimm die Lösung von {$who} an oder lehne sie ab
tickets-yours-decide-wontfix = Du bist dran: Nimm das Schließen ohne Fix von {$who} an oder lehne es ab
tickets-step-quiet = der Schritt von {$agent} ist verstummt
tickets-on-step = {$agent} ist an einem Schritt
tickets-step-resumes-in = {" · "}geht weiter in {$span}
tickets-step-waits-on = {" · "}wartet auf #{$ticket}
tickets-yours-answer = Du bist dran: Antworte {$who}
tickets-holds-you-spoke = {$holder} hält es: Du hast zuletzt gesprochen
tickets-yours-nobody = Du bist dran: Sonst ist niemand daran
tickets-their-turn-of = {$holder} ist dran: Du hast zuletzt gesprochen
tickets-their-turn = Die anderen sind dran: Du hast zuletzt gesprochen
tickets-span-under-minute = unter einer Minute
tickets-span-minutes = {$n} Min.
tickets-span-hours = {$n} Std.

## Der Thread

tickets-images = { $count ->
    [one] 🖼 Bild
   *[other] 🖼 {$count} Bilder
}

tickets-where-it-stands = Stand · {$who}
tickets-edit = ✎ bearbeiten
tickets-edit-tip = Titel und Text bearbeiten
tickets-newest-first = ⇅ neueste zuerst
tickets-newest-last = ⇅ neueste zuletzt
tickets-close-full = ✕  Esc
tickets-more = ⤢ mehr
tickets-comments-spoke = { $count ->
    [one] {$count} Kommentar · {$who} hat zuletzt gesprochen
   *[other] {$count} Kommentare · {$who} hat zuletzt gesprochen
}
tickets-unfold-all = alle aufklappen
tickets-fold-before-summary = vor der Zusammenfassung einklappen
tickets-fold = einklappen
tickets-unfold = aufklappen
tickets-snoozed-until = schlummert bis {$until}
tickets-tokens = {$count} tok
tickets-priority-tip = die Priorität: Ein Klick ändert sie
tickets-priority-label = Priorität
tickets-rel-depends = hängt ab von
tickets-rel-blocks = blockiert
tickets-rel-relates = verwandt mit
tickets-rel-duplicates = Duplikat von
tickets-rel-duplicated = dupliziert durch
tickets-rel-parent = Eltern von
tickets-rel-child = Kind von
tickets-answer = Antworten
tickets-in-reply = ✓ in der Antwort
tickets-step = Schritt
tickets-resumes = geht weiter
tickets-resumes-on = bei
tickets-resumes-or-on = oder bei
tickets-resumes-at = der Agent macht um {$at} weiter
tickets-resumes-when = der Agent macht weiter, wenn sich #{$ticket} bewegt (eine Antwort, eine Entscheidung, ein Schließen)
tickets-or-when = oder wenn sich #{$ticket} bewegt (eine Antwort, eine Entscheidung, ein Schließen)

## Die Aktionen eines Kommentars

tickets-comment-actions = die Aktionen des Kommentars
tickets-edit-comment = Bearbeiten
tickets-delete = Löschen
tickets-really-delete = Wirklich löschen?
tickets-classify-as = als {$noun}
tickets-no-decision = keine Entscheidung
tickets-not-a-step = kein Schritt
tickets-a-step = ein Schritt
tickets-resurface = Wieder hochholen
tickets-cancel = Abbrechen
tickets-save = Speichern

## Schreiben

tickets-reply-placeholder = Antworten… (Strg+Enter sendet)
tickets-editing = Bearbeitung von #{$id}
tickets-edit-keys = Strg+Enter speichert · Esc stellt es zurück
tickets-title-first = Erst ein Titel.
tickets-nothing-to-preview = Noch nichts in der Vorschau.
tickets-sending-answers = { $count ->
    [one] Das Senden beantwortet {$count} Frage.
   *[other] Das Senden beantwortet {$count} Fragen.
}
tickets-close = Schließen
tickets-reopen = Wieder öffnen
tickets-wake = Wecken
tickets-snooze = Schlummern ▾
tickets-snooze-for = Schlummern für
tickets-snooze-hour = 1 Stunde
tickets-snooze-day = einen Tag
tickets-snooze-week = eine Woche
tickets-assign = Zuweisen ▾
tickets-assigned-menu = → {$who} ▾
tickets-assign-to = Zuweisen an
tickets-no-agent = das Projekt hat keinen Agent
tickets-claim-assigns = {$agent} hält es per Claim: Ein Klick weist es ihm zu
tickets-unassign-tip = Zuweisung aufheben: nimmt es {$holder} weg; niemand hält es, es wartet auf den, der es nimmt
tickets-without-notifying = leise
tickets-reply = Antworten

## Die Spalte der Felder

tickets-group-state = Zustand
tickets-group-fields = Felder
tickets-group-people = Personen
tickets-group-links = Links
tickets-group-project = Projekt
tickets-group-tokens = Tokens
tickets-group-payload = Payload
tickets-lifecycle = Status
tickets-lifecycle-closed-resolved = geschlossen, gelöst
tickets-lifecycle-closed = geschlossen
tickets-lifecycle-moderation = wartet auf Moderation
tickets-lifecycle-open = offen
tickets-snoozed = schlummert
tickets-until = bis {$date}
tickets-field-intent = Absicht
tickets-field-priority = Priorität
tickets-field-level = Stufe
tickets-field-scope = Bereich
tickets-field-tags = Tags
tickets-field-milestone = Etappe
tickets-field-reporter = Melder
tickets-field-claimed = Claim von
tickets-field-assigned = zugewiesen
tickets-field-followers = Follower
tickets-field-project = Projekt
tickets-filed = erstellt am {$date}
tickets-lapsed = {$who} (abgelaufen)
tickets-claim-until = {$who}, bis {$date}
tickets-muted = stumm: nicht über dieses Ticket benachrichtigt, auch nicht durch die Rolle — ✕ stellt zurück, was die Rolle sagt
tickets-followers-tip = Wer diesem Ticket aus eigener Wahl folgt (oder es stummgeschaltet hat). Die Eigentümer des Projekts werden über ihre Rolle benachrichtigt: Sie stehen nicht hier.
tickets-sub-ticket = Unterticket
tickets-new-sub = + ein neues
tickets-sub-ticket-of = Unterticket von
tickets-sub-tickets = Untertickets
tickets-remove-relation = diese Verknüpfung entfernen
tickets-relate = verknüpfen
tickets-relate-to = mit…
tickets-relate-add = + ein Ticket
tickets-move-to = Verschieben nach {$project}
tickets-in-out = ein · aus
tickets-cache = Cache
tickets-cache-said = {$written} geschrieben · {$read} gelesen
tickets-payload = dieses Ticket trägt einen Payload (siehe Web-UI)
tickets-pick-project = ein Projekt
tickets-pick-project-search = ein Projekt…
tickets-pick-tag = Tag hinzufügen
tickets-pick-tag-search = ein Tag…
tickets-pick-milestone = keiner
tickets-pick-milestone-search = ein Meilenstein…
tickets-pick-reporter = der Melder
tickets-pick-agent-or-you = ein Agent, oder du…
tickets-pick-follower = hinzufügen
tickets-pick-intent-search = eine Absicht…
tickets-pick-priority-search = eine Priorität…
tickets-pick-level-search = eine Stufe…
tickets-pick-scope-search = ein Bereich…
tickets-pick-nobody = niemand
tickets-pick-agent-search = ein Agent…

## Was eine Geste bewirkt hat (in einer Benachrichtigung gesagt)

tickets-did-reply = Antwort gepostet
tickets-did-with-reply = {$what}, mit einer Antwort
tickets-did-on = {$title} — {$what}
tickets-did-comment-edited = Kommentar bearbeitet
tickets-did-tagged = Tag {$name} hinzugefügt
tickets-did-tag-removed = Tag {$name} entfernt
tickets-did-milestone-set = Meilenstein gesetzt
tickets-did-milestone-removed = Meilenstein entfernt
tickets-did-reporter = Melder jetzt {$name}
tickets-did-assigned = zugewiesen an {$name}
tickets-did-unassigned = Zuweisung aufgehoben
tickets-did-follows = {$name} folgt ihm jetzt
tickets-did-unfollows = {$name} folgt ihm nicht mehr
tickets-did-intent = Absicht {$value}
tickets-did-priority = Priorität {$value}
tickets-did-level = Stufe {$value}
tickets-did-scope = Bereich {$value}
tickets-did-edited = Titel und Text bearbeitet
tickets-did-related = verknüpft mit #{$target} ({$kind})
tickets-did-unrelated = Verknüpfung mit #{$target} entfernt
tickets-did-moved = verschoben nach {$project}
tickets-did-snoozed = schlummert
tickets-did-woken = geweckt
tickets-did-accepted = Entscheidung angenommen
tickets-did-rejected = Entscheidung abgelehnt
tickets-did-approved = freigegeben
tickets-did-rejected-moderation = in der Moderation abgelehnt
tickets-did-closed = geschlossen
tickets-did-reopened = wieder geöffnet
tickets-did-deleted = Kommentar gelöscht
tickets-did-classified = Kommentar als {$noun} eingestuft
tickets-did-plain = Kommentar ohne Entscheidung
tickets-did-step-marked = Schritt markiert
tickets-did-step-unmarked = Schritt entfernt
tickets-did-voted = abgestimmt
tickets-did-resurfaced = Kommentar wieder hochgeholt

## Die Ereignisse des Threads

tickets-event-closed = hat das Ticket geschlossen
tickets-event-reopened = hat das Ticket wieder geöffnet
tickets-event-resolved = hat es als gelöst markiert
tickets-event-blocked = hat es zur Entscheidung markiert
tickets-event-taken-over = hat den Claim übernommen
tickets-event-sub-added = hat ein Unterticket hinzugefügt{$source}
tickets-event-referenced = hat es referenziert aus{$source}
tickets-event-dependency-closed = eine Abhängigkeit wurde geschlossen{$source}
tickets-event-dependency-rejected = eine Abhängigkeit wurde abgelehnt{$source}
tickets-event-related-closed = ein verknüpftes Ticket wurde geschlossen{$source}
tickets-event-linked = verknüpft{$source}
