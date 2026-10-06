# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und dieselben ids
# wie im Englischen (assets/locales/en/), das als Referenz gilt.
# Die Arbeitsbereiche, und die Liste, die Sitzungen auswählt.

## Die Liste der Arbeitsbereiche

workspaces-new = + Arbeitsbereich
workspaces-new-tip = speichert die Gruppen, die gerade laufen, in einer Liste ausgewählt, unter einem Namen
workspaces-none = Noch kein Arbeitsbereich: Ein Arbeitsbereich speichert Gruppen, und wie jede ihrer Sitzungen läuft.
workspaces-default-name = Arbeitsbereich
workspaces-its-name = sein Name
workspaces-act-open = Öffnen
workspaces-act-open-tip = startet, was gestoppt ist, und setzt jede Sitzung wie gespeichert — vorher wird gefragt
workspaces-act-shut = Schließen
workspaces-act-shut-tip = stoppt seine Sitzungen — vorher wird gefragt, welche; eine Gruppe, die auch ein anderer Arbeitsbereich hat, bleibt
workspaces-act-save = speichern
workspaces-act-save-tip = speichert ihn erneut, wie alles jetzt ist, oder fügt eine Gruppe hinzu — in einer Liste ausgewählt
workspaces-again-tip = die angezeigte Gruppe, erneut gespeichert, wie sie jetzt läuft
workspaces-add-tip = fügt die angezeigte Gruppe hinzu, ihre Sitzungen, wie sie laufen
workspaces-act-rename = umbenennen
workspaces-act-rename-tip = ein anderer Name; Enter bestätigt
workspaces-act-delete = löschen
workspaces-act-delete-sure = löschen: sicher?
workspaces-act-delete-tip = vergisst den Arbeitsbereich; seine Sitzungen bleiben unberührt
workspaces-no-project = kein Projekt
workspaces-drop-tip = nimmt diese Gruppe aus dem Arbeitsbereich (ihre Sitzungen bleiben unberührt)
workspaces-now-stopped = gestoppt
workspaces-now-runs = läuft
workspaces-now-held = gehalten
workspaces-kept-own = gespeichert als selbstständig; jetzt {$now}
workspaces-kept-held = gespeichert als gehalten; jetzt {$now}
workspaces-open-sets = {" — "}Öffnen setzt sie wie gespeichert

## Was aus einer Sitzung wird

workspaces-start-own = gestoppt: zu starten, selbstständig
workspaces-start-held = gestoppt: zu starten, gehalten
workspaces-let-go = gehalten: freizugeben
workspaces-to-hold = läuft selbstständig: anzuhalten
workspaces-as-kept = wie gespeichert

## Die Liste, die auswählt

workspaces-quit-title = Auch die Claude-Code-Sitzungen stoppen?
workspaces-quit-summary = An: gestoppt, wenn tvty beendet wird (sie bleiben neu startbar). Aus: sie läuft weiter.
workspaces-new-title = Ein neuer Arbeitsbereich
workspaces-new-summary = An: darin gespeichert, jede, wie sie jetzt läuft (selbstständig oder gehalten).
workspaces-save-title = {$name} speichern, wie alles jetzt ist
workspaces-save-summary = An: darin, jede, wie sie jetzt läuft. Aus: herausgenommen.
workspaces-shut-title = {$name} schließen: seine Sitzungen stoppen?
workspaces-shut-summary = An: gestoppt. Aus: sie läuft weiter. Eine Gruppe, die auch ein anderer Arbeitsbereich hat, bleibt aus.
workspaces-open-title = {$name} öffnen
workspaces-open-summary = An: erledigt. Aus: bleibt, wie sie ist.
workspaces-restart-title = Die Sitzungen neu starten, die beim Beenden von tvty gestoppt wurden?
workspaces-restart-summary = An: neu gestartet, ihre Unterhaltung wird fortgesetzt. Wie sie waren: Eine gehaltene Sitzung wird wieder gehalten; frisch: Jede startet und läuft dann selbstständig.
workspaces-host = Host
workspaces-held-for-good = {" · "}gehalten bis zur Freigabe
workspaces-held-while = {" · "}eine Weile gehalten
workspaces-ran-on = {$place} · lief weiter: ihr Stopp hat nicht gegriffen
workspaces-not-running = läuft nicht: bleibt, wie sie war
workspaces-new-in-group = neu in dieser Gruppe
workspaces-not-in-yet = noch nicht darin
workspaces-also-in = auch in {$others}
workspaces-some = {$ticked} von {$of}
workspaces-whole-group = die ganze Gruppe
workspaces-name = Name
workspaces-cancel = Abbrechen
workspaces-keep = Speichern
workspaces-quit-keep = Beenden, alle weiterlaufen lassen
workspaces-quit-stop = Beenden, die {$count} aktivierten stoppen
workspaces-shut-keep = Schließen, alle weiterlaufen lassen
workspaces-shut-stop = Schließen, die {$count} aktivierten stoppen
workspaces-open-do = Öffnen, die {$count} aktivierten ausführen
workspaces-not-now = Nicht jetzt
workspaces-restart-fresh = Die {$count} frisch neu starten
workspaces-restart-as-were = Die {$count} neu starten, wie sie waren
workspaces-remember-quit = Diese Wahl merken (alle stoppen oder alle behalten)
workspaces-remember-restart = Diese Wahl merken (jedes Mal, alle)
workspaces-remember-tip = Einstellungen > Layout > Sitzungen ändert das

## Was getan wurde

workspaces-none-runs = {$name}: keine seiner Sitzungen läuft
workspaces-all-as-kept = {$name}: jede Sitzung ist wie gespeichert
workspaces-kept = { $count ->
    [one] Arbeitsbereich {$name} gespeichert: {$count} Gruppe
   *[other] Arbeitsbereich {$name} gespeichert: {$count} Gruppen
}
workspaces-shut-run-on = {$name} geschlossen: seine Sitzungen laufen weiter
workspaces-shut-stopped = { $count ->
    [one] {$name} geschlossen: {$count} Sitzung gestoppt
   *[other] {$name} geschlossen: {$count} Sitzungen gestoppt
}
workspaces-shut = schließen
workspaces-no-loop = { $count ->
    [one] {$count} Sitzung auf keinem Loop dieses Rechners: nicht gestoppt
   *[other] {$count} Sitzungen auf keinem Loop dieses Rechners: nicht gestoppt
}
workspaces-nothing-to-start = {$agent}: weder Loop noch Ordner bekannt, um ihn zu starten
workspaces-astray = {$agent}: {$cwd} ist der Ordner von {$other}, nicht gestartet
workspaces-mode-failed = {$agent}: sein Modus: {$error}
workspaces-opened = { $count ->
    [one] {$name} geöffnet: {$count} Sitzung gesetzt
   *[other] {$name} geöffnet: {$count} Sitzungen gesetzt
}
workspaces-opening = {$name} wird geöffnet
workspaces-added = {$project} ist in {$name}
workspaces-shown-tip = das Terminal, das gezeigt wird, sobald die Sitzungen zurück sind
