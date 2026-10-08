# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und IDs wie
# Englisch (assets/locales/en/), die Referenz.
# Der Rest: das Menü und die Markierungen des Terminals, der Bildbetrachter,
# einige Hinweise und Fehler.

## Hinweise

misc-aiball-started = aiball lief nicht: gestartet
misc-aiball-silent = aiball antwortet nicht unter {$at}
misc-service-failed = aiballs Dienst ist nicht gestartet: {$error}
misc-no-service = {$why}; es gibt keinen aiball-Dienst zum Starten
misc-filed = angelegt — {$title}
misc-filed-moderate = {$title} — ein neues Ticket zum Moderieren
misc-filed-other = {$title} — ein neues Ticket
misc-something-new = etwas Neues
misc-held = {$said}: sein Loop ist angehalten, bis du ihn freigibst
misc-go-to = gehe zu
misc-go-to-what = gehe zu {$what}
misc-not-a-ticket = {$typed} ist kein Ticket: eine Nummer, oder der #C.-Link eines Kommentars
misc-copied = In die Zwischenablage kopiert
misc-new-terminal = neues Terminal
misc-no-folder = kein Ordner bekannt für {$project}
misc-not-a-directory = {$folder} ist kein Ordner
misc-not-idle = sein Claude arbeitet: sobald er im Leerlauf ist
misc-stop-not-received = {$agent}: keiner seiner Loops hat das Stoppen erhalten (er läuft nicht, oder nicht dort, wo aiball ihn erreicht)

## Das Terminal

misc-open-link = Link öffnen
misc-copy-link = Link kopieren
misc-copy = Kopieren
misc-paste = Einfügen
misc-frozen = angehalten · Auswahl
misc-frozen-tip = Der Bildschirm bleibt stehen, solange Text ausgewählt ist; die Sitzung läuft weiter. Ein Klick hier, Esc oder eine Taste gibt ihn frei.
misc-size-taken = Größe von einem anderen Client übernommen · klicken, um sie zurückzuholen
misc-session-ended = Die Sitzung ist beendet.
misc-hub-session = Diese Sitzung läuft auf aiballs Hub, einem anderen Rechner: sie lässt sich von hier nicht öffnen. Ihre Tickets sind im Panel.
misc-all-terminals = Alle Terminals
misc-gallery-hint = tippen filtert · Pfeile bewegen · Enter öffnet · Esc schließt

## Das Modell des Agenten, seine Limits, seine Ablehnungen

misc-price = {$input} / {$output} pro M Tokens
misc-model-out = {$name} ist erschienen
misc-prices-from = Preise von {$catalog}
misc-limit = Nutzungslimit erreicht
misc-limit-resets = Nutzungslimit erreicht · setzt zurück {$resets}
misc-denials = { $count ->
    [one] {$count} Tool-Aufruf in der letzten Stunde von Claude Code abgelehnt{$ago}{$why} — ein abgelehnter Agent bleibt dort stehen
   *[other] {$count} Tool-Aufrufe in der letzten Stunde von Claude Code abgelehnt{$ago}{$why} — ein abgelehnter Agent bleibt dort stehen
}
misc-denials-last = , der letzte vor {$ago}

## Bilder

misc-image-too-large = Bild zu groß, um es hier zu zeigen
misc-image-unavailable = Bild nicht verfügbar
misc-fitted = {" "}(eingepasst)
misc-viewer-hint = Mausrad oder + − Zoom · 1 Originalgröße · 0 einpassen · ziehen zum Verschieben · Esc

## The usage arrow in the top bar

usage-ratio = ×{$ratio}
usage-points = {$points} Pkt.
usage-wall = Grenze {$left}
usage-no-wall = keine Grenze
usage-window-five_hour = 5-Stunden-Fenster
usage-window-seven_day = Woche
usage-tip-window = {$window}: {$used} % verbraucht, {$expected} % bei gleichmäßigem Tempo · zurückgesetzt {$resets} · {$end}
usage-tip-wall = beim Tempo der letzten Stunde ist das Kontingent in {$left} aufgebraucht
usage-tip-lasts = beim Tempo der letzten Stunde reicht es bis zum Zurücksetzen
usage-trend = Tempo ×{$speed}
usage-tip-trend = jüngstes Tempo: ×{$recent} das gleichmäßige über 15 min, ×{$hour} über die Stunde
usage-tip-read = vor {$ago} gelesen
usage-tip-click = Ein Klick: der Abstand anders gesagt.
usage-drawing-wall = Wand
usage-drawing-points = Abstand
usage-drawing-trend = Tempo
