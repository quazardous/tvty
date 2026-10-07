# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und ids wie
# Englisch (assets/locales/en/), die Referenz.
# Tastenkürzel (src/keymap.rs, src/options.rs, src/shell.rs). Die Wörter
# eines Befehls gehen nach seinem Namen: notice.open ist keys-notice-open.

## Was jeder Befehl tut

keys-notice-open = Zur neuesten Benachrichtigung: das Terminal des Agenten, sein Ticket
keys-slider-next = Karussell: die Gruppen als Stapel, die neueste zuerst; Strg loslassen öffnet
keys-slider-back = Karussell, rückwärts
keys-tab-next = Der nächste Tab in der gezeigten Gruppe
keys-tab-back = Der vorige Tab in der gezeigten Gruppe
keys-gallery-toggle = Galerie aller Terminals; tippen filtert, Pfeiltasten bewegen, Enter öffnet
keys-panel-toggle = Das Ticket-Panel ein- oder ausklappen
keys-sidebar-toggle = Die Projektliste ein- oder ausklappen
keys-afk-cycle = Der AFK-Modus des gezeigten Agenten: auto → 10 min anhalten → anhalten, wirksam 3 s nach dem letzten Druck (ein Terminal ohne Agent bekommt F9)
keys-sessions-filter = Die Sitzungen filtern: tippen, Enter öffnet, Pfeiltasten bewegen, Esc leert
keys-ticket-goto = Zu einem Ticket: seine Nummer oder der #C.-Link eines Kommentars, Enter öffnet es (das #… der Titelleiste)
keys-options-toggle = Einstellungen
keys-app-quit = tvty beenden, wie beim Schließen des Fensters (laufende Loops: angehalten oder behalten, wie gewählt)
keys-help-menu = Hilfe: über tvty, seine Dokumentation, Neuigkeiten, ein Neustart
keys-window-fullscreen = Das Fenster im Vollbild, oder zurück
keys-font-bigger = Terminalschrift größer
keys-font-smaller = Terminalschrift kleiner
keys-font-reset = Terminalschrift zurück auf Standard
keys-ticket-new = Ein neues Ticket (auch + im Panel und in der vollen Liste)
keys-project-megaphone = Das 📢 des gezeigten Projekts: seine ständige Anweisung und sein Weck-Fokus
keys-list-full = Die Ticketliste im Vollbild
keys-theme-next = Nächstes Farbthema
keys-debug-inspector = Der Inspektor von GPUI: ein Element wählen, seine id sehen und wo es gebaut wird (Debug-Builds)
keys-terminal-copy = Die Auswahl kopieren
keys-terminal-paste = Die Zwischenablage einfügen
keys-terminal-tab = Tab, an das Programm (nicht der Fokus aufs nächste Element)
keys-terminal-back-tab = Umschalt+Tab, an das Programm

## Die Tasten, die keine Befehle sind: was sie tun, die Tasten als .keys

keys-fixed-esc = Die Galerie, das Karussell, die Themenliste, die Einstellungen, die volle Liste schließen
    .keys = Esc
keys-fixed-arrows = Im Karussell und in der Galerie: zur dort gesehenen Karte gehen
    .keys = Pfeiltasten
keys-fixed-select = In einem Terminal: Text · ein Wort · eine Zeile auswählen — in die primäre Auswahl kopiert
    .keys = Ziehen · Doppelklick · Dreifachklick
keys-fixed-middle-click = In einem Terminal: die primäre Auswahl einfügen
    .keys = Mittelklick
keys-fixed-right-click = In einem Terminal: Kopieren, Einfügen — auf einem Link: Link öffnen, Link kopieren
    .keys = Rechtsklick
keys-fixed-ctrl-click = In einem Terminal, auf einem Link: ihn öffnen
    .keys = Strg+Klick
keys-fixed-wheel = Durch den Verlauf scrollen
    .keys = Mausrad
## Die Seite

keys-intro = Jedes Kürzel ist ein Befehl, gebunden in einem Kontext: Terminal, wenn ein Terminal den Fokus hat, Fenster überall sonst. Die tiefste Bindung gewinnt; eine nirgends gebundene Taste geht an das Programm des Terminals.
keys-how = Klick auf eine Taste, um sie zu ändern, + um eine hinzuzufügen, × um sie zu entfernen. Gespeichert in {$file}, das du auch selbst bearbeiten kannst.
keys-reset-all = Alle zurücksetzen
keys-not-applied = Nicht übernommen: {$error}
keys-context-window = Fenster — überall in tvty, Vollbildseiten eingeschlossen
keys-context-workspace = Arbeitsbereich — die Terminals, die Sitzungsliste und das Panel daneben
keys-context-terminal = Terminal — wenn ein Terminal den Fokus hat
keys-press = Drück eine Taste… (Esc bricht ab)
keys-typing-refused = {$key} dient zum Tippen: in einem Terminal bleibt sie beim Programm
keys-remove = diese Taste entfernen
keys-the-program = das Programm
keys-masked = {$what} — in einem Terminal verdeckt durch {$by}
keys-no-key = keine Taste
keys-conflict = {$key} führt {$other} aus: für {$name} übernehmen?
keys-replace = Ersetzen
keys-cancel = Abbrechen
keys-freed-terminal = An das Programm des Terminals zurückgegeben
keys-freed-focus = An das zurückgegeben, was den Fokus hat
