# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und dieselben ids
# wie im Englischen (assets/locales/en/), das als Referenz gilt.
# Die Benachrichtigungen und die kleinen Dialoge.

## Was sich gemeldet hat

messages-proposes-plan = schlägt einen Plan vor
messages-proposes-close = schlägt vor zu schließen
messages-proposes-wontfix = schlägt vor, ohne Korrektur zu schließen
messages-escalates = eskaliert
messages-new-ticket = ein neues Ticket
messages-new-comment = ein neuer Kommentar
messages-new-ticket-moderate = ein neues Ticket zum Moderieren
messages-missed = { $more ->
    [0] {$what} · während tvty geschlossen war
    [one] {$what} · und {$more} weiteres, während tvty geschlossen war
   *[other] {$what} · und {$more} weitere, während tvty geschlossen war
}
messages-dismiss = schließen

## Updates

messages-update-out = Terminal Velocity {$latest} ist erschienen (du nutzt {$running}): {$how}
messages-update-how = Updates… im Menü installiert es
messages-update-dev = dies ist ein Entwicklungs-Build: Aktualisiere seinen Checkout (Updates… im Menü sagt, wie)
messages-aiball-old = aiball {$version} ist älter, als Terminal Velocity braucht ({$needs}): Updates… im Menü aktualisiert es

## Eine beendete Sitzung

messages-detached = — von einem anderen Client getrennt
messages-ended = — die Sitzung ist beendet
messages-attach-again = Wieder verbinden
messages-starting = Wird gestartet…
messages-restart = Neu starten
messages-close = Schließen
messages-copies-failed = die anderen Terminals zu Kopien machen

## Beenden und neu starten

messages-stopping = Die Claude-Code-Sitzungen werden gestoppt, dann wird beendet…
messages-quit-stop-failed = beim Beenden von tvty stoppen
messages-ran-on = {$agents} lief weiter: der Stopp hat nicht gegriffen
messages-restarted-as-were = { $count ->
    [one] {$count} Sitzung neu gestartet, wie sie war, ihre Unterhaltung wird fortgesetzt
   *[other] {$count} Sitzungen neu gestartet, wie sie waren, ihre Unterhaltung wird fortgesetzt
}
messages-restarted-fresh = { $count ->
    [one] {$count} Sitzung frisch neu gestartet, ihre Unterhaltung wird fortgesetzt
   *[other] {$count} Sitzungen frisch neu gestartet, ihre Unterhaltung wird fortgesetzt
}
messages-restart-failed = Neustart der gestoppten Sitzungen
messages-restart-tvty-failed = tvty neu starten

## Welche Unterhaltung fortsetzen

messages-its-agent = sein Agent
messages-ago = vor {$time}
messages-some-time-ago = vor einiger Zeit
messages-last-one = Die letzte, {$when}:
messages-says-nothing = (sie sagt noch nichts)
messages-quoted = „{$said}“
messages-older = { $count ->
    [one] Dazu {$count} ältere: /resume in Claude Code wählt zwischen ihnen.
   *[other] Dazu {$count} ältere: /resume in Claude Code wählt zwischen ihnen.
}
messages-new-conversation = Neue Unterhaltung
messages-resume-last = Letzte Unterhaltung fortsetzen
messages-start = {$who} starten
messages-resume-question = Claude Code hat schon Unterhaltungen in {$cwd}, keine davon aus einem Loop: Welche setzt {$who} fort?

## Die Tipp-Karte

messages-tip-title = Schon gewusst?
messages-tips-browsing = Tipps · {$at} / {$of}
messages-not-now = Nicht jetzt
messages-previous = ‹ Zurück
messages-next = Weiter ›
messages-show-all-again = Alle wieder zeigen
messages-got-it = Verstanden
messages-next-tip = Nächster Tipp
messages-turn-off = Tipps ausschalten
messages-tips-off = Tipps sind aus: Einstellungen > Layout > Tipps schaltet sie wieder ein
messages-tips-again = Jeder Tipp wird wieder gezeigt, einer nach dem anderen
