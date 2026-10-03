# Die Config von aiball, wie Einstellungen > aiball sie zeigt: nach ihrem
# Schlüssel, stabil (ein Schlüssel, den tvty nicht kennt, zeigt das eigene
# Englisch von aiball). Punkte in einem Schlüssel werden zu Bindestrichen.

aiball-group-autopoll = Autopoll
aiball-group-backlog = Backlog
aiball-group-claude-loop = Claude-Loop
aiball-group-defaults = Standardwerte
aiball-group-earn = Verdienst
aiball-group-on-repetitive-denied = Bei wiederholter Ablehnung
aiball-group-rules = Regeln
aiball-group-steps = Schritte
aiball-group-tickets = Tickets
aiball-group-updates = Updates
aiball-group-wait-credit = Wartekredit

aiball-config-updates-check = Nach Updates suchen
    .about = true (default) = beim Start des Daemons und auf Anfrage liest er das neueste aiball-Release auf GitHub, damit Tray, GNOME-Erweiterung und `aiball version` melden können, dass ein Update da ist. false = keine Anfrage nach außen.
aiball-config-tickets-defaults-priority = Standardpriorität
    .about = Die Priorität eines neuen Tickets, das ohne eigene angelegt wird. Setz einen globalen Standard; ein Projekt kann ihn überschreiben.
aiball-config-tickets-defaults-broadcast-new = Neue Tickets als Rundruf
    .about = Wenn an, wird ein neues Ticket ohne eigenen Bereich als Rundruf markiert (die Follower des Projekts werden benachrichtigt). Deklariert; durchgesetzt wird es mit seinem Verbraucher.
aiball-config-tickets-rules-summary-max = Länge der Zusammenfassung (Zeichen)
    .about = Das längste summary_until, das ein Agent in einen Kommentar schreiben darf. Ein längeres wird mit Begründung abgelehnt und nichts gepostet; es wird nie gekürzt. Menschen sind ausgenommen. 0 = keine Grenze.
aiball-config-tickets-rules-require-then = Agenten-Kommentare brauchen then: oder handback
    .about = Wenn an, muss der Kommentar eines Agents ohne then: handback setzen (true: er gibt das Ticket zurück, false: er behält es), sonst wird er mit Begründung abgelehnt und nichts gepostet. Menschen sind ausgenommen.
aiball-config-tickets-rules-require-commits = Agenten-Kommentare brauchen Commits
    .about = Wenn an, muss der Kommentar eines Agents sagen, welche Commits er liefert (commits: ["<sha>"]) oder dass er keine liefert (commits: null oder "none"), sonst wird er mit Begründung abgelehnt. Ein Client von vor diesem Feld wird bis zu seiner Neuverbindung nur gewarnt statt abgelehnt. Menschen, Schließen und Wiederöffnen sind ausgenommen.
aiball-config-tickets-steps-stale = Schritt verstummt nach
    .about = Ein Schritt (then: continue), nach dem so lange nichts kommt, wird in der Inbox markiert: die angekündigte Arbeit ist verstummt. 0 = nie markieren.
aiball-config-tickets-steps-hot = Minuten, die ein Schritt sein Ticket oben im Backlog seines Autors hält
    .about = Nachdem ein Agent einen Schritt (then: continue) gepostet hat, führt dessen Ticket so lange den Backlog dieses Agents an — gleich nach den Ereignissen, vor jedem anderen Ticket —, gezählt ab dem Schritt. Danach reiht es sich wie jedes andere ein. Es ändert nur die Reihenfolge; die sichtbare Markierung 'hot' hat ihre eigene Regel. 0 = ein Schritt bekommt keinen Vorrang.
aiball-config-tickets-steps-max-wait = Längste Wartezeit eines Schritts
    .about = Das Höchste, was ein Agent bei einem Schritt (then: continue) in resume_on.timer angeben darf. Eine längere Wartezeit wird mit dieser Grenze in der Begründung abgelehnt — darüber ist die Arbeit kein Schritt mehr, der auf einen Job wartet: gib das Ticket zurück oder schlag einen Plan vor.
aiball-config-tickets-wait-credit-enabled = Wartekredit
    .about = true (default) = das resume_on.timer eines Schritts verbraucht den Wartekredit eines Agents, verdient durch Arbeitsnachweise. false = Warten ist frei und unbegrenzt (höchstens tickets.steps.max_wait), nichts wird verdient, verbraucht oder erstattet, und Antworten und Wecken sagen nichts über Kredit.
aiball-config-tickets-wait-credit-refund = Wartekredit: frühe Rückkehr erstatten
    .about = true (default) = meldet sich ein Agent auf einem Ticket, bevor die Wartezeit seines Schritts endet, bekommt er den Rest dieser Wartezeit zurück. false = eine Wartezeit wird, einmal angegeben, ganz verbraucht.
aiball-config-tickets-wait-credit-earn-commit-max-age = Wartekredit: ältester Commit, der noch zählt
    .about = Ein zitierter Commit, dessen Commit-Datum älter ist, verdient nichts: der Kredit belohnt frische Arbeit.
aiball-config-tickets-wait-credit-earn-commits-per-comment = Wartekredit: Commits pro Kommentar
    .about = Die meisten Commits, die ein Kommentar für Kredit zitieren kann; die darüber verdienen nichts, und die Antwort sagt es.
aiball-config-tickets-wait-credit-start = Wartekredit zum Start
    .about = Jeder Agent beginnt jedes Projekt mit so viel Wartekredit, damit ein neuer Agent auf einen ersten Build warten kann. Der Kredit wird von Schritt-Timern (resume_on.timer) verbraucht und durch Arbeitsnachweise verdient.
aiball-config-tickets-wait-credit-max = Höchster Wartekredit eines Agents
    .about = Ein Guthaben geht nie darüber: was es übersteigen würde, wird nicht gutgeschrieben, und ein Guthaben darüber wird gekürzt. 0 = keine Obergrenze.
aiball-config-tickets-wait-credit-floor = Wartezeit eines Schritts, auch ohne Kredit
    .about = Fehlt Kredit, wird die Wartezeit eines Schritts auf das Guthaben begrenzt, aber nie darunter, damit ein Agent ohne Kredit nicht pausenlos wiederkommt. Das Guthaben fällt dadurch nie unter null. Ein Schritt, der 0 verlangt (sofort weitermachen), wird immer gewährt.
aiball-config-tickets-wait-credit-earn-resolved = Wartekredit für ein gelöst geschlossenes Ticket, mit Commit
    .about = Einmal pro Ticket verdient vom Agent, dessen Lösung angenommen wurde, wenn er vor dem Schließen einen Commit auf diesem Ticket zitiert hat (commits: [...]).
aiball-config-tickets-wait-credit-earn-resolved-no-commit = Wartekredit für ein gelöst geschlossenes Ticket, ohne Commit
    .about = Einmal pro Ticket verdient vom Agent, dessen Lösung angenommen wurde, wenn er keinen Commit auf diesem Ticket zitiert hat: eine Lösung ohne Code ist weniger wert.
aiball-config-tickets-wait-credit-earn-wontfix = Wartekredit für ein ohne Fix geschlossenes Ticket
    .about = Einmal pro Ticket verdient vom Agent, dessen Schließen ohne Fix (wontfix) angenommen wurde.
aiball-config-tickets-wait-credit-earn-lines-per-minute = Geänderte Zeilen pro Minute Wartekredit aus einem Commit
    .about = Ein Commit, den ein Agent in einer Antwort zitiert (commits: [...]), verdient eine Minute pro so vielen geänderten Zeilen, gelesen im Checkout des Agents. Einmal pro Commit.
aiball-config-tickets-wait-credit-earn-commit-max = Höchster Wartekredit pro Commit
    .about = Die Obergrenze dessen, was ein einzelner Commit verdient, egal wie groß sein Diff ist.
aiball-config-tickets-wait-credit-earn-commit-min = Geringster Wartekredit pro Commit
    .about = Was ein zitierter Commit mit mindestens einer geänderten Zeile verdient, egal wie klein sein Diff ist: ein kurzer Fix ist auch Arbeit. 0 = nur der Satz pro Zeile zählt.
aiball-config-tickets-backlog-claim-protect = Minuten, die der Claim eines arbeitenden Agents geschützt ist
    .about = Wie lange ein Claim gegen den Claim eines anderen Agents hält, gezählt ab der letzten Aktion seines Halters auf dem Ticket — wer daran arbeitet, hält den Schutz aufrecht. Der Claim eines anderen Agents in diesem Zeitraum wird abgelehnt; danach kann das Ticket übernommen werden, und der Thread hält es fest. Eine Zuweisung gewinnt immer gegen einen Claim. 0 = kein Schutz.
aiball-config-tickets-backlog-rest = Ruhezeit eines Tickets nach einem Backlog-Wecken
    .about = Nachdem ein Backlog-Wecken ein Ticket genannt hat, wie lange es aus den nächsten Backlog-Wecken des Agents herausbleibt, solange sich sonst niemand darum kümmert. Ein blockiertes Ticket ruht `blocked_multiplier`-mal länger, ein Ticket, dessen letzte Aktion ein Schritt ist, nur `after_step`. Ein mit CL_BACKLOG_COOLDOWN_SEC gestarteter Loop nimmt stattdessen diesen Wert. 0 = keine Ruhe.
aiball-config-tickets-backlog-depth = Wie tief der Backlog einen Agent weckt
    .about = Die tiefste Backlog-Stufe, für die ein Wecken einen Agent rufen darf, wenn er kein Ereignis zu lesen hat. Kritische, heiße und bearbeitbare Tickets wecken ihn immer. followup: auch ein Ticket, auf dem jemand geantwortet hat, das aber die ausstehende Entscheidung des Agents selbst festhält. waiting: auch ein Ticket, auf dem der Agent zuletzt gesprochen hat und sich seitdem nichts bewegt hat. blocked: auch ein Ticket, das eine offene Abhängigkeit festhält (es kommt ohnehin zurück, wenn sein Blocker schließt). Ein Ticket unterhalb der Einstellung bleibt im Backlog, sichtbar; es weckt nur niemanden.
aiball-config-tickets-backlog-blocked-multiplier = Backlog-Abklingzeit, Faktor für blockierte Tickets
    .about = Wie viel länger ein Backlog-Wecken ein BLOCKIERTES Ticket (durch ein offenes depends_on gesperrt) aus dem Weck-Pool heraushält als jedes andere Ticket. Es muss immer wieder auftauchen, damit es nicht vergessen wird, aber zwischen zwei Wecken bewegt sich nichts daran. 1 = dieselbe Abklingzeit wie der Rest.
aiball-config-tickets-backlog-after-step = Backlog-Abklingzeit nach einem Schritt
    .about = Wie lange ein Backlog-Wecken ein Ticket aus dem Weck-Pool heraushält, wenn seine letzte Aktion ein Schritt ist (then: continue), statt der ganzen Abklingzeit. Absichtlich kurz: der Schritt sagt, dass jetzt Arbeit ansteht, und die Pause lässt nur die Warteschlange weiterlaufen. 0 = nie zurückstellen.
aiball-config-claude-loop-on-repetitive-denied-threshold = Ablehnungen, bevor der Loop antwortet
    .about = Wie viele Tool-Aufrufe das Berechtigungssystem von Claude Code einem Agent in der letzten Stunde ablehnen muss, bevor sein Loop den Prompt unten sendet (oder die Ausgabe des Befehls). Nichts wird gesendet, solange beide leer sind.
aiball-config-claude-loop-on-repetitive-denied-max-per-hour = Höchstens gesendete Prompts pro Stunde
    .about = Höchstens so viele Prompts pro Stunde für wiederholte Ablehnungen, damit ein Kreislauf aus Ablehnung, Prompt, Ablehnung abbricht. 0 = nie senden.
aiball-config-claude-loop-on-repetitive-denied-prompt = Prompt bei wiederholten Ablehnungen
    .about = So wie er ist in den Prompt von Claude getippt, wie ein Wecken und nur, wenn ein Wecken möglich wäre (kein AFK-Halten, Zen, Tippen, Nutzungslimit, Boot oder beschäftigt). Leer = nichts (der Standard).
aiball-config-claude-loop-on-repetitive-denied-command = Befehl, der den Prompt schreibt
    .about = Ein Shell-Befehl, auf dem Rechner des Loops in seinem Ordner ausgeführt, höchstens 10 s. Er bekommt die Ablehnungen als JSON auf stdin (agent, project, cwd, tool, reason, last_hour, recent); was er ausgibt, ist der Prompt. Er hat Vorrang vor dem Prompt oben. Leer = keiner.
aiball-config-autopoll-volatile = Autopoll: einmalige Erinnerungen
    .about = true = nur benachrichtigen, wenn ein echt neuerer Ping ankommt (keine zeitbasierten Erinnerungen). false (default) = eine dauerhafte Erinnerung kommt nach throttle_seconds wieder.
aiball-config-autopoll-throttle = Autopoll: Erinnerungstakt
    .about = Erinnerungstakt in Sekunden, ignoriert bei volatile=true. 0 = bei jedem Stop (lästig). Neue Pings / neue offene Tickets umgehen die Drossel.
aiball-config-autopoll-recent-tickets = Autopoll: neueste Tickettitel
    .about = Bis zu N neueste ungelesene Tickettitel in der Begründung des Hooks, damit der Agent weiß, was wartet, bevor er abarbeitet. 0 = nur die Anzahl.
aiball-config-autopoll-backlog = Autopoll: Backlog als Auslöser
    .about = true (default) = offene Tickets im Bereich lösen Benachrichtigungen aus, auch ohne ungelesene Pings. false = nur Kontext (in der Begründung angezeigt, nie der Auslöser).
aiball-config-autopoll-tone = Autopoll: Ton
    .about = hint = höflich, leicht zu übergehen. directive (default) = nennt die Aktion ausdrücklich. imperative = letztes Mittel, wenn der Agent weiter um Erlaubnis fragt.
aiball-config-assign-window-sec = Zuweisungsfenster
    .about = Wie lange eine Zuweisung oder ein Claim gilt, bevor das Ticket in den gemeinsamen Pool zurückgeht.
aiball-config-hot-window-sec = Dauer „heiß“
    .about = Wie lange eine Bewegung ein Ticket in der Inbox heiß hält.
aiball-config-upstream-transport = Upstream: Transport
    .about = Wie Upstream-Aufrufe (GitHub / GitLab) hinausgehen: gh (die Anmeldedaten der CLI), http oder auto (gh, wenn es funktioniert). Der eigene Transport einer Kopplung hat Vorrang.
aiball-config-upstream-sync = Upstream: beobachten
    .about = pull (default) = ein gekoppeltes Repo wird beobachtet und seine Änderungen gemeldet; off = nur verknüpft. Die eigene Sync-Einstellung einer Kopplung hat Vorrang.
