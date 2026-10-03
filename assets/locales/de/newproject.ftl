# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und dieselben ids
# wie im Englischen (assets/locales/en/), das als Referenz gilt.
# Der Assistent für ein neues Projekt. Was aiball getan zu haben sagt,
# erscheint so, wie aiball es sagt.

newproject-title = Neues Projekt
newproject-step-folder = Ordner
newproject-step-identity = Wer darin arbeitet
newproject-step-done = Einrichtung
newproject-step-next = Wie weiter

## Der Ordner

newproject-which-folder = Welcher Ordner wird das Projekt?
newproject-folder-placeholder = der Ordner des Projekts, z. B. ~/dev/app
newproject-choose-folder = Den Ordner des Projekts wählen
newproject-browse = Durchsuchen…
newproject-folder-empty = Der Ordner, in dem der Agent arbeiten wird: die Wurzel des Projekts.
newproject-folder-missing = Diesen Ordner gibt es nicht.
newproject-folder-file = Das ist eine Datei, kein Ordner.
newproject-folder-checking = aiball wird gefragt…
newproject-folder-project = Schon ein aiball-Projekt: {$name} ({$file}). Weiter zeigt, was darin steht, um es neu einzurichten.
newproject-folder-configured = Schon für aiball eingerichtet ({$file}). Weiter zeigt, was darin steht, um es neu einzurichten.
newproject-folder-git = Ein git-Repository: bereit.
newproject-folder-ready = Bereit (kein git-Repository).
newproject-open-running = {$label} öffnen, läuft
newproject-open = {$name} öffnen
newproject-next = Weiter →
newproject-back = ← Zurück

## Wer darin arbeitet

newproject-project-placeholder = Projekt
newproject-agent-placeholder = Agent
newproject-field-project = Projekt
newproject-field-agent = Agent
newproject-bad-project = Der Name des Projekts: Buchstaben, Ziffern, -, _ und .
newproject-bad-agent = Der Name des Agenten: Buchstaben, Ziffern, -, _ und .
newproject-filled-from = Aus {$file} übernommen: Ändere, was du willst; was hier steht, wird eingerichtet.
newproject-crew = ein Crew-Agent
newproject-crew-about = Neben dem Lead des Projekts, auf den Tickets, die er bekommt; aus: der Lead.
newproject-host = seine Loops auf aiballs Host
newproject-host-about = Wo die in diesem Ordner gestarteten Loops laufen; aus: in {$mux}.
newproject-rc = Remote Control
newproject-rc-about = Sein Claude ist von claude.ai und der Claude-App aus erreichbar.
newproject-private = ein privates Projekt
newproject-private-about = aiball liefert ihm sein privates Kit (keine öffentlichen Tickets, keine Follower).
newproject-noclaim = kein Claim
newproject-noclaim-about = Der Agent arbeitet nur an den Tickets, die ihm zugewiesen sind, nimmt nie eines aus dem Pool.
newproject-in = aiball in {$folder}:
newproject-will-file = {$will} {$file}
newproject-will-created = erstellt
newproject-will-added = ergänzt seinen Eintrag in
newproject-will-rewritten = schreibt seinen Eintrag neu in
newproject-will-patched = aktualisiert
newproject-will-overwrote = überschreibt
newproject-will-kept = behält
newproject-joins = {$name} ist schon auf dem Board: Dieser Ordner schließt sich an (ein weiterer Ordner oder ein Crew-Agent).
newproject-then-sets = setzt dann {$what} in .aiball.yaml
newproject-files = .mcp.json: aiballs MCP-Server für Claude Code · .aiball.yaml: Projekt, Agent, Rolle, wo seine Loops laufen.
newproject-set-up = Einrichten
newproject-setting-up = Wird eingerichtet…
newproject-unsaved = eingerichtet, aber wo seine Loops laufen und sein Remote Control wurden nicht gespeichert: {$error}
newproject-set-in = {$what} in {$file} gesetzt

## Eingerichtet

newproject-done = {$name} ist eingerichtet. aiball sagte:
newproject-failed = aiball konnte es nicht einrichten:
newproject-nothing-said = (nichts gesagt)

## Wie weiter

newproject-next-start = Den Agenten starten
newproject-next-start-host = „Erste Sitzung starten“ startet das Claude Code von {$agent} auf aiballs Host, im Ordner des Projekts; sein Terminal öffnet sich hier, seine Tickets daneben.
newproject-next-start-mux = „Erste Sitzung starten“ startet das Claude Code von {$agent} in {$mux}, im Ordner des Projekts; sein Terminal öffnet sich hier, seine Tickets daneben.
newproject-next-mcp = aiballs MCP-Server annehmen
newproject-next-mcp-about = Beim ersten Start in diesem Ordner fragt Claude Code, ob es den MCP-Server nutzen soll, den .mcp.json angibt (aiball): Nimm ihn an. Ohne ihn kann der Agent weder das Board lesen noch auf seine Tickets antworten. Versehentlich abgelehnt? /mcp in Claude Code schaltet ihn ein.
newproject-next-skill = aiballs Skill installieren
newproject-next-skill-about = Claude Code hat auf diesem Rechner noch keinen aiball-Skill: `aiball init skill`, einmal, installiert ihn — dann kennt der Agent die richtigen Handgriffe auf dem Board.
newproject-next-work = Ihm Arbeit geben
newproject-next-work-about = „+ Neu“ im Ticket-Panel legt ein Ticket auf {$name} an; der Agent nimmt es bei seinem nächsten Aufwachen auf, und seine Pläne und Fragen kommen als Benachrichtigungen zurück.
newproject-close = Schließen
newproject-start-first = Erste Sitzung starten

newproject-step-mcp-created = {$path}: angelegt, mit dem Eintrag von aiball
newproject-step-mcp-added-others = {$path}: Eintrag von aiball hinzugefügt, die anderen Server behalten
newproject-step-mcp-added = {$path}: Eintrag von aiball hinzugefügt
newproject-step-mcp-rewritten = {$path}: Eintrag von aiball in seiner aktuellen Form neu geschrieben
newproject-step-mcp-kept = {$path}: Eintrag von aiball schon da, behalten (force überschreibt ihn)
newproject-step-file-created = {$path}: angelegt ({$set})
newproject-step-file-overwrote = {$path}: überschrieben ({$set})
newproject-step-file-kept = {$path}: schon da, behalten (force überschreibt sie)
newproject-step-consumer = {$path}: Identität gesetzt ({$set})
newproject-step-type-kept = {$path}: Projekttyp schon {$value}
newproject-step-type = {$path}: Projekttyp {$value}
newproject-step-type-was = {$path}: Projekttyp {$value} (vorher: {$previous})
newproject-step-deny = {$path}: Code-Werkzeuge verweigert ({$set})
