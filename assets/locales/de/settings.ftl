# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und ids wie
# Englisch (assets/locales/en/), die Referenz.
# Einstellungen (src/settings.rs, src/options.rs, die Einstellungsseiten
# von src/shell.rs). Die Wörter einer Einstellung gehen nach ihrem Schlüssel:
# appearance.terminal_font_size ist setting-appearance-terminal_font_size,
# ihre Beschreibung .about, die zwei Zustände eines Schalters .on und .off.

## Die Einstellungen

setting-appearance-language = Sprache
    .about = Die Sprache der Oberfläche. Was Menschen schreiben (Tickets, Kommentare, Namen), bleibt, wie es geschrieben ist.
setting-appearance-terminal_font_size = Terminalschrift
    .about = Der Text der Terminals. Auch Strg+Umschalt+= / Strg+Umschalt+− / Strg+Umschalt+0.
setting-appearance-window_font_size = Fenstertext
    .about = Alles rund um die Terminals — Listen, Tickets, Menüs.
setting-notifications-max = Höchstens angezeigt
    .about = In der oberen rechten Ecke des Terminals (unten links über einem Vollbild), die neueste in der Ecke; ältere machen Platz.
setting-notifications-seconds = Sekunden angezeigt
    .about = Dann verschwindet sie, außer der Zeiger liegt darauf.
setting-notifications-own = Deine eigenen Gesten
    .about = Ein Ticket geschlossen, eine Antwort gepostet, ein Plan angenommen: gemeldet, sobald aiball es hat. Eine Ablehnung wird immer gemeldet.
    .on = angezeigt
    .off = verborgen
setting-notifications-copied = Kopiert
    .about = Ein kurzes „In die Zwischenablage kopiert“ unten, wenn Text in die Zwischenablage geht (nicht bei einer bloßen Auswahl).
    .on = angezeigt
    .off = verborgen
setting-terminal-cursor = Cursor
    .about = Im Terminal, das die Tasten hat: blinkend, fest, oder wie das Programm es verlangt. Er bleibt an, während du tippst und Text kommt.
setting-terminal-scrollback = Verlaufszeilen
    .about = Was jedes Terminal zum Zurückscrollen behält. Mehr kostet Speicher: etwa 4 KB pro Zeile bei 160 Spalten, also 5000 Zeilen höchstens etwa 19 MB pro Terminal. Der Verlauf einer tmux-Sitzung ist der von tmux (sein history-limit).
setting-scroll-speed = Mausradtempo
    .about = Mal das von tvty: bei 1 entspricht eine Raste etwa drei Ticketzeilen, fünf Zeilen im Verlauf eines Terminals.
setting-mouse-focus = Fokus
    .about = Was dem Terminal oder einem Eingabefeld die Tastatur gibt: ein Klick, oder der Zeiger, der darüber fährt (wie der Fokus eines Fenstermanagers der Maus folgt). Standardmäßig wie das System.
setting-appearance-theme = Fenster
    .about = Das Farbthema des Fensters. Strg+Umschalt+K blättert durch sie.
setting-appearance-terminal_theme = Terminal
    .about = Das eigene der Terminals, oder das des Fensters — ein dunkles Terminal in einem hellen Fenster.
setting-appearance-terminal_opacity = Terminal-Deckkraft
    .about = Unter 100 % scheint der Desktop live durch den Hintergrund der Terminals. Die Listen, die Tickets und die Farben, die ein Programm setzt, bleiben deckend.
setting-appearance-terminal_blur = Unschärfe dahinter
    .about = Hinter einem durchscheinenden Terminal der Desktop unscharf. Nur wo der Compositor es anbietet (KDE); anderswo bleibt er scharf.
    .on = unscharf
    .off = scharf
setting-updates-check = Nach Updates suchen
    .about = Beim Start und einmal am Tag fragt es GitHub, ob ein neueres Terminal Velocity erschienen ist, und sagt es einmal (sonst wird nichts gesendet). Der Updater (Updates… im Menü) installiert es.
    .on = suchen
    .off = nie
setting-tips-show = Tipps zeigen
    .about = „Schon gewusst?“: ein kurzer Tipp einmal nach dem Start und beim ersten Öffnen einer Seite; nie einer zu etwas, das du schon nutzt. Tipps… im Menü zeigt sie alle.
    .on = zeigen
    .off = nie
setting-sessions-order = Reihenfolge
    .about = Die Projekte in der Liste, im Karussell und in der Galerie: das zuletzt benutzte zuerst, wie ctrl+tab geht; alphabetisch; oder deine, indem du sie in der Liste ziehst. Auch das ⇅ im Kopf der Liste.
setting-sessions-show_hub = Sitzungen des Hubs
    .about = Wenn dieser Rechner aiball über einen Proxy-Knoten erreicht: auch die Sitzungen auflisten, die auf dem Hub von aiball laufen, getrennt, unter „auf dem Hub“. Sie werden von hier gelesen (Zustand, Tickets), nie geöffnet: eine Sitzung wird von ihrem eigenen Rechner aus angehängt. Die Sitzungen anderer Knoten werden nie aufgelistet.
    .on = aufgelistet
    .off = nicht aufgelistet
setting-sessions-on_quit = Beim Beenden
    .about = Wenn tvty beendet wird, während Claude-Code-Loops dieses Rechners laufen: fragen, sie stoppen (über aiball: sie bleiben neu startbar) oder sie weiterlaufen lassen.
setting-sessions-on_start = Beim Start
    .about = Die Loops, die tvty beim Beenden gestoppt hat: fragen, sie neu starten wie sie waren (ihre Unterhaltung fortsetzend, ein angehaltener wieder angehalten), sie frisch neu starten (hochfahrend, dann auf sich gestellt) oder sie gestoppt lassen.
setting-tickets-panel_overlay = Platz
    .about = Neben dem Terminal, das dann schmaler ist — oder darüber, wie die Sitzungsliste links: das Terminal behält seine ganze Breite, und das Panel deckt seine rechte Seite ab, solange es offen ist.
    .on = über dem Terminal
    .off = neben dem Terminal
setting-tickets-newest_first = Neueste zuerst, Vollbild
    .about = Ein Ticket im Vollbild zeigt das neueste Wort seines Threads zuerst (auch ⇅ im Kopf des Tickets).
    .on = neueste zuerst
    .off = neueste zuletzt
setting-tickets-panel_newest_first = Neueste zuerst, im Panel
    .about = Ein Ticket im Panel zeigt das neueste Wort seines Threads zuerst; aus, zuletzt, bei der Antwort (auch ⇅ dort). Das Panel hat seine eigene Reihenfolge: die des Vollbilds ist nicht seine.
    .on = neueste zuerst
    .off = neueste zuletzt
setting-tickets-summary_open = Stand der Dinge, offen
    .about = Im Ticket-Panel zeigt der „Stand der Dinge“ eines Tickets seinen Text sofort; aus, ist er auf seinen Titel eingeklappt, einen Klick entfernt. Im Vollbild wird er immer gezeigt.
    .on = offen
    .off = eingeklappt
setting-tickets-ctrl_enter_opens = Strg+Enter
    .about = In einem neuen Ticket legt Strg+Enter es an und kehrt dorthin zurück, wo du warst („Anlegen und verlassen“), oder legt es an und öffnet es („Ticket anlegen“). Die Buttons tun jeweils das Ihre, egal was hier steht.
    .on = legt an und öffnet
    .off = legt an und schließt

## Die Seiten und Gruppen, nach ihrem Namen (settings-title-<name>)

settings-title-project = Projekt
settings-title-appearance = Darstellung
settings-title-layout = Layout
settings-title-ticket-list = Ticketliste
settings-title-keyboard-shortcuts = Tastenkürzel
settings-title-aiball = aiball
settings-title-about = Über
settings-title-language = Sprache
settings-title-sizes = Größen
settings-title-notifications = Benachrichtigungen
settings-title-mouse = Maus
settings-title-colours = Farben
settings-title-updates = Updates
settings-title-tips = Tipps
settings-title-sessions = Sitzungen
settings-title-ticket-panel = Ticket-Panel
settings-title-thread = Thread
settings-title-new-ticket = Neues Ticket
settings-title-sides = Seiten
settings-title-legend = Legende
settings-title-window = Fenster
settings-title-workspace = Arbeitsbereich
settings-title-terminal = Terminal
settings-title-fixed-keys = Feste Tasten
settings-title-folder = Ordner
settings-title-board = Board
settings-title-board-wide-only = Nur boardweit
settings-title-per-project-only = Nur pro Projekt
settings-title-in-aiball-yaml = In .aiball.yaml

## Die Seite

settings-heading = Einstellungen
settings-search-placeholder = Suchen…
settings-modified-hint = @modified: was du geändert hast
settings-search = Suche: {$text}
settings-close = ✕  Esc
settings-nothing-found = Nichts gefunden. Suche einen Namen, ein Wort, das darin steht, eine Taste (ctrl+shift+b), einen Wert — oder @modified.
settings-default = Standard
settings-default-back = Standard
settings-back-to = zurück auf {$value}
settings-own-themes = Deine eigenen Themen (im Themenformat von gpui-component) gehören nach ~/.config/tvty/themes/: sie erscheinen hier, wenn diese Seite das nächste Mal geöffnet wird.

## Auswahl

setting-language-auto = Automatisch ({$lang})
settings-ask = Fragen
settings-order-recent = Zuletzt benutzt zuerst
settings-cursor-blink = Blinkend
settings-cursor-steady = Fest
settings-cursor-program = Wie das Programm es verlangt
settings-order-alpha = Alphabetisch
settings-order-yours = Deine (in der Liste gezogen)
settings-quit-stop = Stoppen
settings-quit-keep = Weiterlaufen lassen
settings-start-restart = Neu starten wie sie waren
settings-start-fresh = Frisch neu starten
settings-start-leave = Gestoppt lassen
settings-focus-click = Klick
settings-focus-hover = Darüberfahren
settings-same-as-window = Wie das Fenster
settings-dark = Dunkel
settings-light = Hell
settings-theme-dark = DUNKEL
settings-theme-light = HELL
settings-theme-window = Fenster
settings-theme-terminal = Terminal
settings-theme-opacity = Terminal-Deckkraft

## Seiten

settings-side-open = offen
settings-side-folded = eingeklappt
settings-side-said = {$state} · {$width} px
settings-side-list = Projektliste
settings-side-list-about = Über dem Terminal, links. Zieh an ihrem Rand, um ihre Größe zu ändern; ihr Griff klappt sie ein.
settings-side-panel = Ticket-Panel
settings-side-panel-about = Rechts vom Terminal. Zieh an seinem Rand, um seine Größe zu ändern; sein Griff klappt es ein.
settings-widths = Breiten
settings-widths-about = Zurück zu den Standardwerten: eine Liste von 290 px, ein Panel von einem Drittel des Fensters.
settings-reset = Zurücksetzen

## Über

settings-about-version = Version
settings-about-aiball-at = aiball unter
settings-about-acting-as = Handelt als
settings-about-bus = aiball-Bus
settings-about-bus-said = Version {$version}, als {$who}
settings-about-bus-said-kind = Version {$version}, als {$who} ({$kind})
settings-about-not-connected = nicht verbunden
settings-about-live = Live-Board
settings-about-not-subscribed = nicht abonniert
settings-about-subscriptions = { $count ->
    [one] {$count} Abonnement auf dem Bus
   *[other] {$count} Abonnements auf dem Bus
}
settings-about-theme = Thema
settings-about-terminal-theme = Terminal-Thema
settings-about-the-windows = das des Fensters
settings-about-config = Einstellungen, Kürzel, Themen
settings-about-state = Layout und Arbeitsbereich
settings-about-fonts = Schriften
settings-about-what = Ein natives Terminal, um mit vielen KI-Coding-Agenten zugleich zu arbeiten: ihre Terminals nach Projekt gruppiert, die Tickets jedes Projekts daneben — der Agent fragt, du entscheidest, er macht weiter. Gebaut auf aiball, das die Loops der Agenten und ihr Board betreibt.
settings-about-github = GitHub ↗
settings-about-aiball = aiball auf GitHub ↗
settings-about-license = MIT-Lizenz ↗
settings-about-footer = Terminal Velocity (tvty). MIT-Lizenz. Mitgelieferte Themen: siehe themes/README.md.

## Die Config von aiball

settings-aiball-failed = aiball-Config
settings-aiball-unreadable = Die Config von aiball konnte nicht gelesen werden: {$error}. Wähle den Bereich oben links erneut, um es nochmal zu versuchen.
settings-aiball-reading = Lese die Config von aiball…
settings-aiball-board = Die eigene Config des Boards: was jedes Projekt bekommt, sofern es nichts anderes sagt. Wähle oben links ein Projekt für seine eigene.
settings-aiball-project = Was {$project} über der Config des Boards sagt; ↺ gibt einen Schlüssel an den Wert des Boards zurück.
settings-aiball-in-file = aiball liest diese aus der .aiball.yaml jedes Projekts, nicht aus seinem Config-Speicher: ändere sie in dieser Datei.
settings-aiball-global-only = aiball deklariert diese für das ganze Board: ein Wert für alle Projekte, gesetzt in Global.
settings-aiball-project-only = aiball deklariert diese nur pro Projekt: kein boardweiter Wert; wähle oben ein Projekt, um sie zu setzen.
settings-aiball-open-global = Global öffnen
settings-aiball-protected = 🔒 geschützt
settings-aiball-set-in-file = gesetzt in .aiball.yaml
settings-aiball-board-wide = boardweit: gesetzt in Global
settings-aiball-per-project = pro Projekt
settings-aiball-from-board = {" "}Aus der Config des Boards.
settings-aiball-board-back = Board
settings-on = an
settings-off = aus

## Die Legende der Ticketliste

settings-legend-intro = Die Ticketliste, wie aiball sie für dich berechnet: das Band, wer dran ist und das Zustandssymbol kommen von aiball; der Streifen kommt von tvty.
settings-legend-order = Reihenfolge
settings-legend-band-moderate = das Ticket oder Kommentare dazu warten auf Moderation
settings-legend-band-decide = ein Plan, eine Lösung, ein Wontfix oder eine Eskalation wartet auf deine Entscheidung
settings-legend-band-working = ein Agent hält es oder ist bei einem Schritt
settings-legend-band-open = offen, nichts Dringendes
settings-legend-band = {$what}; das neueste zuerst
settings-legend-glyph = Zustandssymbol — farbig, wenn es auf dich wartet, sonst gedämpft
settings-legend-always-blue = {$what} — immer blau
settings-legend-always-amber = {$what} — immer bernsteinfarben
settings-legend-always-red = {$what} — immer rot
settings-legend-stripe = Streifen — wer dran ist
settings-legend-stripe-solid = eine Entscheidung wartet auf dich, und sie ist die letzte Nachricht
settings-legend-stripe-dashed = eine Entscheidung wartet auf dich, aber das Gespräch ging danach weiter
settings-legend-stripe-yours = du bist dran: ein Agent hat dir geantwortet
settings-legend-stripe-waiting = dünn und gepunktet: dein Wort ist das letzte, du wartest
settings-legend-stripe-none = kein Streifen: der Ball liegt beim Agenten
settings-legend-stripe-tape = Absperrband: das Ticket wartet auf Moderation, nichts geht weiter, bis du es durchlässt
settings-legend-rest = Der Rest
settings-legend-unread = Kommentare, blau: etwas Neues für dich, ungelesen (sein Titel auch fett)
settings-legend-others-spoke = hell, seine Spitze links: jemand anderes hat zuletzt gesprochen
settings-legend-you-spoke = dezent, seine Spitze rechts: du hast zuletzt gesprochen
settings-legend-pending = Kommentare, die auf Moderation warten
settings-legend-agent = Agent
settings-legend-holder = der Agent, der es hält; die Flamme, wenn er kürzlich aktiv war
settings-legend-critical = das kritische Ticket des Projekts: es hält 3 offene Tickets auf
settings-legend-priority = Priorität: dringend, hoch, niedrig (normal zeigt nichts)
