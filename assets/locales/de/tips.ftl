# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und dieselben ids
# wie im Englischen (assets/locales/en/), das als Referenz gilt.
# Die Tipps (tips/<id>.md: ihr Englisch ist das der Datei, ein Test prüft
# es). {$key} ist die Taste des Befehls des Tipps, {$key-font-reset} die
# Taste von font.reset: so wie sie sind, nie übersetzt. **fett** und `code` bleiben.

tip-slider = **{$key}** zeigt deine Projekte als Stapel, das neueste zuerst. Lass Strg los, um das gewählte zu öffnen.
tip-gallery = **{$key}** zeigt jedes Terminal live, nebeneinander. Tippen filtert, Pfeiltasten bewegen, Enter öffnet.
tip-gallery-filter = Jedes Vorschaubild ist live: Sieh all deinen Agenten gleichzeitig bei der Arbeit zu und öffne den, der dich braucht.
tip-notice = **{$key}** springt zur neuesten Benachrichtigung: das Terminal des Agenten, sein Ticket daneben.
tip-goto = **{$key}** geht zu einem Ticket: Tippe seine Nummer oder füge den `#C.`-Link eines Kommentars ein.
tip-full-list = **{$key}** zeigt alle Tickets im Vollbild, das kritische ganz oben.
tip-bulk = **Strg+Klick** wählt Tickets, **Umschalt+Klick** einen Bereich: Dann schließe, schlummere oder markiere sie alle auf einmal.
tip-afk = **{$key}** schaltet den AFK-Modus des angezeigten Agenten weiter: selbstständig ▶, 10 Minuten gehalten ‖, gehalten, bis du loslässt ■.
tip-remote-control = **RC** leuchtet in der Leiste des Agenten auf, während sein Claude in Remote Control ist: Verfolge und beantworte ihn von claude.ai aus. `/rc` schaltet es ein.
tip-move-loop = Der Chip **… · Steuerung** in der Leiste des Agenten (Host, tmux oder psmux): Ein Klick nimmt die Steuerung oder gibt sie ab, oder verschiebt den Loop auf die andere Seite.
tip-critical = Ein rotes **!** markiert das kritische Ticket: das offene, das die meisten anderen aufhält. Es voranzubringen macht die meiste Arbeit frei.
tip-references = Jedes **#N** in einem Ticket ist ein Link, sogar zu einem Ticket eines anderen Projekts: Ein Klick öffnet es.
tip-comment-link = Klick auf die Marke **#C.** eines Kommentars, um seinen Link zu kopieren, und füge ihn dann in ein beliebiges Ticket oder das Sprungfeld ein.
tip-reply = **Strg+Enter** sendet deine Antwort; **leise** veröffentlicht sie, ohne jemanden zu benachrichtigen.
tip-preview = **Schreiben / Vorschau**: Sieh dein Markdown gerendert, Bilder inbegriffen, bevor du es abschickst.
tip-escape = **Esc** verlässt zuerst das Feld, in dem du tippst; ein zweites **Esc** schließt die Seite.
tip-sections = Zieh am Titel eines Abschnitts, um seine Größe zu ändern; ein Doppelklick auf einen Titel verteilt den Platz wieder gleichmäßig.
tip-fullscreen = **{$key}** schaltet das Fenster in den Vollbildmodus und zurück.
tip-filter = **{$key}** filtert die Sitzungen: tippen, Enter öffnet, Esc leert.
tip-new-project = **Neues Projekt…** im Menü (das Symbol der App, oben links) richtet einen Ordner für aiball ein und startet seinen Agenten.
tip-restart = **tvty neu starten** im Menü startet nur das Fenster neu: Die Sitzungen deiner Agenten laufen weiter.
tip-shortcuts = Jedes Tastenkürzel lässt sich ändern: hier unter **Tastenkürzel** oder in `keymap.toml` im Konfigurationsordner von tvty.
tip-opacity = **Terminal-Deckkraft** unter 100 % lässt deinen Desktop durch die Terminals scheinen.
tip-aiball-board = **aiball** im Menü öffnet aiballs Board in deinem Browser: die Tickets aller Projekte.
tip-font = **{$key}** macht die Schrift der Terminals größer, **{$key-font-smaller}** kleiner, **{$key-font-reset}** wie zuvor.
tip-new-ticket = **{$key}** legt von überall ein neues Ticket an, im Projekt, in dem du das letzte angelegt hast.
tip-room = **{$key}** klappt das Ticket-Panel weg, **{$key-sidebar-toggle}** die Liste der Projekte: aller Platz für das Terminal.
tip-themes = **{$key}** schaltet durch die Farbthemen; das **◐** der Titelleiste listet sie alle auf.
tip-wizard = Der Assistent zeigt, was er schreiben wird, bevor er irgendetwas schreibt: Nichts ändert sich, bis du es einrichtest.
tip-file-and-exit = **Anlegen und verlassen** legt das Ticket an, ohne es zu öffnen: Du bist zurück im Terminal oder in der vollen Liste.
tip-panel-pin = Die **Nadel** im Kopf des Ticket-Panels: Angeheftet bleibt das Panel neben dem Terminal; nicht angeheftet liegt es darüber und das Terminal behält seine Breite.
tip-followers = **Follower** erfahren von jeder Bewegung dieses Tickets: ✕ beendet einen, die Auswahlliste fügt einen hinzu. Ein stummgeschalteter Follower ist ausgegraut.
tip-thread-order = **⇅** stellt den Thread auf den Kopf: das Neueste zuerst oder zuletzt, im Panel und im Vollbild jeweils auf eigene Weise.
tip-held-selection = Text in einem Terminal auswählen **hält es an**, während du kopierst: Die Ausgabe des Agenten wartet, nichts scrollt unter der Maus davon.
tip-workspaces = Ein **Arbeitsbereich** speichert eine Reihe von Projekten und wie jeder ihrer Agenten läuft, selbstständig ▶ oder gehalten ■. **+ Arbeitsbereich** speichert, was jetzt läuft, unter einem Namen.
tip-workspace-open = **Öffnen** eines Arbeitsbereichs startet, was er speichert, und gibt frei, was er selbstständig laufen lässt; **Schließen** stoppt seine Sitzungen, die mit anderen Arbeitsbereichen geteilten bleiben unberührt.
tip-project-order = Zieh ein Projekt in der Liste auf ein anderes, um sie in deine Reihenfolge zu bringen; **⇅** wechselt zwischen zuletzt, alphabetisch und deiner.
tip-language = Die zwei Buchstaben in der Titelleiste lassen tvty eine andere Sprache sprechen: Englisch, Französisch, Spanisch oder Deutsch.
tip-stop-session = Über einem laufenden Agent in der Liste stoppt **⏹** seinen Loop: ein erster Klick fragt, ein zweiter stoppt. Er bleibt da, um neu zu starten.
