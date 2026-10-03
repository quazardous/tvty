# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und IDs wie
# Englisch (assets/locales/en/), die Referenz.
# Das 📢 eines Projekts und seine Optionen.

## Das 📢

megaphone-no-project = Eine Daueranweisung und ein Weckfokus gehören zu einem Projekt: zeig zuerst eins an
megaphone-standing = Daueranweisung — {$project}
megaphone-standing-hint = Steht am Anfang jedes Weckens seiner Agenten, bei Ereignis wie Backlog. Hinterlass eine, bevor du weggehst; lösche sie, wenn du zurück bist.
megaphone-prompt-placeholder = z. B. erst leichtes Debugging, keine großen Änderungen
megaphone-focus = Weckfokus
megaphone-focus-hint = Nur diese Tickets wecken die Agenten des Projekts, Backlog und Ereignisse. 123, 456 behält nur diese; !789 behält alle außer diesem. 123+ nimmt seine Kinder mit, 123++ alle Nachfahren, +123 / ++123 seine Eltern, 123~ seine verknüpften Tickets. Ereignisse außerhalb bleiben ungelesen, bis du ihn löschst.
megaphone-tickets-placeholder = z. B. 2518, 2523++   oder   !2180
megaphone-until-placeholder = bis (optional): 2026-09-30 18:00
megaphone-not-a-date = bis: {$typed} ist kein Datum (2026-09-30 18:00)
megaphone-not-a-time = bis: {$typed} ist hier keine gültige Uhrzeit
megaphone-clear = Löschen
megaphone-save = Speichern
megaphone-said-nothing = {$project}: nichts lenkt seine Agenten mehr
megaphone-said-prompt = {$project}: seine Agenten lesen „{$prompt}“ bei jedem Wecken
megaphone-said-focus = {$project}: nur sein Fokus weckt seine Agenten
megaphone-tip-standing = Daueranweisung: {$prompt}
megaphone-tip-focus = Weckfokus: {$focus}

## Eine Nachricht an alle Agenten

megaphone-message = Nachricht an alle Agenten
megaphone-message-hint = Jetzt in jede laufende Agenten-Sitzung getippt, egal was sie tut. Senden und halten hält auch jeden Loop an (nicht AFK ∞): kein Wecken beginnt neue Arbeit, bis du sie freigibst. Leer gelassen, wird der angezeigte Text gesendet.
megaphone-no-loop = Kein Agenten-Loop läuft.
megaphone-running = {$count} laufen: {$names}
megaphone-release = Freigeben
megaphone-send = Senden
megaphone-send-hold = Senden und halten
megaphone-typed-into = getippt in {$names}
megaphone-queued-for = eingereiht für {$names}
megaphone-held = gehalten: {$names}
megaphone-released = freigegeben: {$names}
megaphone-hold-not-applied = Halten nicht übernommen

## Die Optionen eines Projekts

projectopts-global = Global
projectopts-a-project = ein Projekt…
projectopts-failed = Projekteinstellungen
projectopts-choose = Wähle oben ein Projekt.
projectopts-no-folder = aiball kennt keinen Ordner dieses Projekts auf diesem Rechner: keiner seiner Agenten arbeitet hier.
projectopts-asking = Frage aiball…
projectopts-could-not-say = aiball konnte es nicht sagen: {$error}
projectopts-no-file = {$folder} hat keine .aiball.yaml, auch kein Ordner darüber: aiballs Standardwerte gelten. Neues Projekt… richtet es ein.
projectopts-written-in = Geschrieben in {$file}.
projectopts-also-serves = Sie gilt auch für {$others}: eine Änderung hier gilt auch für sie.
projectopts-on-named = an: {$name}
projectopts-none-set = {$project} setzt nichts von der Board-Config: es hat die Werte des Boards.
projectopts-all-keys = Alle seine Schlüssel
