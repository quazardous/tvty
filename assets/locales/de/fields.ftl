# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und ids wie
# Englisch (assets/locales/en/), die Referenz.
# Was ein Ticket ist, als Felder: geteilt von der Ticket-Leiste und dem neuen
# Ticket (src/ui/fields.rs, src/composer.rs, src/ui/ticket_text.rs).

## aiballs Werte, ausgesprochen (aiball bekommt weiter seine eigenen)

fields-intent-request = Anfrage
fields-intent-question = Frage
fields-intent-fyi = Info
fields-intent-feature = Feature
fields-intent-panic = Panik
fields-priority-urgent = dringend
fields-priority-high = hoch
fields-priority-normal = normal
fields-priority-low = niedrig
fields-level-task = Aufgabe
fields-level-milestone = Meilenstein
fields-level-roadmap = Roadmap
fields-scope-internal = intern
fields-scope-default = Standard
fields-scope-broadcast = Rundruf

## Unter dem Geltungsbereich

fields-scope-note-internal = benachrichtigt nur, wer erwähnt wird
fields-scope-note-broadcast = benachrichtigt auch die Follower des Projekts
fields-scope-note-default = benachrichtigt die Abonnenten des Tickets und die Eigentümer des Projekts

## Schreiben

fields-take-out = entfernen
fields-write = Schreiben
fields-preview = Vorschau
fields-title = Titel
fields-body = Worum es geht… (@ zum Erwähnen, ctrl+v fügt ein Bild ein)

## Woher eine Einstellung kommt

fields-from-file = aus .aiball.yaml
fields-from-global = aus aiballs globaler Konfiguration
fields-from-mcp = aus .mcp.json
fields-from-env = aus aiballs Umgebung
fields-from-default = Standard
