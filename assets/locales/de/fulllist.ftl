# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und dieselben ids
# wie im Englischen (assets/locales/en/), das als Referenz gilt.
# Die Ticketliste im Vollbild, und ihre Sammelaktionen.

fulllist-title = Tickets — {$project}
fulllist-title-all = Tickets — alle Projekte
fulllist-shown = {$count} angezeigt
fulllist-new = + Neues Ticket
fulllist-search = Titel durchsuchen…
fulllist-projects = Projekte
fulllist-all-projects = Alle Projekte
fulllist-bands = Bänder
fulllist-sort = Sortierung
fulllist-sort-activity = Letzte Aktivität
fulllist-sort-turn = Wer dran ist (Bänder)
fulllist-sort-priority = Priorität
fulllist-sort-created = Erstellt
fulllist-sort-number = Nummer
fulllist-filters = Filter
fulllist-open = Offen
fulllist-all = Alle
fulllist-unread = Ungelesen
fulllist-reading-small = wird gelesen…
fulllist-critical = Kritisch — was am meisten aufhält
fulllist-all-group = Alle
fulllist-reading = Wird gelesen…
fulllist-none = Hier kein Ticket.

## Eine Zeile

fulllist-priority = Priorität: {$priority}
fulllist-rejected = abgelehnt
fulllist-milestone = Meilenstein {$title}
fulllist-assigned-to = zugewiesen an {$who}
fulllist-claimed-by = übernommen von {$who}
fulllist-spoke-last = {$who} hat zuletzt gesprochen
fulllist-you-spoke-last = du hast zuletzt gesprochen
fulllist-blocked = blockiert
fulllist-payload = Payload
fulllist-by = von {$who} · {$date}
fulllist-hot = ein Agent war kürzlich darauf aktiv

## Sammelaktionen

fulllist-selected = {$count} ausgewählt
fulllist-select-all = Alle angezeigten auswählen
fulllist-clear = Leeren · Esc
fulllist-actions = Aktionen
fulllist-actions-about = Jede wirkt auf die ausgewählten Tickets, zu denen sie passt: wie viele, rechts.
fulllist-working = In Arbeit…
fulllist-count-of = {$count} von {$of}
fulllist-confirm = { $count ->
    [one] {$action}: {$count} Ticket?
   *[other] {$action}: {$count} Tickets?
}
fulllist-cancel = Abbrechen
bulk-refused = , {$count} abgewiesen ({$first})
bulk-approve = Freigeben
    .about = Lässt die ausgewählten Tickets durch, die auf Moderation warten
    .done = {$count} freigegeben
bulk-reject = Ablehnen
    .about = Weist die ausgewählten Tickets ab, die auf Moderation warten
    .done = {$count} abgelehnt
bulk-close = Schließen
    .about = Schließt die ausgewählten Tickets, die offen sind (fragt vorher)
    .done = {$count} geschlossen
bulk-reopen = Wieder öffnen
    .about = Öffnet die ausgewählten Tickets, die geschlossen sind, wieder
    .done = {$count} wieder geöffnet
bulk-mark-read = Als gelesen markieren
    .about = Markiert die ausgewählten Tickets mit Neuem als gelesen
    .done = {$count} als gelesen markiert
bulk-mark-unread = Als ungelesen markieren
    .about = Markiert die ausgewählten, schon gelesenen Tickets als ungelesen
    .done = {$count} als ungelesen markiert
bulk-snooze = 3 Tage zurückstellen
    .about = Legt die ausgewählten offenen Tickets für 3 Tage beiseite: dann kommen sie zurück
    .done = {$count} zurückgestellt
bulk-unsnooze = Zurückholen
    .about = Holt die ausgewählten zurückgestellten Tickets jetzt zurück
    .done = {$count} zurückgeholt
bulk-step = Als Schritt markieren
    .about = Markiert das letzte Wort der ausgewählten offenen Tickets als Schritt: Der Agent macht weiter, nichts zu entscheiden
    .done = {$count} als Schritt markiert
bulk-link = Verknüpfen
    .about = Verknüpft die ausgewählten Tickets: Das neueste wird mit jedem der anderen verknüpft
    .done = {$count} verknüpft
