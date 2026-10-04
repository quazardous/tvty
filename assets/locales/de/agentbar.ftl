# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und ids wie
# Englisch (assets/locales/en/), die Referenz.
# Die Leiste des Agenten, unter seinem Terminal (src/shell/agentbar.rs).

## Wer den Loop steuert

agentbar-afk-auto = auto
agentbar-afk-hold-10m = 10 Min. halten
agentbar-afk-hold = halten
agentbar-afk-boot = … Boot
agentbar-afk-arming-tip = scharf: das Männchen zeigt den mit F9 gewählten Modus, gültig 3 s nach dem letzten Drücken — ▶ oder ‖ zeigt den bis dahin gültigen
agentbar-afk-tip = wer den Loop steuert: ▶ er selbst, ‖ für dich gehalten (oder während du tippst); das Männchen ist der AFK-Modus — grau: du bist weg, der Loop läuft allein; die Sekunden eines 10-Min.-Haltens; ∞ gehalten. F9 schaltet weiter (auto → 10 Min. → ∞), gültig 3 s nach dem letzten Drücken; ein Klick wählt

## Was sein Claude tut

agentbar-working = arbeitet
agentbar-starting = startet
agentbar-idle = Leerlauf
agentbar-offline = offline
# Nach der Zeit seit Beginn des Boots: " · noch 12 s".
agentbar-boot-left = {" · "}noch {$seconds} s
agentbar-boot-tip = sein Loop startet: Claude lädt, setzt vielleicht seine Unterhaltung fort oder kompaktiert; der Loop weckt ihn, sobald der Boot endet (mindestens 30 s, länger solange ein Fortsetzen oder Kompaktieren angezeigt wird)
agentbar-state-tip = was sein Claude tut, und seit wann
agentbar-dialog = wartet auf eine Antwort

## Sein Claude neu starten

agentbar-restarting = startet neu…
agentbar-restarting-tip = sein Claude startet neu und setzt seine Unterhaltung fort
agentbar-restart-pending = Neustart ausstehend
agentbar-restart-pending-tip = ein Neustart ist angefordert: sein Claude startet neu, sobald er untätig ist, und setzt sein Gespräch fort; ein Klick nimmt ihn zurück
agentbar-update = Update
agentbar-update-tip = sein Claude Code hat ein Update installiert: ein Klick fordert den Neustart an (sofort, wenn im Leerlauf, sonst sobald er im Leerlauf ist) und setzt seine Unterhaltung fort
agentbar-restart-ask-busy = Sein Claude neu starten, sobald er im Leerlauf ist?
agentbar-restart-ask = Sein Claude jetzt neu starten?
agentbar-restart = Neu starten
agentbar-restart-resumed = seine Unterhaltung wird fortgesetzt
agentbar-cancel = Abbrechen
agentbar-restart-done = Claude von {$agent} startet neu, sobald er im Leerlauf ist, und setzt seine Unterhaltung fort
agentbar-restart-failed = Neustart von Claude von {$agent}

## Wo sein Loop läuft, wessen Hände darauf sind

agentbar-place-host = Host
agentbar-hands-copy = Kopie
agentbar-hands-controls = Steuerung
agentbar-moving = verschiebt…
agentbar-runs-hosted = sein Loop läuft auf aiballs Sitzungs-Host;
agentbar-runs-mux = sein Loop läuft in {$mux} (claude-loop);
agentbar-copy-tip = dieses Terminal ist eine Kopie: du schaust zu, nichts, was du tippst, erreicht seinen Claude, und die Sitzung behält ihre Größe.
agentbar-controls-tip = du hast die Steuerung, geteilt mit jedem anderen Client: die Größe folgt dem, der zuletzt tippt.
agentbar-others-attached = { $others ->
    [one] {$others} weiterer Client verbunden ({$typing} mit der Steuerung).
   *[other] {$others} weitere Clients verbunden ({$typing} mit der Steuerung).
}
agentbar-proxy-alive = Der Terminal-Proxy vor Claude lebt.
agentbar-place-click = Ein Klick: die Steuerung nehmen oder abgeben, den Loop verschieben.
agentbar-others-type = das Terminal von claude-loop tippt auch hinein
agentbar-take-controls = Die Steuerung nehmen
agentbar-leave-copy = Für eine Kopie abgeben
agentbar-close-others = Die anderen schließen ({$others})
agentbar-close-others-tip = { $others ->
    [one] der andere mit dieser Sitzung verbundene Client verlässt sie (das Terminal von claude-loop, ein anderes Terminal Velocity); sein Claude und dieses Terminal laufen weiter
   *[other] die {$others} anderen mit dieser Sitzung verbundenen Clients verlassen sie (das Terminal von claude-loop, ein anderes Terminal Velocity); sein Claude und dieses Terminal laufen weiter
}
agentbar-move-into = In {$mux} verschieben
agentbar-move-to-host = Auf den Host verschieben
agentbar-move-tip-into = sein Claude startet in {$mux} neu und setzt seine Unterhaltung fort
agentbar-move-tip-to-host = sein Claude startet auf aiballs Host neu und setzt seine Unterhaltung fort
agentbar-move-interrupts = {" — "}er arbeitet gerade: das Verschieben unterbricht ihn
agentbar-moved-into = {$agent} in {$mux} verschoben, seine Unterhaltung fortgesetzt
agentbar-moved-to-host = {$agent} auf aiballs Host verschoben, seine Unterhaltung fortgesetzt
agentbar-move-failed-into = Verschieben von {$agent} in {$mux}
agentbar-move-failed-to-host = Verschieben von {$agent} auf aiballs Host
agentbar-hold-failed = Halten von {$agent}
agentbar-closed-others = { $count ->
    [one] den anderen Client dieser Sitzung geschlossen
   *[other] die {$count} anderen Clients dieser Sitzung geschlossen
}
agentbar-closed-others-all = die anderen Clients dieser Sitzung geschlossen
agentbar-closed-others-asked = den Host der Sitzung gebeten, ihre anderen Clients zu schließen
agentbar-others-left = { $count ->
    [one] {$count} anderer Client noch verbunden: {$mux} kann ihn noch nicht von diesem unterscheiden
   *[other] {$count} andere Clients noch verbunden: {$mux} kann sie noch nicht von diesem unterscheiden
}
agentbar-close-others-failed = die anderen Clients schließen

## Remote Control, Warnungen, der Prompt

agentbar-rc-on = Remote Control ist an: dieser Claude kann von claude.ai und der mobilen App übernommen werden
agentbar-rc-off = Remote Control ist aus. /rc in der Sitzung schaltet es ein; die Loops eines Ordners bekommen es aus den Einstellungen des Projekts
agentbar-trust = diesem Ordner vertrauen?
agentbar-not-logged-in = nicht angemeldet
agentbar-api-unreachable = API nicht erreichbar
agentbar-link-down = Loop-Verbindung unterbrochen
agentbar-aiball-unreachable = aiball nicht erreichbar
agentbar-prompt-input = Claudes Prompt ist auf dem Bildschirm, mit noch nicht gesendetem Text
agentbar-prompt-empty = Claudes Prompt ist auf dem Bildschirm, leer
agentbar-typing = ein Mensch hat gerade in sein Terminal getippt: der Loop hält sich zurück
agentbar-zen = Zen
agentbar-zen-tip = Zen-Modus: der Loop bleibt still

## Seine Zähler

agentbar-all = alle:{$count}
agentbar-all-tip = a: alle offenen Tickets des Projekts
agentbar-backlog = Backlog:{$count}
agentbar-backlog-tip = b: sein Backlog, die Tickets, die er ansehen soll; ein Klick listet sie
agentbar-events = Ereig.:{$count}
agentbar-events-tip = e: seine noch nicht gesehenen Ereignisse — Pings, Antworten, Entscheidungen, die auf ihn warten
agentbar-holds = hält:{$count}
agentbar-holds-tip = die Tickets, die er hält
agentbar-wake-tip = Arbeit wartet auf den Loop: er weckt seinen Claude dafür, wenn der Countdown endet
agentbar-pending-tip = Arbeit wartet auf den Loop (Ereignisse oder Backlog)

## Sein Backlog, über der Leiste

agentbar-backlog-of = Backlog von {$agent}
agentbar-reading = liest…
agentbar-backlog-error = Backlog: {$error}
agentbar-backlog-empty = nichts in seinem Backlog
agentbar-tier-critical = kritisch
agentbar-tier-hot = heiß
agentbar-tier-yours = seins
agentbar-tier-decision = seine Entscheidung offen
agentbar-tier-waiting = wartet auf andere
agentbar-tier-blocked = blockiert
agentbar-tier-other = andere

agentbar-info-resuming = wird fortgesetzt
agentbar-info-compacting = komprimiert
agentbar-info-wait = wartet
agentbar-info-interrupted = unterbrochen
agentbar-info-user = ein Mensch an der Tastatur
agentbar-info-picker-session = Sitzungsauswahl
agentbar-info-picker-mode = Modusauswahl
agentbar-info-error-rate-limit = Ratenlimit
agentbar-info-error-overloaded = API überlastet
agentbar-info-error-api = API-Fehler
agentbar-info-retry = Versuch {$attempt}
agentbar-restart-cancel-ask = Den anstehenden Neustart seines Claude zurücknehmen?
agentbar-restart-cancel = Zurücknehmen
agentbar-restart-keep = Behalten
agentbar-restart-cancelled = der Neustart von {$agent}s Claude ist zurückgenommen; das Update bleibt angeboten
agentbar-restart-none = kein Neustart von {$agent}s Claude stand an
agentbar-restart-cancel-failed = Zurücknehmen des Neustarts von {$agent}s Claude
agentbar-restart-too-old = sein Loop startete, bevor aiball einen Neustart zurücknehmen konnte: er startet neu
