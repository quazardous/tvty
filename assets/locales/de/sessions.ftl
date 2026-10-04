# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und ids wie
# Englisch (assets/locales/en/), die Referenz.
# Die Liste der Sitzungen, die Zeile einer Sitzung, die Tabs und die Titelleiste
# (src/shell.rs, src/shell/loopstabs.rs, src/shell/tabs.rs, src/status.rs).

## Der Kopf der Liste

sessions-tab-sessions = Sitzungen
sessions-tab-workspaces = Arbeitsbereiche
sessions-order-recent = das zuletzt benutzte Projekt zuerst, wie ctrl+tab geht; ein Klick: alphabetisch
sessions-order-alpha = alphabetisch; ein Klick: deine Reihenfolge (zieh ein Projekt)
sessions-order-yours = deine Reihenfolge: zieh ein Projekt; ein Klick: das zuletzt benutzte Projekt zuerst
sessions-new-project = + Projekt
sessions-new-project-tip = ein Ordner wird zum aiball-Projekt (aiball init), dann seine erste Sitzung
sessions-new-terminal-tip = ein eigenes Terminal, das tvty überdauert
sessions-filter = Filtern…  ctrl+shift+f
sessions-fold = Die Liste der Sitzungen einklappen

## Ihre Gruppen

sessions-group-live = aktiv
sessions-group-idle = gestoppt
sessions-group-shut = ohne Loop
sessions-group-on-hub = am Hub
sessions-none = Keine Sitzung
sessions-none-found = Keine Sitzung gefunden
sessions-first-project = Richte dein erstes Projekt ein
sessions-none-stopped = Kein gestoppter Loop
sessions-none-stopped-found = Kein gestoppter Loop gefunden
sessions-none-shut = Kein Agent ohne Loop
sessions-none-shut-found = Kein Agent gefunden
sessions-none-hub = Keine Sitzung am Hub
sessions-none-hub-found = Keine Sitzung am Hub gefunden
sessions-no-project = Kein Projekt

## Die Überschrift eines Projekts

sessions-heading-tip = seine Tickets in der Leiste, keine Sitzung geöffnet; zieh es auf ein anderes, um es zu verschieben
sessions-new-session = + Sitzung
sessions-project-options = seine Einstellungen: wo seine Loops laufen, Remote Control, seine Board-Konfiguration
sessions-project-terminal = ein Terminal im Ordner des Projekts, mit ihm aufgelistet

## Die Zeile einer Sitzung

sessions-open = in tvty geöffnet: sein Terminal läuft hier
sessions-looped = {$mux}: sein Claude läuft in claude-loop, geöffnet über {$mux} (nicht auf aiballs Host)
sessions-attached = { $others ->
    [one] {$others} weiterer Client verbunden (das Terminal von claude-loop, ein anderes tvty), {$typing} mit der Steuerung
   *[other] {$others} weitere Clients verbunden (das Terminal von claude-loop, ein anderes tvty), {$typing} mit der Steuerung
}
sessions-attached-copy = : hier als Kopie geöffnet
sessions-host-shell = ein Terminal auf aiballs Host, ohne Claude
sessions-update = sein Claude Code hat ein Update installiert: starte es über seine Leiste neu
sessions-update-pending = sein Neustart ist angefordert: sein Claude startet neu, sobald er untätig ist (das Abzeichen ist auf seiner Leiste)
sessions-starting = startet…
sessions-start = ▶ starten
sessions-astray = ⚠ Ordner von {$other}
sessions-astray-tip = {$other} arbeitet in {$cwd}: hier gestartet, würde dieser Agent dessen Unterhaltung fortsetzen. Nicht gestartet.
sessions-stop-ask = stoppen?
sessions-stop-tip = den Loop von {$who} stoppen: über aiball bleibt er neu startbar (unter den gestoppten) — ein zweiter Klick stoppt ihn
sessions-stopped = {$who} gestoppt
sessions-stop-failed-loop = {$who} stoppen
sessions-forget-ask = vergessen?
sessions-forget-tip = {$who} vergessen: aiball listet ihn nicht mehr; sein Ordner, seine .aiball.yaml und die Tickets des Projekts bleiben — ein zweiter Klick vergisst
sessions-forgot = {$who} vergessen
sessions-forget-failed = {$who} vergessen

## Der Zustand einer Sitzung, in einer Zeile

sessions-offline = offline
sessions-offline-tip = sein Loop ist nicht mit aiball verbunden
sessions-working = arbeitet
sessions-starting-state = startet
sessions-idle = Leerlauf
sessions-booting = bootet
sessions-claude-working = Claude arbeitet
sessions-claude-starting = Claude startet
sessions-claude-idle = Claude ist im Leerlauf
sessions-held-typing-for-good = gehalten bis zur Freigabe (ein Mensch tippt darin): der Loop weckt es nicht
sessions-typing = ein Mensch tippt darin: der Loop wartet
sessions-held-for-good = gehalten bis zur Freigabe: der Loop weckt es nicht
sessions-held-while = eine Weile gehalten: der Loop weckt es nicht
sessions-loop-drives = der Loop steuert es selbstständig
sessions-mark-held-for-good = gehalten bis zur Freigabe
sessions-mark-held-while = eine Weile gehalten
sessions-mark-own = selbstständig

## Die Markierungen der eingeklappten Liste

sessions-mark-tip = {$agent}: {$said}
sessions-mark-limit = {$agent}: {$said} — {$limit}

## „+ Sitzung“

sessions-form-cwd = Arbeitsverzeichnis
sessions-form-agent = Agent
sessions-form-where = wo sein Claude arbeitet
sessions-form-host-here = aiballs Host startet ihn hier
sessions-form-loop-here = claude-loop startet hier
sessions-form-no-dir = kein solches Verzeichnis
sessions-form-crew = ein Crew-Agent, neben dem Haupt-Loop
sessions-form-on-host = auf aiballs Host, ohne {$mux}
sessions-form-cancel = Abbrechen
sessions-form-start = Starten
sessions-a-loop = ein Loop
sessions-astray-said = {$cwd} ist der Ordner von {$other}: {$agent} würde dessen Unterhaltung fortsetzen
sessions-start-failed = starten
sessions-start-host-failed = auf dem Host starten
sessions-started-host = {$agent} auf aiballs Host in {$cwd} gestartet
sessions-started = {$agent} in {$cwd} gestartet
sessions-runs-already = {$agent} läuft schon: als Kopie geöffnet

## Die Abzeichen: eines Projekts, eines Agenten

sessions-badge-critical = das kritische Ticket, #{$ticket}, ist hier: es blockiert die meisten offenen Tickets — ein Klick öffnet es
sessions-badge-decisions = { $count ->
    [one] {$count} Ticket wartet auf deine Entscheidung
   *[other] {$count} Tickets warten auf deine Entscheidung
}
sessions-badge-unread = { $count ->
    [one] {$count} Ticket mit etwas Neuem für dich
   *[other] {$count} Tickets mit etwas Neuem für dich
}
sessions-light-backlog = { $count ->
    [one] Backlog: {$count} Ticket für ihn zum Ansehen
   *[other] Backlog: {$count} Tickets für ihn zum Ansehen
}
sessions-light-backlog-unknown = Backlog: unbekannt, bis sein Loop es meldet
sessions-light-critical = er hält das kritische Ticket: das, welches die meisten offenen Tickets blockiert
sessions-light-events = Ereignisse: {$count} noch nicht gesehen — Pings, Antworten, Entscheidungen, die auf ihn warten
sessions-hub-mark = Auf aiballs Hub, eine andere Maschine: von hier gelesen (sein Zustand, seine Tickets), nicht geöffnet — eine Sitzung wird von ihrer eigenen Maschine aus verbunden.

## Die Tabs

sessions-tab-name = sein Name
sessions-rename-failed = das Terminal umbenennen
sessions-stop-failed = das Terminal stoppen
sessions-rename-tip = umbenennen (oder ein Doppelklick, F2)
sessions-close-shell = schließen: stoppt dieses Terminal
sessions-close-tab = den Tab schließen: sein Claude läuft weiter
sessions-tab-shell-tip = ein Terminal auf aiballs Host, ohne Claude; ein Doppelklick (oder F2) benennt es um
sessions-tab-tip = ctrl+pgup / ctrl+pgdn: der Tab davor, danach
sessions-new-tab-project = ein Terminal im Ordner des Projekts
sessions-new-tab-home = ein Terminal im Home-Verzeichnis

## Das Fenster

sessions-pick = Wähle links ein Terminal · ctrl+shift+space zeigt alle
sessions-on-hub = {$agent} — am Hub
sessions-window-hub-project = {$name} — {$project} · {$agent} · am Hub
sessions-window-hub = {$name} — {$agent} · am Hub
sessions-bus-down = aiballs Bus ist ausgefallen: tvty verbindet sich neu; die Listen können solange hinterherhinken
sessions-bus-failing = das Abonnieren von {$what} ist fehlgeschlagen: erneut versucht, ganz gelesen; die Listen können solange hinterherhinken
sessions-menu = Menü: Über, Hilfe, Neustart, Beenden
sessions-goto = Zu einem Ticket: seine Nummer oder der #C.-Link eines Kommentars, Enter · ctrl+shift+g
sessions-themes = Die Farbthemen — das nächste
sessions-message-all = Eine Nachricht an jeden laufenden Agenten: senden, senden & halten, Halten aufheben
sessions-settings = Einstellungen
sessions-user = als wer tvty auf aiballs Board handelt
